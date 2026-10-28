#!/usr/bin/env node
/* Tiny headless-Chrome harness (no deps, Node 22+).
   usage: node tools/shot.mjs <url> <out.png> [--w=1440 --h=900 --wait=1500 --dark --eval="js" --full]
   Prints console messages + uncaught exceptions, saves a screenshot.          */
import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [url, out = "shot.png", ...rest] = process.argv.slice(2);
const opt = Object.fromEntries(rest.map((a) => { const [k, ...v] = a.replace(/^--/, "").split("="); return [k, v.length ? v.join("=") : true]; }));
const W = +(opt.w || 1440), H = +(opt.h || 900), WAIT = +(opt.wait || 1500);
const port = 9300 + Math.floor(Math.random() * 500);
const chrome = spawn("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", [
  "--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "shot-"))}`,
  "--hide-scrollbars", "--no-first-run", `--window-size=${W},${H}`, "about:blank",
], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let ws;
try {
  let target;
  for (let i = 0; i < 50 && !target; i++) { try { target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page"); } catch { await sleep(100); } }
  ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let id = 0; const pending = new Map();
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); }
    if (d.method === "Runtime.consoleAPICalled") console.log(`[console.${d.params.type}]`, d.params.args.map((a) => a.value ?? a.description).join(" "));
    if (d.method === "Runtime.exceptionThrown") console.log("[EXCEPTION]", d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text, "@", d.params.exceptionDetails.url, d.params.exceptionDetails.lineNumber);
    if (d.method === "Log.entryAdded" && d.params.entry.level !== "verbose") console.log(`[log.${d.params.entry.level}]`, d.params.entry.text, d.params.entry.url || "");
  };
  const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  await send("Runtime.enable"); await send("Log.enable"); await send("Page.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  if (opt.dark) await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "dark" }] });
  await send("Page.navigate", { url });
  await sleep(WAIT);
  if (opt.eval) {
    const r = await send("Runtime.evaluate", { expression: opt.eval, awaitPromise: true, returnByValue: true });
    console.log("[eval]", JSON.stringify(r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description));
    await sleep(+(opt.after || 1200));
  }
  const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: !!opt.full });
  writeFileSync(out, Buffer.from(shot.result.data, "base64"));
  console.log("saved", out);
} finally { ws?.close(); chrome.kill(); }
