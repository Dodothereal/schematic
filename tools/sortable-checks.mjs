#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { browser, delay } from "./browser.mjs";

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const url = pathToFileURL(join(workspace, "dist/schematic.html")).href;
const checks = [], traces = {};
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const near = (a, b, tolerance = 1) => Number.isFinite(a) && Number.isFinite(b) && Math.abs(a - b) <= tolerance;
const check = (name, pass, detail) => {
  checks.push({ name, pass: !!pass, ...(detail === undefined ? {} : { detail }) });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${!pass && detail !== undefined ? `: ${JSON.stringify(detail)}` : ""}`);
};
const group = async (name, run) => {
  try { await run(); }
  catch (error) {
    check(`${name}: completed without an exception`, false, error.stack);
    // A failed assertion group must not leave a held pointer contaminating the
    // remaining keyboard/reduced/lifecycle coverage.
    if (held) { await key("Escape", "Escape", 27).catch(() => {}); await release().catch(() => {}); }
  }
};

// Installed before the runtime. Descendant callbacks retain their originating
// route, so an auto-scroll loop cannot hide a leak by scheduling its next frame
// after navigation. Probe frames use the native scheduler and are not counted.
function installWorkTracker() {
  const raf = window.requestAnimationFrame, caf = window.cancelAnimationFrame;
  const pending = new Map(), callbacks = new Map();
  let owner = null;
  window.requestAnimationFrame = callback => {
    const page = owner || window.Schematic?._shell?.state.current || "boot";
    const id = raf.call(window, time => {
      pending.delete(id); callbacks.set(page, (callbacks.get(page) || 0) + 1);
      const previous = owner; owner = page;
      try { callback(time); } finally { owner = previous; }
    });
    pending.set(id, page); return id;
  };
  window.cancelAnimationFrame = id => { pending.delete(id); caf.call(window, id); };
  window.sortableWorkQA = {
    snapshot: page => ({ callbacks: callbacks.get(page) || 0, pending: [...pending.values()].filter(p => p === page).length }),
    frames: async (count = 2) => { for (let i = 0; i < count; i++) await new Promise(resolve => raf.call(window, resolve)); },
  };
}

function installProbe() {
  const rect = element => {
    const r = element?.getBoundingClientRect();
    return r ? { x: r.x, y: r.y, width: r.width, height: r.height } : null;
  };
  const rows = selector => [...document.querySelectorAll(selector)];
  const identityKey = row => row.hasAttribute("data-pr-sort-row") ? `sort:${row.dataset.prSortRow}` : `pin:${row.dataset.prPinRow}`;
  const identities = new Map(rows("[data-pr-sort-row], [data-pr-pin-row]").map(row => [identityKey(row), row]));
  const events = [];
  for (const type of ["pointerdown", "pointermove", "pointerup", "pointercancel"]) {
    document.addEventListener(type, event => events.push({
      type, trusted: event.isTrusted, id: event.pointerId, pointerType: event.pointerType, buttons: event.buttons,
      x: event.clientX, y: event.clientY, target: event.target.closest("[data-pr-sort-row], [data-pr-pin-row]")?.dataset.prSortRow
        || event.target.closest("[data-pr-pin-row]")?.dataset.prPinRow || null,
    }), true);
  }
  const running = (root = document) => root.getAnimations({ subtree: true }).filter(animation => ["running", "pending"].includes(animation.playState));
  const sortOrder = () => rows("[data-pr-sort-list] > [data-pr-sort-row]").map(row => row.dataset.prSortRow);
  const pins = () => Object.fromEntries(["pinned", "unpinned"].map(group => [group,
    rows(`[data-pr-pin-zone="${group}"] > [data-pr-pin-row]`).map(row => row.dataset.prPinRow)]));
  const snapshot = () => ({
    order: sortOrder(), pins: pins(),
    active: rows(".pg-primitives [data-sortable-active]").map(el => el.dataset.sortableActive),
    ghosts: rows(".pg-primitives [data-sortable-ghost]").map(el => ({ rect: rect(el), inert: el.inert, hidden: el.getAttribute("aria-hidden"), ids: el.querySelectorAll("[id]").length, hooks: el.querySelectorAll("[data-pr-sort-row], [data-pr-pin-row]").length })),
    indicators: rows(".pg-primitives [data-sortable-indicator]").map(rect),
    focus: document.activeElement?.outerHTML.slice(0, 500),
    rows: rows("[data-pr-sort-list] > [data-pr-sort-row], [data-pr-pin-zone] > [data-pr-pin-row]").map(row => {
      const style = getComputedStyle(row), m = new DOMMatrixReadOnly(style.transform === "none" ? undefined : style.transform);
      return { key: row.dataset.prSortRow || row.dataset.prPinRow, rect: rect(row), opacity: Number(style.opacity), x: m.m41, y: m.m42,
        original: identities.get(identityKey(row)) === row };
    }),
    running: running(document.querySelector(".pg-primitives")).map(animation => ({ target: animation.effect.target?.className, iterations: animation.effect.getTiming().iterations })),
  });
  const settle = async (limit = 2400) => {
    const started = performance.now(); let idle = 0;
    do {
      await sortableWorkQA.frames(1);
      idle = running(document.querySelector(".pg-primitives")).length === 0 ? idle + 1 : 0;
      if (idle >= 4) return snapshot();
    } while (performance.now() - started < limit);
    throw new Error("Sortable motion did not settle within the bounded observation window.");
  };
  window.sortableQA = { rect, snapshot, sortOrder, pins, running, events, settle };
}

let b, held = null;
const evaluate = expression => b.evaluate(expression);
const frames = (count = 2) => evaluate(`sortableWorkQA.frames(${count})`);
const snapshot = () => evaluate("sortableQA.snapshot()");
const settle = () => evaluate("sortableQA.settle()");
const click = selector => evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing '+${JSON.stringify(selector)});e.click();})()`);
const focus = selector => evaluate(`document.querySelector(${JSON.stringify(selector)}).focus()`);
const go = async page => { await evaluate(`Schematic.navigate(${JSON.stringify(page)})`); await frames(3); };
const key = async (key, code, keyCode) => {
  const params = { key, code, windowsVirtualKeyCode: keyCode };
  await b.send("Input.dispatchKeyEvent", { type: "keyDown", ...params, ...(key === "Enter" ? { text: "\r", unmodifiedText: "\r" } : {}) });
  await b.send("Input.dispatchKeyEvent", { type: "keyUp", ...params });
};
const box = async selector => evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing '+${JSON.stringify(selector)});return sortableQA.rect(e);})()`);
const show = async selector => {
  await evaluate(`document.querySelector(${JSON.stringify(selector)}).scrollIntoView({block:'center',inline:'nearest',behavior:'instant'})`);
  await frames(3);
};
const center = r => ({ x: r.x + r.width / 2, y: r.y + r.height / 2 });
const press = async point => {
  await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...point, buttons: 0 });
  await b.send("Input.dispatchMouseEvent", { type: "mousePressed", ...point, button: "left", buttons: 1, clickCount: 1 });
  held = point;
};
const move = async (point, steps = 12) => {
  if (!held) throw new Error("A real held mouse is required before dragging.");
  const start = held, samples = [];
  for (let i = 1; i <= steps; i++) {
    const next = { x: start.x + (point.x - start.x) * i / steps, y: start.y + (point.y - start.y) * i / steps };
    await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...next, button: "left", buttons: 1 });
    held = next; await frames(1); samples.push(await snapshot());
  }
  return samples;
};
const release = async () => {
  if (!held) return;
  await b.send("Input.dispatchMouseEvent", { type: "mouseReleased", ...held, button: "left", buttons: 0, clickCount: 1 });
  held = null;
};

