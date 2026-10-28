import { spawn } from "node:child_process";
import { constants } from "node:fs";
import { access, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { homedir, tmpdir } from "node:os";
import { delimiter, isAbsolute, join, resolve } from "node:path";

export const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Find an installed Chrome/Chromium without downloading anything. */
export async function chromeExecutable() {
  const executable = async (path) => {
    try {
      await access(path, process.platform === "win32" ? constants.F_OK : constants.X_OK);
      return (await stat(path)).isFile();
    } catch { return false; }
  };
  const onPath = (name) => (process.env.PATH || "").split(delimiter).filter(Boolean).flatMap((dir) =>
    process.platform === "win32" && !name.endsWith(".exe") ? [join(dir, `${name}.exe`), join(dir, name)] : [join(dir, name)]);
  const override = process.env.SCHEMATIC_CHROME?.trim();
  if (override) {
    const candidates = isAbsolute(override) || /[/\\]/.test(override) ? [resolve(override)] : onPath(override);
    for (const candidate of candidates) if (await executable(candidate)) return candidate;
    throw new Error(`SCHEMATIC_CHROME does not name an executable browser: ${override}. Set it to the full Chrome/Chromium executable path, not the application directory.`);
  }
  let candidates = [];
  if (process.platform === "darwin") {
    candidates = ["/Applications", join(homedir(), "Applications")].flatMap((dir) => [
      join(dir, "Google Chrome.app/Contents/MacOS/Google Chrome"),
      join(dir, "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"),
      join(dir, "Chromium.app/Contents/MacOS/Chromium"),
      join(dir, "Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"),
    ]);
  } else if (process.platform === "win32") {
    candidates = [process.env.PROGRAMFILES, process.env["PROGRAMFILES(X86)"], process.env.LOCALAPPDATA].filter(Boolean).flatMap((dir) => [
      join(dir, "Google/Chrome/Application/chrome.exe"),
      join(dir, "Chromium/Application/chrome.exe"),
      join(dir, "Google/Chrome SxS/Application/chrome.exe"),
    ]);
  } else {
    candidates = ["/opt/google/chrome/chrome", "/opt/google/chrome-for-testing/chrome", "/snap/bin/chromium"];
  }
  candidates.push(...["google-chrome", "google-chrome-stable", "chrome", "chromium", "chromium-browser"].flatMap(onPath));
  for (const candidate of new Set(candidates)) if (await executable(candidate)) return candidate;
  throw new Error(`Chrome/Chromium was not found on ${process.platform}. Install Chrome or Chromium, or set SCHEMATIC_CHROME to its executable path. No browser is downloaded by these tools.`);
}

/** An isolated, owned headless Chrome process. Never attaches to user tabs. */
export async function browser({ width = 1440, height = 1000, dark = false, reduced = false } = {}) {
  if (typeof WebSocket !== "function") throw new Error("The browser tools require Node.js 22.13 or newer (built-in WebSocket).");
  const metrics = (w, h) => {
    if (![w, h].every((n) => Number.isInteger(n) && n > 0)) throw new Error("Browser width and height must be positive integers.");
    return { width: w, height: h, deviceScaleFactor: 1, mobile: w < 600 };
  };
  metrics(width, height);
  const chrome = await chromeExecutable();
  const profile = await mkdtemp(join(tmpdir(), "schematic-qa-"));
  const args = ["--headless=new", "--remote-debugging-address=127.0.0.1", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--no-first-run", "--no-default-browser-check", "--disable-background-networking", "--disable-extensions", "--hide-scrollbars"];
  if (process.platform === "linux") {
    args.push("--disable-dev-shm-usage");
    // CI/container hosts can prohibit Chrome's user-namespace sandbox.
    if (process.env.CI || process.getuid?.() === 0) args.push("--no-sandbox");
  }
  let child;
  try { child = spawn(chrome, [...args, "about:blank"], { stdio: ["ignore", "ignore", "pipe"], windowsHide: true }); }
  catch (error) { await rm(profile, { recursive: true, force: true }); throw error; }
  let launchError, stderr = "", exited = false, socket, id = 0, closing;
  const pending = new Map(), events = [], failures = [];
  child.stderr.on("data", (data) => { stderr = (stderr + data).slice(-4000); });
  const exit = new Promise((resolve) => {
    child.once("error", (error) => { launchError = error; exited = true; resolve(); });
    child.once("close", () => { exited = true; resolve(); });
  });
  const rejectPending = (error) => {
    for (const request of pending.values()) { clearTimeout(request.timer); request.reject(error); }
    pending.clear();
  };
  const waitForExit = async (ms) => {
    let timer;
    try { await Promise.race([exit, new Promise((resolve) => { timer = setTimeout(resolve, ms); })]); }
    finally { clearTimeout(timer); }
  };
  const signals = new Map();
  const close = () => closing ||= (async () => {
    for (const [signal, handler] of signals) process.removeListener(signal, handler);
    rejectPending(new Error("Browser closed"));
    socket?.close();
    if (!exited) { child.kill(); await waitForExit(1500); }
    if (!exited) { child.kill("SIGKILL"); await waitForExit(1500); }
    await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 150 });
  })();
  for (const [signal, code] of [["SIGINT", 130], ["SIGTERM", 143]]) {
    const handler = () => { close().finally(() => process.exit(code)); };
    signals.set(signal, handler); process.once(signal, handler);
  }
  try {
    let port;
    // Cold Linux runners can need more than eight seconds before Chrome writes
    // its port file. Keep the startup bounded without restarting the browser.
    for (let i = 0; i < 400 && !port; i++) {
      if (exited) throw new Error(`Chrome exited before opening its debug endpoint: ${launchError?.message || stderr.trim() || "no diagnostic output"}`);
      try { port = (await readFile(join(profile, "DevToolsActivePort"), "utf8")).split("\n")[0]; } catch { await delay(50); }
    }
    if (!/^\d+$/.test(port || "")) throw new Error(`Chrome failed to start its debug endpoint. ${stderr.trim()}`);
    const response = await fetch(`http://127.0.0.1:${port}/json`, { signal: AbortSignal.timeout(5000) });
    if (!response.ok) throw new Error(`Chrome debug endpoint returned HTTP ${response.status}`);
    const target = (await response.json()).find((t) => t.type === "page");
    if (!target?.webSocketDebuggerUrl) throw new Error("Chrome did not expose a page debug target");
    socket = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("Chrome debug connection timed out")), 5000);
      socket.onopen = () => { clearTimeout(timer); resolve(); };
      socket.onerror = () => { clearTimeout(timer); reject(new Error("Chrome debug connection failed")); };
    });
    socket.onclose = () => rejectPending(new Error("Chrome debug connection closed"));
    socket.onerror = () => rejectPending(new Error("Chrome debug connection failed"));
    socket.onmessage = (message) => {
      const data = JSON.parse(message.data);
      if (data.id) {
        const request = pending.get(data.id);
        if (request) { clearTimeout(request.timer); pending.delete(data.id); data.error ? request.reject(new Error(data.error.message)) : request.resolve(data.result); }
      } else {
        events.push(data);
        if (data.method === "Runtime.exceptionThrown") failures.push(data.params.exceptionDetails.exception?.description || data.params.exceptionDetails.text);
        if (data.method === "Runtime.consoleAPICalled" && ["error", "warning"].includes(data.params.type)) failures.push(data.params.args.map((a) => a.value ?? a.description).join(" "));
        if (data.method === "Network.loadingFailed" && !data.params.canceled) failures.push(data.params.errorText);
      }
    };
    const send = (method, params = {}) => new Promise((resolve, reject) => {
      if (socket.readyState !== WebSocket.OPEN || closing) return reject(new Error("Chrome debug connection is not open"));
      const n = ++id, timer = setTimeout(() => { pending.delete(n); reject(new Error(`CDP timeout: ${method}`)); }, 15000);
      pending.set(n, { resolve, reject, timer });
      try { socket.send(JSON.stringify({ id: n, method, params })); }
      catch (error) { clearTimeout(timer); pending.delete(n); reject(error); }
    });
    const evaluate = async (expression) => {
      const result = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true, userGesture: true });
      if (result.exceptionDetails) throw new Error(result.exceptionDetails.exception?.description || result.exceptionDetails.text);
      return result.result?.value;
    };
    await send("Runtime.enable"); await send("Log.enable"); await send("Network.enable"); await send("Page.enable");
    await send("Emulation.setDeviceMetricsOverride", metrics(width, height));
    await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: dark ? "dark" : "light" }, { name: "prefers-reduced-motion", value: reduced ? "reduce" : "no-preference" }] });
    return {
      send, evaluate, events, failures, close,
      async open(url) {
        const result = await send("Page.navigate", { url });
        if (result.errorText) throw new Error(`Could not open ${url}: ${result.errorText}`);
        for (let i = 0; i < 100; i++) {
          if (await evaluate("document.readyState === 'complete' && !!window.Schematic?._shell?.state.current")) return;
          await delay(50);
        }
        throw new Error(`Page did not boot: ${url}${failures.length ? `\n${failures.join("\n")}` : ""}`);
      },
      async screenshot(path) { const result = await send("Page.captureScreenshot", { format: "png" }); await writeFile(path, Buffer.from(result.data, "base64")); },
      async size(w, h) { await send("Emulation.setDeviceMetricsOverride", metrics(w, h)); },
    };
  } catch (error) { await close(); throw error; }
}
