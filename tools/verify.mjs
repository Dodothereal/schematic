#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { browser, delay } from "./browser.mjs";
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = join(workspace, "scratch/qa"); await mkdir(output, { recursive: true });
const checks = [], screenshots = [];
function check(name, pass, detail = "") { checks.push({ name, pass: !!pass, detail }); console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? `: ${typeof detail === "string" ? detail : JSON.stringify(detail)}` : ""}`); }
const source = await readFile(join(workspace, "prototype/index.html"), "utf8");
for (const [, asset] of source.matchAll(/(?:href|src)="((?:css|js|pages)\/[^\"]+)"/g)) { try { await readFile(join(workspace, "prototype", asset)); } catch { check("Referenced asset exists", false, asset); } }
const b = await browser();
try {
  await b.open(pathToFileURL(join(workspace, "prototype/index.html")).href);
  const pages = await b.evaluate("Schematic.pages.map(p => p.id)"); check("Eleven complete routes", pages.length === 11 && ['overview','foundations','motion','icons','components','primitives','navigation','forms','data','overlays','workspace'].every(id=>pages.includes(id)), pages.join(", "));
  const css = (await readFile(join(workspace, "prototype/css/tokens.css"), "utf8")).replace(/\/\*[\s\S]*?\*\//g, "");
  const names = [...new Set([...css.matchAll(/(--[\w-]+)\s*:/g)].map(m=>m[1]))].sort();
  check("Exported token manifest matches the source stylesheet", await b.evaluate(`JSON.stringify(Schematic.tokenNames())===${JSON.stringify(JSON.stringify(names))}`), `${names.length} tokens`);
  for (const page of pages) {
    await b.evaluate(`Schematic.navigate(${JSON.stringify(page)})`); await delay(1000);
    const info = await b.evaluate(`(() => { const el = document.querySelector('.page:not([hidden])'); return {id:el.dataset.page,h1:el.querySelector('h1')?.textContent, sections:el.querySelectorAll('section[data-toc]').length,overflow:document.documentElement.scrollWidth>innerWidth,controls:[...el.querySelectorAll('button')].filter(e=>e.offsetWidth&&!e.textContent.trim()&&!e.getAttribute('aria-label')&&!e.title&&!e.dataset.tip).map(e=>e.outerHTML.slice(0,200))}; })()`);
    check(`Route ${page} renders`, info.id === page && !!info.h1, `${info.sections} sections`);
    check(`Route ${page} control names`, !info.controls.length, info.controls.length ? info.controls : "");
    if (["overview", "components", "data", "forms", "overlays", "motion"].includes(page)) { const path = join(output, `${page}-light.png`); await b.screenshot(path); screenshots.push(path); }
  }
  const duplicateIds = await b.evaluate("[...document.querySelectorAll('[id]')].map(e=>e.id).filter((id,i,all)=>all.indexOf(id)!==i)"); check("No duplicate DOM IDs", !duplicateIds.length, duplicateIds.length ? duplicateIds : "");
  check("Replay controls retain their overlay positioning", await b.evaluate("[...document.querySelectorAll('.demo-replay')].every(el=>getComputedStyle(el).position==='absolute')"));
  await b.evaluate("Promise.all(['icons','components','forms','motion'].map(id=>Schematic.navigate(id)))"); await delay(600);
  check("Interrupted View Transitions resolve latest route", await b.evaluate("Schematic._shell.state.current==='motion'&&document.querySelectorAll('.page:not([hidden])').length===1"));
  await b.evaluate("Schematic.navigate('components')"); await b.evaluate("Schematic.navigate('forms')");
  await b.evaluate("history.back()"); await delay(700);
  check("Browser Back restores the prior route", await b.evaluate("Schematic._shell.state.current==='components'&&location.hash==='#/components'"));
  await b.evaluate("history.forward()"); await delay(700);
  check("Browser Forward restores the next route", await b.evaluate("Schematic._shell.state.current==='forms'&&location.hash==='#/forms'"));
  await b.evaluate("document.startViewTransition=undefined;Schematic.navigate('icons')"); await delay(650);
  check("Navigation works without View Transitions", await b.evaluate("Schematic._shell.state.current==='icons'&&document.querySelector('.page:not([hidden])').dataset.page==='icons'"));
  await b.evaluate("delete document.startViewTransition");
  await b.evaluate("Schematic.setMotion('reduced')");
  await b.evaluate("Promise.all(['components','forms','motion','data','overview'].map(id=>Schematic.navigate(id)))"); await delay(100);
  check("Rapid navigation resolves latest intent", await b.evaluate("Schematic._shell.state.current === 'overview' && document.querySelectorAll('.page:not([hidden])').length === 1"));
  await b.evaluate("Schematic.navigate('forms')");
  const preserved = await b.evaluate("(() => { const input=document.querySelector('.pg-forms input[type=text]');if(!input)return null;input.value='Preserve this draft';input.dispatchEvent(new Event('input',{bubbles:true}));return input.id; })()");
  await b.evaluate("Schematic.navigate('icons')"); await b.evaluate("Schematic.navigate('forms')");
  check("Form state survives navigation", preserved && await b.evaluate(`document.getElementById(${JSON.stringify(preserved)})?.value === 'Preserve this draft'`));
  await b.evaluate("Schematic.navigate('icons');"); await delay(40);
  check("Icon search filters and clears", await b.evaluate("(() => {const q=document.querySelector('#icons-q');q.value='bell';q.dispatchEvent(new Event('input',{bubbles:true}));const n=document.querySelectorAll('.ic-tile:not([hidden])').length;q.value='';q.dispatchEvent(new Event('input',{bubbles:true}));return n===1&&document.querySelectorAll('.ic-tile:not([hidden])').length===Schematic.iconNames.length;})()"));
  check("Unknown route falls back", await b.evaluate("(() => {history.replaceState(null,'','#/missing');return Schematic._shell.routeFromHash()==='overview';})()"));
  await b.evaluate("Schematic.cmdk.open()"); await delay(50);
  check("Command palette opens with input focus", await b.evaluate("document.querySelector('.cmdk').open && document.activeElement.matches('.cmdk-input')"));
  await b.evaluate("(() => {const i=document.querySelector('.cmdk-input');i.value='motion';i.dispatchEvent(new Event('input',{bubbles:true}));})()");
  await b.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 }); await b.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 }); await delay(150);
  check("Command palette keyboard navigation", await b.evaluate("Schematic._shell.state.current==='motion'&&!document.querySelector('.cmdk').open"));
  await b.evaluate("Schematic.setTheme('dark');Schematic.setHue(150)");
  await b.size(390, 844);
  for (const page of pages) {
    await b.evaluate(`Schematic.navigate(${JSON.stringify(page)})`); await delay(80);
    const overflow = await b.evaluate("(() => {const r=document.querySelector('.page:not([hidden])');const contained=e=>{for(let p=e.parentElement;p&&p!==r;p=p.parentElement){if(['auto','scroll','hidden','clip'].includes(getComputedStyle(p).overflowX)&&p.getBoundingClientRect().right<=innerWidth+2)return true;}return false;}; return [...r.querySelectorAll('*')].filter(e=>e.getBoundingClientRect().width&&e.getBoundingClientRect().right>innerWidth+2&&!e.closest('svg')&&!e.closest('[role=listbox],.pop,dialog,pre')&&!e.matches('.roll-strip,.roll-strip *')&&!contained(e)).slice(0,5).map(e=>({tag:e.tagName,cls:e.className,right:Math.round(e.getBoundingClientRect().right)}));})()");
    check(`Mobile ${page} stays within viewport`, !overflow.length, overflow.length ? overflow : "");
    const path = join(output, `${page}-mobile-dark.png`); await b.screenshot(path); screenshots.push(path);
  }
  await b.evaluate("document.querySelector('#menu-btn').click()");
  check("Mobile navigation opens accessibly", await b.evaluate("document.querySelector('#menu-btn').getAttribute('aria-expanded')==='true'&&!document.querySelector('.sidebar').inert&&!!document.activeElement.closest('.sidebar')"));
  await b.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 }); await delay(50);
  check("Mobile Escape restores focus", await b.evaluate("!document.documentElement.hasAttribute('data-nav-open')&&document.activeElement.id==='menu-btn'&&document.querySelector('.sidebar').inert"));
  check("Reduced motion stops infinite CSS loops", await b.evaluate("document.getAnimations().filter(a=>a.effect?.getTiming().iterations===Infinity&&a.playState==='running').length===0"));
  const contrasts = await b.evaluate(`(() => {
    const r=document.documentElement; r.style.transition='none';
    const c=document.createElement('canvas').getContext('2d');
    const lum=(token)=>{c.clearRect(0,0,1,1);c.fillStyle=Schematic.token(token);c.fillRect(0,0,1,1);const a=[...c.getImageData(0,0,1,1).data].slice(0,3).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4});return a[0]*.2126+a[1]*.7152+a[2]*.0722;};
    const contrast=(a,b)=>{const x=lum(a),y=lum(b);return (Math.max(x,y)+.05)/(Math.min(x,y)+.05);};
    const results=[];
    for(const theme of ['light','dark']){r.dataset.theme=theme;for(const hue of [0,18,55,150,195,235,262,290,330]){r.style.setProperty('--accent-h',hue);results.push({theme,hue,primary:contrast('--accent-contrast','--accent'),hover:contrast('--accent-contrast','--accent-hover'),caption:contrast('--text-3','--surface-3'),danger:contrast('--danger-contrast','--danger')});}}
    return results;
  })()`);
  const lowContrast = contrasts.filter((v)=>[v.primary,v.hover,v.caption,v.danger].some((n)=>n<4.5));
  check("Primary labels, captions and danger labels meet 4.5:1 across 18 theme/hue combinations", !lowContrast.length, lowContrast.length ? lowContrast : "");
  check("No runtime errors or warnings", !b.failures.length, b.failures.length ? b.failures : "");
} catch (error) { check("Browser verification completed", false, error.stack); }
finally { await b.close(); }
const reducedBrowser = await browser({ reduced: true, width: 390, height: 844 });
try {
  await reducedBrowser.open(pathToFileURL(join(workspace, "dist/schematic.html")).href); await delay(400);
  check("Fresh standalone launch respects OS reduced motion", await reducedBrowser.evaluate("Schematic.reduced()&&document.getAnimations().filter(a=>a.effect?.getTiming().iterations===Infinity&&a.playState==='running').length===0"));
  check("Idle preparation builds all routes without mounting inactive pages", await reducedBrowser.evaluate("Schematic._shell.state.built.size===Schematic.pages.length&&Schematic._shell.state.mounted.size===1"));
  check("Fresh standalone launch has no runtime errors", !reducedBrowser.failures.length, reducedBrowser.failures.length ? reducedBrowser.failures : "");
} finally { await reducedBrowser.close(); }
const result = { date: new Date().toISOString(), passed: checks.filter(c=>c.pass).length, failed: checks.filter(c=>!c.pass).length, checks, screenshots };
await writeFile(join(output, "results.json"), JSON.stringify(result, null, 2));
console.log(`${result.passed} passed, ${result.failed} failed; report ${join(output, "results.json")}`);
process.exitCode = result.failed ? 1 : 0;
