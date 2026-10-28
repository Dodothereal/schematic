#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { browser } from "./browser.mjs";

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const url = pathToFileURL(join(workspace, "dist/schematic.html")).href;
const checks = [], traces = {};
const check = (name, pass, detail) => {
  checks.push({ name, pass: !!pass, ...(detail === undefined ? {} : { detail }) });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail === undefined ? "" : `: ${JSON.stringify(detail)}`}`);
};
const near = (a, b, tolerance = 0.6) => Number.isFinite(a) && Number.isFinite(b) && Math.abs(a - b) <= tolerance;
const rectNear = (a, b, tolerance = 0.6) => a && b && ["x", "y", "width", "height"].every((key) => near(a[key], b[key], tolerance));
const allFrames = (trace) => [trace.immediate, ...trace.frames];
const lastFrame = (trace) => trace.frames.at(-1) || trace.immediate;
const distinct = (values) => new Set(values.map((value) => Math.round(value * 1000))).size;
const intermediate = (values) => values.filter((value) => value > 0.02 && value < 0.98);
const bounded = (rect, viewport) => rect && rect.x >= -0.6 && rect.y >= -0.6
  && rect.x + rect.width <= viewport.width + 0.6 && rect.y + rect.height <= viewport.height + 0.6;
const settledOn = (frame, step) => frame.open && frame.active === step && frame.activeCount === 1
  && frame.panels.every((panel, i) => i === step
    ? !panel.inert && panel.ariaHidden === "false" && near(panel.opacity, 1, 0.001) && near(panel.x, 0) && near(panel.y, 0)
    : panel.inert && panel.ariaHidden === "true" && panel.visibility === "hidden" && near(panel.opacity, 0, 0.001))
  && frame.running === 0 && frame.headingInActive && frame.focusInActive;

// This code runs in the page. Samples come from real requestAnimationFrame
// callbacks, without pausing animations or guessing one exact wall-clock frame.
function installProbe() {
  const dialog = document.querySelector("#schematic-guide");
  const panels = [...dialog.querySelectorAll(".onboarding-stage > .onboarding-panel")];
  if (panels.length !== 4) throw new Error("Expected four retained onboarding panels. Rebuild dist before running the motion checks.");
  const rect = (element) => {
    const r = element?.getBoundingClientRect();
    return r ? { x: r.x, y: r.y, width: r.width, height: r.height } : null;
  };
  const transform = (style) => {
    const m = new DOMMatrixReadOnly(style.transform === "none" ? undefined : style.transform);
    return { x: m.m41, y: m.m42, scaleX: m.m11, scaleY: m.m22 };
  };
  const visual = (element, pseudo) => {
    const style = getComputedStyle(element, pseudo);
    return { opacity: Number(style.opacity), ...transform(style), visibility: style.visibility };
  };
  const running = () => dialog.getAnimations({ subtree: true }).filter((animation) =>
    ["running", "pending"].includes(animation.playState) && animation.effect?.getTiming().iterations !== Infinity).length;
  const controlKey = (element) => element.classList.contains("onboarding-body") ? "scroll-region"
    : element.hasAttribute("data-guide-close") ? "close"
    : element.hasAttribute("data-guide-back") ? "back"
      : element.hasAttribute("data-guide-next") ? "next"
        : element.hasAttribute("data-guide-motion") ? "motion"
          : element.dataset.value || element.id || element.textContent.trim();
  const controls = () => [...dialog.querySelectorAll("button, [href], input, select, textarea, [tabindex]")].filter((element) =>
    !element.disabled && element.tabIndex >= 0 && !element.closest("[inert], [aria-hidden='true']")
    && element.getClientRects().length && getComputedStyle(element).visibility === "visible");
  const frame = () => {
    const active = panels.findIndex((panel) => panel.hasAttribute("data-active"));
    const style = visual(dialog), focus = document.activeElement;
    return {
      t: performance.now(), open: dialog.open, ...style, dialog: rect(dialog),
      footer: rect(dialog.querySelector(".onboarding-foot")), next: rect(dialog.querySelector("[data-guide-next]")),
      viewport: { width: innerWidth, height: innerHeight }, running: running(),
      backdrop: Number(getComputedStyle(dialog, "::backdrop").opacity), active,
      activeCount: panels.filter((panel) => panel.hasAttribute("data-active")).length,
      count: dialog.querySelector("[data-guide-count]").textContent,
      headingInActive: active >= 0 && panels[active].contains(document.querySelector("#onboarding-title")),
      focusInActive: active >= 0 && panels[active].contains(focus),
      focusInGuide: dialog.contains(focus) && !focus.closest("[inert], [aria-hidden='true']"),
      focusKey: controlKey(focus),
      panels: panels.map((panel) => ({ ...visual(panel), rect: rect(panel), inert: panel.inert, ariaHidden: panel.getAttribute("aria-hidden") })),
      fills: [...dialog.querySelectorAll(".onboarding-progress li")].map((item) => visual(item, "::after").scaleX),
      marker: visual(dialog.querySelector(".onboarding-motion-marker")),
    };
  };
  const raf = () => new Promise((resolve) => requestAnimationFrame(resolve));
  const frames = async (count = 1) => { for (let i = 0; i < count; i++) await raf(); };
  const until = async (predicate, limit = 2200) => {
    const started = performance.now();
    do {
      await raf();
      const sample = frame();
      if (predicate(sample)) return sample;
    } while (performance.now() - started < limit);
    throw new Error("No requested intermediate onboarding frame was observed before the timeout.");
  };
  const trace = async (action = () => {}, limit = 2400) => {
    const before = frame();
    action(); // Do not await close(): its intermediate frames are the subject.
    const immediate = frame(), samples = [];
    let idle = 0;
    do {
      await raf();
      const sample = frame();
      samples.push(sample);
      idle = sample.running === 0 ? idle + 1 : 0;
      if (idle >= 4 && samples.length >= 5) return { before, immediate, frames: samples };
    } while (performance.now() - immediate.t < limit);
    throw new Error(`Onboarding did not settle within ${limit}ms (${running()} animations remain).`);
  };
  window.guideMotionQA = {
    frame, frames, until, trace, controlKey,
    next: () => dialog.querySelector("[data-guide-next]").click(),
    back: () => dialog.querySelector("[data-guide-back]").click(),
    controls: () => controls().map(controlKey),
    focusClose: () => dialog.querySelector("[data-guide-close]").focus(),
    focusNext: () => dialog.querySelector("[data-guide-next]").focus(),
    focusVisible: () => {
      const element = document.activeElement;
      const bounds = rect(element), body = dialog.querySelector(".onboarding-body");
      const clip = element.closest(".onboarding-body") ? rect(body) : rect(dialog);
      return dialog.contains(element) && !element.closest("[inert], [aria-hidden='true']")
        && bounds.width > 0 && bounds.height > 0 && bounds.x >= clip.x - 0.6
        && bounds.x + bounds.width <= clip.x + clip.width + 0.6
        && bounds.y >= clip.y - 0.6 && bounds.y + bounds.height <= clip.y + clip.height + 0.6;
    },
  };
}

const install = (b) => b.evaluate(`(${installProbe.toString()})()`);
const trace = (b, expression) => b.evaluate(`guideMotionQA.trace(()=>{${expression}})`);
const key = async (b, key, code, keyCode, modifiers = 0) => {
  const params = { key, code, windowsVirtualKeyCode: keyCode, modifiers };
  await b.send("Input.dispatchKeyEvent", { type: "keyDown", ...params });
  await b.send("Input.dispatchKeyEvent", { type: "keyUp", ...params });
};

function checkNavigation(name, transition, step) {
  const frames = allFrames(transition), incoming = frames.map((frame) => frame.panels[step]);
  check(`${name}: reading surface has multiple intermediate opacity and travel frames`,
    distinct(intermediate(incoming.map((panel) => panel.opacity))) >= 3
      && incoming.some((panel) => panel.opacity > 0.05 && panel.opacity < 0.95 && Math.abs(panel.x) > 0.5));
  check(`${name}: dialog, footer and Next bounds remain anchored`, frames.every((frame) =>
    rectNear(transition.before.dialog, frame.dialog) && rectNear(transition.before.footer, frame.footer)
      && rectNear(transition.before.next, frame.next)));
  check(`${name}: incoming and outgoing prose do not overlap at readable opacity`, frames.every((frame) =>
    frame.panels.filter((panel) => panel.visibility === "visible" && panel.opacity > 0.18).length <= 1));
  check(`${name}: settles on one accessible panel with no animation leftovers`, settledOn(lastFrame(transition), step));
}

async function reversal(b, phase) {
  const result = await b.evaluate(`(async()=>{
    const q=guideMotionQA;
    q.next();
    await q.until(f=>f.panels[${phase === "outgoing" ? 0 : 1}].opacity>0.2&&f.panels[${phase === "outgoing" ? 0 : 1}].opacity<0.85);
    const before=q.frame();
    q.back();
    const after=q.frame();
    return {before,after,settle:await q.trace()};
  })()`);
  traces[`reverse-${phase}`] = result;
  const visible = result.before.panels.map((panel, index) => ({ panel, index })).filter(({ panel }) => panel.opacity > 0.02 && panel.visibility === "visible");
  const continuous = visible.length > 0 && visible.every(({ panel, index }) => {
    const after = result.after.panels[index];
    return near(panel.opacity, after.opacity, 0.025) && near(panel.x, after.x, 0.6)
      && near(panel.y, after.y, 0.6) && rectNear(panel.rect, after.rect);
  });
  check(`Reverse during ${phase} motion continues from the displayed frame`, continuous, {
    visiblePanels: visible.map(({ index }) => index),
    opacityJump: Math.max(...visible.map(({ panel, index }) => Math.abs(panel.opacity - result.after.panels[index].opacity))),
    xJump: Math.max(...visible.map(({ panel, index }) => Math.abs(panel.x - result.after.panels[index].x))),
  });
  check(`Reverse during ${phase} motion returns to Explore and releases hidden panels`, settledOn(lastFrame(result.settle), 0));
}

async function mobileChecks(b, width, height) {
  await trace(b, "Schematic.onboarding.close()");
  await b.size(width, height);
  const opening = await trace(b, "document.querySelector('#guide-btn').focus();Schematic.onboarding.open()");
  const anchor = lastFrame(opening);
  let geometry = true, keyboard = true, hiddenSkipped = true;
  const details = [];
  for (let step = 0; step < 4; step++) {
    const current = await b.evaluate("guideMotionQA.frame()");
    geometry &&= bounded(current.dialog, current.viewport) && bounded(current.footer, current.viewport)
      && bounded(current.next, current.viewport) && rectNear(anchor.dialog, current.dialog)
      && rectNear(anchor.footer, current.footer) && rectNear(anchor.next, current.next);
    const expected = await b.evaluate("guideMotionQA.controls()");
    await b.evaluate("guideMotionQA.focusClose()");
    const visited = [];
    for (let index = 0; index <= expected.length; index++) {
      const focus = await b.evaluate("({frame:guideMotionQA.frame(),visible:guideMotionQA.focusVisible()})");
      visited.push(focus.frame.focusKey);
      keyboard &&= focus.visible;
      hiddenSkipped &&= focus.frame.focusInGuide;
      await key(b, "Tab", "Tab", 9);
    }
    keyboard &&= expected.every((control) => visited.includes(control));
    details.push({ step: step + 1, expected, visited });
    if (step < 3) {
      const change = await trace(b, "guideMotionQA.next()");
      traces[`mobile-${width}-step-${step + 2}`] = change;
      geometry &&= allFrames(change).every((frame) => bounded(frame.dialog, frame.viewport)
        && bounded(frame.footer, frame.viewport) && rectNear(anchor.dialog, frame.dialog)
        && rectNear(anchor.footer, frame.footer) && rectNear(anchor.next, frame.next));
    }
  }
  check(`${width}x${height}: all four steps keep dialog and navigation inside the viewport`, geometry);
  check(`${width}x${height}: keyboard reaches every current control and scrolls it into view`, keyboard, details);
  check(`${width}x${height}: Tab never enters a retained inactive panel or the page behind`, hiddenSkipped);
  await b.evaluate("guideMotionQA.focusNext()");
  await key(b, "Tab", "Tab", 9);
  const forwardWrap = await b.evaluate("document.activeElement.hasAttribute('data-guide-close')");
  await key(b, "Tab", "Tab", 9, 8);
  check(`${width}x${height}: forward and reverse Tab wrap inside the modal`, forwardWrap
    && await b.evaluate("document.activeElement.hasAttribute('data-guide-next')"));
}

async function reducedChecks(b, name) {
  const opening = await trace(b, "document.querySelector('#guide-btn').focus();Schematic.onboarding.open()");
  traces[`${name}-open`] = opening;
  check(`${name}: guide opens immediately without surface or backdrop travel`, allFrames(opening).every((frame) =>
    frame.open && near(frame.opacity, 1, 0.001) && near(frame.x, 0) && near(frame.y, 0)
      && near(frame.scaleX, 1, 0.001) && near(frame.scaleY, 1, 0.001) && near(frame.backdrop, 1, 0.001)));
  const transitions = [];
  for (let step = 1; step <= 2; step++) transitions.push(await trace(b, "guideMotionQA.next()"));
  traces[`${name}-steps`] = transitions;
  check(`${name}: navigation changes state immediately with no intermediate travel`, transitions.every((transition, index) =>
    allFrames(transition).every((frame) => settledOn(frame, index + 1))));
  const sample = await trace(b, "document.querySelector('[data-guide-motion]').click()");
  traces[`${name}-sample`] = sample;
  check(`${name}: motion sample remains operable but changes position without travel`,
    Math.abs(sample.before.marker.x - sample.immediate.marker.x) > 20
      && allFrames(sample).every((frame) => near(frame.marker.x, sample.immediate.marker.x, 0.05))
      && await b.evaluate("document.querySelector('[data-guide-motion]').getAttribute('aria-pressed')==='true'"));
  const closing = await trace(b, "Schematic.onboarding.close()");
  traces[`${name}-close`] = closing;
  check(`${name}: dismissal is immediate and returns focus to Guide`, !closing.immediate.open
    && await b.evaluate("document.activeElement.id==='guide-btn'"));
}

let b;
try {
  b = await browser();
  await b.open(url);
  await install(b);
  await b.evaluate("Schematic.setMotion('full')");
  const opening = traces.open = await trace(b, "document.querySelector('#guide-btn').focus();Schematic.onboarding.open()");
  const openFrames = allFrames(opening);
  check("Normal opening paints several real intermediate surface frames", distinct(intermediate(openFrames.map((frame) => frame.opacity))) >= 3
    && openFrames.some((frame) => frame.opacity > 0.1 && frame.opacity < 0.9 && Math.abs(frame.y) > 0.1));
  check("Normal opening fades the backdrop rather than snapping it on", distinct(intermediate(openFrames.map((frame) => frame.backdrop))) >= 3
    && near(lastFrame(opening).backdrop, 1, 0.001));
  check("Opening settles with focus on the active step heading", settledOn(lastFrame(opening), 0));

  const forward = traces.forward = await trace(b, "guideMotionQA.next()");
  checkNavigation("Forward navigation", forward, 1);
  check("Progress fill advances through multiple intermediate scales", distinct(intermediate(allFrames(forward).map((frame) => frame.fills[1]))) >= 3
    && near(lastFrame(forward).fills[1], 1, 0.001));
  const backward = traces.backward = await trace(b, "guideMotionQA.back()");
  checkNavigation("Backward navigation", backward, 0);
  check("Back uses the reverse direction of travel", allFrames(forward).some((frame) => frame.panels[1].opacity > 0.02 && frame.panels[1].x > 0.5)
    && allFrames(backward).some((frame) => frame.panels[0].opacity > 0.02 && frame.panels[0].x < -0.5));
  await reversal(b, "outgoing");
  await reversal(b, "incoming");

  const rapid = traces.rapid = await b.evaluate(`(async()=>{
    const q=guideMotionQA, sequence=['next','next','back','next','back','next','next','back','next','back'];
    const frames=[];
    for(const action of sequence){q[action]();await q.frames();frames.push(q.frame());}
    return {sequence,frames,settle:await q.trace()};
  })()`);
  check("Rapid alternating navigation obeys the latest intent", settledOn(lastFrame(rapid.settle), 2)
    && rapid.frames.every((frame) => frame.activeCount === 1));
  check("Rapid navigation leaves the frame and actions anchored", [...rapid.frames, ...allFrames(rapid.settle)].every((frame) =>
    rectNear(opening.frames.at(-1).dialog, frame.dialog) && rectNear(opening.frames.at(-1).footer, frame.footer)
      && rectNear(opening.frames.at(-1).next, frame.next)));

  const reopening = traces.reopen = await b.evaluate(`(async()=>{
    const q=guideMotionQA;
    Schematic.onboarding.close();
    await q.until(f=>f.open&&f.opacity>0.2&&f.opacity<0.9);
    const before=q.frame();
    Schematic.onboarding.open();
    const after=q.frame(),settle=await q.trace();
    await q.frames(12);
    return {before,after,settle,later:q.frame()};
  })()`);
  check("Reopening during exit preserves the displayed modal and backdrop frame", near(reopening.before.opacity, reopening.after.opacity, 0.025)
    && near(reopening.before.backdrop, reopening.after.backdrop, 0.025) && rectNear(reopening.before.dialog, reopening.after.dialog));
  check("Cancelled dismissal cannot close a reopened guide later", settledOn(lastFrame(reopening.settle), 0) && settledOn(reopening.later, 0));
  const closing = traces.close = await trace(b, "Schematic.onboarding.close()");
  check("Normal dismissal fades both modal and backdrop through intermediate frames", distinct(intermediate(allFrames(closing).map((frame) => frame.opacity))) >= 3
    && distinct(intermediate(allFrames(closing).map((frame) => frame.backdrop))) >= 3);
  check("Dismissal restores focus only after the dialog leaves the top layer", closing.immediate.open && !lastFrame(closing).open
    && await b.evaluate("document.activeElement.id==='guide-btn'"));

  await mobileChecks(b, 390, 844);
  await mobileChecks(b, 320, 568);
  await trace(b, "Schematic.onboarding.close()");
  await b.size(1440, 1000);
  await b.evaluate("Schematic.setMotion('reduced')");
  await b.evaluate("guideMotionQA.frames(2)");
  await reducedChecks(b, "Explicit Reduced");
  check("Normal-speed, mobile and explicit Reduced runs have no runtime errors", b.failures.length === 0, b.failures.length ? b.failures : undefined);
  await b.close(); b = null;

  b = await browser({ reduced: true });
  await b.open(url);
  await install(b);
  check("OS Reduced run starts without an explicit Full override", await b.evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches&&document.documentElement.dataset.motion!=='full'&&Schematic.reduced()"));
  await reducedChecks(b, "OS Reduced");
  check("OS Reduced run has no runtime errors", b.failures.length === 0, b.failures.length ? b.failures : undefined);
} catch (error) {
  check("Motion browser run completed", false, error.stack);
} finally {
  if (b) await b.close();
}

const output = join(workspace, "scratch/qa/onboarding-motion.json");
await mkdir(dirname(output), { recursive: true });
await writeFile(output, JSON.stringify({ date: new Date().toISOString(), checks, traces }, null, 2));
const failed = checks.filter((result) => !result.pass).length;
console.log(`${checks.length - failed} passed, ${failed} failed`);
process.exitCode = failed ? 1 : 0;
