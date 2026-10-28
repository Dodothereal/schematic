#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { browser, delay } from "./browser.mjs";

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const url = pathToFileURL(join(workspace, "dist/schematic.html")).href;
const checks = [], traces = {}, inventory = {};
const routes = ["overview", "foundations", "motion", "icons", "components", "forms", "data", "overlays", "primitives", "navigation"];
const check = (name, pass, detail) => {
  const result = { name, pass: !!pass, ...(detail === undefined ? {} : { detail }) };
  checks.push(result);
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${!pass && detail !== undefined ? `: ${JSON.stringify(detail)}` : ""}`);
};
const group = async (name, run) => {
  try { await run(); }
  catch (error) { check(`${name}: completed without an exception`, false, error.stack); }
};
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const near = (a, b, tolerance = 1) => Number.isFinite(a) && Number.isFinite(b) && Math.abs(a - b) <= tolerance;
const framesOf = (trace) => [trace.immediate, ...trace.frames];
const finalOf = (trace) => trace.frames.at(-1) || trace.immediate;
const varied = (values) => new Set(values.map(value => Math.round(value * 1000))).size >= 3;
const between = (values, start, end, tolerance = 0.005) => {
  const lo = Math.min(start, end), hi = Math.max(start, end);
  return values.filter(value => Number.isFinite(value) && value > lo + tolerance && value < hi - tolerance);
};

// Track chains rather than just outstanding IDs: a recursive timeout/rAF which
// survives leave() still belongs to the route that originally scheduled it.
function installWorkTracker() {
  let owner = null;
  const native = { raf: requestAnimationFrame, caf: cancelAnimationFrame, timeout: setTimeout, clear: clearTimeout, interval: setInterval, clearInterval };
  const pending = new Map(), calls = new Map();
  const route = () => owner || window.Schematic?._shell?.state.current || "boot";
  const count = (kind, page) => { const key = `${page}:${kind}`; calls.set(key, (calls.get(key) || 0) + 1); };
  const invoke = (callback, page, kind, args) => {
    const previous = owner; owner = page; count(kind, page);
    try { return callback(...args); } finally { owner = previous; }
  };
  window.requestAnimationFrame = callback => {
    const page = route();
    const id = native.raf.call(window, time => { pending.delete(`raf:${id}`); invoke(callback, page, "raf", [time]); });
    pending.set(`raf:${id}`, { kind: "raf", page }); return id;
  };
  window.cancelAnimationFrame = id => { pending.delete(`raf:${id}`); native.caf.call(window, id); };
  window.setTimeout = (callback, ms, ...args) => {
    if (typeof callback !== "function") return native.timeout.call(window, callback, ms, ...args);
    const page = route();
    const id = native.timeout.call(window, () => { pending.delete(`timeout:${id}`); invoke(callback, page, "timeout", args); }, ms);
    pending.set(`timeout:${id}`, { kind: "timeout", page }); return id;
  };
  window.clearTimeout = id => { pending.delete(`timeout:${id}`); native.clear.call(window, id); };
  window.setInterval = (callback, ms, ...args) => {
    if (typeof callback !== "function") return native.interval.call(window, callback, ms, ...args);
    const page = route();
    const id = native.interval.call(window, () => invoke(callback, page, "interval", args), ms);
    pending.set(`interval:${id}`, { kind: "interval", page }); return id;
  };
  window.clearInterval = id => { pending.delete(`interval:${id}`); native.clearInterval.call(window, id); };
  window.libraryWorkQA = page => ({
    pending: [...pending.values()].filter(item => item.page === page),
    callbacks: Object.fromEntries([...calls].filter(([key]) => key.startsWith(`${page}:`))),
  });
}

// The probe samples the real rendered frame. It does not finish animations,
// disable transitions, replace RAF or derive motion from implementation values.
function installProbe() {
  const rect = element => { const r = element?.getBoundingClientRect(); return r ? { x: r.x, y: r.y, width: r.width, height: r.height } : null; };
  const visual = (selector, pseudo) => {
    const element = typeof selector === "string" ? document.querySelector(selector) : selector;
    if (!element) throw new Error(`Missing component selector: ${selector}`);
    const style = getComputedStyle(element, pseudo);
    const transform = new DOMMatrixReadOnly(style.transform === "none" ? undefined : style.transform);
    return { rect: rect(element), opacity: Number(style.opacity), x: transform.m41, y: transform.m42,
      scaleX: transform.m11, scaleY: transform.m22, transform: style.transform,
      clip: style.clipPath, grid: style.gridTemplateRows, visibility: style.visibility,
      text: element.textContent.trim(), expanded: element.getAttribute("aria-expanded"),
      selected: element.getAttribute("aria-selected"), pressed: element.getAttribute("aria-pressed"),
      inert: element.inert, hidden: element.hidden, dash: Number(style.strokeDashoffset) };
  };
  const running = (root = document) => root.getAnimations({ subtree: true }).filter(animation => ["running", "pending"].includes(animation.playState));
  const raf = () => new Promise(resolve => requestAnimationFrame(resolve));
  const frames = async (n = 2) => { for (let i = 0; i < n; i++) await raf(); };
  const trace = async (action, sample, rootSelector, limit = 2200) => {
    const root = document.querySelector(rootSelector);
    if (!root) throw new Error(`Missing motion root: ${rootSelector}`);
    const snap = () => ({ t: performance.now(), ...sample(), running: running(root).filter(a => a.effect.getTiming().iterations !== Infinity).length });
    const before = snap(); action(); const immediate = snap(), samples = [];
    let idle = 0;
    do {
      await raf(); const frame = snap(); samples.push(frame);
      idle = frame.running === 0 ? idle + 1 : 0;
      if (idle >= 4 && samples.length >= 6) return { before, immediate, frames: samples };
    } while (performance.now() - immediate.t < limit);
    return { before, immediate, frames: samples, timedOut: true };
  };
  const until = async (predicate, limit = 1800) => {
    const start = performance.now();
    do { await raf(); if (predicate()) return; } while (performance.now() - start < limit);
    throw new Error("The requested intermediate library animation frame did not occur.");
  };
  window.libraryQA = { visual, rect, running, frames, trace, until };
}

let b;
const evaluate = expression => b.evaluate(expression);
const click = selector => evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing '+${JSON.stringify(selector)});e.click();})()`);
const input = (selector, value) => evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing '+${JSON.stringify(selector)});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`);
const frameWait = (n = 3) => evaluate(`libraryQA.frames(${n})`);
const go = async id => { await evaluate(`Schematic.navigate(${JSON.stringify(id)})`); await frameWait(); };
const key = async (key, code, keyCode, modifiers = 0) => {
  const params = { key, code, windowsVirtualKeyCode: keyCode, modifiers };
  await b.send("Input.dispatchKeyEvent", { type: "keyDown", ...params, ...(key === "Enter" ? { text: "\r", unmodifiedText: "\r" } : {}) });
  await b.send("Input.dispatchKeyEvent", { type: "keyUp", ...params });
};
const focus = selector => evaluate(`document.querySelector(${JSON.stringify(selector)}).focus()`);
const trace = (name, action, sample, root) => evaluate(`libraryQA.trace(()=>{${action}},()=>(${sample}),${JSON.stringify(root)})`).then(result => (traces[name] = result));

