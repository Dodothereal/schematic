#!/usr/bin/env node
import { browser, delay } from "./browser.mjs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { mkdir, writeFile } from "node:fs/promises";
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), ".."),
  b = await browser(),
  results = [];
const ev = (s) => b.evaluate(s),
  scope = ".pg-workspace ";
const click = async (selector) => {
  await ev(
    `document.querySelector(${JSON.stringify(scope + selector)}).click();void 0`,
  );
  await delay(30);
};
const fill = async (selector, value) =>
  ev(
    `(()=>{const e=document.querySelector(${JSON.stringify(scope + selector)});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`,
  );
const check = async (name, expression) => {
  const pass =
    typeof expression === "boolean" ? expression : await ev(expression);
  results.push({ name, pass: pass === true });
  console.log(`${pass === true ? "PASS" : "FAIL"} ${name}`);
};
const key = async (key, code, n, modifiers = 0) => {
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key,
    code,
    windowsVirtualKeyCode: n,
    modifiers,
    ...(key === "Enter" ? { text: "\r", unmodifiedText: "\r" } : {}),
  });
  await b.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key,
    code,
    windowsVirtualKeyCode: n,
    modifiers,
  });
};
const done = async () => {
  const end = Date.now() + 8000;
  while (Date.now() < end) {
    if (await ev("document.querySelector('[data-ws-stop]').hidden")) return;
    await delay(80);
  }
  throw Error("Workspace task did not settle");
};
const tab = async (name) => click(`[data-ws-tab="${name}"]`);
try {
  await b.open(pathToFileURL(join(workspace, "dist/schematic.html")).href);
  await ev("Schematic.setMotion('reduced');Schematic.navigate('workspace')");
  await delay(70);
  await check(
    "Workspace is one discoverable complete app",
    "Schematic.catalog().some(e=>e.page==='workspace')&&document.querySelectorAll('.pg-workspace .ws-app').length===1&&document.querySelectorAll('[data-ws-threads] [data-ws-thread]').length===3",
  );
  await check(
    "Initial failed check blocks merging",
    "document.querySelector('[data-ws-merge]').disabled&&document.querySelector('[data-ws-status]').textContent.includes('failed')",
  );
  await tab("checks");
  await check(
    "Check rows and failure logs identify the actual failure",
    "document.querySelectorAll('.ws-check-row').length===3&&document.querySelectorAll('.ws-check-row[data-status=failed]').length===1&&document.querySelector('[data-ws-log]').textContent.includes('defaultPrevented')&&document.querySelector('[data-ws-log]').textContent.includes('code 1')",
  );
  await click("[data-ws-check=types]");
  await check(
    "Selecting a check shows its own command and output",
    "document.querySelector('[data-ws-log]').textContent.includes('typecheck')&&document.querySelector('[data-ws-log]').textContent.includes('No type errors')",
  );
  await tab("changes");
  await check(
    "Diff contains real context and changes with line numbers",
    "document.querySelectorAll('.ws-diff-context').length>1&&document.querySelectorAll('.ws-diff-added').length>0&&document.querySelectorAll('.ws-diff-removed').length>0&&!document.querySelector('.ws-diff').textContent.includes('event.preventDefault();')",
  );
  await click('[data-ws-file="1"]');
  await check(
    "File selection changes the diff",
    "document.querySelector('.ws-diff-heading').textContent.includes('search.test.ts')",
  );
  await tab("pr");
  await click("[data-ws-approve]");
  await check(
    "Approval cannot bypass a failed check",
    "document.querySelector('[data-ws-merge]').disabled&&document.querySelector('[data-ws-approve]').textContent.includes('Revoke')",
  );
  await tab("checks");
  await click("[data-ws-run-checks]");
  await check(
    "Rerun exposes a running task and Stop control",
    "!document.querySelector('[data-ws-stop]').hidden&&document.querySelector('[data-ws-run-checks]').disabled",
  );
  await done();
  await check(
    "Rerunning without a fix preserves the failed check",
    "document.querySelectorAll('.ws-check-row[data-status=failed]').length===1&&[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent.includes('failed')",
  );
  await click('.ws-suggestions [data-ws-prompt="Fix the failed check"]');
  await delay(100);
  const partial = await ev(
    "[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent",
  );
  await delay(120);
  await check(
    "Assistant text advances during generation",
    await ev(
      `[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent.length>${partial.length}`,
    ),
  );
  await done();
  await tab("checks");
  await check(
    "Repair runs all three checks successfully",
    "document.querySelectorAll('.ws-check-row[data-status=passed]').length===3&&!document.querySelector('.ws-check-row[data-status=failed]')",
  );
  await tab("changes");
  await click('[data-ws-file="0"]');
  await check(
    "Repair changes the displayed patch rather than only the badge",
    "[...document.querySelectorAll('.ws-diff-added')].some(e=>e.textContent.includes('event.preventDefault();'))",
  );
  await tab("pr");
  await check(
    "Code changes invalidate the preceding approval",
    "document.querySelector('[data-ws-approve]').textContent.includes('Approve')&&document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-approve]");
  await check(
    "Passing checks and approval enable Merge",
    "!document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-merge]");
  await check(
    "Merge opens a concrete confirmation with branch and checks",
    "document.querySelector('[data-ws-confirm]').open&&document.querySelector('[data-ws-merge-summary]').textContent.includes('fix/search-shortcut')&&document.querySelector('[data-ws-merge-summary]').textContent.includes('3 checks passed')",
  );
  await click("[data-ws-cancel-merge]");
  await check(
    "Cancelling merge preserves the open PR",
    "!document.querySelector('[data-ws-confirm]').open&&!document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-merge]");
  await click("[data-ws-confirm-merge]");
  await check(
    "Confirmed merge updates the PR, branch and conversation",
    "document.querySelector('[data-ws-merge]').disabled&&document.querySelector('[data-ws-merge]').textContent.includes('Merged')&&document.querySelector('[data-ws-branch]').textContent.trim()==='main'&&[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent.includes('Merged #128')",
  );
  await tab("activity");
  await check(
    "Activity records repair, approval and merge events",
    "['Applied file changes','Approved by you','Merged #128'].every(s=>document.querySelector('.ws-activity').textContent.includes(s))",
  );
  await click("[data-ws-thread=sorting]");
  await fill("[data-ws-prompt-input]", "Keep this draft");
  await click("[data-ws-thread=retry]");
  await click("[data-ws-thread=sorting]");
  await check(
    "Draft text survives conversation changes",
    "document.querySelector('[data-ws-prompt-input]').value==='Keep this draft'",
  );
  await tab("pr");
  await click("[data-ws-approve]");
  await click("[data-ws-mode=plan]");
  await click('.ws-suggestions [data-ws-prompt="Explain these changes"]');
  await done();
  await check(
    "Plan mode preserves approval and does not alter the patch",
    "document.querySelector('[data-ws-approve]').textContent.includes('Revoke')&&!document.querySelector('[data-ws-merge]').disabled&&[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent.includes('Files:')",
  );
  await click("[data-ws-rename]");
  await fill("#ws-name", "Sorting review");
  await ev("document.querySelector('[data-ws-name-form]').requestSubmit()");
  await check(
    "Renaming updates the conversation and sidebar",
    "document.querySelector('[data-ws-title]').textContent==='Sorting review'&&document.querySelector('[data-ws-threads] [aria-current=true]').textContent.includes('Sorting review')",
  );
  await click("[data-ws-new]");
  await check(
    "New conversation has a draft PR and focuses the composer",
    "document.querySelectorAll('[data-ws-threads] [data-ws-thread]').length===4&&document.activeElement.matches('[data-ws-prompt-input]')&&document.querySelector('[data-ws-merge]').disabled&&document.querySelector('.ws-pr-status').textContent.includes('Draft')",
  );
  await click("[data-ws-mode=build]");
  await fill("[data-ws-prompt-input]", "Fix keyboard shortcuts");
  await ev("document.querySelector('[data-ws-prompt-input]').focus()");
  await key("Enter", "Enter", 13);
  await delay(120);
  await click("[data-ws-stop]");
  const stopped = await ev(
    "document.querySelector('[data-ws-messages]').textContent",
  );
  await delay(650);
  await check(
    "Stop cancels generation without reporting a successful run",
    await ev(
      `document.querySelector('[data-ws-stop]').hidden&&document.querySelector('[data-ws-messages]').textContent===${JSON.stringify(stopped)}&&document.querySelector('[data-ws-status]').textContent==='Stopped'`,
    ),
  );
  await fill("[data-ws-prompt-input]", "Fix the shortcut");
  await click("[data-ws-send]");
  await delay(130);
  await ev("Schematic.navigate('forms')");
  await delay(70);
  const hidden = await ev(
    "document.querySelector('.pg-workspace').textContent",
  );
  await delay(650);
  await check(
    "Leaving the workspace stops all conversation mutations",
    await ev(
      `document.querySelector('.pg-workspace').textContent===${JSON.stringify(hidden)}&&document.querySelector('.page[data-page=workspace]').hidden`,
    ),
  );
  await ev("Schematic.navigate('workspace')");
  await delay(70);
  await click("[data-ws-reset]");
  await check(
    "Reset restores the three initial conversations and failed check",
    "document.querySelectorAll('[data-ws-threads] [data-ws-thread]').length===3&&document.querySelector('[data-ws-title]').textContent==='Search shortcut'&&document.querySelector('[data-ws-merge]').disabled",
  );
  await ev("Schematic.setMotion('full')");
  await tab("changes");
  await check(
    "Inspector transition produces a real intermediate frame",
    await ev(
      "document.querySelector('[data-ws-review-content]').getAnimations().some(a=>a.playState==='running')",
    ),
  );
  await ev("Schematic.setMotion('reduced')");
  await delay(40);
  await check(
    "Reduced motion settles an active inspector transition",
    "document.querySelector('[data-ws-review-content]').getAnimations().length===0",
  );
  for (const width of [390, 320]) {
    await b.size(width, 844);
    await delay(300);
    await check(
      `${width}px workspace stays inside the viewport`,
      await ev(
        "document.documentElement.scrollWidth<=innerWidth&&document.querySelector('.ws-app').getBoundingClientRect().right<=innerWidth+1",
      ),
    );
    await click("[data-ws-review-toggle]");
    await check(
      `${width}px review opens as a native modal`,
      "document.querySelector('[data-ws-panel-dialog]').open&&document.activeElement.closest('[data-ws-panel-dialog]')!==null",
    );
    await ev("document.querySelector('[data-ws-review-close]').focus()");
    await key("Tab", "Tab", 9, 8);
    await check(
      `${width}px reverse Tab stays in the review modal`,
      "document.activeElement.closest('[data-ws-panel-dialog]')!==null",
    );
    await key("Escape", "Escape", 27);
    await delay(50);
    await check(
      `${width}px Escape restores the review trigger`,
      "!document.querySelector('[data-ws-panel-dialog]').open&&document.activeElement.hasAttribute('data-ws-review-toggle')",
    );
    await click("[data-ws-sidebar]");
    await check(
      `${width}px conversation picker opens`,
      "document.querySelector('[data-ws-nav-dialog]').open",
    );
    await click("[data-ws-mobile-threads] [data-ws-thread=retry]");
    await check(
      `${width}px choosing a conversation closes the picker`,
      "!document.querySelector('[data-ws-nav-dialog]').open&&document.querySelector('[data-ws-title]').textContent==='API retry backoff'",
    );
  }
  await b.size(1440, 1000);
  await ev(
    "document.documentElement.dataset.theme='dark';Schematic.setPalette('mono')",
  );
  await b.screenshot(join(workspace, "scratch/qa/workspace-dark.png"));
  await check(
    "Workspace supports dark Mono with no exceptions",
    "document.documentElement.dataset.theme==='dark'&&document.documentElement.dataset.palette==='mono'",
  );
  await check(
    "Workspace has no runtime or network errors",
    b.failures.length === 0,
  );
} catch (error) {
  results.push({
    name: "Workspace browser run completed",
    pass: false,
    error: error.stack,
  });
  console.error(error.stack);
} finally {
  await b.close();
}
await mkdir(join(workspace, "scratch/qa"), { recursive: true });
await writeFile(
  join(workspace, "scratch/qa/workspace.json"),
  JSON.stringify({ date: new Date().toISOString(), checks: results }, null, 2),
);
const failed = results.filter((c) => !c.pass).length;
console.log(`${results.length - failed} passed, ${failed} failed`);
process.exitCode = failed ? 1 : 0;