// Specimen selectors below are deliberately separate from the input driver.
// Drag coverage always uses trusted CDP mouse input, never dispatches a fake
// pointermove and never substitutes the pre-existing arrow-button shortcut.
const sortHandle = key => `[data-pr-sort-row="${key}"] [data-sortable-handle]`;
const pinHandle = key => `[data-pr-pin-row="${key}"] [data-sortable-handle]`;

async function pointerChecks() {
  await evaluate("Schematic.setMotion('full')"); await settle();
  const handles = await evaluate(`({sort:document.querySelectorAll('[data-pr-sort-row] [data-sortable-handle]').length,pins:document.querySelectorAll('[data-pr-pin-row] [data-sortable-handle]').length,help:[...document.querySelectorAll('.pg-primitives [data-sortable-handle]')].every(e=>e.tagName==='BUTTON'&&e.getAttribute('aria-label')&&e.getAttribute('aria-describedby')?.split(' ').every(id=>document.getElementById(id))&&e.getAttribute('aria-keyshortcuts')?.includes('Escape'))})`);
  check("Sortable and pinned rows expose dedicated labelled keyboard drag handles", handles.sort === 4 && handles.pins === 4 && handles.help, handles);
  await click("[data-pr-sort-reset]"); await settle();
  const initial = await snapshot();
  check("Sortable fixture starts with the actual four task rows", equal(initial.order, ["audit", "structure", "motion", "handoff"]), initial.order);
  await show("[data-pr-sort-list]");
  const start = center(await box(sortHandle("audit")));
  await press(start); await move({ x: start.x + 1, y: start.y + 1 }, 1);
  const small = await snapshot(); await release(); await settle();
  check("Movement below the pickup threshold does not reorder or spawn a drag preview", equal(small.order, initial.order) && small.active.length === 0 && small.ghosts.length === 0, small);

  const first = await dragSort("audit", "handoff", "after", "sort-pointer-downward");
  check("A held mouse reorders the actual sortable DOM before release", equal(first.during.at(-1).order, ["structure", "motion", "handoff", "audit"]), first.during.at(-1).order);
  check("Pointer pickup uses one inert, ID-free visual copy and one insertion indicator", first.during.some(frame => frame.active.includes("pointer") && frame.ghosts.length === 1 && frame.ghosts[0].inert && frame.ghosts[0].hidden === "true" && frame.ghosts[0].ids === 0 && frame.ghosts[0].hooks === 0 && frame.indicators.length === 1), first.during.at(-1));
  check("The drag preview follows distinct real mouse positions through intermediate frames", new Set(first.during.map(frame => Math.round(frame.ghosts[0]?.rect.y))).size >= 5 && first.during.filter(frame => frame.ghosts.length).every(frame => near(frame.ghosts[0].rect.x, first.source.x, 4)), { positions: first.during.map(frame => frame.ghosts[0]?.rect), start: first.start, source: first.source, grip: first.grip });
  check("Displaced task rows animate through intermediate layout positions during the held drag", ["structure", "motion", "handoff"].some(key => {
    const offsets = first.during.map(frame => frame.rows.find(row => row.key === key)?.y).filter(Number.isFinite);
    return offsets.some(y => Math.abs(y) > 1) && new Set(offsets.map(y => Math.round(y * 100))).size >= 3;
  }), first.during.map(frame => frame.rows.map(row => ({ key: row.key, y: row.y }))));
  check("Dropping commits the requested order with original rows and restores the handle's focus", equal(first.after.order, ["structure", "motion", "handoff", "audit"]) && first.after.rows.every(row => row.original) && await evaluate(`document.activeElement.matches('${sortHandle("audit")}')`), first.after);
  check("Drop removes visual copies, source markers and active pickup semantics", clean(first.after) && await evaluate("!document.querySelector('.pg-primitives [data-sortable-source]')&&[...document.querySelectorAll('.pg-primitives [data-sortable-handle]')].every(e=>e.getAttribute('aria-pressed')==='false')"), first.after);
  const input = first.events;
  check("Sort movement came from trusted held-pointer events rather than a shortcut or synthetic move", input.some(e => e.type === "pointerdown" && e.trusted && e.pointerType === "mouse" && e.buttons === 1) && input.filter(e => e.type === "pointermove" && e.trusted && e.buttons === 1).length >= 8 && input.some(e => e.type === "pointerup" && e.trusted && e.buttons === 0), input);
  check("Pointer reorder refreshes the real task position labels and boundary buttons", await evaluate("JSON.stringify([...document.querySelectorAll('[data-pr-position]')].map(e=>e.textContent.trim()))===JSON.stringify(['01','02','03','04'])&&document.querySelector('[data-pr-sort-row] [data-pr-move=up]').disabled&&document.querySelector('[data-pr-sort-row]:last-child [data-pr-move=down]').disabled"));
  const reverse = await dragSort("audit", "structure", "before", "sort-pointer-upward");
  check("Dragging upward restores the desired actual order without replacing rows", equal(reverse.after.order, initial.order) && reverse.after.rows.every(row => row.original) && clean(reverse.after), reverse.after);

  await show("[data-pr-sort-list]");
  const beforeCancel = await snapshot(), grip = await box(sortHandle("structure")), target = await box('[data-pr-sort-row="handoff"]');
  await press(center(grip)); const cancelledFrames = await move({ x: center(grip).x, y: target.y + target.height * .85 });
  check("Escape rollback is exercised after the pointer has actually changed the order", !equal(cancelledFrames.at(-1).order, beforeCancel.order), cancelledFrames.at(-1));
  await key("Escape", "Escape", 27); await release(); const cancelled = await settle();
  traces["sort-pointer-escape"] = { before: beforeCancel, frames: cancelledFrames, after: cancelled };
  check("Escape restores every sortable position, clears pickup and ignores the later mouse release", equal(cancelled.order, beforeCancel.order) && cancelled.rows.every(row => row.original) && clean(cancelled) && await evaluate(`document.activeElement.matches('${sortHandle("structure")}')`), cancelled);

  await touchCancel("audit", "handoff", false, "sort-trusted-pointercancel");
  await crossGroupChecks();
}