async function catalogChecks() {
  const registered = await evaluate("Schematic.pages.map(page=>page.id)");
  check("All ten library routes are registered once", registered.length === routes.length && equal([...registered].sort(), [...routes].sort()), registered);
  for (const id of routes.filter(id => id !== "overview")) {
    await go(id);
    inventory[id] = await evaluate(`(()=>{const page=document.querySelector('[data-page="${id}"]');return [...page.querySelectorAll('.demo')].map((demo,index)=>({page:${JSON.stringify(id)},index,title:demo.querySelector('.demo-title')?.textContent.trim(),desc:demo.querySelector('.demo-desc')?.textContent.trim()||''}));})()`);
    if (["primitives", "navigation"].includes(id)) check(`${id}: exactly eight actual specimens`, inventory[id].length === 8, inventory[id]);
  }
  await go("overview");
  const actual = Object.values(inventory).flat(), catalog = await evaluate("Schematic.catalog()"), links = await evaluate("[...document.querySelectorAll('.pg-overview [data-catalog-id]')].map(e=>({id:e.dataset.catalogId,page:e.getAttribute('href').slice(2),index:Number(e.dataset.specimen),title:e.querySelector('strong').textContent.trim()}))");
  const lookup = new Map(actual.map(entry => [`${entry.page}:${entry.index}`, entry]));
  check("Catalog covers every real non-overview demo exactly once", catalog.length === actual.length && new Set(catalog.map(e => `${e.page}:${e.index}`)).size === actual.length && catalog.every(e => lookup.has(`${e.page}:${e.index}`)), { actual: actual.length, catalog: catalog.length });
  check("Catalog names and descriptions match mounted specimens", catalog.every(e => { const card = lookup.get(`${e.page}:${e.index}`); return card && card.title === e.title && (!card.desc || card.desc === e.desc); }), catalog.filter(e => lookup.get(`${e.page}:${e.index}`)?.title !== e.title));
  check("Every catalog link targets its exact real demo", links.length === actual.length && new Set(links.map(e => `${e.page}:${e.index}`)).size === actual.length && links.every(e => lookup.get(`${e.page}:${e.index}`)?.title === e.title), { links: links.length, actual: actual.length });
  const ledger = await evaluate("[...document.querySelectorAll('.ov-ledger > span')].map(e=>e.textContent.trim())");
  check("Overview ledger reports actual specimen and collection counts", Number(ledger[0]?.match(/\d+/)?.[0]) === actual.length && Number(ledger[1]?.match(/\d+/)?.[0]) === routes.length, ledger);
  const visible = () => evaluate("[...document.querySelectorAll('[data-catalog-id]')].filter(e=>!e.hidden).map(e=>({page:e.getAttribute('href').slice(2),title:e.querySelector('strong').textContent.trim(),text:e.textContent.trim(),id:e.dataset.catalogId,index:Number(e.dataset.specimen)}))");
  const count = () => evaluate("Number(document.querySelector('[data-catalog-count]').textContent.match(/\\d+/)?.[0])");
  const initial = await visible();
  check("Initial catalog is a useful finite preview with Show more available", initial.length > 0 && initial.length < actual.length && await evaluate("!document.querySelector('[data-catalog-more]').hidden"), { visible: initial.length, total: actual.length });
  await click("[data-catalog-more]"); const expanded = await visible();
  check("Show more reveals additional unique real specimens", expanded.length > initial.length && new Set(expanded.map(e => e.id)).size === expanded.length && initial.every(e => expanded.some(n => n.id === e.id)), { before: initial.length, after: expanded.length });
  for (let i = 0; i < 20 && await evaluate("!document.querySelector('[data-catalog-more]').hidden"); i++) await click("[data-catalog-more]");
  const allVisible = await visible();
  check("Repeated Show more reaches the whole library and then disappears", allVisible.length === actual.length && await evaluate("document.querySelector('[data-catalog-more]').hidden"), { visible: allVisible.length, actual: actual.length });
  await input("[data-catalog-search]", "accordion");
  const accordion = await visible(), actualAccordion = actual.filter(e => `${e.title} ${e.desc}`.toLowerCase().includes("accordion"));
  check("Searching accordion returns real matching specimens with an accurate count", accordion.length > 0 && accordion.every(e => e.text.toLowerCase().includes("accordion")) && await count() === actualAccordion.length, { visible: accordion, expected: actualAccordion.length, count: await count() });
  await input("[data-catalog-search]", "zz-no-such-library-detail-8844");
  check("An unmatched search shows an empty state and zero results", (await visible()).length === 0 && await count() === 0 && await evaluate("!document.querySelector('[data-catalog-empty]').hidden&&!document.querySelector('[data-catalog-clear]').hidden&&document.querySelector('[data-catalog-more]').hidden"));
  await click("[data-catalog-clear]");
  check("Clear filters restores the full count and focuses the empty search", await count() === actual.length && await evaluate("document.activeElement.matches('[data-catalog-search]')&&document.activeElement.value===''&&document.querySelector('[data-catalog-empty]').hidden&&document.querySelector('[data-catalog-clear]').hidden"));
  const familyPages = { controls: ["components", "forms", "navigation"], motion: ["primitives", "motion", "icons"], data: ["data"], feedback: ["overlays"], foundations: ["foundations"] };
  for (const [family, pages] of Object.entries(familyPages)) {
    await click(`[data-catalog-filter="${family}"]`);
    const items = await visible(), expected = actual.filter(e => pages.includes(e.page)).length;
    check(`${family}: category filter has real results and an accurate count`, items.length > 0 && items.every(e => pages.includes(e.page)) && await count() === expected && await evaluate(`document.querySelectorAll('[data-catalog-filter][aria-pressed="true"]').length===1&&document.querySelector('[data-catalog-filter="${family}"]').getAttribute('aria-pressed')==='true'`), { count: await count(), expected, pages: [...new Set(items.map(e => e.page))] });
  }
  await click("[data-catalog-clear]");
  await focus("[data-catalog-search]");
  await key("Escape", "Escape", 27);
  await focus(".ov-title").catch(() => {});
  await evaluate("document.activeElement.blur()"); await key("/", "Slash", 191);
  check("Slash shortcut reaches the catalog search by keyboard", await evaluate("document.activeElement.matches('[data-catalog-search]')"));
  await input("[data-catalog-search]", "accordion");
  const result = (await visible())[0];
  if (!result) throw new Error("No accordion result exists to test an exact specimen jump.");
  await focus(`[data-catalog-id="${result.id}"]`); await key("Enter", "Enter", 13); await frameWait(6);
  const landing = await evaluate(`(()=>{const h=document.activeElement,d=h.closest('.demo'),v=document.querySelector('#viewport'),r=d?.getBoundingClientRect(),vr=v.getBoundingClientRect();return {page:Schematic._shell.state.current,title:h.textContent.trim(),isHeading:h.matches('.demo-title'),index:[...document.querySelectorAll('[data-page="${result.page}"] .demo')].indexOf(d),rect:r?{top:r.top,bottom:r.bottom,height:r.height}:null,viewport:{top:vr.top,bottom:vr.bottom},scroll:v.scrollTop};})()`);
  check("Keyboard result activation jumps to the exact demo and focuses its title", landing.page === result.page && landing.isHeading && landing.index === result.index && landing.title === result.title, { expected: result, actual: landing });
  check("The exact result is actually brought into the reading viewport", landing.rect && landing.rect.top >= landing.viewport.top - 1 && landing.rect.top < landing.viewport.bottom - 30, landing);
  await go("overview");
  check("Catalog search state survives navigation", await evaluate("document.querySelector('[data-catalog-search]').value==='accordion'"));
  await click("[data-catalog-clear]");
}

async function overviewChecks() {
  await go("overview");
  await click("[data-study-reset]");
  await click('.ov-study-tabs [data-value="0"]');
  await focus("[data-study-task]:nth-child(2)"); await key(" ", "Space", 32);
  const selection = await evaluate("({selected:[...document.querySelectorAll('[data-study-task]')].map(e=>e.getAttribute('aria-pressed')),total:document.querySelector('[data-study-total]').textContent,meter:new DOMMatrixReadOnly(getComputedStyle(document.querySelector('[data-study-meter]')).transform).m11})");
  check("Select study changes the real selection, count and meter with Space", equal(selection.selected, ["true", "true", "false"]) && selection.total === "2 / 3" && near(selection.meter, 2 / 3, 0.01), selection);
  await click('.ov-study-tabs [data-value="1"]');
  const original = await evaluate("[...document.querySelectorAll('[data-study-pin]')].map(e=>e.querySelector('span').textContent.trim())");
  await focus("[data-study-pin]:last-child"); await key("Enter", "Enter", 13);
  const pinned = await evaluate("[...document.querySelectorAll('[data-study-pin]')].map(e=>({title:e.querySelector('span').textContent.trim(),selected:e.getAttribute('aria-pressed')}))");
  check("Arrange study pins the chosen row and reorders the actual list", pinned[0]?.title === original.at(-1) && pinned[0]?.selected === "true" && equal(pinned.map(e => e.title).sort(), [...original].sort()), { original, pinned });
  await click('.ov-study-tabs [data-value="2"]');
  const closedHeight = await evaluate("document.querySelector('.ov-disclosure-grid').getBoundingClientRect().height");
  await focus("[data-study-reveal]"); await key("Enter", "Enter", 13); await frameWait();
  const disclosure = await evaluate("(()=>{const b=document.querySelector('[data-study-reveal]'),p=document.getElementById(b.getAttribute('aria-controls'));return {expanded:b.getAttribute('aria-expanded'),inert:p.inert,height:p.getBoundingClientRect().height,text:p.textContent.trim()};})()");
  check("Reveal study exposes actual content and expanded semantics with Enter", disclosure.expanded === "true" && !disclosure.inert && disclosure.height > closedHeight + 10 && disclosure.text.length > 20, { closedHeight, ...disclosure });
  await go("foundations"); await go("overview");
  check("All home study state survives leaving and returning", await evaluate(`document.querySelectorAll('[data-study-task][aria-pressed=true]').length===2&&document.querySelector('[data-study-pin]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-study-reveal]').getAttribute('aria-expanded')==='true'&&document.querySelector('[data-study="2"]').hasAttribute('data-active')`));
  await click("[data-study-reset]");
  check("Home reset restores selection and closes every disclosure", await evaluate("document.querySelectorAll('[data-study-task][aria-pressed=true]').length===1&&document.querySelector('[data-study-task]').getAttribute('aria-pressed')==='true'&&!document.querySelector('[data-study-pin][aria-pressed=true]')&&!document.querySelector('[data-study-reveal][aria-expanded=true]')&&[...document.querySelectorAll('.ov-disclosure-grid')].every(e=>e.inert)"));
}

