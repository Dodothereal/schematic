#!/usr/bin/env node
// Structural release checks. All builds happen in an isolated temporary copy.
// --defer-missing is for work in progress; --strict makes optional doc deferrals fail.
import { spawnSync } from "node:child_process";
import { cp, mkdtemp, readFile, readdir, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, extname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { runInNewContext, Script } from "node:vm";
import { executableCode } from "./javascript-code.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = new Set(process.argv.slice(2));
if ([...args].some((arg) => !["--strict", "--defer-missing"].includes(arg)) || (args.has("--strict") && args.has("--defer-missing"))) {
  console.error("Usage: node tools/check.mjs [--strict | --defer-missing]"); process.exit(2);
}
let passed = 0, failed = 0, deferred = 0;
const label = (file) => relative(root, file).split(sep).join("/");
const pass = (message) => { passed++; console.log(`PASS ${message}`); };
const fail = (message) => { failed++; console.error(`FAIL ${message}`); };
const defer = (message) => { deferred++; console.log(`DEFER ${message}`); };
const missing = (message, optional = false) => (args.has("--defer-missing") || (optional && !args.has("--strict")) ? defer : fail)(message);
const exists = async (file) => { try { return (await stat(file)).isFile(); } catch { return false; } };
async function walk(dir) {
  const files = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) files.push(...await walk(path));
    else if (entry.isFile()) files.push(path);
  }
  return files.sort();
}
const withinRoot = (file) => { const rel = relative(root, file); return !isAbsolute(rel) && rel !== ".." && !rel.startsWith(`..${sep}`); };