const clean = frame => frame.active.length === 0 && frame.ghosts.length === 0 && frame.indicators.length === 0;
async function dragSort(item, target, edge, name) {
  await show("[data-pr-sort-list]"); await settle();
  const before = await snapshot(), grip = await box(sortHandle(item)), source = await box(`[data-pr-sort-row="${item}"]`), dest = await box(`[data-pr-sort-row="${target}"]`);
  const start = center(grip), end = { x: start.x, y: dest.y + dest.height * (edge === "before" ? .15 : .85) };
  const eventStart = await evaluate("sortableQA.events.length");
  await press(start); const during = await move(end, 18); await release(); const immediate = await snapshot(), after = await settle();
  const result = { before, grip, source, start, end, during, immediate, after, events: await evaluate(`sortableQA.events.slice(${eventStart})`) };
  traces[name] = result; return result;
}

async function touchCancel(item, target, pin, name) {
  await show(pin ? "[data-pr-pinned-list]" : "[data-pr-sort-list]"); await settle();
  const before = await snapshot(), start = center(await box(pin ? pinHandle(item) : sortHandle(item)));
  const dest = await box(pin ? `[data-pr-pin-zone="${target}"]` : `[data-pr-sort-row="${target}"]`);
  const end = { x: pin ? dest.x + dest.width / 2 : start.x, y: dest.y + dest.height * .85 };
  const eventStart = await evaluate("sortableQA.events.length");
  await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 1 });
  const point = (x, y) => ({ x, y, id: 17, radiusX: 1, radiusY: 1, force: 1 });
  try {
    await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [point(start.x, start.y)] });
    for (let i = 1; i <= 14; i++) {
      await b.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [point(start.x + (end.x - start.x) * i / 14, start.y + (end.y - start.y) * i / 14)] });
      await frames(1);
    }
    const during = await snapshot();
    check(`${pin ? "Pinning" : "Sorting"}: trusted pointercancel starts from a real reordered touch drag`, !equal(pin ? during.pins : during.order, pin ? before.pins : before.order) && during.active.includes("pointer"), during);
    await b.send("Input.dispatchTouchEvent", { type: "touchCancel", touchPoints: [] });
    const after = await settle(), events = await evaluate(`sortableQA.events.slice(${eventStart})`);
    traces[name] = { before, during, after, events };
    check(`${pin ? "Pinning" : "Sorting"}: browser-generated pointercancel rolls the whole transaction back`, equal(pin ? after.pins : after.order, pin ? before.pins : before.order) && clean(after) && after.rows.every(row => row.original) && events.some(event => event.type === "pointercancel" && event.trusted && event.pointerType === "touch"), { after, events });
  } finally {
    await b.send("Input.dispatchTouchEvent", { type: "touchCancel", touchPoints: [] }).catch(() => {});
    await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
  }
}