async function narrowChecks() {
  await evaluate("Schematic.setMotion('reduced');Schematic.setPalette('mono');Schematic.setTheme('dark');void 0"); await frameWait();
  for (const width of [390, 320]) {
    await b.size(width, 844);
    for (const id of routes) {
      await go(id);
      const geometry = await evaluate(`(()=>{
        const v=document.querySelector('#viewport'),p=document.querySelector('[data-page="${id}"]'),r=v.getBoundingClientRect();
        const describe=e=>({tag:e.tagName,id:e.id,class:e.className?.baseVal??e.className,rect:libraryQA.rect(e),scroll:e.scrollWidth,client:e.clientWidth});
        const leaks=[...p.querySelectorAll('*')].filter(e=>{
          const b=e.getBoundingClientRect(),s=getComputedStyle(e);if(!b.width||!b.height||s.visibility==='hidden'||s.display==='none')return false;
          if(b.left>=r.left-1&&b.right<=r.right+1)return false;
          for(let a=e.parentElement;a&&a!==v;a=a.parentElement){if(['hidden','clip','auto','scroll'].includes(getComputedStyle(a).overflowX))return false;}
          return true;
        }).slice(0,8).map(describe);
        return {route:Schematic._shell.state.current,width:innerWidth,document:{client:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth},viewport:describe(v),page:describe(p),leaks,demos:[...p.querySelectorAll('.demo')].map(describe),theme:document.documentElement.dataset.theme,palette:document.documentElement.dataset.palette};
      })()`);
      check(`${width}px dark Mono: ${id} keeps the reading canvas inside the viewport`, geometry.route === id && geometry.document.scroll <= width + 1 && geometry.viewport.scroll <= geometry.viewport.client + 1 && geometry.leaks.length === 0 && geometry.theme === "dark" && geometry.palette === "mono", geometry);
    }
  }
  await b.size(1440, 1000); await evaluate("Schematic.setTheme('light');Schematic.setPalette('color');void 0"); await frameWait();
}

async function hiddenWorkChecks() {
  for (const id of ["overview", "primitives", "navigation"]) {
    await go(id); await evaluate("Schematic.setMotion('full')"); await frameWait();
    // Finish the entrance, then deliberately navigate away during a fresh
    // transition. An already-idle page cannot prove leave() cancels motion.
    await delay(750);
    const actions = {
      overview: `document.querySelector('.ov-study-tabs [data-value="1"]').click();document.querySelector('[data-study-pin]:last-child').click()`,
      primitives: "document.querySelector('[data-pr-collapse]').click()",
      navigation: `document.querySelector('[data-nv-page="2"]').click()`,
    };
    const interruption = await evaluate(`(async()=>{
      const root=document.querySelector('[data-page="${id}"]');
      const animations=()=>root.getAnimations({subtree:true}).filter(a=>['running','pending'].includes(a.playState)).map(a=>({state:a.playState,target:a.effect.target?.className,iterations:a.effect.getTiming().iterations}));
      ${actions[id]};const active=animations();const navigation=Schematic.navigate('foundations');
      await libraryQA.until(()=>Schematic._shell.state.current==='foundations');const hidden=animations();await navigation;
      return {active,hidden,hiddenAttribute:root.hidden};
    })()`);
    check(`${id}: navigating away cancels motion that was actually in flight`, interruption.active.length > 0 && interruption.hidden.length === 0 && interruption.hiddenAttribute, interruption);
    await delay(750);
    const before = await evaluate(`({work:libraryWorkQA('${id}'),animations:document.querySelector('[data-page="${id}"]').getAnimations({subtree:true}).filter(a=>['running','pending'].includes(a.playState)).map(a=>({state:a.playState,iterations:a.effect.getTiming().iterations,target:a.effect.target?.className}))})`);
    await delay(420);
    const after = await evaluate(`libraryWorkQA('${id}')`);
    check(`${id}: hidden page has no running animation or recurring callbacks`, before.animations.length === 0 && equal(before.work.callbacks, after.callbacks) && after.pending.every(item => item.kind !== "raf" && item.kind !== "interval"), { before, after });
  }
  await evaluate("Schematic.setMotion('reduced')");
}