async function localTarget(from, value, { dependency = false, portable = false } = {}) {
  value = value.trim().replaceAll("&amp;", "&");
  if (!value || value.startsWith("#") || /^(data|blob):/i.test(value)) return;
  if (/^(?:[a-z][a-z\d+.-]*:|\/\/)/i.test(value)) {
    if (dependency) fail(`${label(from)} requires an external asset: ${value}`);
    return;
  }
  if (value.includes("${")) return; // Runtime templates are covered by browser tests.
  if (portable && dependency) { fail(`${label(from)} is not self-contained: ${value}`); return; }
  let path;
  try {
    const target = decodeURIComponent(value.split(/[?#]/)[0]);
    path = target.startsWith("/") ? resolve(root, `.${target}`) : resolve(dirname(from), target);
  } catch { fail(`${label(from)} has an invalid local URL: ${value}`); return; }
  if (!withinRoot(path)) { fail(`${label(from)} links outside the repository: ${value}`); return; }
  try {
    await stat(path);
  } catch { missing(`${label(from)} references a missing local target: ${value}`); }
}

// Inspect resource attributes, including HTML inside page templates. Ordinary
// outbound <a> links and SVG namespace declarations are not runtime dependencies.
async function assets(file, text, portable = false) {
  const before = failed + deferred;
  const clean = text.replace(/<!--[\s\S]*?-->/g, "").replace(/\/\*[\s\S]*?\*\//g, "");
  for (const match of clean.matchAll(/<(script|link|img|image|video|audio|source|track|iframe|embed|object|input|use)\b([^>]*?)>/gi)) {
    const attrs = new Map([...match[2].matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g)].map((m) => [m[1].toLowerCase(), m[2] ?? m[3] ?? m[4]]));
    const tag = match[1].toLowerCase();
    for (const key of ["src", "poster", "data", "href", "xlink:href"]) {
      if (!attrs.has(key) || (key === "data" && tag !== "object")) continue;
      if (tag === "link" && key === "href" && /^(canonical|alternate)$/i.test(attrs.get("rel") || "")) continue;
      await localTarget(file, attrs.get(key), { dependency: true, portable });
    }
    if (attrs.has("srcset") && !attrs.get("srcset").startsWith("data:")) {
      for (const source of attrs.get("srcset").split(",")) await localTarget(file, source.trim().split(/\s+/)[0], { dependency: true, portable });
    }
  }
  // Scope CSS parsing so JavaScript's new URL(value) is not an asset reference.
  const css = extname(file) === ".css" ? clean : [
    ...[...clean.matchAll(/<style\b[^>]*>([\s\S]*?)<\/style>/gi)].map((m) => m[1]),
    ...[...clean.matchAll(/\b(?:style|fill|stroke|filter|mask|clip-path|marker-start|marker-mid|marker-end)\s*=\s*(?:"([^"]*)"|'([^']*)')/gi)].map((m) => m[1] ?? m[2]),
  ].join("\n");
  for (const match of css.matchAll(/\burl\(\s*(?:"([^"]*)"|'([^']*)'|([^)]*))\s*\)/gi)) {
    await localTarget(file, match[1] ?? match[2] ?? match[3], { dependency: true, portable });
  }
  for (const match of css.matchAll(/@import\s+["']([^"']+)["']/gi)) await localTarget(file, match[1], { dependency: true, portable });
  if (failed + deferred === before) return true;
  return false;
}

function inlineSyntax(file, text) {
  let count = 0;
  for (const match of text.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script\s*>/gi)) {
    if (/\bsrc\s*=/.test(match[1]) || !match[2].trim()) continue;
    const type = match[1].match(/\btype\s*=\s*["']([^"']+)["']/i)?.[1];
    if (type && !["module", "text/javascript", "application/javascript"].includes(type)) continue;
    count++;
    try {
      if (type === "module") {
        const result = spawnSync(process.execPath, ["--input-type=module", "--check"], { input: match[2], encoding: "utf8", timeout: 10000 });
        if (result.status !== 0) throw new Error(result.error?.message || result.stderr);
      } else new Script(match[2], { filename: `${label(file)}:inline-${count}` });
    } catch (error) { fail(`JavaScript syntax in ${label(file)} inline script ${count}: ${error.message}`); }
  }
}

async function sourceChecks() {
  const scannerCases = [
    ['fetch("/api")', true],
    ['const sample="fetch(\\"/api\\")"', false],
    ['const sample=`fetch("/api")`', false],
    ['const sample=`${fetch("/api")}`', true],
    ['const regex=/"/; fetch("/api")', true],
    ['const regex=/fetch\\(/;', false],
    ['// fetch("/api")\nconst x=1', false],
    ['const x=`<div>${(()=>fetch("/api"))()}</div>`', true],
  ];
  if (scannerCases.every(([code, expected]) => /\bfetch\s*\(/.test(executableCode(code)) === expected)) pass('Offline scanner distinguishes literal samples from executable calls');
  else fail('Offline scanner literal/expression checks failed');
  const files = [...await walk(join(root, "prototype")), ...await walk(join(root, "tools"))];
  let count = 0;
  for (const file of files.filter((file) => /\.(?:m?js|cjs)$/.test(file))) {
    const result = spawnSync(process.execPath, ["--check", file], { encoding: "utf8", timeout: 10000 });
    if (result.status !== 0) fail(`JavaScript syntax: ${label(file)}\n${result.error?.message || result.stderr}`);
    else count++;
  }
  pass(`${count} JavaScript files parsed`);
  let inspected = 0;
  const before = failed + deferred;
  for (const file of files.filter((file) => file.startsWith(join(root, "prototype") + sep) && /\.(?:html|css|m?js)$/.test(file))) {
    const text = await readFile(file, "utf8");
    inspected++;
    await assets(file, text);
    if (extname(file) === ".html") inlineSyntax(file, text);
    if (/\.(?:html|m?js)$/.test(file)) {
      const code = text;
      const executable = executableCode(code);
      if (/\b(?:fetch|importScripts)\s*\(|\bnew\s+(?:XMLHttpRequest|WebSocket|EventSource)\s*\(|\bsendBeacon\s*\(/.test(executable)) fail(`Network API in offline runtime: ${label(file)}`);
      for (const match of code.matchAll(/\b(?:import|export)\s+(?:[^;\n]*?\s+from\s*)?["']([^"']+)["']|\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)) {
        if (!executable.slice(match.index, match.index + 6).trim()) continue;
        const specifier = match[1] ?? match[2];
        if (!specifier.startsWith(".")) fail(`Nonlocal module dependency in ${label(file)}: ${specifier}`);
        else await localTarget(file, specifier, { dependency: true });
      }
    }
  }
  if (failed + deferred === before) pass(`${inspected} runtime files have local assets and no network dependencies`);
  const pkg = JSON.parse(await readFile(join(root, "package.json"), "utf8"));
  const deps = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"].flatMap((key) => Object.keys(pkg[key] || {}));
  deps.length ? fail(`Unexpected npm dependencies: ${deps.join(", ")}`) : pass("No npm dependencies");
}

async function tokenCheck() {
  const css = (await readFile(join(root, "prototype/css/tokens.css"), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const expected = [...new Set([...css.matchAll(/(--[\w-]+)\s*:/g)].map((match) => match[1]))].sort();
  const manifest = await readFile(join(root, "prototype/js/token-names.js"), "utf8");
  const names = runInNewContext(`${manifest}\nwindow.Schematic.tokenNames()`, { window: {} }, { timeout: 1000 });
  if (expected.length && JSON.stringify(Array.from(names)) === JSON.stringify(expected)) pass(`Token manifest matches all ${expected.length} source tokens`);
  else fail("Token manifest differs from tokens.css; run npm run build and review the generated manifest");
}

async function docsCheck() {
  const quickstarts = ["QUICKSTART.md", "docs/QUICKSTART.md"];
  const docs = ["README.md"];
  for (const file of quickstarts) if (await exists(join(root, file))) docs.push(file);
  if (docs.length === 1) missing("QUICKSTART.md is not available yet; quickstart link validation deferred", true);
  if (!await exists(join(root, "docs/watch/index.html"))) missing("docs/watch/index.html is not available yet; walkthrough player validation deferred", true);
  for (const doc of docs) {
    const file = join(root, doc);
    if (!await exists(file)) { missing(`${doc} is missing; documentation link validation deferred`); continue; }
    const text = (await readFile(file, "utf8")).replace(/^\s*(```|~~~)[^\n]*\n[\s\S]*?^\s*\1\s*$/gm, "").replace(/`[^`\n]*`/g, "");
    const before = failed + deferred;
    // Match destinations rather than labels, including linked poster images.
    const targets = [...text.matchAll(/\]\(\s*(<[^>]+>|[^\s)]+)(?:\s+["'][^\n]*?["'])?\s*\)/g)].map((m) => m[1]);
    targets.push(...[...text.matchAll(/^\s*\[[^\]]+\]:\s*(<[^>]+>|[^\s]+)/gm)].map((m) => m[1]));
    for (const target of targets) await localTarget(file, target.replace(/^<|>$/g, ""));
    for (const match of text.matchAll(/\b(?:src|href|poster)=["']([^"']+)["']/g)) await localTarget(file, match[1]);
    if (failed + deferred === before) pass(`${doc} local link targets exist`);
  }
  try {
    for (const file of (await walk(join(root, "docs"))).filter((file) => extname(file) === ".html")) {
      const text = await readFile(file, "utf8"), before = failed + deferred;
      await assets(file, text); inlineSyntax(file, text);
      if (failed + deferred === before) pass(`${label(file)} scripts and local media references are valid`);
    }
  } catch (error) { if (error.code !== "ENOENT") throw error; }
}

async function buildCheck() {
  const temp = await mkdtemp(join(tmpdir(), "schematic-check-"));
  try {
    await cp(join(root, "prototype"), join(temp, "prototype"), { recursive: true });
    await cp(join(root, "tools"), join(temp, "tools"), { recursive: true });
    const build = () => {
      const result = spawnSync(process.execPath, [join(temp, "tools/bundle.mjs")], { cwd: temp, encoding: "utf8", timeout: 30000 });
      if (result.status !== 0) throw new Error(result.error?.message || result.stderr || result.stdout);
    };
    try { build(); }
    catch (error) {
      if (args.has("--defer-missing") && /ENOENT/.test(error.message)) defer(`Bundle checks await referenced assets: ${error.message.trim()}`);
      else fail(`Isolated build failed: ${error.message.trim()}`);
      return;
    }
    const output = join(temp, "dist"), generated = await walk(output);
    for (const name of ["schematic.html", "schematic-mono.html"]) {
      if (!await exists(join(output, name))) fail(`Build did not produce dist/${name}`);
    }
    const first = new Map();
    for (const file of generated) first.set(relative(output, file), await readFile(file));
    build();
    const second = await walk(output);
    let deterministic = first.size === second.length, consistent = true;
    for (const file of second) {
      const name = relative(output, file), bytes = await readFile(file);
      if (!first.get(name)?.equals(bytes)) deterministic = false;
      const committed = join(root, "dist", name);
      const present = await exists(committed);
      // The host-specific embed preview is intentionally ignored by Git. Its
      // build still participates in determinism and offline/syntax checks.
      if (name === "preview.html") console.log("INFO dist/preview.html is a local embed preview; public consistency checks exclude it");
      else if (!present) { missing(`dist/${name} is missing; run npm run build`); consistent = false; }
      else if (!(await readFile(committed)).equals(bytes)) { fail(`dist/${name} is stale; run npm run build and review the output`); consistent = false; }
      if (extname(file) === ".html") {
        const text = bytes.toString("utf8");
        inlineSyntax(committed, text);
        await assets(committed, text, true);
      }
    }
    deterministic ? pass("Two isolated builds are byte-for-byte deterministic") : fail("Isolated builds produced different outputs");
    if (consistent) pass("Generated dist matches source; the working tree was not modified");
  } finally { await rm(temp, { recursive: true, force: true, maxRetries: 3, retryDelay: 150 }); }
}

for (const [name, check] of [["Source", sourceChecks], ["Tokens", tokenCheck], ["Documentation", docsCheck], ["Build", buildCheck]]) {
  try { await check(); } catch (error) { fail(`${name} check: ${error.message}`); }
}
console.log(`${passed} passed, ${failed} failed, ${deferred} deferred${deferred ? "; deferred checks are not release validation" : ""}`);
process.exitCode = failed ? 1 : 0;