async function dragPin(item, destination, name) {
  await show("[data-pr-pinned-list]"); await settle();
  const before = await snapshot(), start = center(await box(pinHandle(item))), dest = await box(`[data-pr-pin-zone="${destination}"]`);
  const end = { x: dest.x + dest.width / 2, y: dest.y + dest.height - 8 };
  await press(start); const during = await move(end, 18); await release(); const immediate = await snapshot(), after = await settle();
  const result = { before, start, end, during, immediate, after }; traces[name] = result; return result;
}

async function crossGroupChecks() {
  const initial = await snapshot(), source = Object.entries(initial.pins).find(([, keys]) => keys.includes("tokens"))[0], destination = source === "pinned" ? "unpinned" : "pinned";
  const pinned = await dragPin("tokens", destination, "pin-pointer-cross-group");
  check("Dragging between pin groups moves the actual original row into the target list", pinned.after.pins[destination].includes("tokens") && !pinned.after.pins[source].includes("tokens") && pinned.after.rows.every(row => row.original), pinned.after);
  check("Cross-group dragging keeps exactly the same four unique documents", Object.values(pinned.after.pins).flat().length === 4 && new Set(Object.values(pinned.after.pins).flat()).size === 4, pinned.after.pins);
  check("Pointer pinning refreshes group counts and native pin-button state", await pinSemantics());
  const unpinned = await dragPin("tokens", source, "pin-pointer-return");
  check("The same row can be dragged back across groups without stale pickup state", unpinned.after.pins[source].includes("tokens") && !unpinned.after.pins[destination].includes("tokens") && clean(unpinned.after) && await pinSemantics(), unpinned.after);

  await show("[data-pr-pinned-list]");
  const before = await snapshot(), grip = center(await box(pinHandle("motion"))), dest = await box('[data-pr-pin-zone="pinned"]');
  await press(grip); const during = await move({ x: dest.x + dest.width / 2, y: dest.y + dest.height - 8 }, 18);
  check("Cross-group Escape is exercised after a document actually entered another list", !equal(during.at(-1).pins, before.pins), during.at(-1));
  await key("Escape", "Escape", 27); await release(); const after = await settle();
  traces["pin-pointer-escape"] = { before, during, after };
  check("Escape restores both pin-group orders, counts, identity and button state", equal(after.pins, before.pins) && clean(after) && after.rows.every(row => row.original) && await pinSemantics(), after);
  await touchCancel("review", "pinned", true, "pin-trusted-pointercancel");
}

