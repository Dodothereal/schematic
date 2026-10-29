import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { browser, delay } from "./browser.mjs";
const checks = [];
const check = (name, pass, detail) => {
  checks.push({ name, pass: !!pass, detail });
  console.log(
    `${pass ? "PASS" : "FAIL"} ${name}${!pass ? " " + JSON.stringify(detail) : ""}`,
  );
};
const b = await browser({ width: 1440, height: 1000 });
const evaluate = (s) => b.evaluate(s);
const click = async (s) => {
  await evaluate(`document.querySelector(${JSON.stringify(s)}).click();void 0`);
};
const rect = async (s) =>
  evaluate(
    `(()=>{const r=document.querySelector(${JSON.stringify(s)}).getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height}})()`,
  );
try {
  await b.open(pathToFileURL(resolve("dist/schematic.html")).href);
  await delay(1300);
  check(
    "Opening exposes five inspectable components",
    await evaluate('document.querySelectorAll("[data-as-piece]").length===5'),
  );
  check(
    "Opening has no marketing paragraph or gradient headline",
    await evaluate(
      '!document.querySelector(".ov-opening p")&&getComputedStyle(document.querySelector(".ov-wordmark")).backgroundImage==="none"',
    ),
  );
  check(
    "Component assembly is visible in the first viewport",
    await evaluate(
      'document.querySelector("[data-as-stage]").getBoundingClientRect().bottom<innerHeight',
    ),
  );
  const exploded = await rect('[data-as-piece="chart"]');
  await click('[data-as-view="assembled"]');
  await delay(140);
  const intermediate = await rect('[data-as-piece="chart"]');
  await delay(1000);
  const assembled = await rect('[data-as-piece="chart"]');
  check(
    "Assemble has a rendered intermediate position",
    Math.abs(intermediate.x - exploded.x) > 2 &&
      Math.abs(intermediate.x - assembled.x) > 2,
    { exploded, intermediate, assembled },
  );
  check(
    "Assembled parts fit their application frame",
    await evaluate(
      `(()=>{const w=document.querySelector('.as-window').getBoundingClientRect();return [...document.querySelectorAll('[data-as-piece]')].every(el=>{const r=el.getBoundingClientRect();return r.left>=w.left-1&&r.right<=w.right+1&&r.top>=w.top&&r.bottom<=w.bottom+1})})()`,
    ),
  );
  await click('[data-as-view="wireframe"]');
  await delay(900);
  check(
    "Wireframe removes fills and exposes layout strokes",
    await evaluate(
      'getComputedStyle(document.querySelector(".as-piece")).backgroundColor==="rgba(0, 0, 0, 0)"&&getComputedStyle(document.querySelector(".as-piece")).borderTopStyle==="dashed"',
    ),
  );
  await click('[data-as-view="exploded"]');
  await delay(900);
  await evaluate('document.querySelector("[data-as-piece=metric]").focus()');
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Enter",
    code: "Enter",
    windowsVirtualKeyCode: 13,
    text: "\r",
    unmodifiedText: "\r",
  });
  await b.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "Enter",
    code: "Enter",
    windowsVirtualKeyCode: 13,
  });
  check(
    "Keyboard selection updates inspector and destination",
    await evaluate(
      'document.querySelector("[data-as-title]").textContent==="Metric"&&document.querySelector("[data-as-piece=metric]").getAttribute("aria-pressed")==="true"&&document.querySelector("[data-as-open]").getAttribute("href")==="#/data"',
    ),
  );
  await evaluate(
    '(()=>{const r=document.querySelector("[data-as-radius]");r.value="0";r.dispatchEvent(new Event("input",{bubbles:true}))})()',
  );
  await delay(400);
  check(
    "Radius control changes the actual component geometry",
    await evaluate(
      'getComputedStyle(document.querySelector("[data-as-piece=metric]")).borderTopLeftRadius==="0px"&&document.querySelector("[data-as-radius-value]").textContent==="0"',
    ),
  );
  await click("[data-as-replay]");
  await delay(260);
  const dash = await evaluate(
    'parseFloat(getComputedStyle(document.querySelector(".as-chart-line")).strokeDashoffset)',
  );
  check(
    "Replay traces the chart through a real intermediate frame",
    dash > 0 && dash < 1,
    dash,
  );
  await evaluate('Schematic.setMotion("reduced")');
  await delay(80);
  check(
    "Reduced motion settles active entrance and tracing",
    await evaluate(
      'document.querySelector("[data-assembly]").getAnimations({subtree:true}).filter(a=>a.playState==="running").length===0&&parseFloat(getComputedStyle(document.querySelector(".as-chart-line")).strokeDashoffset)===0',
    ),
  );
  await click('[data-as-view="assembled"]');
  check(
    "Reduced-motion view change applies immediately",
    await evaluate(
      'document.querySelector("[data-assembly]").dataset.view==="assembled"&&document.querySelector("[data-assembly]").getAnimations({subtree:true}).every(a=>a.playState!=="running")',
    ),
  );
  await click("[data-as-browse]");
  await delay(100);
  check(
    "Browse moves to the library and focuses search",
    await evaluate(
      'document.activeElement.matches("[data-catalog-search]")&&document.querySelector("[data-catalog-search]").getBoundingClientRect().top<150',
    ),
  );
  await evaluate(
    'document.querySelector("#viewport").scrollTop=0;Schematic.navigate("foundations");void 0',
  );
  await delay(100);
  await evaluate('Schematic.navigate("overview");void 0');
  await delay(100);
  check(
    "Assembly selection and radius survive route changes",
    await evaluate(
      'document.querySelector("[data-as-title]").textContent==="Metric"&&document.querySelector("[data-as-radius]").value==="0"',
    ),
  );
  await evaluate(
    'Schematic.setMotion("full");document.querySelector("#viewport").scrollTop=0',
  );
  await click("[data-as-replay]");
  await evaluate('Schematic.navigate("foundations");void 0');
  await delay(300);
  check(
    "Leaving the opening cancels owned entrance work",
    await evaluate(
      'document.querySelector("[data-assembly]").getAnimations({subtree:true}).filter(a=>a.playState==="running").length===0',
    ),
  );
  await evaluate(
    'Schematic.setMotion("reduced");Schematic.navigate("overview");void 0',
  );
  await delay(100);
  for (const width of [390, 320]) {
    await b.size(width, 844);
    await delay(250);
    await click('[data-as-view="exploded"]');
    await click('[data-as-select="chart"]');
    const stage = await rect("[data-as-stage]"),
      chart = await rect('[data-as-piece="chart"]');
    check(
      `${width}px shows a readable full-size selected component`,
      chart.width >= 260 &&
        chart.x >= stage.x &&
        chart.x + chart.width <= stage.x + stage.width + 1,
      { stage, chart },
    );
    check(
      `${width}px component picker keeps usable targets`,
      await evaluate(
        '[...document.querySelectorAll("[data-as-select]")].every(e=>e.getBoundingClientRect().height>=38)',
      ),
    );
    await click('[data-as-select="navigation"]');
    check(
      `${width}px picker changes component and announces selection`,
      await evaluate(
        'document.querySelector("[data-as-title]").textContent==="Navigation"&&document.querySelector("[data-as-status]").textContent.includes("Navigation selected")',
      ),
    );
    check(
      `${width}px opening has no horizontal overflow`,
      await evaluate(
        'document.querySelector(".pg-overview").scrollWidth<=document.querySelector("#viewport").clientWidth&&document.documentElement.scrollWidth<=innerWidth',
      ),
    );
    await b.screenshot(`scratch/qa/opening-${width}.png`);
  }
  await b.size(1440, 1000);
  await evaluate(
    'Schematic.setPalette("mono");Schematic.setTheme("dark");void 0',
  );
  await delay(150);
  await click('[data-as-view="wireframe"]');
  check(
    "Dark Mono preserves a visible assembly and component labels",
    await evaluate(
      'getComputedStyle(document.querySelector(".as-piece")).color!==getComputedStyle(document.querySelector(".as-surface")).backgroundColor&&document.querySelector("[data-as-title]").textContent==="Navigation"',
    ),
  );
  await b.screenshot("scratch/qa/opening-dark-mono.png");
  await evaluate('Schematic.navigate("workspace");void 0');
  await delay(100);
  check(
    "Immersive workspace uses full width and removes hidden library links from focus",
    await evaluate(
      'document.querySelector("[data-ws-app]").getBoundingClientRect().width>=innerWidth-20&&document.querySelector(".sidebar").inert&&document.querySelector(".sidebar").getBoundingClientRect().right<=0',
    ),
  );
  await click("#menu-btn");
  await delay(20);
  check(
    "Desktop workspace can reopen the library navigation",
    await evaluate(
      '!document.querySelector(".sidebar").inert&&document.querySelector(".sidebar").getBoundingClientRect().left===0&&!!document.activeElement.closest(".sidebar")',
    ),
  );
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Tab",
    code: "Tab",
    windowsVirtualKeyCode: 9,
    modifiers: 8,
  });
  await b.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "Tab",
    code: "Tab",
    windowsVirtualKeyCode: 9,
    modifiers: 8,
  });
  check(
    "Immersive navigation contains reverse Tab",
    await evaluate(
      '(()=>{const a=[...document.querySelector(".sidebar").querySelectorAll("a[href],button:not(:disabled),input:not(:disabled)")].filter(e=>e.offsetParent!==null);return document.activeElement===a.at(-1)})()',
    ),
  );
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await b.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await delay(20);
  check(
    "Immersive navigation Escape restores its trigger",
    await evaluate(
      'document.activeElement.id==="menu-btn"&&document.querySelector(".sidebar").inert',
    ),
  );
  await evaluate('Schematic.navigate("overview");void 0');
  await delay(100);
  check(
    "Returning to the library restores normal desktop navigation",
    await evaluate(
      '!document.querySelector(".sidebar").inert&&document.querySelector(".sidebar").getBoundingClientRect().left===0',
    ),
  );
  check(
    "Opening has no runtime, icon or network errors",
    b.failures.length === 0,
    b.failures,
  );
} catch (error) {
  check("Opening suite completed", false, error.stack);
} finally {
  await b.close();
}
await mkdir("scratch/qa", { recursive: true });
await writeFile("scratch/qa/assembly.json", JSON.stringify(checks, null, 2));
console.log(
  `${checks.filter((x) => x.pass).length} passed, ${checks.filter((x) => !x.pass).length} failed`,
);
if (checks.some((x) => !x.pass)) process.exitCode = 1;