async function primitivesChecks() {
  await go("primitives");
  await evaluate("Schematic.setMotion('reduced')"); await frameWait();
  const root = ".pg-primitives";
  const specimenNames = await evaluate("[...document.querySelectorAll('.pg-primitives .demo-title')].map(e=>e.textContent.trim())");
  check("Primitives expose eight distinct functional building blocks", specimenNames.length === 8 && new Set(specimenNames).size === 8 && ["Accordion", "Collapsible", "Pinned list", "Sortable list", "Text swap", "Animated counter", "Flip card", "Expandable surface"].every(name => specimenNames.includes(name)), specimenNames);

  await focus('[data-pr-accordion-trigger="0"]'); await key("ArrowDown", "ArrowDown", 40);
  check("Accordion arrow navigation moves heading focus without changing selection", await evaluate(`document.activeElement.matches('[data-pr-accordion-trigger="1"]')&&document.querySelector('[data-pr-accordion-trigger="0"]').getAttribute('aria-expanded')==='true'`));
  await key(" ", "Space", 32); await frameWait();
  check("Accordion Space opens one real accessible panel and closes the previous one", await evaluate(`(()=>{const b=document.querySelector('[data-pr-accordion-trigger="1"]'),p=document.getElementById(b.getAttribute('aria-controls'));return b.getAttribute('aria-expanded')==='true'&&!p.inert&&p.getAttribute('aria-hidden')==='false'&&p.getBoundingClientRect().height>20&&document.querySelectorAll('[data-pr-accordion-trigger][aria-expanded=true]').length===1&&document.querySelector('#pr-accordion-panel-0').inert;})()`));
  await key("End", "End", 35);
  check("Accordion End reaches the final heading", await evaluate(`document.activeElement.matches('[data-pr-accordion-trigger="2"]')`));
  await key("Home", "Home", 36);
  check("Accordion Home reaches the first heading", await evaluate(`document.activeElement.matches('[data-pr-accordion-trigger="0"]')`));

  await focus("[data-pr-collapse]"); await key("Enter", "Enter", 13); await frameWait();
  const collapsed = await evaluate("libraryQA.visual('[data-pr-collapse-panel]')");
  check("Collapsible Enter reveals actual checklist content", await evaluate("document.querySelector('[data-pr-collapse]').getAttribute('aria-expanded')==='true'&&!document.querySelector('[data-pr-collapse-panel]').inert") && collapsed.rect.height > 20, collapsed);
  await focus("[data-pr-detail-toggle]"); await key("Enter", "Enter", 13); await frameWait();
  const withNotes = await evaluate("libraryQA.visual('[data-pr-collapse-panel]')");
  check("Collapsible adapts its real height when optional notes are added", withNotes.rect.height > collapsed.rect.height + 20 && await evaluate("!document.querySelector('[data-pr-detail-notes]').hidden&&document.querySelector('[data-pr-detail-toggle]').getAttribute('aria-expanded')==='true'"), { checklist: collapsed.rect, withNotes: withNotes.rect });
  await click("[data-pr-collapse]"); await frameWait();
  check("Closing a collapsible with focus inside restores trigger focus and excludes the content", await evaluate("document.activeElement.matches('[data-pr-collapse]')&&document.querySelector('[data-pr-collapse-panel]').inert&&document.querySelector('[data-pr-collapse]').getAttribute('aria-expanded')==='false'&&document.querySelector('[data-pr-collapse-panel]').getBoundingClientRect().height<1"));

  await evaluate("window.libraryPinnedRow=document.querySelector('[data-pr-pin-row=\"tokens\"]')");
  await focus('[data-pr-pin="tokens"]'); await key("Enter", "Enter", 13); await frameWait();
  check("Pinning moves the original document into Pinned and preserves keyboard focus", await evaluate(`libraryPinnedRow===document.querySelector('[data-pr-pin-zone="pinned"] [data-pr-pin-row="tokens"]')&&document.activeElement.matches('[data-pr-pin="tokens"]')&&document.querySelector('[data-pr-pin="tokens"]').getAttribute('aria-pressed')==='true'&&Number(document.querySelector('[data-pr-pin-count]').textContent)===2&&Number(document.querySelector('[data-pr-unpin-count]').textContent)===2`));
  await key("Enter", "Enter", 13);
  check("Unpinning returns that same row to its original list in document order", await evaluate(`libraryPinnedRow===document.querySelector('[data-pr-pin-zone="unpinned"] [data-pr-pin-row="tokens"]')&&document.querySelector('[data-pr-pin-zone="unpinned"]').firstElementChild===libraryPinnedRow&&document.querySelector('[data-pr-pin="tokens"]').getAttribute('aria-pressed')==='false'`));
  await click('[data-pr-pin="motion"]');

  const originalOrder = await evaluate("[...document.querySelectorAll('[data-pr-sort-row]')].map(e=>e.dataset.prSortRow)");
  await focus('[data-pr-sort-row="motion"] [data-pr-move="up"]'); await key("Enter", "Enter", 13); await frameWait();
  const order = await evaluate("[...document.querySelectorAll('[data-pr-sort-row]')].map(e=>e.dataset.prSortRow)");
  check("Sortable move buttons reorder the plan and keep focus on the moved item", equal(order, ["audit", "motion", "structure", "handoff"]) && await evaluate(`document.activeElement.closest('[data-pr-sort-row]').dataset.prSortRow==='motion'&&document.querySelector('[data-pr-sort-row] [data-pr-move="up"]').disabled&&document.querySelector('[data-pr-sort-row]:last-child [data-pr-move="down"]').disabled`), { originalOrder, order });
  await click("[data-pr-sort-reset]");
  check("Sort reset restores the actual order and visible position numbers", equal(await evaluate("[...document.querySelectorAll('[data-pr-sort-row]')].map(e=>e.dataset.prSortRow)"), originalOrder) && equal(await evaluate("[...document.querySelectorAll('[data-pr-position]')].map(e=>e.textContent.trim())"), ["01", "02", "03", "04"]));

  await focus('[data-pr-swap="0"]'); await key("End", "End", 35); await frameWait();
  check("Text swap keyboard selection changes the visible and accessible message", await evaluate(`document.activeElement.matches('[data-pr-swap="2"]')&&document.querySelector('[data-pr-swap="2"]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-pr-swap-accessible]').textContent.includes('what comes next.')&&document.querySelectorAll('[data-pr-word-value]').length===1&&document.querySelector('[data-pr-word-value]').textContent.trim()==='what comes next.'`));
  const counterBefore = await evaluate("libraryQA.rect(document.querySelector('[data-pr-counter]'))");
  await click('[data-pr-count="1"]'); await click('[data-pr-count="1"]'); await frameWait();
  const counterAfter = await evaluate("libraryQA.rect(document.querySelector('[data-pr-counter]'))");
  check("Counter carries 129 to 130 without changing the tally frame width", await evaluate("document.querySelector('[data-pr-counter]').dataset.value==='130'&&document.querySelector('[data-pr-counter-accessible]').textContent==='130 items'") && near(counterBefore.width, counterAfter.width, 0.5), { before: counterBefore, after: counterAfter });
  await focus('[data-pr-count="-1"]'); await key(" ", "Space", 32);
  check("Counter decrement is keyboard operable and changes the actual value", await evaluate("document.querySelector('[data-pr-counter]').dataset.value==='129'"));

  await focus('[data-pr-flip-action="save"]'); await key("Enter", "Enter", 13);
  await focus("[data-pr-flip]"); await key("Enter", "Enter", 13); await frameWait();
  check("Flip card exposes its reverse while making the hidden face inert", await evaluate(`document.querySelector('[data-pr-flip]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-pr-face="front"]').inert&&document.querySelector('[data-pr-face="front"]').getAttribute('aria-hidden')==='true'&&!document.querySelector('[data-pr-face="back"]').inert&&document.querySelector('[data-pr-face="back"]').getAttribute('aria-hidden')==='false'`));
  await focus('[data-pr-flip-action="review"]'); await key(" ", "Space", 32);
  await click("[data-pr-flip]"); await frameWait();
  check("Turning away from an active face restores focus and retains each face's action state", await evaluate(`document.activeElement.matches('[data-pr-flip]')&&!document.querySelector('[data-pr-face="front"]').inert&&document.querySelector('[data-pr-face="back"]').inert&&document.querySelector('[data-pr-flip-action="save"]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-pr-flip-action="review"]').getAttribute('aria-pressed')==='true'`));

  await focus("[data-pr-expand]"); await key("Enter", "Enter", 13); await frameWait();
  check("Expandable surface reveals real project context", await evaluate("document.querySelector('[data-pr-expand]').getAttribute('aria-expanded')==='true'&&!document.querySelector('[data-pr-expand-panel]').inert&&document.querySelector('[data-pr-expand-panel]').getBoundingClientRect().height>40"));
  await focus("[data-pr-review]"); await key(" ", "Space", 32);
  check("Expanded project review changes both its action and summary status", await evaluate("document.querySelector('[data-pr-review]').getAttribute('aria-pressed')==='true'&&document.querySelector('[data-pr-review-state]').textContent==='Reviewed'&&document.querySelector('[data-pr-status=\"expandable-surface\"]').textContent.includes('complete')"));
  await click("[data-pr-expand]"); await frameWait();
  check("Closing project context restores focus and removes hidden controls from interaction", await evaluate("document.activeElement.matches('[data-pr-expand]')&&document.querySelector('[data-pr-expand-panel]').inert&&document.querySelector('[data-pr-expand-panel]').getBoundingClientRect().height<1"));

  const retained = () => evaluate("({accordion:[...document.querySelectorAll('[data-pr-accordion-trigger]')].map(e=>e.getAttribute('aria-expanded')),notes:document.querySelector('[data-pr-detail-toggle]').getAttribute('aria-expanded'),pinned:[...document.querySelectorAll('[data-pr-pin-zone=\"pinned\"] [data-pr-pin-row]')].map(e=>e.dataset.prPinRow),sort:[...document.querySelectorAll('[data-pr-sort-row]')].map(e=>e.dataset.prSortRow),text:document.querySelector('[data-pr-swap-accessible]').textContent,count:document.querySelector('[data-pr-counter]').dataset.value,saved:document.querySelector('[data-pr-flip-action=\"save\"]').getAttribute('aria-pressed'),review:document.querySelector('[data-pr-review]').getAttribute('aria-pressed')})");
  const beforeLeave = await retained(); await go("overview"); await go("primitives");
  check("All eight primitive specimens preserve their user state across navigation", equal(beforeLeave, await retained()), { before: beforeLeave, after: await retained() });

  // Measure actual frames for every animation family, with normal motion.
  await evaluate(`(()=>{const b=document.querySelector('[data-pr-accordion-trigger="0"]');if(b.getAttribute('aria-expanded')!=='true')b.click();})()`);
  await evaluate("Schematic.setMotion('full')"); await frameWait();
  const accordionTrace = await trace("primitive-accordion", `document.querySelector('[data-pr-accordion-trigger="1"]').click()`, `({panel:libraryQA.visual('#pr-accordion-panel-1'),copy:libraryQA.visual('#pr-accordion-panel-1 .pr-accordion-copy')})`, '[data-pr-accordion]');
  check("Normal accordion expansion contains multiple real height and text-opacity frames", !accordionTrace.timedOut && varied(between(framesOf(accordionTrace).map(f => f.panel.rect.height), accordionTrace.before.panel.rect.height, finalOf(accordionTrace).panel.rect.height, 0.5)) && varied(between(framesOf(accordionTrace).map(f => f.copy.opacity), 0, 1)), { before: accordionTrace.before, end: finalOf(accordionTrace), frameCount: accordionTrace.frames.length });

  const collapseTrace = await trace("primitive-collapsible", "document.querySelector('[data-pr-collapse]').click()", "({panel:libraryQA.visual('[data-pr-collapse-panel]')})", '[data-pr-collapsible]');
  check("Normal measured collapsible expands through intermediate rendered heights", !collapseTrace.timedOut && varied(between(framesOf(collapseTrace).map(f => f.panel.rect.height), collapseTrace.before.panel.rect.height, finalOf(collapseTrace).panel.rect.height, 0.5)), { before: collapseTrace.before, end: finalOf(collapseTrace), frameCount: collapseTrace.frames.length });
  const collapseReversal = traces["primitive-collapse-reversal"] = await evaluate(`(async()=>{const b=document.querySelector('[data-pr-collapse]'),p=document.querySelector('[data-pr-collapse-panel]'),height=p.getBoundingClientRect().height;b.click();await libraryQA.until(()=>p.getBoundingClientRect().height>10&&p.getBoundingClientRect().height<height-10);const before=libraryQA.visual(p);b.click();const after=libraryQA.visual(p);const settle=await libraryQA.trace(()=>{},()=>({panel:libraryQA.visual(p)}),'[data-pr-collapsible]');return {before,after,settle,expanded:b.getAttribute('aria-expanded')};})()`);
  check("Reversing measured collapse keeps its displayed height continuous and honors the latest action", near(collapseReversal.before.rect.height, collapseReversal.after.rect.height, 1) && collapseReversal.expanded === "true" && finalOf(collapseReversal.settle).panel.rect.height > 30 && !collapseReversal.settle.timedOut, collapseReversal);

  await evaluate("window.libraryPinMotionTarget=document.querySelector('[data-pr-pin-row=\"tokens\"]')");
  const pinTrace = await trace("primitive-pinned-list", `document.querySelector('[data-pr-pin="tokens"]').click()`, "({row:libraryQA.visual(libraryPinMotionTarget)})", '[data-pr-pinned-list]');
  check("Pinning animates the real row continuously between its two list positions", !pinTrace.timedOut && varied(between(framesOf(pinTrace).map(f => f.row.rect.y), pinTrace.before.row.rect.y, finalOf(pinTrace).row.rect.y, 0.5)) && await evaluate(`libraryPinMotionTarget===document.querySelector('[data-pr-pin-zone="pinned"] [data-pr-pin-row="tokens"]')`), { before: pinTrace.before, end: finalOf(pinTrace), frameCount: pinTrace.frames.length });
  const sortTrace = await trace("primitive-sortable-list", `document.querySelector('[data-pr-sort-row="motion"] [data-pr-move="up"]').click()`, `({row:libraryQA.visual('[data-pr-sort-row="motion"]')})`, '[data-pr-sortable-list]');
  check("Sorting travels through multiple intermediate row positions", !sortTrace.timedOut && varied(between(framesOf(sortTrace).map(f => f.row.rect.y), sortTrace.before.row.rect.y, finalOf(sortTrace).row.rect.y, 0.5)), { before: sortTrace.before, end: finalOf(sortTrace), frameCount: sortTrace.frames.length });

  const wordTrace = await trace("primitive-text-swap", `document.querySelector('[data-pr-swap="0"]').click()`, `({words:[...document.querySelectorAll('[data-pr-word-value]')].map(e=>({key:e.dataset.prWordValue,...libraryQA.visual(e)})),window:libraryQA.rect(document.querySelector('[data-pr-word-window]'))})`, '[data-pr-text-swap]');
  const wordFrames = framesOf(wordTrace).map(f => f.words.find(w => w.key === "0")).filter(Boolean);
  check("Normal text swap blends and travels through intermediate frames inside a stable window", !wordTrace.timedOut && varied(between(wordFrames.map(w => w.opacity), 0, 1)) && varied(wordFrames.map(w => w.y)) && wordTrace.frames.every(f => near(f.window.height, wordTrace.before.window.height, 0.5)), { before: wordTrace.before, end: finalOf(wordTrace), frameCount: wordTrace.frames.length });
  const wordReversal = traces["primitive-word-reversal"] = await evaluate(`(async()=>{document.querySelector('[data-pr-swap="1"]').click();await libraryQA.until(()=>{const e=document.querySelector('[data-pr-word-value="1"]');return e&&Number(getComputedStyle(e).opacity)>.15&&Number(getComputedStyle(e).opacity)<.85;});const snap=()=>[...document.querySelectorAll('[data-pr-word-value]')].map(e=>({key:e.dataset.prWordValue,...libraryQA.visual(e)}));const before=snap();document.querySelector('[data-pr-swap="0"]').click();const after=snap();const settle=await libraryQA.trace(()=>{},()=>({words:snap()}),'[data-pr-text-swap]');return {before,after,settle};})()`);
  check("Rapid text reversal preserves every visible word layer and settles on the latest message", wordReversal.before.every(w => { const after = wordReversal.after.find(a => a.key === w.key); return after && near(w.opacity, after.opacity, 0.025) && near(w.y, after.y, 0.6); }) && finalOf(wordReversal.settle).words.length === 1 && finalOf(wordReversal.settle).words[0].key === "0" && near(finalOf(wordReversal.settle).words[0].opacity, 1, 0.001), wordReversal);

  const counterTrace = await trace("primitive-counter", `document.querySelector('[data-pr-count="1"]').click()`, "({digits:[...document.querySelectorAll('[data-pr-digit]')].map(e=>libraryQA.visual(e)),counter:libraryQA.rect(document.querySelector('[data-pr-counter]'))})", '[data-pr-counter-card]');
  check("Counter rolls actual digits through intermediate frames while its width stays stable", !counterTrace.timedOut && counterTrace.before.digits.some((digit, i) => varied(framesOf(counterTrace).map(f => f.digits[i].y))) && counterTrace.frames.every(f => near(f.counter.width, counterTrace.before.counter.width, 0.5)), { before: counterTrace.before, end: finalOf(counterTrace), frameCount: counterTrace.frames.length });
  const flipTrace = await trace("primitive-flip-card", "document.querySelector('[data-pr-flip]').click()", "({rotor:libraryQA.visual('.pr-flip-rotor')})", '[data-pr-flip-card]');
  check("Normal flip card rotates through multiple rendered 3D frames", !flipTrace.timedOut && varied(between(framesOf(flipTrace).map(f => f.rotor.scaleX), flipTrace.before.rotor.scaleX, finalOf(flipTrace).rotor.scaleX)), { before: flipTrace.before, end: finalOf(flipTrace), frameCount: flipTrace.frames.length });
  const contextTrace = await trace("primitive-expandable", "document.querySelector('[data-pr-expand]').click()", "({panel:libraryQA.visual('[data-pr-expand-panel]')})", '[data-pr-expandable]');
  check("Normal project expansion has multiple actual content-height frames", !contextTrace.timedOut && varied(between(framesOf(contextTrace).map(f => f.panel.rect.height), contextTrace.before.panel.rect.height, finalOf(contextTrace).panel.rect.height, 0.5)), { before: contextTrace.before, end: finalOf(contextTrace), frameCount: contextTrace.frames.length });

  await evaluate("Schematic.setMotion('reduced')"); await frameWait();
  const reduced = await evaluate(`(()=>{document.querySelector('[data-pr-collapse]').click();document.querySelector('[data-pr-expand]').click();document.querySelector('[data-pr-flip]').click();document.querySelector('[data-pr-swap="2"]').click();document.querySelector('[data-pr-count="1"]').click();return {collapse:libraryQA.visual('[data-pr-collapse-panel]'),context:libraryQA.visual('[data-pr-expand-panel]'),words:[...document.querySelectorAll('[data-pr-word-value]')].map(e=>e.dataset.prWordValue),rotor:libraryQA.visual('.pr-flip-rotor'),running:libraryQA.running(document.querySelector('.pg-primitives')).length};})()`);
  check("Reduced primitives apply their final state immediately without travel animations", reduced.collapse.rect.height < 1 && reduced.context.rect.height < 1 && equal(reduced.words, ["2"]) && near(reduced.rotor.scaleX, 1, 0.001) && reduced.running === 0, reduced);
}
async function navigationChecks() {
  await go("navigation");
  await evaluate("Schematic.setMotion('reduced')"); await frameWait();
  const names = await evaluate("[...document.querySelectorAll('.pg-navigation .demo-title')].map(e=>e.textContent.trim())");
  check("Navigation exposes eight distinct working patterns", names.length === 8 && new Set(names).size === 8, names);

  await focus('[data-nv-menu-trigger="0"]'); await key("ArrowDown", "ArrowDown", 40);
  check("Navigation menu opens with ArrowDown and focuses a real destination", await evaluate(`document.querySelector('[data-nv-menu-trigger="0"]').getAttribute('aria-expanded')==='true'&&!document.querySelector('[data-nv-menu-viewport]').inert&&document.activeElement.matches('[data-nv-destination]')&&document.activeElement.closest('[data-nv-pane]').dataset.nvPane==='0'`));
  await key("Escape", "Escape", 27);
  check("Menu Escape dismisses its content and restores the owning trigger", await evaluate(`document.activeElement.matches('[data-nv-menu-trigger="0"]')&&document.querySelector('[data-nv-menu-viewport]').inert&&!document.querySelector('[data-nv-menu-trigger][aria-expanded=true]')`));
  await key("End", "End", 35); await key("Enter", "Enter", 13); await frameWait();
  check("Menu End and Enter open the final destination group", await evaluate(`document.activeElement.matches('[data-nv-menu-trigger="2"]')&&document.querySelector('[data-nv-menu-trigger="2"]').getAttribute('aria-expanded')==='true'&&document.querySelector('[data-nv-menu-stack] [data-nv-pane="2"]').getAttribute('aria-hidden')==='false'`));
  await click('[data-nv-menu-stack] [data-nv-pane="2"] [data-nv-destination]');
  check("Choosing a menu destination changes its status and dismisses the viewport", await evaluate("document.querySelector('[data-nv-menu-status]').textContent.includes('Field guide selected')&&document.querySelector('[data-nv-menu-viewport]').inert"));

  await focus("[data-nv-combo-input]"); await input("[data-nv-combo-input]", "motion");
  const matches = await evaluate("[...document.querySelectorAll('[data-nv-option]')].filter(e=>!e.hidden).map(e=>({id:e.id,text:e.textContent.trim()}))");
  check("Collection picker filters real options and exposes a visible active descendant", matches.length === 1 && matches[0].text.includes("Motion studies") && await evaluate("document.querySelector('[data-nv-combo-input]').getAttribute('aria-activedescendant')===document.querySelector('[data-nv-option]:not([hidden])').id"), matches);
  await key("ArrowDown", "ArrowDown", 40); await key("Enter", "Enter", 13);
  check("Picker Enter selects the filtered collection while retaining input focus", await evaluate("document.activeElement.matches('[data-nv-combo-input]')&&document.activeElement.getAttribute('aria-expanded')==='false'&&document.querySelector('[data-nv-combo-selected]').textContent==='Motion studies'&&document.querySelector('[data-nv-combo-list]').hidden&&document.querySelectorAll('[data-nv-option][aria-selected=true]').length===1"));
  await input("[data-nv-combo-input]", "unknown-collection-448");
  check("Picker no-results state has no invisible active descendant", await evaluate("!document.querySelector('[data-nv-no-results]').hidden&&[...document.querySelectorAll('[data-nv-option]')].every(e=>e.hidden)&&!document.querySelector('[data-nv-combo-input]').hasAttribute('aria-activedescendant')"));
  await key("Escape", "Escape", 27);
  check("Picker Escape preserves the previously selected collection", await evaluate("document.querySelector('[data-nv-combo-selected]').textContent==='Motion studies'&&document.querySelector('[data-nv-combo-list]').hidden"));

  await focus('[data-nv-folder-view] [data-nv-folder="projects"]'); await key("Enter", "Enter", 13);
  await click('[data-nv-folder-view] [data-nv-folder="fieldwork"]'); await click('[data-nv-folder-view] [data-nv-folder="research"]');
  const path = await evaluate("[...document.querySelectorAll('[data-nv-crumbs] [data-nv-folder]')].map(e=>({key:e.dataset.nvFolder,current:e.getAttribute('aria-current')}))");
  check("Breadcrumb drilling builds a real four-level path to the chosen folder", equal(path.map(e => e.key), ["workspace", "projects", "fieldwork", "research"]) && path.at(-1).current === "page" && await evaluate("document.querySelector('[data-nv-folder-view]').textContent.includes('Research is ready')"), path);
  await click('[data-nv-crumbs] [data-nv-folder="projects"]');
  check("A breadcrumb ancestor opens that folder's actual children and restores its focus", await evaluate(`document.activeElement.matches('[data-nv-crumbs] [data-nv-folder="projects"]')&&document.querySelector('[data-nv-crumbs] [aria-current]').dataset.nvFolder==='projects'&&document.querySelectorAll('[data-nv-folder-view] [data-nv-folder]').length===2`));
  await click("[data-nv-folder-up]");
  check("Breadcrumb Up returns to the root and disables itself at the boundary", await evaluate("document.querySelector('[data-nv-crumbs] [aria-current]').dataset.nvFolder==='workspace'&&document.querySelector('[data-nv-folder-up]').disabled"));

  const firstNotes = await evaluate("[...document.querySelectorAll('[data-nv-page-stack] [data-nv-active=true] li strong')].map(e=>e.textContent.trim())");
  await focus("[data-nv-page-next]"); await key("Enter", "Enter", 13);
  const secondNotes = await evaluate("[...document.querySelectorAll('[data-nv-page-stack] [data-nv-active=true] li strong')].map(e=>e.textContent.trim())");
  check("Pagination Next changes the actual three notes, range and current page", secondNotes.length === 3 && secondNotes.every(note => !firstNotes.includes(note)) && await evaluate(`document.querySelector('[data-nv-page="1"]').getAttribute('aria-current')==='page'&&document.querySelector('[data-nv-page-range]').textContent.includes('04')&&document.querySelector('[data-nv-page-range]').textContent.includes('06')`), { firstNotes, secondNotes });
  await click('[data-nv-page="3"]');
  check("Pagination last page contains the final notes and disables Next", await evaluate(`document.querySelector('[data-nv-page-next]').disabled&&document.querySelector('[data-nv-page-stack] [data-nv-active=true]').dataset.nvPane==='3'&&document.querySelector('[data-nv-page-range]').textContent==='10—12 / 12'`));
  await click('[data-nv-page="0"]');
  check("Pagination first page disables Previous and has exactly one interactive panel", await evaluate("document.querySelector('[data-nv-page-prev]').disabled&&document.querySelectorAll('[data-nv-page-stack] [data-nv-active=true]').length===1&&[...document.querySelectorAll('[data-nv-page-stack] [data-nv-pane]')].filter(e=>!e.inert).length===1"));
  await click('[data-nv-page="1"]');

  await focus('[data-nv-node="source"]'); await key("ArrowRight", "ArrowRight", 39); await key("ArrowRight", "ArrowRight", 39); await key("ArrowRight", "ArrowRight", 39); await key("Enter", "Enter", 13);
  check("File tree arrows traverse nested folders and Enter selects the actual file", await evaluate(`document.activeElement.matches('[data-nv-node="navigation-file"]')&&document.activeElement.getAttribute('aria-selected')==='true'&&document.querySelector('[data-nv-file-name]').textContent==='navigation.js'&&document.querySelector('[data-nv-file-desc]').textContent.includes('JavaScript')&&document.querySelectorAll('[data-nv-node][tabindex="0"]').length===1`));
  await key("ArrowLeft", "ArrowLeft", 37); await key("ArrowLeft", "ArrowLeft", 37);
  check("Tree Left returns to the parent and collapses its inaccessible descendants", await evaluate(`document.activeElement.matches('[data-nv-node="components"]')&&document.activeElement.getAttribute('aria-expanded')==='false'&&document.activeElement.querySelector('[data-nv-tree-children]').inert`));
  await key("End", "End", 35);
  check("Tree End skips closed descendants and reaches the last visible root", await evaluate(`document.activeElement.matches('[data-nv-node="notes"]')`));

  await focus('[data-nv-dock-view="0"]'); await key("ArrowRight", "ArrowRight", 39);
  check("Dock arrows change the actual workspace view with a single roving tab", await evaluate(`document.activeElement.matches('[data-nv-dock-view="1"]')&&document.activeElement.getAttribute('aria-selected')==='true'&&document.querySelector('[data-nv-dock-stack] [data-nv-active=true]').dataset.nvPane==='1'&&document.querySelectorAll('[data-nv-dock-view][tabindex="0"]').length===1&&document.querySelector('[data-nv-dock-stack] [data-nv-active=true]').textContent.includes('Map the journey')`));
  await key("End", "End", 35);
  check("Dock End reaches the real activity view", await evaluate(`document.activeElement.matches('[data-nv-dock-view="2"]')&&document.querySelector('[data-nv-dock-stack] [data-nv-active=true]').textContent.includes('Research is underway.')`));

  const wizardActionWidths = [await evaluate("document.querySelector('[data-nv-wizard-next]').getBoundingClientRect().width")];
  await click("[data-nv-wizard-next]");
  check("Wizard blocks an empty name with a focused, labelled native field error", await evaluate("document.activeElement.id==='nv-workspace-name'&&document.activeElement.getAttribute('aria-invalid')==='true'&&document.querySelector('[data-nv-name-error]').textContent.length>5&&document.querySelector('[data-nv-wizard-stack] [data-nv-active=true]').dataset.nvPane==='0'"));
  await input("#nv-workspace-name", "Regression workspace"); await click("[data-nv-wizard-next]");
  wizardActionWidths.push(await evaluate("document.querySelector('[data-nv-wizard-next]').getBoundingClientRect().width"));
  check("Wizard advances only after a valid name and focuses the new step heading", await evaluate("document.querySelector('[data-nv-wizard-stack] [data-nv-active=true]').dataset.nvPane==='1'&&document.activeElement.matches('[data-nv-wizard-stack] [data-nv-pane=\"1\"] h3')"));
  await input("#nv-workspace-email", "invalid"); await click("[data-nv-wizard-next]");
  check("Wizard rejects an invalid owner email without advancing", await evaluate("document.activeElement.id==='nv-workspace-email'&&document.activeElement.getAttribute('aria-invalid')==='true'&&document.querySelector('[data-nv-email-error]').textContent.length>5&&document.querySelector('[data-nv-wizard-stack] [data-nv-active=true]').dataset.nvPane==='1'"));
  await input("#nv-workspace-email", "owner@example.com"); await evaluate("document.querySelector('#nv-workspace-access').value='Shared';document.querySelector('#nv-workspace-access').dispatchEvent(new Event('change',{bubbles:true}))"); await click("[data-nv-wizard-next]");
  wizardActionWidths.push(await evaluate("document.querySelector('[data-nv-wizard-next]').getBoundingClientRect().width"));
  check("Wizard review shows the user's actual field values", await evaluate("document.querySelector('[data-nv-review-name]').textContent==='Regression workspace'&&document.querySelector('[data-nv-review-email]').textContent==='owner@example.com'&&document.querySelector('[data-nv-review-access]').textContent==='Shared'"));
  await click("[data-nv-wizard-back]");
  check("Wizard Back retains owner, name and visibility", await evaluate("document.querySelector('#nv-workspace-name').value==='Regression workspace'&&document.querySelector('#nv-workspace-email').value==='owner@example.com'&&document.querySelector('#nv-workspace-access').value==='Shared'"));
  await click("[data-nv-wizard-next]"); await click("[data-nv-wizard-next]");
  wizardActionWidths.push(await evaluate("document.querySelector('[data-nv-wizard-next]').getBoundingClientRect().width"));
  check("Wizard Create displays a local success with the actual workspace and owner", await evaluate("!document.querySelector('[data-nv-wizard-success]').hidden&&document.querySelector('[data-nv-wizard-created]').textContent.includes('Regression workspace')&&document.querySelector('[data-nv-wizard-created]').textContent.includes('owner@example.com')&&document.querySelector('[data-nv-wizard-status]').textContent.includes('No data was sent')&&document.activeElement.matches('[data-nv-wizard-success] h3')"));
  check("Wizard action stays the same width across Next, Create and Start again", wizardActionWidths.every(width => near(width, wizardActionWidths[0], 0.5)), wizardActionWidths);

  await focus("[data-nv-carousel]"); await key("End", "End", 35);
  check("Carousel End shows the Essential slide and updates the shared counter and dot", await evaluate(`document.querySelector('[data-nv-carousel-stack] [data-nv-active=true]').getAttribute('aria-label').includes('Essential')&&document.querySelector('[data-nv-slide-count]').textContent==='03 / 03'&&document.querySelector('[data-nv-slide="2"]').getAttribute('aria-pressed')==='true'`));
  await key("ArrowRight", "ArrowRight", 39);
  check("Carousel wraps from the final slide to the first using the keyboard", await evaluate("document.querySelector('[data-nv-carousel-stack] [data-nv-active=true]').dataset.nvPane==='0'&&document.querySelectorAll('[data-nv-slide][aria-pressed=true]').length===1"));
  await key("ArrowLeft", "ArrowLeft", 37); await click("[data-nv-use-direction]");
  check("Carousel action uses the selected direction's actual content", await evaluate("document.querySelector('[data-nv-carousel-status]').textContent.includes('Essential selected for your project')"));

  const retained = () => evaluate("({collection:document.querySelector('[data-nv-combo-selected]').textContent,folder:document.querySelector('[data-nv-crumbs] [aria-current]').dataset.nvFolder,page:document.querySelector('[data-nv-page][aria-current]').dataset.nvPage,file:document.querySelector('[data-nv-file-name]').textContent,dock:document.querySelector('[data-nv-dock-view][aria-selected=true]').dataset.nvDockView,wizard:document.querySelector('[data-nv-wizard-created]').textContent,name:document.querySelector('#nv-workspace-name').value,email:document.querySelector('#nv-workspace-email').value,slide:document.querySelector('[data-nv-carousel-stack] [data-nv-active=true]').dataset.nvPane})");
  const beforeLeave = await retained(); await go("overview"); await go("navigation");
  check("Navigation retains selected content, wizard fields and completion across route changes", equal(beforeLeave, await retained()), { before: beforeLeave, after: await retained() });

  await click("[data-nv-wizard-next]");
  check("Wizard Start again resets its native fields and returns to the first step", await evaluate("document.querySelector('#nv-workspace-name').value===''&&document.querySelector('#nv-workspace-email').value===''&&document.querySelector('[data-nv-wizard-stack] [data-nv-active=true]').dataset.nvPane==='0'&&document.querySelector('[data-nv-wizard-success]').hidden"));
  await evaluate("Schematic.setMotion('full')"); await frameWait();
  const menuTrace = await trace("navigation-menu", `document.querySelector('[data-nv-menu-trigger="0"]').click()`, "({viewport:libraryQA.visual('[data-nv-menu-viewport]')})", '[data-nv-menu]');
  check("Normal menu opening fades and travels through real intermediate frames", !menuTrace.timedOut && varied(between(framesOf(menuTrace).map(f => f.viewport.opacity), 0, 1)) && varied(framesOf(menuTrace).map(f => f.viewport.y)), { before: menuTrace.before, end: finalOf(menuTrace), frameCount: menuTrace.frames.length });
  const menuReversal = traces["navigation-menu-reversal"] = await evaluate(`(async()=>{document.querySelector('[data-nv-menu-trigger="0"]').click();await libraryQA.until(()=>{const o=Number(getComputedStyle(document.querySelector('[data-nv-menu-viewport]')).opacity);return o>.2&&o<.8;});const before=libraryQA.visual('[data-nv-menu-viewport]');document.querySelector('[data-nv-menu-trigger="1"]').click();const after=libraryQA.visual('[data-nv-menu-viewport]');const settle=await libraryQA.trace(()=>{},()=>({viewport:libraryQA.visual('[data-nv-menu-viewport]')}),'[data-nv-menu]');return {before,after,settle,open:document.querySelector('[data-nv-menu-trigger="1"]').getAttribute('aria-expanded')};})()`);
  check("Closing then reopening a menu preserves the displayed frame and latest destination", near(menuReversal.before.opacity, menuReversal.after.opacity, 0.025) && near(menuReversal.before.y, menuReversal.after.y, 0.6) && menuReversal.open === "true" && near(finalOf(menuReversal.settle).viewport.opacity, 1, 0.001), menuReversal);
  await focus('[data-nv-menu-trigger="1"]'); await key("Escape", "Escape", 27); await frameWait(20);
  await input("[data-nv-combo-input]", ""); await focus('[data-nv-menu-trigger="0"]'); await frameWait(20);
  const comboTrace = await trace("navigation-combobox", "document.querySelector('[data-nv-combo-input]').focus()", "({list:libraryQA.visual('[data-nv-combo-list]')})", '[data-nv-combo]');
  check("Normal collection picker enters through intermediate opacity and travel frames", !comboTrace.timedOut && varied(between(framesOf(comboTrace).map(f => f.list.opacity), 0, 1)) && varied(framesOf(comboTrace).map(f => f.list.y)), { before: comboTrace.before, end: finalOf(comboTrace), frameCount: comboTrace.frames.length });
  await key("Escape", "Escape", 27);

  const folderTrace = await trace("navigation-breadcrumb", `document.querySelector('[data-nv-folder-view] [data-nv-folder="projects"]').click()`, "({view:libraryQA.visual('[data-nv-folder-view]'),line:libraryQA.visual('[data-nv-path-line]')})", '[data-nv-breadcrumb]');
  check("Normal breadcrumb navigation animates its actual folder view and path line", !folderTrace.timedOut && varied(between(framesOf(folderTrace).map(f => f.view.opacity), 0, 1)) && varied(framesOf(folderTrace).map(f => f.view.x)) && varied(between(framesOf(folderTrace).map(f => f.line.scaleX), folderTrace.before.line.scaleX, finalOf(folderTrace).line.scaleX)), { before: folderTrace.before, end: finalOf(folderTrace), frameCount: folderTrace.frames.length });

  const paneSample = stack => `({panes:[...document.querySelectorAll('${stack} > [data-nv-pane]')].map(e=>({key:e.dataset.nvPane,active:e.dataset.nvActive,...libraryQA.visual(e)})),frame:libraryQA.rect(document.querySelector('${stack}'))})`;
  const pageTrace = await trace("navigation-pagination", `document.querySelector('[data-nv-page="2"]').click()`, paneSample('[data-nv-page-stack]'), '[data-nv-pagination]');
  const incoming = framesOf(pageTrace).map(f => f.panes.find(p => p.key === "2"));
  check("Normal pagination moves and crossfades real content inside a stable frame", !pageTrace.timedOut && varied(between(incoming.map(p => p.opacity), 0, 1)) && varied(incoming.map(p => p.x)) && pageTrace.frames.every(f => near(f.frame.width, pageTrace.before.frame.width, 0.5) && near(f.frame.height, pageTrace.before.frame.height, 0.5)), { before: pageTrace.before, end: finalOf(pageTrace), frameCount: pageTrace.frames.length });
  const pageReversal = traces["navigation-pagination-reversal"] = await evaluate(`(async()=>{document.querySelector('[data-nv-page="3"]').click();await libraryQA.until(()=>{const o=Number(getComputedStyle(document.querySelector('[data-nv-page-stack] [data-nv-pane="3"]')).opacity);return o>.15&&o<.85;});const snap=()=>[...document.querySelectorAll('[data-nv-page-stack] > [data-nv-pane]')].map(e=>({key:e.dataset.nvPane,active:e.dataset.nvActive,...libraryQA.visual(e)}));const before=snap();document.querySelector('[data-nv-page="2"]').click();const after=snap();const settle=await libraryQA.trace(()=>{},()=>({panes:snap()}),'[data-nv-pagination]');return {before,after,settle};})()`);
  check("Rapid pagination reversal keeps every visible layer continuous and settles on the newest page", pageReversal.before.filter(p => p.visibility === "visible").every(p => { const a = pageReversal.after.find(a => a.key === p.key); return a && near(p.opacity, a.opacity, 0.025) && near(p.x, a.x, 0.6); }) && finalOf(pageReversal.settle).panes.filter(p => p.active === "true").length === 1 && finalOf(pageReversal.settle).panes.find(p => p.active === "true")?.key === "2" && finalOf(pageReversal.settle).panes.every(p => p.active === "true" || p.visibility === "hidden"), pageReversal);

  const treeTrace = await trace("navigation-file-tree", `document.querySelector('[data-nv-node="notes"] > .nv-tree-row').click()`, `({children:libraryQA.visual('[data-nv-node="notes"] > [data-nv-tree-children]')})`, '[data-nv-tree]');
  check("Normal file tree expansion animates the real nested group height", !treeTrace.timedOut && varied(between(framesOf(treeTrace).map(f => f.children.rect.height), treeTrace.before.children.rect.height, finalOf(treeTrace).children.rect.height, 0.5)), { before: treeTrace.before, end: finalOf(treeTrace), frameCount: treeTrace.frames.length });
  const dockTrace = await trace("navigation-dock", `document.querySelector('[data-nv-dock-view="0"]').click()`, `({...${paneSample('[data-nv-dock-stack]')},marker:libraryQA.visual('[data-nv-dock-marker]')})`, '[data-nv-dock]');
  check("Normal dock selection travels to its control while the new view blends in", !dockTrace.timedOut && varied(between(framesOf(dockTrace).map(f => f.marker.rect.x), dockTrace.before.marker.rect.x, finalOf(dockTrace).marker.rect.x, 0.5)) && varied(between(framesOf(dockTrace).map(f => f.panes.find(p => p.key === "0").opacity), 0, 1)), { before: dockTrace.before, end: finalOf(dockTrace), frameCount: dockTrace.frames.length });

  await input("#nv-workspace-name", "Motion workspace");
  const wizardTrace = await trace("navigation-wizard", "document.querySelector('[data-nv-wizard-next]').click()", paneSample('[data-nv-wizard-stack]'), '[data-nv-wizard]');
  check("Normal wizard transitions the actual new step through intermediate frames without resizing its stage", !wizardTrace.timedOut && varied(between(framesOf(wizardTrace).map(f => f.panes.find(p => p.key === "1").opacity), 0, 1)) && varied(framesOf(wizardTrace).map(f => f.panes.find(p => p.key === "1").x)) && wizardTrace.frames.every(f => near(f.frame.height, wizardTrace.before.frame.height, 0.5)), { before: wizardTrace.before, end: finalOf(wizardTrace), frameCount: wizardTrace.frames.length });
  const carouselTrace = await trace("navigation-carousel", "document.querySelector('[data-nv-slide-next]').click()", paneSample('[data-nv-carousel-stack]'), '[data-nv-carousel]');
  check("Normal carousel animates the actual slide through opacity and travel inside a stable frame", !carouselTrace.timedOut && varied(between(framesOf(carouselTrace).map(f => f.panes.find(p => p.key === "0").opacity), 0, 1)) && varied(framesOf(carouselTrace).map(f => f.panes.find(p => p.key === "0").x)) && carouselTrace.frames.every(f => near(f.frame.width, carouselTrace.before.frame.width, 0.5) && near(f.frame.height, carouselTrace.before.frame.height, 0.5)), { before: carouselTrace.before, end: finalOf(carouselTrace), frameCount: carouselTrace.frames.length });

  await evaluate("Schematic.setMotion('reduced')"); await frameWait();
  const reduced = await evaluate(`(()=>{document.querySelector('[data-nv-page="0"]').click();document.querySelector('[data-nv-dock-view="1"]').click();document.querySelector('[data-nv-slide="1"]').click();document.querySelector('[data-nv-wizard-back]').click();const stacks=['[data-nv-page-stack]','[data-nv-dock-stack]','[data-nv-carousel-stack]','[data-nv-wizard-stack]'];return {panels:stacks.map(s=>{const es=[...document.querySelectorAll(s+' > [data-nv-pane]')],a=es.find(e=>e.dataset.nvActive==='true');return {key:a?.dataset.nvPane,active:es.filter(e=>!e.inert).length,...libraryQA.visual(a)};}),running:libraryQA.running(document.querySelector('.pg-navigation')).length};})()`);
  check("Reduced navigation reaches final panel geometry immediately without running transitions", equal(reduced.panels.map(p => p.key), ["0", "1", "1", "0"]) && reduced.panels.every(p => p.active === 1 && near(p.opacity, 1, 0.001) && near(p.x, 0, 0.001)) && reduced.running === 0, reduced);
}

try {
  b = await browser();
  await b.send("Page.addScriptToEvaluateOnNewDocument", { source: `(${installWorkTracker.toString()})()` });
  await b.open(url);
  await evaluate(`(${installProbe.toString()})()`);
  await evaluate("Schematic.setMotion('reduced')"); await frameWait();
  await group("Catalog", catalogChecks);
  await group("Overview studies", overviewChecks);
  await group("Primitives", primitivesChecks);
  await group("Navigation", navigationChecks);
  await group("Narrow appearances", narrowChecks);
  await group("Inactive work", hiddenWorkChecks);
  check("Expanded library browser run has no runtime or network errors", b.failures.length === 0, b.failures);
} catch (error) { check("Expanded library browser run completed", false, error.stack); }
finally { if (b) await b.close(); }

const output = join(workspace, "scratch/qa/library.json");
await mkdir(dirname(output), { recursive: true });
await writeFile(output, JSON.stringify({ date: new Date().toISOString(), checks, inventory, traces }, null, 2));
const failed = checks.filter(result => !result.pass).length;
console.log(`${checks.length - failed} passed, ${failed} failed`);
process.exitCode = failed ? 1 : 0;