const pinSemantics = () => evaluate(`(()=>{const groups=['pinned','unpinned'];return groups.every(group=>[...document.querySelectorAll('[data-pr-pin-zone="'+group+'"] > [data-pr-pin-row]')].every(row=>row.dataset.prPinned===String(group==='pinned')&&row.querySelector('[data-pr-pin]').getAttribute('aria-pressed')===String(group==='pinned')))&&Number(document.querySelector('[data-pr-pin-count]').textContent)===document.querySelectorAll('[data-pr-pin-zone="pinned"] > [data-pr-pin-row]').length&&Number(document.querySelector('[data-pr-unpin-count]').textContent)===document.querySelectorAll('[data-pr-pin-zone="unpinned"] > [data-pr-pin-row]').length;})()`);

async function keyboardChecks() {
  await evaluate("Schematic.setMotion('full')"); await click("[data-pr-sort-reset]"); await settle();
  await show("[data-pr-sort-list]"); await focus(sortHandle("audit"));
  await key(" ", "Space", 32);
  const pickup = await snapshot();
  check("Space picks up the focused sortable row with accessible keyboard state", equal(pickup.active, ["keyboard"]) && pickup.ghosts.length === 0 && await evaluate(`document.activeElement.matches('${sortHandle("audit")}')&&document.activeElement.getAttribute('aria-pressed')==='true'&&document.querySelector('[data-pr-sort-row="audit"]').getAttribute('data-sortable-source')==='keyboard'`), pickup);
  await key("ArrowDown", "ArrowDown", 40); await key("ArrowDown", "ArrowDown", 40);
  const moved = await snapshot();
  check("Keyboard arrows move the grabbed original row through actual DOM positions", equal(moved.order, ["structure", "motion", "audit", "handoff"]) && moved.rows.every(row => row.original) && await evaluate(`document.activeElement.matches('${sortHandle("audit")}')`), moved);
  check("Keyboard moves announce the selected item's real new position", await evaluate("(()=>{const text=document.querySelector('[data-pr-status=\"sortable-list\"]').textContent,title=document.querySelector('[data-pr-sort-row=\"audit\"] strong').textContent;return text.includes(title)&&text.includes('3')&&text.includes('4');})()"));
  await key("Enter", "Enter", 13); const dropped = await settle();
  traces["sort-keyboard-drop"] = { pickup, moved, after: dropped };
  check("Enter drops without submitting another action and preserves the keyboard order", equal(dropped.order, moved.order) && clean(dropped) && await evaluate(`document.activeElement.matches('${sortHandle("audit")}')&&document.activeElement.getAttribute('aria-pressed')==='false'`), dropped);

  const before = await snapshot(); await focus(sortHandle("motion")); await key("Enter", "Enter", 13); await key("Home", "Home", 36);
  const during = await snapshot(); await key("Escape", "Escape", 27); const after = await settle();
  traces["sort-keyboard-escape"] = { before, during, after };
  check("Keyboard Escape is tested after Home changes the actual ordering", !equal(during.order, before.order) && during.order[0] === "motion", during.order);
  check("Keyboard Escape restores the original order and handle focus", equal(after.order, before.order) && clean(after) && await evaluate(`document.activeElement.matches('${sortHandle("motion")}')&&document.activeElement.getAttribute('aria-pressed')==='false'`), after);

  await show("[data-pr-pinned-list]"); await focus(pinHandle("tokens"));
  const groups = await evaluate("[...document.querySelectorAll('[data-pr-pin-zone]')].map(e=>e.dataset.prPinZone)"), pinsBefore = await snapshot();
  const from = groups.findIndex(group => pinsBefore.pins[group].includes("tokens")), destination = from === 0 ? 1 : 0;
  await key(" ", "Space", 32); await key(destination > from ? "ArrowRight" : "ArrowLeft", destination > from ? "ArrowRight" : "ArrowLeft", destination > from ? 39 : 37);
  const cross = await snapshot();
  check("Keyboard horizontal arrows move a grabbed document between real lists", cross.pins[groups[destination]].includes("tokens") && !cross.pins[groups[from]].includes("tokens") && cross.active.includes("keyboard") && await pinSemantics(), cross);
  await key(" ", "Space", 32); const pinDrop = await settle();
  check("Space drops a cross-list keyboard move and retains its new pin state", equal(pinDrop.pins, cross.pins) && clean(pinDrop) && await evaluate(`document.activeElement.matches('${pinHandle("tokens")}')&&document.activeElement.getAttribute('aria-pressed')==='false'`) && await pinSemantics(), pinDrop);

  const pinBeforeCancel = await snapshot(); await key("Enter", "Enter", 13); await key(destination > from ? "ArrowLeft" : "ArrowRight", destination > from ? "ArrowLeft" : "ArrowRight", destination > from ? 37 : 39);
  const pinDuringCancel = await snapshot(); await key("Escape", "Escape", 27); const pinAfterCancel = await settle();
  check("Cross-list keyboard cancellation restores both complete lists and counts", !equal(pinDuringCancel.pins, pinBeforeCancel.pins) && equal(pinAfterCancel.pins, pinBeforeCancel.pins) && clean(pinAfterCancel) && await pinSemantics(), { before: pinBeforeCancel.pins, during: pinDuringCancel.pins, after: pinAfterCancel.pins });
  const retained = await snapshot(); await go("foundations"); await go("primitives"); await settle(); const returned = await snapshot();
  check("Pointer and keyboard changes preserve both list states across navigation", equal(returned.order, retained.order) && equal(returned.pins, retained.pins) && returned.rows.every(row => row.original) && await pinSemantics(), { before: retained, after: returned });
}

