#!/usr/bin/env node
import { browser, delay } from "./browser.mjs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { mkdir, writeFile } from "node:fs/promises";
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), ".."), output = join(workspace, "scratch/qa/refinements");
await mkdir(output, { recursive: true });
const b = await browser(), checks = [];
const check = (name, pass, detail) => { checks.push({ name, pass: !!pass, detail }); console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail === undefined ? "" : `: ${JSON.stringify(detail)}`}`); };
const evaluate = (js) => b.evaluate(js);
async function hover(selector) {
  await evaluate(`document.querySelector(${JSON.stringify(selector)}).scrollIntoView({block:'center'})`); await delay(70);
  const rect = await evaluate(`(()=>{const r=document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();return{x:r.left+r.width/2,y:r.top+r.height/2}})()`);
  await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...rect }); await delay(250);
}
async function key(key, code, n) { await b.send("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: n }); await b.send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: n }); }
try {
  await b.open(pathToFileURL(join(workspace, "dist/schematic.html")).href + "#/data");
  await delay(180);
  check("Fresh palette is forest on warm paper", await evaluate("document.documentElement.dataset.palette==='color'&&parseFloat(Schematic.token('--accent-h'))===168&&parseFloat(Schematic.token('--neutral-c'))>0"));
  check("Overview headline has no gradient", await evaluate("(()=>{Schematic.prepare('overview');return getComputedStyle(document.querySelector('.ov-wordmark')).backgroundImage==='none'})()"));
  const waiting = await evaluate("document.querySelector('[data-current-line]').style.strokeDashoffset");
  check("Below-fold line waits to draw", +waiting === 1);
  await evaluate("document.querySelector('[data-main-chart]').closest('.demo').scrollIntoView({block:'start'});window.qaPath=document.querySelector('[data-current-line]').getAttribute('d');window.qaGrid=document.querySelector('[data-chart-grid]').innerHTML");
  await delay(180); const early = await evaluate("+document.querySelector('[data-current-line]').style.strokeDashoffset");
  await b.screenshot(join(output, "line-early.png"));
  await delay(340); const middle = await evaluate("+document.querySelector('[data-current-line]').style.strokeDashoffset");
  await b.screenshot(join(output, "line-middle.png"));
  await delay(700); const end = await evaluate("+document.querySelector('[data-current-line]').style.strokeDashoffset");
  await b.screenshot(join(output, "line-finished.png"));
  check("Line advances as a drawn trace", early > middle && middle > end && end === 0, { early, middle, end });
  check("Draw does not rebuild the path or axes every frame", await evaluate("qaPath===document.querySelector('[data-current-line]').getAttribute('d')&&qaGrid===document.querySelector('[data-chart-grid]').innerHTML"));
  check("Range changes restart a fresh trace", await evaluate(`(()=>{document.querySelector('[data-control=period] [data-value="7"]').click();return +document.querySelector('[data-current-line]').style.strokeDashoffset===1&&qaPath!==document.querySelector('[data-current-line]').getAttribute('d');})()`));
  await delay(120); await evaluate(`document.querySelector('[data-control=period] [data-value="90"]').click();document.querySelector('[data-control=period] [data-value="30"]').click()`); await delay(1250);
  check("Rapid range changes settle on the final data", await evaluate("document.querySelector('#dt-day').max==='29'&&+document.querySelector('[data-current-line]').style.strokeDashoffset===0"));
  await hover('[data-bar-group="2"]');
  check("Bar hover links exact values and series emphasis", await evaluate("document.querySelector('[data-bar-group=\"2\"]').classList.contains('is-highlight')&&document.querySelector('[data-bar-insight]').textContent.includes('orders')"));
  await b.screenshot(join(output, "bars-hover.png"));
  await evaluate(`document.querySelector('[data-bar-group="2"]').focus()`); await key("Enter", "Enter", 13);
  check("Bar inspection can be pinned with a keyboard", await evaluate(`document.querySelector('[data-bar-group="2"]').getAttribute('aria-pressed')==='true'`));
  await hover('[data-rank-inspect="0"]');
  check("Channel hover reveals its contribution", await evaluate(`document.querySelector('[data-rank-inspect="0"]').classList.contains('is-highlight')&&document.querySelector('[data-rank-summary]').textContent.includes('Organic search')`));
  await hover('[data-device="1"]');
  check("Legend hover links its donut arc and centre label", await evaluate(`document.querySelector('[data-donut="1"]').classList.contains('is-highlight')&&document.querySelector('[data-donut-label]').textContent.includes('mobile')`));
  await b.screenshot(join(output, "donut-hover.png"));
  await hover('[data-goal="0"]');
  check("Target hover reveals remaining progress", await evaluate(`document.querySelector('[data-goal="0"]').classList.contains('is-highlight')&&document.querySelector('[data-goal-insight]').textContent.includes('24% remaining')`));
  await b.screenshot(join(output, "targets-hover.png"));
  await evaluate("Schematic.setMotion('reduced');document.querySelector('.pg-data [data-search]').value='Orbit Supply';document.querySelector('.pg-data [data-search]').dispatchEvent(new Event('input',{bubbles:true}))");
  const row = '[data-account="SC-1026"]';
  await evaluate(`document.querySelector(${JSON.stringify(row)}).scrollIntoView({block:'center'})`); await delay(70);
  const blank = await evaluate(`(()=>{const r=document.querySelector(${JSON.stringify(row)}+' td.dt-money').getBoundingClientRect();return{x:r.right-8,y:r.top+r.height/2}})()`);
  await b.send("Input.dispatchMouseEvent", { type: "mousePressed", button: "left", clickCount: 1, ...blank }); await b.send("Input.dispatchMouseEvent", { type: "mouseReleased", button: "left", clickCount: 1, ...blank });
  check("Orbit Supply whitespace selects the whole row", await evaluate(`document.querySelector('${row}').getAttribute('aria-selected')==='true'&&document.querySelector('[data-select="SC-1026"]').checked`));
  await evaluate(`document.querySelector('[data-select="SC-1026"]').click()`);
  check("Direct checkbox click toggles exactly once", await evaluate(`!document.querySelector('[data-select="SC-1026"]').checked&&document.querySelector('${row}').getAttribute('aria-selected')==='false'`));
  await evaluate(`document.querySelector('${row}').focus()`); await key(" ", "Space", 32);
  check("Row selection also works with Space", await evaluate(`document.querySelector('[data-select="SC-1026"]').checked`));
  await evaluate("Schematic.setMotion('full');document.querySelector('[data-select=\"SC-1026\"]').click()"); await delay(400); await evaluate("document.querySelector('[data-select=\"SC-1026\"]').click()"); await delay(70);
  const tick = await evaluate("getComputedStyle(document.querySelector('[data-select=\"SC-1026\"]').nextElementSibling.querySelector('path')).strokeDashoffset");
  check("Checkbox checkmark draws through an intermediate frame", parseFloat(tick) > 0 && parseFloat(tick) < 1, tick);
  await delay(400); await b.screenshot(join(output, "row-selected.png"));
  await evaluate("Schematic.setPalette('mono');Schematic.setMotion('reduced')"); await delay(100);
  check("Mono keeps the interface neutral and charts coloured", await evaluate("+Schematic.token('--neutral-c')===0&&+Schematic.token('--accent-c')===0&&!Schematic.token('--chart-1').includes('0 0')"));
  await evaluate("document.querySelector('#tune-btn').click()"); await delay(60);
  check("Mono selector is accessible and hides hue controls", await evaluate("document.querySelector('#palette-seg [data-value=mono]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-accent-options]').inert"));
  await evaluate("document.querySelector('#tune-btn').click();Schematic.navigate('overview')"); await delay(100); await b.screenshot(join(output, "mono-overview.png"));
  await b.size(390, 844); await evaluate("Schematic.navigate('data')"); await delay(80); await b.screenshot(join(output, "mono-mobile.png"));
  check("Refined mobile data layout stays inside viewport", await evaluate("document.documentElement.scrollWidth<=innerWidth"));
  await evaluate("Schematic.setPalette('color');Schematic.setMotion('full');Schematic.setTheme('light')"); await delay(100);
  await b.size(1440, 1000);
  for (const theme of ["dark", "light"]) {
    await evaluate(`window.qaTheme=Schematic.setTheme('${theme}',document.querySelector('#theme-btn'));void 0`);
    await delay(130);
    const pseudo = await evaluate("(()=>{const a=document.getAnimations().find(a=>a.animationName==='vt-reveal');if(!a)return null;const duration=a.effect.getTiming().duration;const all=document.getAnimations();all.forEach(a=>a.pause());a.currentTime=duration*.5;return {duration,oldOpacity:getComputedStyle(document.documentElement,'::view-transition-old(root)').opacity,newClip:getComputedStyle(document.documentElement,'::view-transition-new(root)').clipPath,paint:getComputedStyle(document.querySelector('.app'),'::before').backgroundColor};})()");
    check(`Theme ${theme} covers the background at mid-reveal`, !!pseudo && pseudo.oldOpacity === "1" && pseudo.newClip.includes("circle"), pseudo);
    await b.screenshot(join(output, `theme-${theme}-middle.png`));
    await evaluate("document.getAnimations().forEach(a=>{if(a.playState==='paused')a.play()})"); await delay(800);
  }
  await evaluate("Promise.all([Schematic.setTheme('dark'),Schematic.setTheme('light'),Schematic.setTheme('dark')])");
  check("Rapid theme toggles settle cleanly", await evaluate("document.documentElement.dataset.theme==='dark'&&!document.documentElement.classList.contains('vt-theme')"));
  check("Refinement interactions produce no runtime errors", b.failures.length === 0, b.failures.length ? b.failures : undefined);
} catch (error) { check("Refinement run completed", false, error.stack); }
finally { await b.close(); }
await writeFile(join(output, "results.json"), JSON.stringify({ date: new Date().toISOString(), checks }, null, 2));
const failed = checks.filter(c=>!c.pass).length; console.log(`${checks.length-failed} passed, ${failed} failed`); process.exitCode = failed ? 1 : 0;
