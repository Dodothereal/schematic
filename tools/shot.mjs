#!/usr/bin/env node
// No dependencies. Node 22.13+, installed Chrome/Chromium or SCHEMATIC_CHROME.
// node tools/shot.mjs <url> [out.png] [--w=1440 --h=900 --wait=1500 --dark --eval="js" --after=1200 --full]
import { writeFile } from "node:fs/promises";
import { browser, delay } from "./browser.mjs";

const usage = "Usage: node tools/shot.mjs <url> [out.png] [--w=1440 --h=900 --wait=1500 --dark --eval=\"js\" --after=1200 --full]";
let b, eventIndex = 0;
function printEvents() {
  for (const event of b?.events.slice(eventIndex) || []) {
    if (event.method === "Runtime.consoleAPICalled") console.log(`[console.${event.params.type}]`, event.params.args.map((a) => a.value ?? a.description).join(" "));
    if (event.method === "Runtime.exceptionThrown") console.log("[EXCEPTION]", event.params.exceptionDetails.exception?.description || event.params.exceptionDetails.text, "@", event.params.exceptionDetails.url, event.params.exceptionDetails.lineNumber);
    if (event.method === "Log.entryAdded" && event.params.entry.level !== "verbose") console.log(`[log.${event.params.entry.level}]`, event.params.entry.text, event.params.entry.url || "");
  }
  eventIndex = b?.events.length || 0;
}
try {
  const args = process.argv.slice(2);
  if (args.includes("--help")) { console.log(usage); }
  else {
    const positional = [], options = {};
    for (const arg of args) {
      if (!arg.startsWith("--")) { positional.push(arg); continue; }
      const [name, ...parts] = arg.slice(2).split("=");
      if (!["w", "h", "wait", "dark", "eval", "after", "full"].includes(name)) throw new Error(`Unknown option: --${name}`);
      if (!["dark", "full"].includes(name) && !parts.length) throw new Error(`--${name} requires a value: --${name}=...`);
      options[name] = parts.length ? parts.join("=") : true;
    }
    const [url, out = "shot.png"] = positional;
    if (!url || positional.length > 2) throw new Error(usage);
    new URL(url);
    const number = (name, fallback, positive = false) => {
      const value = Number(options[name] ?? fallback);
      if (!Number.isInteger(value) || value < (positive ? 1 : 0)) throw new Error(`--${name} must be ${positive ? "a positive" : "a nonnegative"} integer`);
      return value;
    };
    const width = number("w", 1440, true), height = number("h", 900, true), wait = number("wait", 1500), after = number("after", 1200);
    b = await browser({ width, height, dark: !!options.dark && options.dark !== "false" });
    const navigation = await b.send("Page.navigate", { url });
    if (navigation.errorText) throw new Error(`Could not open ${url}: ${navigation.errorText}`);
    await delay(wait); printEvents();
    if (options.eval) { console.log("[eval]", JSON.stringify(await b.evaluate(options.eval))); await delay(after); printEvents(); }
    const full = !!options.full && options.full !== "false";
    const params = { format: "png", captureBeyondViewport: full };
    if (full) {
      const layout = await b.send("Page.getLayoutMetrics");
      const { x, y, width, height } = layout.cssContentSize;
      params.clip = { x, y, width, height, scale: 1 };
    }
    const shot = await b.send("Page.captureScreenshot", params);
    await writeFile(out, Buffer.from(shot.data, "base64"));
    printEvents(); console.log("saved", out);
  }
} catch (error) { console.error(error.message); process.exitCode = 1; }
finally { printEvents(); await b?.close(); }