async function reducedChecks() {
  await evaluate("Schematic.setMotion('reduced')"); await frames(3); await click("[data-pr-sort-reset]");
  const sort = await dragSort("audit", "handoff", "after", "sort-reduced-pointer");
  check("Reduced motion preserves direct pointer sorting and reaches the same final order", equal(sort.after.order, ["structure", "motion", "handoff", "audit"]) && sort.after.rows.every(row => row.original), sort.after);
  check("Reduced sorting has no animated row travel or lingering landing transition", sort.during.every(frame => frame.rows.every(row => near(row.x, 0, .01) && near(row.y, 0, .01))) && clean(sort.immediate) && sort.immediate.running.length === 0, { immediate: sort.immediate, travel: sort.during.map(frame => frame.rows.map(row => ({ key: row.key, x: row.x, y: row.y }))) });
  const before = await snapshot(), source = Object.keys(before.pins).find(group => before.pins[group].includes("tokens")), destination = source === "pinned" ? "unpinned" : "pinned";
  const pin = await dragPin("tokens", destination, "pin-reduced-pointer");
  check("Reduced motion also supports actual cross-group pointer pinning", pin.after.pins[destination].includes("tokens") && !pin.after.pins[source].includes("tokens") && clean(pin.immediate) && pin.immediate.running.length === 0 && await pinSemantics(), pin.after);
  await show("[data-pr-sort-list]"); await focus(sortHandle("audit")); await key(" ", "Space", 32); await key("Home", "Home", 36); await key("Enter", "Enter", 13);
  const keyboard = await snapshot();
  check("Reduced keyboard sorting reaches final geometry immediately and remains keyboard accessible", keyboard.order[0] === "audit" && clean(keyboard) && keyboard.rows.every(row => near(row.x, 0, .01) && near(row.y, 0, .01)) && await evaluate(`document.activeElement.matches('${sortHandle("audit")}')`), keyboard);
}

