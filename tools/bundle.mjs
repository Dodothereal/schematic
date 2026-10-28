#!/usr/bin/env node
// Bundle the reference into an offline HTML file. No dependencies, no server.
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const input = join(workspace, "prototype/index.html");
const tokenCSS = (await readFile(join(workspace, "prototype/css/tokens.css"), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
const tokenNames = [...new Set([...tokenCSS.matchAll(/(--[\w-]+)\s*:/g)].map((match) => match[1]))].sort();
await writeFile(join(workspace, "prototype/js/token-names.js"), `// Generated from css/tokens.css by tools/bundle.mjs.\n(() => { const S = window.Schematic = window.Schematic || {}; const names = Object.freeze(${JSON.stringify(tokenNames)}); S.tokenNames = () => [...names]; })();\n`);
let html = await readFile(input, "utf8");
for (const match of [...html.matchAll(/<link rel="stylesheet" href="([^"]+)">/g)]) {
  const css = await readFile(join(dirname(input), match[1]), "utf8");
  html = html.replace(match[0], () => `<style data-source="${match[1]}">\n${css}\n</style>`);
}
for (const match of [...html.matchAll(/<script defer src="([^"]+)"><\/script>/g)]) {
  const js = await readFile(join(dirname(input), match[1]), "utf8");
  html = html.replace(match[0], () => `<script data-source="${match[1]}">\n${js.replace(/<\/script/gi, "<\\/script")}\n</script>`);
}
const dest = join(workspace, "dist/schematic.html");
await mkdir(dirname(dest), { recursive: true });
await writeFile(dest, html);
await writeFile(join(workspace, "dist/schematic-mono.html"), html.replace('data-palette="color"', 'data-palette="mono"'));
// The embedded artifact uses a bounded scroll region and the host's canvas.
const embedSource = html.replace(/(<style\b[^>]*>)([\s\S]*?)(<\/style>)/gi, (_, open, css, close) => open + css.replace(/\/\*[\s\S]*?\*\//g, "") + close).replace(/^[\t ]+/gm, "");
const embed = embedSource.replaceAll("100dvh", "880px").replaceAll("min(16vh, 140px)", "80px").replace("height: 100%;", "height: auto;").replace("min-height: 100%;", "min-height: 0;").replace("background: var(--bg);", "background: transparent;")
  .replace("</head>", `<style>html,body{background:transparent!important}.app{height:880px}.viewport{scrollbar-gutter:auto}:root{--background:var(--bg);--foreground:var(--text);--muted:var(--text-3);--card:var(--surface);--secondary:var(--surface-2);--primary:var(--accent);--radius:var(--r-lg)}.page-wrap{padding-top:var(--s-4)}.reveal> *{animation:none}</style></head>`);
await writeFile(join(workspace, "dist/preview.html"), embed);
console.log(`Bundled ${Buffer.byteLength(html).toLocaleString()} bytes -> ${dest}`);
