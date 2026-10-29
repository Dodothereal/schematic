import { browser, delay } from "./browser.mjs";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";
import { writeFile } from "node:fs/promises";
const b = await browser({ width: 1440, height: 1000 }),
  checks = [];
const ev = (code) => b.evaluate(code),
  check = (name, pass, detail) => {
    checks.push({ name, pass: !!pass, detail });
    console.log(
      `${pass ? "PASS" : "FAIL"} ${name}${!pass ? " " + JSON.stringify(detail) : ""}`,
    );
  };
const click = async (s) => {
  await ev(`document.querySelector(${JSON.stringify(s)}).click();void 0`);
};
const reveal = async (s) => {
  await ev(
    `document.querySelector(${JSON.stringify(s)}).scrollIntoView({block:'center'});void 0`,
  );
  await delay(140);
};
const key = async (key, code, n) => {
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key,
    code,
    windowsVirtualKeyCode: n,
    ...(key === "Enter" ? { text: "\r" } : {}),
  });
  await b.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key,
    code,
    windowsVirtualKeyCode: n,
  });
};
try {
  await b.open(pathToFileURL(resolve("dist/schematic.html")).href);
  await delay(1100);
  check(
    "Opening uses v3 library structure with six working previews and no assembly",
    await ev(
      'document.querySelectorAll("[data-gallery-kind]").length===6&&!document.querySelector("[data-assembly]")&&document.querySelector("[data-catalog-search]").getBoundingClientRect().top<250',
    ),
  );
  await click("[data-gallery-action]");
  await delay(230);
  const progress = await ev(
    'new DOMMatrixReadOnly(getComputedStyle(document.querySelector("[data-gallery-progress]")).transform).m11',
  );
  check(
    "Action shows real pending progress and blocks duplicate activation",
    progress > 0 &&
      progress < 1 &&
      (await ev(
        'document.querySelector("[data-gallery-action]").disabled&&document.querySelector("[data-gallery-action-status]").textContent==="Running"',
      )),
    progress,
  );
  await delay(1250);
  check(
    "Action resolves and becomes replayable",
    await ev(
      'document.querySelector("[data-gallery-action]").dataset.state==="complete"&&!document.querySelector("[data-gallery-action]").disabled',
    ),
  );
  await click("[data-gallery-action]");
  await click('[data-catalog-filter="data"]');
  await delay(1450);
  check(
    "Filtering away from pending work cancels it without false success",
    await ev(
      '!document.querySelector("[data-gallery-action]").dataset.state&&document.querySelector("[data-gallery-action-caption]").textContent==="Cancelled"',
    ),
  );
  check(
    "Filters hide entire preview cards without leaving blank tiles",
    await ev(
      `[...document.querySelectorAll('[data-gallery-card]')].every(el=>el.hidden===el.querySelector('[data-catalog-id]').hidden)&&!document.querySelector('[data-gallery-card="data-0"]').hidden&&document.querySelector('[data-gallery-card="components-2"]').hidden`,
    ),
  );
  await reveal("[data-gallery-chart]");
  await click('[data-gallery-range="30"]');
  await delay(170);
  const trace = await ev(
    '({dash:parseFloat(getComputedStyle(document.querySelector("[data-gallery-line]")).strokeDashoffset),clip:getComputedStyle(document.querySelector("[data-gallery-area]")).clipPath})',
  );
  check(
    "Range changes trace final chart geometry and reveal its area",
    trace.dash > 0 &&
      trace.dash < 1 &&
      trace.clip !== "none" &&
      !trace.clip.includes("100%"),
    trace,
  );
  await delay(1050);
  const firstPoint = await ev(
    "(()=>{const r=document.querySelector('[data-gallery-chart]').getBoundingClientRect();return{x:r.left+r.width*8/300,y:r.top+r.height*.8}})()",
  );
  await b.send("Input.dispatchMouseEvent", {
    type: "mouseMoved",
    ...firstPoint,
  });
  check(
    "Pointer inspection maps the chart inset to the first data point",
    await ev(
      'document.querySelector("[data-gallery-chart-caption]").textContent==="Day 1 of 30"',
    ),
  );
  await ev('document.querySelector("[data-gallery-chart]").focus()');
  await key("End", "End", 35);
  check(
    "Keyboard chart inspection exposes the selected day and marker",
    await ev(
      'document.querySelector("[data-gallery-chart-caption]").textContent==="Day 30 of 30"&&!document.querySelector("[data-gallery-inspect]").hasAttribute("hidden")',
    ),
  );
  await click('[data-gallery-range="7"]');
  await delay(100);
  await click('[data-gallery-range="30"]');
  await click('[data-gallery-range="7"]');
  await delay(1100);
  check(
    "Interrupted chart draws settle on the last period and correct total",
    await ev(
      'document.querySelector("[data-gallery-chart-value]").textContent==="$4,190"&&document.querySelector("[data-gallery-chart-caption]").textContent==="7 days"&&parseFloat(getComputedStyle(document.querySelector("[data-gallery-line]")).strokeDashoffset)===0',
    ),
  );
  await click("[data-catalog-clear]");
  await reveal("[data-gallery-sort]");
  await ev(
    `document.querySelector('[data-gallery-task="0"] [data-sortable-handle]').focus()`,
  );
  await key(" ", "Space", 32);
  await key("ArrowDown", "ArrowDown", 40);
  await key(" ", "Space", 32);
  await delay(550);
  check(
    "Preview sorting moves original rows and preserves grip focus",
    await ev(
      '[...document.querySelectorAll("[data-gallery-task]")].map(e=>e.dataset.galleryTask).join() === "1,0,2"&&document.activeElement.closest("[data-gallery-task]").dataset.galleryTask==="0"',
    ),
  );
  await click('[data-gallery-task="1"] input');
  check(
    "Preview checklist uses real state and updates its count",
    await ev(
      'document.querySelector("[data-gallery-task-count]").textContent==="2 / 3"',
    ),
  );
  await reveal(".ov-gallery-tabs");
  await click('.ov-gallery-tabs [data-value="2"]');
  await delay(400);
  await click('[data-gallery-panel="2"] input');
  await click('.ov-gallery-tabs [data-value="1"]');
  await delay(400);
  check(
    "Tabs reveal the selected content and hide inactive controls",
    await ev(
      `!document.querySelector('[data-gallery-panel="1"]').hidden&&document.querySelector('[data-gallery-panel="2"]').hidden`,
    ),
  );
  await click('.ov-gallery-tabs [data-value="2"]');
  check(
    "Settings retain input state through tab changes",
    await ev(
      `!document.querySelector('[data-gallery-panel="2"] input').checked`,
    ),
  );
  await reveal("[data-gallery-counter]");
  await ev(`document.querySelector('[data-gallery-step="1"]').focus()`);
  await key(" ", "Space", 32);
  await delay(100);
  check(
    "Counter responds to Space and rolls through an intermediate frame",
    await ev(
      'document.querySelector("[data-gallery-counter]").getAttribute("aria-label")==="025"&&document.querySelector("[data-gallery-counter]").getAnimations({subtree:true}).some(a=>a.playState==="running")',
    ),
  );
  await delay(700);
  await click("[data-gallery-counter-reset]");
  await delay(700);
  check(
    "Counter reset restores its actual value",
    await ev(
      'document.querySelector("[data-gallery-counter]").getAttribute("aria-label")==="024"',
    ),
  );
  await reveal('[data-gallery-icon="send"]');
  await ev('document.querySelector("[data-gallery-icon=send]").focus()');
  await delay(80);
  check(
    "Gallery icons use their own animation on keyboard focus",
    await ev(
      'document.querySelector("[data-gallery-icon=send] svg").classList.contains("is-anim")&&document.querySelector("[data-gallery-icon-name]").textContent==="send"',
    ),
  );
  await ev('Schematic.setMotion("reduced")');
  await reveal("[data-gallery-action]");
  await click("[data-gallery-action]");
  await delay(220);
  check(
    "Reduced motion keeps asynchronous operation timing intact",
    await ev('document.querySelector("[data-gallery-action]").disabled'),
  );
  await ev('Schematic.navigate("foundations");void 0');
  await delay(160);
  check(
    "Leaving the page cancels work and all owned preview animations",
    await ev(
      '!document.querySelector("[data-gallery-action]").disabled&&document.querySelector("[data-gallery-action-caption]").textContent==="Cancelled"&&[...document.querySelectorAll(".ov-live-preview")].every(el=>el.getAnimations({subtree:true}).every(a=>a.playState!=="running"))',
    ),
  );
  await ev('Schematic.navigate("overview");void 0');
  await delay(200);
  check(
    "Component state survives navigation independently",
    await ev(
      `[...document.querySelectorAll('[data-gallery-task]')].map(e=>e.dataset.galleryTask).join()==="1,0,2"&&document.querySelector('[data-gallery-task-count]').textContent==="2 / 3"&&!document.querySelector('[data-gallery-panel="2"] input').checked`,
    ),
  );
  for (const width of [390, 320]) {
    await b.size(width, 844);
    await delay(180);
    await ev('document.querySelector("#viewport").scrollTop=0');
    check(
      `${width}px previews and controls stay within the reading canvas`,
      await ev(
        'document.querySelector(".pg-overview").scrollWidth<=document.querySelector("#viewport").clientWidth&&document.documentElement.scrollWidth<=innerWidth',
      ),
    );
    await b.screenshot(`scratch/qa/gallery-${width}.png`);
  }
  await b.size(1440, 1000);
  await ev('Schematic.setPalette("mono");Schematic.setTheme("dark");void 0');
  await delay(120);
  await b.screenshot("scratch/qa/gallery-dark.png");
  check(
    "Dark Mono retains readable preview controls and colored chart series",
    await ev(
      'getComputedStyle(document.querySelector("[data-gallery-action]")).color!==getComputedStyle(document.querySelector("[data-gallery-action]")).backgroundColor&&getComputedStyle(document.querySelector("[data-gallery-line]")).stroke!=="none"',
    ),
  );
  check(
    "Live previews have no runtime, icon or network errors",
    b.failures.length === 0,
    b.failures,
  );
} catch (error) {
  check("Gallery checks completed", false, error.stack);
} finally {
  await b.close();
}
await writeFile(
  "scratch/qa/gallery-results.json",
  JSON.stringify(checks, null, 2),
);
console.log(
  `${checks.filter((x) => x.pass).length} passed, ${checks.filter((x) => !x.pass).length} failed`,
);
if (checks.some((x) => !x.pass)) process.exitCode = 1;
