import { browser, delay } from "./browser.mjs";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";

const b = await browser({ width: 1440, height: 1000 }), checks = [];
const ev = (code) => b.evaluate(code);
const check = (name, pass, detail) => {
  checks.push({ name, pass: !!pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${!pass ? ` ${JSON.stringify(detail)}` : ""}`);
};
const click = (selector) => ev(`document.querySelector(${JSON.stringify(selector)}).click();void 0`);
const query = (value) => ev(`(()=>{const input=document.querySelector('[data-catalog-search]');input.value=${JSON.stringify(value)};input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
const indicator = () => ev(`(()=>{const e=document.querySelector('.ov-filter-indicator'),r=e.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height,animations:e.getAnimations().length};})()`);
const close = (a, b, tolerance = 0.75) => Math.abs(a - b) <= tolerance;
const settledIndicator = () => ev(`(()=>{const a=document.querySelector('.ov-filter-indicator').getBoundingClientRect(),b=document.querySelector('[data-catalog-filter][aria-pressed="true"]').getBoundingClientRect();return Math.max(Math.abs(a.x-b.x),Math.abs(a.y-b.y),Math.abs(a.width-b.width),Math.abs(a.height-b.height))<1;})()`);
const visible = () => ev(`Array.from(document.querySelectorAll('[data-gallery-card]')).filter(e=>!e.hidden).map(e=>e.dataset.galleryCard)`);
try {
  await mkdir("scratch/qa", { recursive: true });
  await b.open(pathToFileURL(resolve("dist/schematic.html")).href);
  await delay(100);
  const opening = await ev(`Array.from(document.querySelectorAll('.ov-title-word'),e=>({opacity:Number(getComputedStyle(e).opacity),y:new DOMMatrixReadOnly(getComputedStyle(e).transform).m42,animations:e.getAnimations().length}))`);
  check("Overview heading reveals its words in staggered intermediate frames", opening.length === 2 && opening.every(e => e.animations === 1) && opening[0].opacity > 0 && opening[0].opacity < 1 && opening[1].y > opening[0].y, opening);
  await delay(800);
  check("Header and toolbar entrance finishes without persistent animations", await ev(`Array.from(document.querySelectorAll('.ov-title-word,.ov-search,.ov-filters'),e=>e.getAnimations().length).every(n=>n===0)`));

  const field = await ev(`document.querySelector('.ov-search').getBoundingClientRect().toJSON()`);
  await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: field.x + 8, y: field.y + field.height / 2 });
  await ev(`document.querySelector('[data-catalog-search]').focus()`);
  await delay(110);
  const focusFrame = await ev(`new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.ov-search > .ic .a')).transform).m12`);
  await delay(500);
  check("Search focus smoothly settles the magnifier into a stable pose", focusFrame < 0 && focusFrame > -0.174 && await ev(`Math.abs(new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.ov-search > .ic .a')).transform).m12+0.173648)<0.001`), focusFrame);
  for (const x of [field.x + 26, field.x + 120, field.right - 24, field.x + 65, field.x - 10, field.x + 50]) {
    await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y: field.y + field.height / 2 });
    await delay(35);
  }
  check("Crossing and re-entering the search field never replays the scanning icon", await ev(`(()=>{const svg=document.querySelector('.ov-search > .ic');return !svg.hasAttribute('data-anim')&&!svg.classList.contains('is-anim')&&svg.getAnimations({subtree:true}).length===0;})()`));

  const before = await ev(`({thumb:document.querySelector('.ov-filter-indicator').getBoundingClientRect().toJSON(),height:document.querySelector('.ov-results').getBoundingClientRect().height,card:document.querySelector('[data-gallery-card="data-0"]').getBoundingClientRect().toJSON()})`);
  const firstFrame = await ev(`(()=>{document.querySelector('[data-catalog-filter="data"]').click();return {thumb:document.querySelector('.ov-filter-indicator').getBoundingClientRect().toJSON(),height:document.querySelector('.ov-results').getBoundingClientRect().height,card:document.querySelector('[data-gallery-card="data-0"]').getBoundingClientRect().toJSON(),ghosts:document.querySelector('.ov-result-exits').children.length};})()`);
  check("Category changes begin at the displayed thumb, card and results height", close(before.thumb.x, firstFrame.thumb.x) && close(before.card.x, firstFrame.card.x) && close(before.card.y, firstFrame.card.y) && close(before.height, firstFrame.height), { before, firstFrame });
  check("Outgoing cards have a short, inert exit without duplicate application hooks", firstFrame.ghosts > 0 && await ev(`(()=>{const layer=document.querySelector('.ov-result-exits');return layer.inert&&layer.getAttribute('aria-hidden')==='true'&&!layer.querySelector('[data-gallery-card],[data-catalog-id],[data-gallery-kind]')&&Array.from(layer.children).every(e=>e.inert&&!e.hidden);})()`));
  await delay(170);
  const moving = await indicator(), destination = await ev(`document.querySelector('[data-catalog-filter="data"]').getBoundingClientRect().toJSON()`);
  check("The category indicator visibly travels between choices", moving.x > before.thumb.x + 5 && moving.x < destination.x - 0.5 && moving.animations === 1, { moving, destination });
  check("New results enter progressively after outgoing cards clear", await ev(`(()=>{const e=document.querySelector('[data-gallery-card="data-1"]'),opacity=Number(getComputedStyle(e).opacity);return opacity>0&&opacity<1&&document.querySelector('.ov-result-exits').children.length===0;})()`));

  const reverse = await ev(`(()=>{const e=document.querySelector('.ov-filter-indicator'),a=e.getBoundingClientRect();document.querySelector('[data-catalog-filter="controls"]').click();const c=e.getBoundingClientRect();return {dx:c.x-a.x,dy:c.y-a.y,dw:c.width-a.width};})()`);
  check("Reversing a category transition has no first-frame geometry jump", Math.max(...Object.values(reverse).map(Math.abs)) < 0.75, reverse);
  await delay(700);
  check("Reversed categories settle on one selected button and exact indicator bounds", await settledIndicator() && await ev(`document.querySelectorAll('[data-catalog-filter][aria-pressed="true"]').length===1&&document.querySelector('[data-catalog-filter="controls"]').getAttribute('aria-pressed')==='true'`));
  let allAnimated = true;
  for (const category of ["examples", "motion", "data", "feedback", "foundations", "all"]) {
    await click(`[data-catalog-filter="${category}"]`);
    allAnimated &&= (await indicator()).animations === 1;
    await delay(570);
    allAnimated &&= await settledIndicator();
  }
  check("Apps, Controls, Motion, Data, Feedback, Foundations and Everything share the animated selection", allAnimated);

  await ev(`document.querySelector('[data-catalog-search]').focus()`);
  await query("accordion");
  await delay(140);
  const typing = await ev(`(()=>{const line=document.querySelector('.ov-search-activity'),result=document.querySelector('[data-gallery-card]:not([hidden])'),matrix=new DOMMatrixReadOnly(getComputedStyle(result).transform);return {line:Number(getComputedStyle(line).opacity),scale:new DOMMatrixReadOnly(getComputedStyle(line).transform).m11,resultMotion:Math.abs(matrix.m41)+Math.abs(matrix.m42),focus:document.activeElement.matches('[data-catalog-search]'),reset:!document.querySelector('[data-catalog-reset-query]').hidden,count:document.querySelector('[data-catalog-count]').textContent};})()`);
  check("Typing animates search feedback and matching cards while preserving input focus", typing.line > 0 && typing.scale > 0 && typing.scale < 1 && typing.resultMotion > 1 && typing.focus && typing.reset, typing);
  await delay(650);
  check("The search activity line finishes instead of looping", await ev(`getComputedStyle(document.querySelector('.ov-search-activity')).opacity==='0'&&document.querySelector('.ov-search-activity').getAnimations().length===0`));
  for (const value of ["button", "bu", "counter", "spring", "zz-no-component-921"]) { await query(value); await delay(40); }
  await delay(150);
  check("Rapid search ends at the current empty state with animated feedback", (await visible()).length === 0 && await ev(`!document.querySelector('[data-catalog-empty]').hidden&&Number(getComputedStyle(document.querySelector('[data-catalog-empty]')).opacity)>0&&document.querySelector('[data-catalog-count]').textContent==='0 examples'`));
  await click("[data-catalog-reset-query]");
  await delay(650);
  check("Clear search restores results, its shortcut and keyboard focus", (await visible()).length === 12 && await ev(`document.activeElement.matches('[data-catalog-search]')&&document.activeElement.value===''&&document.querySelector('[data-catalog-reset-query]').hidden&&!document.querySelector('.ov-search').hasAttribute('data-has-query')`));
  await click('[data-catalog-filter="data"]'); await delay(620); await query("revenue"); await delay(620); await click("[data-catalog-reset-query]");
  check("Clearing the query preserves the selected category", await ev(`document.querySelector('[data-catalog-filter="data"]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-catalog-search]').value===''`) && (await visible()).length === 5);
  await click("[data-catalog-clear]"); await delay(650);
  const original = await visible();
  await click("[data-catalog-more]");
  check("Show more preserves existing controls and animates the added results", (await visible()).length > original.length && await ev(`(()=>{const cards=Array.from(document.querySelectorAll('[data-gallery-card]:not([hidden])'));return cards.slice(0,12).every(e=>!e.getAnimations().length)&&cards.slice(12).some(e=>e.getAnimations().length===1);})()`));
  await delay(700);

  await query("accordion"); await delay(60);
  await ev(`document.documentElement.dataset.motion='reduced';void 0`); await delay(40);
  check("Changing to Reduced settles ongoing results, exits and search motion", await ev(`document.querySelector('.ov-result-exits').children.length===0&&Array.from(document.querySelectorAll('.ov-title-word,.ov-filter-indicator,.ov-results,.ov-search-activity,[data-gallery-card]')).every(e=>e.getAnimations().length===0)`));
  await click('[data-catalog-filter="motion"]');
  check("Reduced motion applies category geometry immediately", await settledIndicator() && (await indicator()).animations === 0);
  await ev(`delete document.documentElement.dataset.motion;void 0`);
  await b.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await click('[data-catalog-filter="all"]');
  check("System reduced motion also skips the category animation", await settledIndicator() && (await indicator()).animations === 0);
  await ev(`document.documentElement.dataset.motion='full';void 0`);
  await click('[data-catalog-filter="controls"]');
  check("An explicit Full preference restores motion over system Reduced", (await indicator()).animations === 1);
  await b.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "no-preference" }] });
  await click("[data-catalog-clear]"); await delay(700);

  await ev(`document.documentElement.dataset.theme='dark';document.documentElement.dataset.palette='mono';void 0`);
  for (const width of [390, 320]) {
    await b.size(width, width === 320 ? 568 : 844); await delay(100);
    await click('[data-catalog-filter="all"]'); await delay(550);
    const start = await indicator();
    await click('[data-catalog-filter="foundations"]'); await delay(100);
    const frame = await indicator();
    await delay(500);
    check(`${width}px: category indicator travels across wrapped rows and settles without overflow`, frame.y > start.y + 1 && frame.animations === 1 && await settledIndicator() && await ev(`(()=>{const p=document.querySelector('.pg-overview'),r=p.getBoundingClientRect();return p.scrollWidth<=r.width+1&&document.documentElement.scrollWidth<=innerWidth;})()`), { start, frame });
    await b.screenshot(`scratch/qa/overview-motion-${width}.png`);
  }
  await b.size(1440, 1000); await delay(100); await click('[data-catalog-filter="all"]'); await delay(650);
  await query("accordion");
  await ev(`Object.defineProperty(document,'hidden',{value:true,configurable:true});document.dispatchEvent(new Event('visibilitychange'));void 0`);
  check("Document visibility cleanup cancels ongoing Overview motion", await ev(`!document.querySelector('.ov-result-exits').children.length&&Array.from(document.querySelectorAll('.ov-filter-indicator,.ov-results,.ov-search-activity,[data-gallery-card]')).every(e=>e.getAnimations().length===0)`));
  await ev(`delete document.hidden;document.dispatchEvent(new Event('visibilitychange'));void 0`);
  await click("[data-catalog-reset-query]"); await delay(30);
  await ev(`Schematic.navigate('icons');void 0`); await delay(600);
  check("Leaving Overview removes exit paint and cancels its pending transitions", await ev(`Schematic._shell.state.current==='icons'&&!document.querySelector('.ov-result-exits').children.length&&Array.from(document.querySelectorAll('.ov-title-word,.ov-filter-indicator,.ov-results,.ov-search-activity,[data-gallery-card]')).every(e=>e.getAnimations().length===0)`));
  await ev(`Schematic.navigate('overview');void 0`);
  let returning = false;
  for (let i = 0; i < 20 && !returning; i++) {
    await delay(25);
    returning = await ev(`Schematic._shell.state.current==='overview'&&Array.from(document.querySelectorAll('.ov-title-word'),e=>e.getAnimations().length).every(n=>n===1)`);
  }
  check("Returning to Overview replays its heading without hidden-page resize cancellation", returning);
  await delay(850);
  check("Returning preserves the catalog state and restores the indicator", (await visible()).length === 12 && await settledIndicator());
  check("Overview motion produces no browser errors or warnings", b.failures.length === 0, b.failures);
} finally {
  await writeFile("scratch/qa/overview-motion.json", JSON.stringify({ checks, failures: b.failures }, null, 2));
  await b.close();
}
const failed = checks.filter(e => !e.pass);
console.log(`${checks.length - failed.length} passed, ${failed.length} failed`);
if (failed.length) process.exitCode = 1;