async function narrowChecks() {
  await b.size(390, 844); await evaluate("Schematic.setPalette('mono');Schematic.setTheme('dark');void 0"); await frames(3);
  await click("[data-pr-sort-reset]"); await evaluate("Schematic.setMotion('full')"); await settle();
  const trace = await dragSort("audit", "motion", "after", "sort-390-dark-mono");
  check("390px dark Mono supports actual held-mouse sorting on a narrow canvas", equal(trace.after.order, ["structure", "motion", "audit", "handoff"]) && clean(trace.after) && trace.after.rows.every(row => row.original), trace.after);
  const geometry = await evaluate("({width:innerWidth,document:document.documentElement.scrollWidth,viewport:document.querySelector('#viewport').clientWidth,scroll:document.querySelector('#viewport').scrollWidth,theme:document.documentElement.dataset.theme,palette:document.documentElement.dataset.palette})");
  check("Drag completion preserves the narrow viewport and selected appearance", geometry.width === 390 && geometry.document <= 391 && geometry.scroll <= geometry.viewport + 1 && geometry.theme === "dark" && geometry.palette === "mono", geometry);
  await b.size(1440, 1000); await evaluate("Schematic.setMotion('reduced');Schematic.setTheme('light');Schematic.setPalette('color');void 0"); await frames(3);
}

