#!/usr/bin/env node
import { browser, delay } from "./browser.mjs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { mkdir, writeFile } from "node:fs/promises";
const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const qa = join(workspace, "scratch/qa");
const entry =
  process.env.SCHEMATIC_WORKSPACE_SOURCE === "1"
    ? "prototype/index.html"
    : "dist/schematic.html";
const b = await browser({ width: 1440, height: 1000 }),
  results = [];
const ev = (expression) => b.evaluate(expression);
const q = (selector) => `.pg-workspace ${selector}`;
const check = async (name, expression) => {
  const pass =
    (typeof expression === "boolean" ? expression : await ev(expression)) ===
    true;
  results.push({ name, pass });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}`);
};
// Trusted pointer events catch hidden or overlaid controls that .click() misses.
const click = async (selector) => {
  const point = await ev(
    `(()=>{const e=[...document.querySelectorAll(${JSON.stringify(q(selector))})].find(e=>e.getClientRects().length&&!e.closest('[inert]'));if(!e)throw Error('No visible control: '+${JSON.stringify(selector)});e.scrollIntoView({block:'nearest'});const r=e.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2};})()`,
  );
  await b.send("Input.dispatchMouseEvent", {
    type: "mousePressed",
    ...point,
    button: "left",
    clickCount: 1,
  });
  await b.send("Input.dispatchMouseEvent", {
    type: "mouseReleased",
    ...point,
    button: "left",
    clickCount: 1,
  });
  await delay(25);
};
const key = async (key, code, n, modifiers = 0) => {
  const args = { key, code, windowsVirtualKeyCode: n, modifiers };
  await b.send("Input.dispatchKeyEvent", {
    type: "keyDown",
    ...args,
    ...(key === "Enter" ? { text: "\r", unmodifiedText: "\r" } : {}),
  });
  await b.send("Input.dispatchKeyEvent", { type: "keyUp", ...args });
};
const fill = async (selector, value) =>
  ev(
    `(()=>{const e=document.querySelector(${JSON.stringify(q(selector))});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`,
  );
const done = async () => {
  const deadline = Date.now() + 8000;
  while (Date.now() < deadline) {
    if (await ev("document.querySelector('[data-ws-stop]').hidden")) return;
    await delay(70);
  }
  throw Error("Workspace task did not settle");
};
const thread = async (id) => {
  await click("[data-ws-sessions]");
  await click(`[data-ws-thread="${id}"]`);
};
const review = () => click("[data-ws-review-toggle]");
const closeReview = () => click("[data-ws-review-close]");
const view = (id) => click(`[data-ws-view="${id}"]`);
try {
  await mkdir(qa, { recursive: true });
  await b.open(pathToFileURL(join(workspace, entry)).href);
  await ev("Schematic.setMotion('reduced');Schematic.navigate('workspace')");
  await delay(140);
  await check(
    "Workspace stays one discoverable complete app",
    "Schematic.catalog().some(e=>e.page==='workspace')&&document.querySelectorAll('.pg-workspace .demo').length===1&&document.querySelector('.ws-local').textContent==='Local demo'",
  );
  await check(
    "Code is the largest pane, with explorer and docked console",
    "(()=>{const code=document.querySelector('.ws-center').getBoundingClientRect(),chat=document.querySelector('.ws-chat').getBoundingClientRect(),explorer=document.querySelector('.ws-explorer').getBoundingClientRect();return code.width>chat.width*2&&code.width>780&&explorer.width>150&&document.querySelector('.ws-console').getBoundingClientRect().bottom<=code.bottom;})()",
  );
  const geometry = await ev(
    "(()=>{const e=document.querySelector('.ws-app'),r=e.getBoundingClientRect(),s=getComputedStyle(e);return{width:r.width,bottom:r.bottom,viewportWidth:innerWidth,viewportHeight:innerHeight,radius:s.borderRadius,padding:s.padding,gutter:getComputedStyle(document.querySelector('#viewport')).scrollbarGutter}})()",
  );
  console.log("IDE geometry: " + JSON.stringify(geometry));
  await check(
    "IDE fills shell viewport without an outer card",
    geometry.width === geometry.viewportWidth &&
      geometry.bottom === geometry.viewportHeight &&
      geometry.radius === "0px" &&
      geometry.padding === "0px" &&
      geometry.gutter === "auto",
  );
  await check(
    "Primary file has substantive syntax-rendered code and line numbers",
    "document.querySelectorAll('.ws-code-row').length>=40&&document.querySelectorAll('.ws-code-row').length<=80&&document.querySelectorAll('.ws-token-keyword').length>20&&document.querySelectorAll('.ws-token-string').length>10&&document.querySelectorAll('.ws-line-number').length>=40&&document.querySelector('[data-ws-source]').textContent.includes('removeEventListener')",
  );
  await check(
    "Failure is attached to relevant source line",
    "document.querySelector('[data-diagnostic]').textContent.includes('openSearch()')&&document.querySelector('[data-ws-source]').scrollTop>0",
  );
  await check(
    "Code lines are plain text surfaces without inline-code pills",
    "(()=>{const s=getComputedStyle(document.querySelector('.ws-code-row code'));return s.backgroundColor==='rgba(0, 0, 0, 0)'&&s.borderTopWidth==='0px';})()",
  );
  await check(
    "Initial CI log names failing assertion, file and exit status",
    "document.querySelectorAll('.ws-check-row[data-status=failed]').length===1&&document.querySelector('[data-ws-log]').textContent.includes('defaultPrevented')&&document.querySelector('[data-ws-log]').textContent.includes('src/search.test.ts:31:36')&&document.querySelector('[data-ws-log-exit]').textContent==='exit 1'",
  );
  await b.screenshot(join(qa, "workspace-ide-desktop.png"));
  await click("[data-ws-check=types]");
  await check(
    "Selecting a CI check shows its own output",
    "document.querySelector('[data-ws-log]').textContent.includes('No type errors')&&document.querySelector('[data-ws-log-command]').textContent==='pnpm typecheck'",
  );
  await click('[data-ws-file-tabs] [data-ws-file="1"]');
  await check(
    "File tab opens substantive test source",
    "document.querySelector('[data-ws-file-path]').textContent.includes('search.test.ts')&&document.querySelector('[data-ws-source]').textContent.includes('KeyboardEventInit')",
  );
  await key("ArrowRight", "ArrowRight", 39);
  await check(
    "File tabs support roving keyboard selection and focus",
    "document.querySelector('[data-ws-file-tabs] [data-ws-file=" +
      '"2"' +
      "]').getAttribute('aria-selected')==='true'&&document.activeElement.dataset.wsFile==='2'&&document.querySelector('[data-ws-file-path]').textContent==='CHANGELOG.md'",
  );
  await click('[data-ws-file-tabs] [data-ws-file="0"]');
  await click("[data-ws-editor-mode=diff]");
  await check(
    "Split diff has aligned old/new cells and real changes",
    "document.querySelectorAll('[data-ws-diff-side=old] .ws-diff-cell').length>40&&document.querySelectorAll('[data-ws-diff-side=old] .ws-diff-cell').length===document.querySelectorAll('[data-ws-diff-side=new] .ws-diff-cell').length&&document.querySelectorAll('.ws-diff-added').length>0&&document.querySelectorAll('.ws-diff-removed').length>0&&document.querySelectorAll('.ws-diff-context').length>20",
  );
  await check(
    "Both diff panes receive equal visible width",
    "(()=>{const a=document.querySelector('[data-ws-diff-side=old]').getBoundingClientRect(),b=document.querySelector('[data-ws-diff-side=new]').getBoundingClientRect(),p=document.querySelector('[data-ws-diff]').getBoundingClientRect();return Math.abs(a.width-b.width)<2&&a.width>400&&b.right<=p.right+1;})()",
  );
  await ev(
    "const side=document.querySelector('[data-ws-diff-side=new]');side.scrollTop=242;side.dispatchEvent(new Event('scroll'));void 0",
  );
  await check(
    "Diff panes synchronize vertical context while retaining horizontal scrolling",
    "document.querySelector('[data-ws-diff-side=old]').scrollTop===document.querySelector('[data-ws-diff-side=new]').scrollTop&&document.querySelector('[data-ws-diff-side=old]').scrollWidth>document.querySelector('[data-ws-diff-side=old]').clientWidth",
  );
  await check(
    "Unrepaired diff does not invent preventDefault",
    "!document.querySelector('[data-ws-diff]').textContent.includes('event.preventDefault();')",
  );
  await review();
  await check(
    "PR controls live in a native dialog, not a permanent pane",
    "document.querySelector('[data-ws-panel-dialog]').open&&document.querySelector('[data-ws-merge]').disabled&&document.activeElement.hasAttribute('data-ws-review-close')",
  );
  await click("[data-ws-approve]");
  await check(
    "Approval cannot bypass failed CI",
    "document.querySelector('[data-ws-approve]').textContent.includes('Revoke')&&document.querySelector('[data-ws-merge]').disabled",
  );
  await ev("document.querySelector('[data-ws-approve]').focus()");
  await key("Tab", "Tab", 9);
  await check(
    "Review dialog wraps focus past its last enabled control",
    "document.activeElement.hasAttribute('data-ws-review-close')",
  );
  await key("Escape", "Escape", 27);
  await check(
    "Escape closes review and restores trigger focus",
    "!document.querySelector('[data-ws-panel-dialog]').open&&document.activeElement.hasAttribute('data-ws-review-toggle')",
  );
  await click("[data-ws-edit]");
  await check(
    "Edit mode focuses a native code textarea",
    "document.activeElement.hasAttribute('data-ws-code-input')&&!document.querySelector('[data-ws-edit-area]').hidden&&!document.querySelector('[data-ws-code-input]').readOnly",
  );
  await check(
    "Highlight overlay matches textarea metrics",
    "(()=>{const a=getComputedStyle(document.querySelector('[data-ws-code-input]')),b=getComputedStyle(document.querySelector('[data-ws-highlight]'));return a.fontFamily===b.fontFamily&&a.fontSize===b.fontSize&&a.lineHeight===b.lineHeight&&a.padding===b.padding;})()",
  );
  await ev(
    "document.querySelector('[data-ws-code-input]').setSelectionRange(0,0)",
  );
  await b.send("Input.insertText", {
    text: "// Local edit <script> is text\n",
  });
  await check(
    "Native code edit changes buffer and invalidates all checks and approval",
    "document.querySelector('[data-ws-code-input]').value.startsWith('// Local edit')&&document.querySelectorAll('.ws-check-row[data-status=stale]').length===3&&document.querySelector('[data-ws-approve]').textContent.includes('Approve changes')&&document.querySelector('[data-ws-merge]').disabled",
  );
  await check(
    "Code highlighting escapes user text",
    "!document.querySelector('[data-ws-highlight] script')&&document.querySelector('[data-ws-highlight]').textContent.includes('<script>')",
  );
  await ev(
    "const e=document.querySelector('[data-ws-code-input]');e.scrollTop=330;e.scrollLeft=70;e.dispatchEvent(new Event('scroll'));void 0",
  );
  await check(
    "Editor gutter and highlighted text track scroll",
    "document.querySelector('[data-ws-highlight]').scrollTop===document.querySelector('[data-ws-code-input]').scrollTop&&document.querySelector('[data-ws-edit-gutter]').scrollTop===document.querySelector('[data-ws-code-input]').scrollTop&&document.querySelector('[data-ws-highlight]').scrollLeft===document.querySelector('[data-ws-code-input]').scrollLeft",
  );
  await key("s", "KeyS", 83, 2);
  await check(
    "Ctrl-S returns to highlighted source with focus retained",
    "document.querySelector('[data-ws-edit-area]').hidden&&document.activeElement.hasAttribute('data-ws-edit')&&document.querySelector('[data-ws-source]').textContent.includes('Local edit')",
  );
  await click("[data-ws-run-checks]");
  await delay(100);
  await check(
    "Check rerun queues every check and exposes Stop",
    "!document.querySelector('[data-ws-stop]').hidden&&document.querySelector('[data-ws-run-checks]').disabled&&document.querySelector('[data-ws-code-input]').readOnly",
  );
  await done();
  await check(
    "Rerunning without repair stays failed",
    "document.querySelectorAll('.ws-check-row[data-status=failed]').length===1&&document.querySelector('[data-ws-status]').textContent.includes('failed')",
  );
  await click('.ws-suggestions [data-ws-prompt="Fix the failed check"]');
  await delay(100);
  const partial = await ev(
    "[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent",
  );
  await delay(150);
  await check(
    "Reply streams partial text before patch/check completion",
    partial.length > 0 &&
      (await ev(
        "[...document.querySelectorAll('[data-ws-message-text]')].at(-1).textContent.length",
      )) > partial.length,
  );
  await done();
  await check(
    "Repair reruns all checks against changed code",
    "document.querySelectorAll('.ws-check-row[data-status=passed]').length===3&&document.querySelector('[data-ws-source]').textContent.includes('event.preventDefault();')&&!document.querySelector('[data-diagnostic]')",
  );
  await click("[data-ws-editor-mode=diff]");
  await check(
    "Repair produces actual added line on modified side only",
    "(()=>{const row=[...document.querySelectorAll('[data-ws-diff-side=new] .ws-diff-cell')].find(e=>e.textContent.includes('event.preventDefault();'));return !!row&&row.classList.contains('ws-diff-added')&&!document.querySelector('[data-ws-diff-side=old]').textContent.includes('event.preventDefault();');})()",
  );
  await b.screenshot(join(qa, "workspace-ide-diff.png"));
  await review();
  await check(
    "Passing checks still require approval",
    "document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-approve]");
  await closeReview();
  await click("[data-ws-mode=plan]");
  await fill("[data-ws-prompt-input]", "Explain these changes");
  await click("[data-ws-send]");
  await done();
  await check(
    "Read-only planning preserves approval and passed checks",
    "document.querySelector('[data-ws-approve]').textContent.includes('Revoke')&&!document.querySelector('[data-ws-merge]').disabled&&document.querySelectorAll('.ws-check-row[data-status=passed]').length===3",
  );
  await click("[data-ws-mode=build]");
  await fill("[data-ws-prompt-input]", "Keep this draft");
  await click('[data-ws-file-tabs] [data-ws-file="1"]');
  await thread("sorting");
  await check(
    "Sessions have independent code, checks and approvals",
    "document.querySelector('[data-ws-file-path]').textContent.includes('task-list.ts')&&document.querySelectorAll('.ws-check-row[data-status=passed]').length===3&&document.querySelector('[data-ws-approve]').textContent.includes('Approve changes')",
  );
  await fill("[data-ws-prompt-input]", "Sorting draft");
  await thread("search");
  await check(
    "Returning restores conversation draft and selected file",
    "document.querySelector('[data-ws-prompt-input]').value==='Keep this draft'&&document.querySelector('[data-ws-file-path]').textContent.includes('search.test.ts')&&document.querySelector('[data-ws-approve]').textContent.includes('Revoke')",
  );
  await thread("sorting");
  await check(
    "Other session keeps its own composer draft",
    "document.querySelector('[data-ws-prompt-input]').value==='Sorting draft'",
  );
  await click("[data-ws-run-checks]");
  await delay(500);
  await click("[data-ws-stop]");
  await check(
    "Stop cancels queued/running checks without success",
    "document.querySelector('[data-ws-stop]').hidden&&document.querySelectorAll('.ws-check-row[data-status=cancelled]').length>0&&[...document.querySelectorAll('.ws-message')].at(-1).textContent.includes('Stopped')",
  );
  await delay(1800);
  await check(
    "Cancelled task never later reports passed",
    "document.querySelectorAll('.ws-check-row[data-status=cancelled]').length>0&&document.querySelector('[data-ws-status]').textContent==='Stopped'",
  );
  await click("[data-ws-run-checks]");
  await delay(120);
  await thread("retry");
  await delay(1800);
  await thread("sorting");
  await check(
    "Switching sessions cancels work in the previous session",
    "document.querySelectorAll('.ws-check-row[data-status=cancelled]').length>0&&document.querySelector('[data-ws-stop]').hidden",
  );
  await click("[data-ws-run-checks]");
  await delay(150);
  await ev("Schematic.navigate('icons')");
  await delay(150);
  await ev("Schematic.navigate('workspace')");
  await delay(100);
  await check(
    "Route navigation cancels work and closes dialogs",
    "document.querySelector('[data-ws-stop]').hidden&&document.querySelectorAll('.pg-workspace dialog[open]').length===0&&document.querySelectorAll('.ws-check-row[data-status=cancelled]').length>0",
  );
  await click("[data-ws-run-checks]");
  await delay(150);
  await ev(
    "Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));Object.defineProperty(document,'hidden',{configurable:true,value:false});",
  );
  await check(
    "Hidden-document cleanup records cancellation",
    "document.querySelector('[data-ws-stop]').hidden&&document.querySelectorAll('.ws-check-row[data-status=cancelled]').length>0",
  );
  await thread("search");
  await review();
  await click("[data-ws-merge]");
  await check(
    "Merge opens explicit native confirmation with Cancel focused",
    "document.querySelector('[data-ws-confirm]').open&&document.activeElement.hasAttribute('data-ws-cancel-merge')",
  );
  await key("Tab", "Tab", 9, 8);
  await check(
    "Merge dialog wraps backward focus",
    "document.activeElement.hasAttribute('data-ws-confirm-merge')",
  );
  await key("Escape", "Escape", 27);
  await check(
    "Cancelling merge preserves PR and restores merge focus",
    "!document.querySelector('[data-ws-confirm]').open&&!document.querySelector('[data-ws-merge]').disabled&&document.activeElement.hasAttribute('data-ws-merge')",
  );
  await click("[data-ws-merge]");
  await click("[data-ws-confirm-merge]");
  await check(
    "Successful merge updates branch, timeline and read-only state",
    "document.querySelector('[data-ws-branch]').textContent.includes('main')&&document.querySelector('[data-ws-prompt-input]').disabled&&document.querySelector('[data-ws-edit]').disabled&&document.querySelector('[data-ws-merge]').disabled&&document.querySelector('[data-ws-status]').textContent.includes('Merged')",
  );
  await click("[data-ws-tab=activity]");
  await check(
    "Activity records actual approval and merge events",
    "document.querySelector('.ws-timeline').textContent.includes('Approved by you')&&document.querySelector('.ws-timeline').textContent.includes('Merged #128')",
  );
  await closeReview();
  await click("[data-ws-reset]");
  await click("[data-ws-new]");
  await check(
    "New session starts draft PR with no passing checks",
    "document.querySelector('[data-ws-title]').textContent==='New session'&&document.querySelectorAll('.ws-check-row[data-status=stale]').length===3&&document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-rename]");
  await fill("#ws-name", "Keyboard repair");
  await click("[data-ws-name-form] button[type=submit]");
  await check(
    "Rename updates session identity and restores focus",
    "document.querySelector('[data-ws-title]').textContent==='Keyboard repair'&&document.activeElement.hasAttribute('data-ws-rename')",
  );
  await click("[data-ws-reset]");
  // Normal-speed intermediate frames and interruption, rather than endpoints only.
  await ev("Schematic.setMotion('full')");
  await click("[data-ws-console-toggle]");
  await delay(65);
  await check(
    "Console collapse has a real intermediate geometry",
    "(()=>{const h=document.querySelector('.ws-console-wrap').getBoundingClientRect().height;const full=document.querySelector('.ws-console').getBoundingClientRect().height;return h>1&&h<full-1;})()",
  );
  await b.screenshot(join(qa, "workspace-ide-console-mid.png"));
  await click("[data-ws-console-toggle]");
  await delay(450);
  await check(
    "Reversing console transition settles open with usable checks",
    "document.querySelector('.ws-app').dataset.console==='open'&&Math.abs(document.querySelector('.ws-console-wrap').getBoundingClientRect().height-document.querySelector('.ws-console').getBoundingClientRect().height)<1&&!document.querySelector('[data-ws-console-wrap]').inert",
  );
  await click('[data-ws-file-tabs] [data-ws-file="1"]');
  await delay(35);
  await check(
    "File selection animates the code pane, not its geometry",
    "(()=>{const a=document.querySelector('[data-ws-editor-content]').getAnimations();return a.some(a=>a.playState==='running')&&document.querySelector('.ws-center').getBoundingClientRect().width>780;})()",
  );
  await click('[data-ws-file-tabs] [data-ws-file="0"]');
  await delay(230);
  await check(
    "Interrupted file transition settles on the latest file",
    "document.querySelector('[data-ws-file-path]').textContent.includes('search.ts')&&document.querySelector('[data-ws-editor-content]').getAnimations().every(a=>a.playState!=='running')",
  );
  await click('[data-ws-file-tabs] [data-ws-file="1"]');
  await ev("Schematic.setMotion('reduced')");
  await delay(20);
  await check(
    "Reduced motion settles an active editor transition",
    "document.querySelector('[data-ws-editor-content]').getAnimations().every(a=>a.playState!=='running')",
  );
  await click('[data-ws-file-tabs] [data-ws-file="0"]');
  await ev("Schematic.setTheme('dark')");
  await delay(100);
  await b.screenshot(join(qa, "workspace-ide-dark.png"));
  await check(
    "Dark appearance inherits semantic editor and syntax colours",
    "getComputedStyle(document.querySelector('.ws-app')).color===getComputedStyle(document.documentElement).getPropertyValue('--text').trim()||getComputedStyle(document.querySelector('.ws-token-keyword')).color!==getComputedStyle(document.querySelector('.ws-source')).color",
  );
  await ev("Schematic.setPalette('mono')");
  await delay(100);
  await b.screenshot(join(qa, "workspace-ide-mono.png"));
  await check(
    "Mono keeps code readable and selected controls identifiable",
    "document.documentElement.dataset.palette==='mono'&&document.querySelector('[data-ws-editor-mode=source]').getAttribute('aria-pressed')==='true'&&document.querySelectorAll('.ws-code-row').length===63",
  );
  await ev("Schematic.setTheme('light');Schematic.setPalette('color')");
  await b.size(390, 844);
  await delay(100);
  await check(
    "Mobile shows one full-width editor instead of cramped panes",
    "(()=>{const visible=s=>document.querySelector(s).getClientRects().length>0;return visible('.ws-editor-pane')&&!visible('.ws-chat')&&!visible('.ws-console-wrap')&&!visible('.ws-explorer')&&document.querySelector('.ws-center').getBoundingClientRect().width===innerWidth&&document.documentElement.scrollWidth===innerWidth;})()",
  );
  await b.screenshot(join(qa, "workspace-ide-mobile-editor.png"));
  await view("chat");
  await check(
    "Mobile view switch gives chat the complete body",
    "document.querySelector('.ws-chat').getClientRects().length>0&&document.querySelector('.ws-center').getClientRects().length===0&&document.querySelector('.ws-chat').getBoundingClientRect().width===innerWidth",
  );
  await fill("[data-ws-prompt-input]", "Mobile draft");
  await b.screenshot(join(qa, "workspace-ide-mobile-chat.png"));
  await view("checks");
  await check(
    "Mobile checks view has selectable checks and readable logs",
    "document.querySelector('.ws-console').getBoundingClientRect().height>500&&document.querySelector('.ws-chat').getClientRects().length===0&&document.querySelector('.ws-editor-pane').getClientRects().length===0&&document.querySelector('[data-ws-log]').getBoundingClientRect().width>220",
  );
  await click("[data-ws-check=types]");
  await check(
    "Mobile CI selection changes log output",
    "document.querySelector('[data-ws-log]').textContent.includes('No type errors')",
  );
  await b.screenshot(join(qa, "workspace-ide-mobile-checks.png"));
  await view("editor");
  await click("[data-ws-sidebar]");
  await check(
    "Mobile project files open in a native dialog",
    "document.querySelector('[data-ws-files-dialog]').open&&document.activeElement.hasAttribute('data-ws-file')",
  );
  await click('[data-ws-mobile-files] [data-ws-file="1"]');
  await check(
    "Mobile file selection closes picker and opens selected source",
    "!document.querySelector('[data-ws-files-dialog]').open&&document.querySelector('[data-ws-file-path]').textContent.includes('search.test.ts')&&document.activeElement.dataset.wsFile==='1'",
  );
  await review();
  await key("Escape", "Escape", 27);
  await check(
    "Mobile review Escape restores visible PR trigger",
    "!document.querySelector('[data-ws-panel-dialog]').open&&document.activeElement.hasAttribute('data-ws-review-toggle')",
  );
  await view("chat");
  await check(
    "Mobile view switches preserve composer draft",
    "document.querySelector('[data-ws-prompt-input]').value==='Mobile draft'",
  );
  await b.size(320, 740);
  await view("editor");
  await delay(50);
  await check(
    "Small mobile viewport contains toolbar and editor",
    "document.documentElement.scrollWidth===innerWidth&&document.querySelector('.ws-topbar').getBoundingClientRect().right<=innerWidth&&document.querySelector('.ws-editor-toolbar').getBoundingClientRect().right<=innerWidth",
  );
  // Manual code edits must affect CI independently of the chat repair action.
  await b.size(1440, 1000);
  await click("[data-ws-reset]");
  await click("[data-ws-edit]");
  await ev(
    "(()=>{const input=document.querySelector('[data-ws-code-input]'),at=input.value.indexOf('    openSearch();');input.setSelectionRange(at,at);})()",
  );
  await b.send("Input.insertText", { text: "    event.preventDefault();\n" });
  await key("Escape", "Escape", 27);
  await check(
    "Escape ends editing without trapping keyboard focus",
    "document.activeElement.hasAttribute('data-ws-edit')&&document.querySelector('[data-ws-edit-area]').hidden",
  );
  await click("[data-ws-run-checks]");
  await done();
  await check(
    "A manually typed repair changes the CI outcome",
    "document.querySelectorAll('.ws-check-row[data-status=passed]').length===3&&document.querySelector('[data-ws-source]').textContent.includes('event.preventDefault();')",
  );
  await review();
  await click("[data-ws-approve]");
  await closeReview();
  await click("[data-ws-run-checks]");
  await done();
  await check(
    "Rerunning an unchanged revision preserves approval",
    "document.querySelector('[data-ws-approve]').textContent.includes('Revoke')&&!document.querySelector('[data-ws-merge]').disabled",
  );
  await click("[data-ws-edit]");
  await ev(
    "(()=>{const input=document.querySelector('[data-ws-code-input]'),at=input.value.indexOf('    event.preventDefault();');input.setSelectionRange(at,at+'    event.preventDefault();'.length+1);})()",
  );
  await key("Backspace", "Backspace", 8);
  await check(
    "Removing a repair revokes approval and marks every check stale",
    "document.querySelectorAll('.ws-check-row[data-status=stale]').length===3&&document.querySelector('[data-ws-approve]').textContent.includes('Approve changes')",
  );
  await key("Escape", "Escape", 27);
  await click("[data-ws-run-checks]");
  await done();
  await check(
    "Manual regression fails again without a cached repair flag",
    "document.querySelectorAll('.ws-check-row[data-status=failed]').length===1&&document.querySelector('[data-ws-merge]').disabled",
  );
  await b.size(1440, 1000);
  await ev('Schematic.setMotion("reduced");void 0');
  await delay(50);
  await check(
    "Immersive workspace removes hidden library links from focus",
    "document.querySelector('.sidebar').inert&&document.querySelector('.sidebar').getBoundingClientRect().right<=0",
  );
  await ev('document.querySelector("#menu-btn").click();void 0');
  await delay(25);
  await check(
    "Desktop workspace can reopen library navigation",
    "!document.querySelector('.sidebar').inert&&document.querySelector('.sidebar').getBoundingClientRect().left===0&&!!document.activeElement.closest('.sidebar')",
  );
  await key("Tab", "Tab", 9, 8);
  await check(
    "Immersive navigation contains reverse Tab",
    "(()=>{const a=[...document.querySelector('.sidebar').querySelectorAll('a[href],button:not(:disabled),input:not(:disabled)')].filter(e=>e.offsetParent!==null);return document.activeElement===a.at(-1)})()",
  );
  await key("Escape", "Escape", 27);
  await delay(25);
  await check(
    "Immersive navigation Escape restores its trigger",
    "document.activeElement.id==='menu-btn'&&document.querySelector('.sidebar').inert",
  );
  await ev('Schematic.navigate("overview");void 0');
  await delay(100);
  await check(
    "Returning to the library restores desktop navigation",
    "!document.querySelector('.sidebar').inert&&document.querySelector('.sidebar').getBoundingClientRect().left===0",
  );
  await check(
    "No browser exceptions or console errors",
    b.failures.length === 0,
  );
  await writeFile(
    join(qa, "workspace-results.json"),
    JSON.stringify({ entry, results, failures: b.failures }, null, 2),
  );
  console.log(
    `\n${results.filter((r) => r.pass).length} passed, ${results.filter((r) => !r.pass).length} failed`,
  );
  if (results.some((r) => !r.pass)) process.exitCode = 1;
} finally {
  await b.close();
}