async function hiddenWorkChecks() {
  await evaluate("Schematic.setMotion('full')"); await settle();
  for (const [name, handle, root] of [["sortable", sortHandle("audit"), "[data-pr-sort-list]"], ["cross-group", pinHandle("tokens"), "[data-pr-pinned-list]"]]) {
    await show(root); await settle(); const before = await snapshot(), start = center(await box(handle));
    await press(start);
    const bounds = await box("#viewport");
    await move({ x: start.x, y: Math.min(995, bounds.y + bounds.height - 4) }, 12);
    await frames(8);
    const active = await snapshot(), work = await evaluate("sortableWorkQA.snapshot('primitives')");
    check(`${name}: lifecycle check interrupts a real pickup and active auto-scroll work`, active.active.includes("pointer") && work.pending > 0, { active, work });
    await go("foundations"); await release(); await delay(350);
    const hidden = await evaluate("({work:sortableWorkQA.snapshot('primitives'),hidden:document.querySelector('[data-page=primitives]').hidden,animations:document.querySelector('[data-page=primitives]').getAnimations({subtree:true}).filter(a=>['running','pending'].includes(a.playState)).length,state:sortableQA.snapshot()})");
    await delay(420); const later = await evaluate("sortableWorkQA.snapshot('primitives')");
    traces[`hidden-${name}`] = { before, active, work, hidden, later };
    check(`${name}: leaving rolls back the active drag and removes all drag artifacts`, hidden.hidden && equal(hidden.state.order, before.order) && equal(hidden.state.pins, before.pins) && clean(hidden.state), hidden);
    check(`${name}: hidden route has no recurring RAF callbacks or running animations`, hidden.work.pending === 0 && later.pending === 0 && hidden.work.callbacks === later.callbacks && hidden.animations === 0, { hidden: hidden.work, later, animations: hidden.animations });
    await go("primitives"); await settle();
  }
}

try {
  b = await browser();
  await b.send("Page.addScriptToEvaluateOnNewDocument", { source: `(${installWorkTracker.toString()})()` });
  await b.open(url); await go("primitives");
  await evaluate(`(${installProbe.toString()})()`);
  await group("Pointer drag", pointerChecks);
  await group("Keyboard drag", keyboardChecks);
  await group("Reduced drag", reducedChecks);
  await group("Narrow drag", narrowChecks);
  await group("Hidden drag work", hiddenWorkChecks);
  check("Sortable browser run has no runtime or network errors", b.failures.length === 0, b.failures);
} catch (error) { check("Sortable browser run completed", false, error.stack); }
finally { if (b) { await release().catch(() => {}); await b.close(); } }

const output = join(workspace, "scratch/qa/sortable.json");
await mkdir(dirname(output), { recursive: true });
await writeFile(output, JSON.stringify({ date: new Date().toISOString(), checks, traces }, null, 2));
const failed = checks.filter(result => !result.pass).length;
console.log(`${checks.length - failed} passed, ${failed} failed`);
process.exitCode = failed ? 1 : 0;
