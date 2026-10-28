/* Local coding-workspace simulation. No model, Git or CI service is contacted. */
(() => {
  const S = window.Schematic,
    instances = new WeakMap();
  const ic = (name, size = 16, cls = "") =>
    S.icon(S.iconNames.includes(name) ? name : "cube", { size, cls });
  const esc = S.esc;
  const patch = (path, before, after) => ({
    path,
    before: before.split("\n"),
    after: after.split("\n"),
  });
  const fixtures = [
    {
      id: "search",
      title: "Search shortcut",
      branch: "fix/search-shortcut",
      number: 128,
      failed: true,
      request:
        "Fix Cmd+K so it opens search without triggering the browser shortcut.",
      summary:
        "Updated the shortcut handler and added a test. The keyboard check failed; its log is available in Checks.",
      files: [
        patch(
          "src/search.ts",
          "export function onKey(event: KeyboardEvent) {\n  if (event.key === 'k') {\n    openSearch();\n  }\n}",
          "export function onKey(event: KeyboardEvent) {\n  const modifier = event.metaKey || event.ctrlKey;\n  if (modifier && event.key.toLowerCase() === 'k') {\n    openSearch();\n  }\n}",
        ),
        patch(
          "src/search.test.ts",
          "it('opens search', () => {\n  press('k');\n  expect(search.isOpen).toBe(true);\n});",
          "it('handles the platform shortcut', () => {\n  const event = press('k', { metaKey: true });\n  expect(event.defaultPrevented).toBe(true);\n  expect(search.isOpen).toBe(true);\n});",
        ),
        patch(
          "CHANGELOG.md",
          "## Unreleased",
          "## Unreleased\n- Fix the search keyboard shortcut.",
        ),
      ],
    },
    {
      id: "sorting",
      title: "List drag handles",
      branch: "feat/drag-handles",
      number: 129,
      failed: false,
      request: "Add drag handles and keyboard sorting to the task list.",
      summary:
        "Added pointer dragging, keyboard sorting and a cancel path. All three checks passed.",
      files: [
        patch(
          "src/task-list.ts",
          "export function moveTask(id, position) {\n  tasks.move(id, position);\n}",
          "export function moveTask(id, position) {\n  const previous = tasks.snapshot();\n  tasks.move(id, position);\n  return () => tasks.restore(previous);\n}\n\nexport const dragOptions = {\n  handle: '[data-drag-handle]',\n  keyboard: true,\n};",
        ),
        patch(
          "src/task-list.css",
          ".task {\n  display: flex;\n}",
          ".task {\n  display: flex;\n}\n.drag-handle {\n  touch-action: none;\n  cursor: grab;\n}\n.drag-handle:active {\n  cursor: grabbing;\n}",
        ),
        patch(
          "src/task-list.test.ts",
          "describe('task list', () => {});",
          "describe('task list', () => {\n  it('reorders with a pointer', pointerSort);\n  it('reorders with a keyboard', keyboardSort);\n  it('restores order on Escape', cancelSort);\n});",
        ),
      ],
    },
    {
      id: "retry",
      title: "API retry backoff",
      branch: "fix/retry-backoff",
      number: 130,
      failed: false,
      request: "Limit retries and add exponential backoff to the API client.",
      summary:
        "Added a three-attempt limit, backoff and a cancellation test. All checks passed.",
      files: [
        patch(
          "src/client.ts",
          "export async function load(url) {\n  return fetch(url);\n}",
          "export async function load(url, signal) {\n  for (let attempt = 0; attempt < 3; attempt++) {\n    const response = await fetch(url, { signal });\n    if (response.ok) return response;\n    if (attempt === 2) throw new Error('Retry limit');\n    await wait(200 * 2 ** attempt, signal);\n  }\n}",
        ),
        patch(
          "src/client.test.ts",
          "describe('client', () => {});",
          "describe('client', () => {\n  it('stops after three attempts', retryLimit);\n  it('backs off between requests', backoff);\n  it('cancels pending retries', cancellation);\n});",
        ),
        patch(
          "README.md",
          "## API client",
          "## API client\nRequests retry at most three times.\nPass an AbortSignal to cancel.",
        ),
      ],
    },
  ];
  const checks = [
    {
      id: "types",
      title: "TypeScript",
      command: "pnpm typecheck",
      success: [
        "> tsc --noEmit",
        "No type errors.",
        "Process exited with code 0.",
      ],
    },
    {
      id: "tests",
      title: "Unit tests",
      command: "pnpm test",
      success: [
        "> vitest run",
        "PASS src/search.test.ts",
        "PASS src/task-list.test.ts",
        "PASS src/client.test.ts",
        "Tests  24 passed (24)",
        "Process exited with code 0.",
      ],
    },
    {
      id: "build",
      title: "Production build",
      command: "pnpm build",
      success: [
        "> vite build",
        "32 modules transformed.",
        "dist/index.html  1.42 kB",
        "Build completed in 318ms.",
        "Process exited with code 0.",
      ],
    },
  ];
  function diffLines(file) {
    const a = file.before,
      b = file.after,
      table = Array.from({ length: a.length + 1 }, () =>
        Array(b.length + 1).fill(0),
      );
    for (let i = a.length - 1; i >= 0; i--)
      for (let j = b.length - 1; j >= 0; j--)
        table[i][j] =
          a[i] === b[j]
            ? table[i + 1][j + 1] + 1
            : Math.max(table[i + 1][j], table[i][j + 1]);
    const lines = [];
    let i = 0,
      j = 0;
    while (i < a.length || j < b.length) {
      if (i < a.length && j < b.length && a[i] === b[j]) {
        lines.push({ kind: "context", old: ++i, next: ++j, text: a[i - 1] });
      } else if (
        i < a.length &&
        (j === b.length || table[i + 1][j] >= table[i][j + 1])
      ) {
        lines.push({ kind: "removed", old: ++i, next: "", text: a[i - 1] });
      } else {
        lines.push({ kind: "added", old: "", next: ++j, text: b[j - 1] });
      }
    }
    return lines;
  }
  const createThread = (fixture) => ({
    ...fixture,
    files: fixture.files.map((f) => ({ ...f })),
    draft: "",
    messages: [
      { role: "user", text: fixture.request },
      {
        role: "assistant",
        text: fixture.summary,
        tools: [
          {
            kind: "read",
            label: `Read ${fixture.files[0].path}`,
            status: "passed",
          },
          {
            kind: "edit",
            label: `Updated ${fixture.files.length} files`,
            status: "passed",
          },
          {
            kind: "test",
            label: "Run checks",
            status: fixture.failed ? "failed" : "passed",
          },
        ],
      },
    ],
    checks: Object.fromEntries(
      checks.map((c) => [
        c.id,
        fixture.failed && c.id === "tests" ? "failed" : "passed",
      ]),
    ),
    approved: false,
    merged: false,
    repaired: !fixture.failed,
    phase: "idle",
    events: [
      { text: `Opened pull request #${fixture.number}`, time: "09:41" },
      {
        text: fixture.failed ? "Unit tests failed" : "All checks passed",
        time: "09:42",
      },
    ],
  });
  const statusIcon = (status) =>
    status === "passed"
      ? "check-circle"
      : status === "failed"
        ? "x"
        : status === "running"
          ? "loader"
          : status === "cancelled"
            ? "stop"
            : "clock";
  const render = () => `<div class="pg-workspace">
    <div class="demo ws-app" data-ws-app>
      <header class="ws-topbar"><div class="ws-product">${ic("code", 20)}<h1 class="demo-title">Coding workspace</h1><span class="ws-demo-label">Demo</span></div><span class="demo-desc ws-sr">Chat, file diffs, pull requests, CI checks and merges.</span><div class="ws-repository">${ic("folder", 14)} atlas / storefront</div><button class="btn btn-ghost btn-icon btn-sm ws-mobile-only" data-ws-sidebar aria-label="Open conversations">${ic("menu")}</button><button class="btn btn-secondary btn-sm" data-ws-review-toggle>${ic("git-pull-request", 14)} Review <span data-ws-pr-number>#128</span></button></header>
      <aside class="ws-sidebar" data-ws-sidebar-panel><div class="ws-sidebar-heading"><h2>Conversations</h2><button class="btn btn-ghost btn-icon btn-sm" data-ws-new aria-label="New conversation">${ic("plus", 16)}</button></div><div class="ws-threads" data-ws-threads></div><div class="ws-sidebar-heading ws-files-heading"><h2>Changed files</h2><span data-ws-file-total>3</span></div><div class="ws-files" data-ws-files></div><div class="ws-sidebar-footer"><button class="btn btn-ghost btn-sm" data-ws-reset>${ic("refresh", 14)} Reset demo</button></div></aside>
      <section class="ws-chat" aria-label="Conversation"><header class="ws-chat-heading"><div><h2 data-ws-title>Search shortcut</h2><span class="ws-branch" data-ws-branch>${ic("git-branch", 12)} fix/search-shortcut</span></div><button class="btn btn-ghost btn-icon btn-sm" data-ws-rename aria-label="Rename conversation">${ic("pencil", 14)}</button></header><div class="ws-messages" data-ws-messages role="log" aria-label="Conversation messages" aria-live="off"></div><div class="ws-chat-bottom"><div class="ws-suggestions" data-ws-suggestions><button data-ws-prompt="Explain these changes">Explain changes</button><button data-ws-prompt="Fix the failed check">Fix failed check</button><button data-ws-prompt="Run checks">Run checks</button></div><form class="ws-composer" data-ws-composer><label class="ws-sr" for="ws-prompt">Message</label><textarea id="ws-prompt" data-ws-prompt-input rows="2" placeholder="Describe a change…" maxlength="2000"></textarea><div class="ws-composer-bar"><div class="ws-mode" role="group" aria-label="Assistant mode"><button type="button" data-ws-mode="build" aria-pressed="true">${ic("code", 14)} Build</button><button type="button" data-ws-mode="plan" aria-pressed="false">${ic("document", 14)} Plan</button></div><span class="ws-shortcut">Enter to send</span><button type="submit" class="ws-send" data-ws-send aria-label="Send message">${ic("send", 17)}</button><button type="button" class="ws-send ws-stop" data-ws-stop aria-label="Stop generation" hidden>${ic("stop", 17)}</button></div></form><div class="ws-run-status" role="status" data-ws-status>1 check failed · Review required</div></div></section>
      <aside class="ws-review" data-ws-review aria-label="Code review"><div class="ws-review-heading"><strong>Review</strong><button class="btn btn-ghost btn-icon btn-sm" data-ws-review-close aria-label="Close review">${ic("x", 14)}</button></div><div class="ws-review-tabs" role="tablist" aria-label="Review panels">${[
        ["pr", "Pull request"],
        ["changes", "Changes"],
        ["checks", "Checks"],
        ["activity", "Activity"],
      ]
        .map(
          ([id, label], i) =>
            `<button role="tab" id="ws-tab-${id}" aria-controls="ws-review-content" aria-selected="${i === 0}" tabindex="${i === 0 ? 0 : -1}" data-ws-tab="${id}">${label}</button>`,
        )
        .join(
          "",
        )}</div><div class="ws-review-content" id="ws-review-content" role="tabpanel" aria-labelledby="ws-tab-pr" data-ws-review-content></div></aside>
    </div>
    <dialog class="ws-panel-dialog" data-ws-panel-dialog aria-label="Review panel"></dialog>
    <dialog class="ws-nav-dialog" data-ws-nav-dialog aria-label="Conversations"><div class="ws-dialog-top"><strong>Conversations</strong><button class="btn btn-ghost btn-icon" data-ws-nav-close aria-label="Close conversations">${ic("x")}</button></div><div data-ws-mobile-threads></div><button class="btn btn-secondary" data-ws-new>${ic("plus", 14)} New conversation</button></dialog>
    <dialog class="ws-confirm-dialog" data-ws-confirm aria-labelledby="ws-confirm-title"><h2 id="ws-confirm-title">Merge pull request</h2><div data-ws-merge-summary></div><div class="ws-dialog-actions"><button class="btn btn-secondary" data-ws-cancel-merge>Cancel</button><button class="btn btn-primary" data-ws-confirm-merge>${ic("git-merge", 16)} Merge</button></div></dialog>
    <dialog class="ws-confirm-dialog" data-ws-name-dialog aria-labelledby="ws-name-title"><form data-ws-name-form><h2 id="ws-name-title">Rename conversation</h2><label class="ws-sr" for="ws-name">Conversation name</label><input id="ws-name" maxlength="60" required><div class="ws-dialog-actions"><button type="button" class="btn btn-secondary" data-ws-name-cancel>Cancel</button><button type="submit" class="btn btn-primary">Save</button></div></form></dialog>
  </div>`;

  function mount(root) {
    const page = root.querySelector(".pg-workspace"),
      $ = (q) => page.querySelector(q),
      $$ = (q) => [...page.querySelectorAll(q)];
    const state = {
      active: false,
      threads: fixtures.map(createThread),
      selected: "search",
      tab: "pr",
      file: 0,
      check: "tests",
      mode: "build",
      job: null,
      timers: new Set(),
      animations: new Set(),
      serial: 0,
      returnFocus: null,
    };
    instances.set(root, state);
    const current = () => state.threads.find((t) => t.id === state.selected);
    const clock = () =>
      new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    const event = (thread, text) => thread.events.push({ text, time: clock() });
    const animate = (el) => {
      if (!state.active || S.reduced() || document.hidden) return;
      const a = el.animate(
        [
          { opacity: 0, transform: "translateY(5px)" },
          { opacity: 1, transform: "none" },
        ],
        { duration: S.dur(220), easing: S.curve.out },
      );
      state.animations.add(a);
      a.finished.catch(() => {}).finally(() => state.animations.delete(a));
    };
    const cancelAnimations = () => {
      state.animations.forEach((a) => a.cancel());
      state.animations.clear();
    };
    const later = (fn, ms) => {
      const id = setTimeout(() => {
        state.timers.delete(id);
        if (state.active && !document.hidden) fn();
      }, ms);
      state.timers.add(id);
      return id;
    };
    const clearTimers = () => {
      state.timers.forEach(clearTimeout);
      state.timers.clear();
    };
    const passed = (t) => checks.every((c) => t.checks[c.id] === "passed");
    const canMerge = (t) =>
      passed(t) && t.approved && !t.merged && t.phase === "idle";
    const threadHTML = (t) =>
      `<button class="ws-thread" data-ws-thread="${t.id}" aria-current="${t.id === state.selected ? "true" : "false"}"><span class="ws-thread-icon">${ic(t.merged ? "git-merge" : t.phase === "running" ? "loader" : "chat", 15)}</span><span><strong>${esc(t.title)}</strong><small>#${t.number} · ${t.merged ? "Merged" : t.draftPR ? "Draft" : t.phase === "running" ? "Running" : Object.values(t.checks).includes("failed") ? "Check failed" : "Open"}</small></span></button>`;
    function renderThreads() {
      const html = state.threads.map(threadHTML).join("");
      if (state.threadHTML === html) return;
      const focus = document.activeElement?.dataset.wsThread;
      state.threadHTML = html;
      $("[data-ws-threads]").innerHTML = html;
      $("[data-ws-mobile-threads]").innerHTML = html;
      if (focus) {
        const container = $("[data-ws-nav-dialog]").open
          ? $("[data-ws-mobile-threads]")
          : $("[data-ws-threads]");
        container
          .querySelector(`[data-ws-thread="${focus}"]`)
          ?.focus({ preventScroll: true });
      }
    }
    function renderFiles() {
      const t = current();
      const signature = t.id + t.files.map((file) => file.path).join("|");
      if (signature === state.fileHTML) return;
      state.fileHTML = signature;
      $("[data-ws-file-total]").textContent = t.files.length;
      $("[data-ws-files]").innerHTML = t.files
        .map(
          (f, i) =>
            `<button data-ws-file="${i}" title="${esc(f.path)}">${ic("document", 14)}<span>${esc(f.path)}</span><small>M</small></button>`,
        )
        .join("");
    }
    function toolHTML(tool, index) {
      const kindIcon =
        tool.kind === "read"
          ? "document"
          : tool.kind === "edit"
            ? "file-diff"
            : "terminal";
      return `<details class="ws-tool" data-ws-tool="${index}" data-status="${tool.status}"><summary>${ic(kindIcon, 15)}<span>${esc(tool.label)}</span><span class="ws-tool-state">${ic(statusIcon(tool.status), 14)}</span>${ic("chevron-down", 12)}</summary><div class="ws-tool-output">${tool.kind === "read" ? `<code>${esc(current().files[0].before.join("\n"))}</code>` : tool.kind === "edit" ? `<button class="ws-inline-link" data-ws-open-tab="changes">View file changes ${ic("arrow-right", 12)}</button>` : `<button class="ws-inline-link" data-ws-open-tab="checks">Open check logs ${ic("arrow-right", 12)}</button>`}</div></details>`;
    }
    function renderMessages(follow = true) {
      const view = $("[data-ws-messages]");
      const nearBottom =
        view.scrollHeight - view.scrollTop - view.clientHeight < 90;
      const opened = new Set(
        [...view.querySelectorAll("details[open]")].map(
          (el) =>
            el.closest("[data-ws-message]").dataset.wsMessage +
            ":" +
            el.dataset.wsTool,
        ),
      );
      const active = view.contains(document.activeElement)
        ? document.activeElement.closest("[data-ws-tool]")
        : null;
      const focusKey = active
        ? active.closest("[data-ws-message]").dataset.wsMessage +
          ":" +
          active.dataset.wsTool
        : null;
      view.innerHTML = current()
        .messages.map(
          (m, i) =>
            `<article class="ws-message ws-message-${m.role}" data-ws-message="${i}"><div class="ws-message-avatar">${ic(m.role === "user" ? "user" : "code", 15)}</div><div class="ws-message-body"><header><strong>${m.role === "user" ? "You" : "Assistant"}</strong>${m.streaming ? '<span class="ws-streaming-label">Working</span>' : ""}</header><p data-ws-message-text>${esc(m.text)}</p>${m.tools?.map(toolHTML).join("") || ""}${m.stopped ? '<span class="ws-muted">Stopped</span>' : ""}</div></article>`,
        )
        .join("");
      view.querySelectorAll("[data-ws-tool]").forEach((el) => {
        const key =
          el.closest("[data-ws-message]").dataset.wsMessage +
          ":" +
          el.dataset.wsTool;
        el.open = opened.has(key);
        if (key === focusKey)
          el.querySelector("summary").focus({ preventScroll: true });
      });
      state.toolStatuses ||= new Map();
      current().messages.forEach((message, i) =>
        message.tools?.forEach((tool, j) => {
          const key = current().id + ":" + i + ":" + j,
            previous = state.toolStatuses.get(key);
          state.toolStatuses.set(key, tool.status);
          if (
            state.active &&
            previous !== undefined &&
            previous !== tool.status
          ) {
            S.playIcon(
              view.querySelector(
                `[data-ws-message="${i}"] [data-ws-tool="${j}"] .ws-tool-state .ic`,
              ),
            );
          }
        }),
      );
      if (follow && nearBottom) view.scrollTop = view.scrollHeight;
    }
    function logLines(t, id) {
      const c = checks.find((c) => c.id === id),
        status = t.checks[id];
      if (status === "queued")
        return ["$ " + c.command, "Waiting for a runner."];
      if (status === "running") return ["$ " + c.command, "Running…"];
      if (status === "cancelled") return ["$ " + c.command, "Run cancelled."];
      if (status === "failed")
        return [
          "$ " + c.command,
          "> vitest run",
          "FAIL src/search.test.ts",
          "Expected: defaultPrevented === true",
          "Received: false",
          "Fix: prevent the default browser shortcut.",
          "Tests  1 failed | 23 passed",
          "Process exited with code 1.",
        ];
      return ["$ " + c.command, ...c.success];
    }
    function renderInspector() {
      const t = current(),
        panel = $("[data-ws-review-content]"),
        oldFocus = panel.contains(document.activeElement)
          ? document.activeElement?.dataset.wsFocus
          : null;
      const scroll = panel.scrollTop;
      const deltas = t.files.flatMap(diffLines),
        additions = deltas.filter((line) => line.kind === "added").length,
        deletions = deltas.filter((line) => line.kind === "removed").length;
      if (state.tab === "pr")
        panel.innerHTML = `<div class="ws-pr-status">${ic(t.merged ? "git-merge" : "git-pull-request", 17)}<span>${t.merged ? "Merged" : t.draftPR ? "Draft" : "Open"}</span><span class="ws-muted">#${t.number}</span></div><h2 class="ws-pr-title">${esc(t.title)}</h2><div class="ws-branch-pair"><code>${esc(t.branch)}</code>${ic("arrow-right", 12)}<code>main</code></div><div class="ws-change-count"><button data-ws-open-tab="changes">${t.files.length} files changed</button><span class="ws-added">+${additions}</span><span class="ws-deleted">−${deletions}</span></div><div class="ws-pr-checklist"><button data-ws-open-tab="checks">${ic(passed(t) ? "check-circle" : "x", 16)}<span>${passed(t) ? "All checks passed" : Object.values(t.checks).includes("running") ? "Checks running" : Object.values(t.checks).includes("failed") ? "1 check failed" : "Checks incomplete"}</span>${ic("chevron-right", 12)}</button><div>${ic(t.approved ? "check-circle" : "user", 16)}<span>${t.approved ? "Approved by you" : "Review required"}</span></div></div><button class="btn ${t.approved ? "btn-secondary" : "btn-soft"} ws-approve" data-ws-approve data-ws-focus="approve" ${t.merged || t.phase === "running" ? "disabled" : ""}>${ic(t.approved ? "check" : "user", 14)}${t.approved ? "Revoke approval" : "Approve changes"}</button><button class="btn btn-primary ws-merge" data-ws-merge data-ws-focus="merge" ${canMerge(t) ? "" : "disabled"}>${ic("git-merge", 16)}${t.merged ? "Merged" : "Merge pull request"}</button>${t.merged ? '<p class="ws-panel-note">Merged into main. See Activity for the commit.</p>' : !passed(t) ? '<button class="ws-inline-link" data-ws-prompt="Fix the failed check">Fix failed check ' + ic("arrow-right", 12) + "</button>" : ""}`;
      else if (state.tab === "changes") {
        const f = t.files[state.file] || t.files[0];
        panel.innerHTML = `<div class="ws-file-tabs">${t.files.map((file, i) => `<button data-ws-file="${i}" data-ws-focus="file-${i}" aria-pressed="${i === state.file}">${ic("file-diff", 13)}${esc(file.path.split("/").at(-1))}</button>`).join("")}</div><div class="ws-diff-heading"><code>${esc(f.path)}</code><button class="btn btn-ghost btn-icon btn-sm" data-ws-copy-path aria-label="Copy file path">${ic("copy", 14)}</button></div><div class="ws-diff" aria-label="File diff"><div class="ws-diff-line ws-diff-context"><span></span><span></span><code>@@ −1,${f.before.length} +1,${f.after.length} @@</code></div>${diffLines(
          f,
        )
          .map(
            (line) =>
              `<div class="ws-diff-line ws-diff-${line.kind}"><span>${line.old}</span><span>${line.next}</span><code>${line.kind === "added" ? "+" : line.kind === "removed" ? "−" : " "} ${esc(line.text)}</code></div>`,
          )
          .join("")}</div>`;
      } else if (state.tab === "checks")
        panel.innerHTML = `<div class="ws-checks-heading"><strong>${passed(t) ? "3 / 3 passed" : t.phase === "running" ? "Running checks" : "Check results"}</strong><button class="btn btn-ghost btn-sm" data-ws-run-checks ${t.phase === "running" || t.merged ? "disabled" : ""}>${ic("refresh", 14)} Rerun</button></div><div class="ws-checks-list">${checks.map((c) => `<button class="ws-check-row" data-ws-check="${c.id}" data-ws-focus="check-${c.id}" data-status="${t.checks[c.id]}" aria-pressed="${state.check === c.id}">${ic(statusIcon(t.checks[c.id]), 15)}<span>${c.title}</span><small>${t.checks[c.id]}</small></button>`).join("")}</div><div class="ws-log-heading">${ic("terminal", 13)}<span>${esc(checks.find((c) => c.id === state.check).command)}</span><span class="ws-log-code">${t.checks[state.check] === "failed" ? "exit 1" : t.checks[state.check] === "passed" ? "exit 0" : ""}</span></div><pre class="ws-log" data-ws-log>${logLines(
          t,
          state.check,
        )
          .map(
            (line) =>
              `<span class="${/FAIL|Expected:|Received:|code 1/.test(line) ? "is-error" : /PASS|code 0|passed/.test(line) ? "is-success" : ""}">${esc(line)}</span>`,
          )
          .join("")}</pre>`;
      else
        panel.innerHTML = `<ol class="ws-activity">${t.events.map((e) => `<li><span>${ic(/Merged/.test(e.text) ? "git-merge" : /Approved|passed/.test(e.text) ? "check-circle" : /failed|Stopped/.test(e.text) ? "x" : "git-commit", 14)}</span><div><p>${esc(e.text)}</p><time>${esc(e.time)}</time></div></li>`).join("")}</ol>`;
      panel.scrollTop = scroll;
      if (oldFocus)
        panel
          .querySelector(`[data-ws-focus="${oldFocus}"]`)
          ?.focus({ preventScroll: true });
    }
    function renderState() {
      const t = current();
      renderThreads();
      renderFiles();
      $("[data-ws-title]").textContent = t.title;
      $("[data-ws-pr-number]").textContent = "#" + t.number;
      $("[data-ws-branch]").innerHTML =
        ic("git-branch", 12) + " " + esc(t.merged ? "main" : t.branch);
      $("[data-ws-send]").hidden = t.phase === "running";
      $("[data-ws-stop]").hidden = t.phase !== "running";
      $("[data-ws-prompt-input]").disabled = t.merged;
      $("[data-ws-suggestions]").hidden = t.merged || t.phase === "running";
      $("[data-ws-status]").textContent = t.merged
        ? "Merged into main"
        : t.phase === "running"
          ? "Working…"
          : passed(t)
            ? t.approved
              ? "Ready to merge"
              : "Checks passed · Review required"
            : Object.values(t.checks).includes("failed")
              ? "Check failed · Review required"
              : "Checks incomplete · Review required";
      renderInspector();
    }
    function selectTab(tab, focus = false) {
      state.tab = tab;
      $$("[data-ws-tab]").forEach((b) => {
        const selected = b.dataset.wsTab === tab;
        b.setAttribute("aria-selected", String(selected));
        b.tabIndex = selected ? 0 : -1;
      });
      $("[data-ws-review-content]").setAttribute(
        "aria-labelledby",
        "ws-tab-" + tab,
      );
      renderInspector();
      if (focus) $(`[data-ws-tab="${tab}"]`).focus({ preventScroll: true });
      animate($("[data-ws-review-content]"));
    }
    const review = $("[data-ws-review]"),
      reviewDialog = $("[data-ws-panel-dialog]"),
      app = $("[data-ws-app]"),
      small = matchMedia("(max-width:1320px)");
    function openReview(tab = state.tab, trigger) {
      selectTab(tab);
      if (small.matches && !reviewDialog.open) {
        state.reviewReturn = trigger || $("[data-ws-review-toggle]");
        reviewDialog.append(review);
        reviewDialog.showModal();
        $(`[data-ws-tab="${tab}"]`).focus();
      } else review.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
    function closeReview() {
      if (reviewDialog.open) {
        reviewDialog.close();
        app.append(review);
        const target =
          state.reviewReturn?.isConnected &&
          state.reviewReturn.getClientRects().length
            ? state.reviewReturn
            : $("[data-ws-review-toggle]");
        target.focus({ preventScroll: true });
        state.reviewReturn = null;
      }
    }
    reviewDialog.addEventListener("cancel", (e) => {
      e.preventDefault();
      closeReview();
    });
    reviewDialog.addEventListener("close", () => {
      if (!reviewDialog.open && review.parentElement === reviewDialog)
        app.append(review);
    });
    small.addEventListener("change", () => {
      if (!small.matches) closeReview();
    });
    function stopJob(announce = true) {
      if (!state.job) return;
      const job = state.job,
        t = state.threads.find((t) => t.id === job.thread);
      clearTimers();
      state.job = null;
      t.phase = "idle";
      const msg = t.messages[job.message];
      msg.streaming = false;
      msg.stopped = true;
      msg.tools?.forEach((tool) => {
        if (tool.status === "running" || tool.status === "queued")
          tool.status = "cancelled";
      });
      checks.forEach((c) => {
        if (t.checks[c.id] === "running" || t.checks[c.id] === "queued")
          t.checks[c.id] = "cancelled";
      });
      event(t, "Stopped generation");
      if (state.selected === t.id) {
        renderMessages(false);
        renderState();
        if (announce) $("[data-ws-status]").textContent = "Stopped";
      }
    }
    function closeNav() {
      const dialog = $("[data-ws-nav-dialog]");
      if (!dialog.open) return;
      dialog.close();
      state.navReturn?.focus({ preventScroll: true });
      state.navReturn = null;
    }
    $("[data-ws-nav-dialog]").addEventListener("cancel", (e) => {
      e.preventDefault();
      closeNav();
    });
    function selectThread(id) {
      if (id === state.selected) {
        closeNav();
        return;
      }
      current().draft = $("[data-ws-prompt-input]").value;
      stopJob(false);
      state.selected = id;
      state.file = 0;
      state.check = "tests";
      $("[data-ws-prompt-input]").value = current().draft;
      renderMessages();
      renderState();
      $("[data-ws-messages]").scrollTop = $("[data-ws-messages]").scrollHeight;
      closeNav();
    }
    function addThread() {
      stopJob(false);
      const source = fixtures[0],
        id = "new-" + ++state.serial,
        t = createThread({
          ...source,
          id,
          title: "New conversation",
          branch: "work/session-" + state.serial,
          number: 130 + state.serial,
          failed: false,
        });
      t.messages = [];
      t.draft = "";
      t.draftPR = true;
      t.repaired = false;
      t.events = [{ text: "Draft session created", time: clock() }];
      checks.forEach((c) => (t.checks[c.id] = "queued"));
      t.approved = false;
      state.threads.unshift(t);
      selectThread(id);
      $("[data-ws-prompt-input]").focus();
    }
    function send(text) {
      const t = current();
      text = text.trim();
      if (!text || t.phase === "running" || t.merged) return;
      if (t.title === "New conversation") {
        t.title = text.length > 34 ? text.slice(0, 31) + "…" : text;
      }
      t.messages.push({ role: "user", text });
      $("[data-ws-prompt-input]").value = "";
      t.draft = "";
      const plan =
          state.mode === "plan" ||
          /^(explain|summari[sz]e|why|what)\b/i.test(text),
        runOnly =
          /^(run|rerun|retry)\b/i.test(text) && !/fix|repair/i.test(text);
      const response = plan
        ? `Files: ${t.files.map((f) => f.path).join(", ")}. ${t.failed && !t.repaired ? "The failing test expects the shortcut handler to prevent the browser default. Add preventDefault, then rerun the checks." : "The changes are ready for review. All three checks must pass and approval is required before merging."}`
        : runOnly
          ? "Running TypeScript, unit tests and the production build."
          : `Updating ${t.files[0].path}. I’ll run the checks after applying the patch.`;
      const message = {
        role: "assistant",
        text: "",
        streaming: true,
        tools: [],
      };
      t.messages.push(message);
      const index = t.messages.length - 1;
      t.phase = "running";
      if (!plan && !runOnly) t.approved = false;
      const job = (state.job = {
        thread: t.id,
        message: index,
        kind: plan ? "plan" : runOnly ? "checks" : "fix",
      });
      renderMessages();
      renderState();
      $("[data-ws-messages]").scrollTop = $("[data-ws-messages]").scrollHeight;
      let cursor = 0;
      function stream() {
        if (state.job !== job) return;
        cursor = Math.min(response.length, cursor + 11);
        message.text = response.slice(0, cursor);
        const textNode = $(
          `[data-ws-message="${index}"] [data-ws-message-text]`,
        );
        if (textNode) textNode.textContent = message.text;
        if (cursor < response.length) later(stream, 24);
        else {
          message.streaming = false;
          if (plan) {
            finish();
            return;
          }
          startWork();
        }
      }
      function finish() {
        if (state.job !== job) return;
        t.phase = "idle";
        message.streaming = false;
        state.job = null;
        event(
          t,
          plan
            ? "Reviewed changes"
            : passed(t)
              ? "All checks passed"
              : "Unit tests failed",
        );
        if (!plan)
          message.text = passed(t)
            ? `Checks passed. ${t.files.length} files are ready for review.`
            : "Unit tests failed. Open Checks to inspect the log.";
        renderMessages();
        renderState();
      }
      function startWork() {
        message.tools = [
          {
            kind: "read",
            label: "Read " + t.files[0].path,
            status: runOnly ? "passed" : "running",
          },
        ];
        renderMessages();
        later(
          () => {
            if (state.job !== job) return;
            message.tools[0].status = "passed";
            if (!runOnly) {
              if (!t.repaired) {
                const file = t.files.find((f) => f.path === "src/search.ts");
                if (
                  file &&
                  !file.after.some((line) =>
                    line.includes("event.preventDefault()"),
                  )
                ) {
                  const at = file.after.findIndex((line) =>
                    line.includes("openSearch()"),
                  );
                  file.after.splice(at, 0, "    event.preventDefault();");
                }
              }
              t.repaired = true;
              t.failed = false;
              t.draftPR = false;
              message.tools.push({
                kind: "edit",
                label: `Patched ${t.files.length} files`,
                status: "passed",
              });
              event(t, "Applied file changes");
            }
            message.tools.push({
              kind: "test",
              label: "Run checks",
              status: "running",
            });
            checks.forEach((c) => (t.checks[c.id] = "queued"));
            runCheck(0);
          },
          runOnly ? 180 : 500,
        );
      }
      function runCheck(i) {
        if (state.job !== job) return;
        if (i >= checks.length) {
          message.tools.at(-1).status = passed(t) ? "passed" : "failed";
          finish();
          return;
        }
        const c = checks[i];
        t.checks[c.id] = "running";
        renderMessages(false);
        renderState();
        later(
          () => {
            if (state.job !== job) return;
            t.checks[c.id] =
              c.id === "tests" && !t.repaired ? "failed" : "passed";
            renderState();
            runCheck(i + 1);
          },
          c.id === "tests" ? 750 : 500,
        );
      }
      later(stream, 70);
    }
    function requestMerge() {
      const t = current();
      if (!canMerge(t)) return;
      $("[data-ws-merge-summary]").innerHTML =
        `<p><strong>#${t.number} ${esc(t.title)}</strong></p><p><code>${esc(t.branch)}</code> ${ic("arrow-right", 12)} <code>main</code></p><p>3 checks passed · Approved by you</p>`;
      state.mergeReturn = document.activeElement;
      $("[data-ws-confirm]").showModal();
      $("[data-ws-cancel-merge]").focus();
    }
    function merge() {
      const t = current();
      if (!canMerge(t)) return;
      t.merged = true;
      event(
        t,
        `Merged #${t.number} into main · ${("a" + t.number + "c72e").slice(0, 7)}`,
      );
      $("[data-ws-confirm]").close();
      t.messages.push({
        role: "assistant",
        text: `Merged #${t.number} into main.`,
        tools: [
          { kind: "edit", label: "Merge commit created", status: "passed" },
        ],
      });
      renderMessages();
      renderState();
      (reviewDialog.open
        ? $("[data-ws-review-close]")
        : $("[data-ws-review-toggle]")
      ).focus({ preventScroll: true });
      S.playIcon($("[data-ws-review-toggle] .ic"));
    }
    page.addEventListener("click", async (e) => {
      const button = e.target.closest("button");
      if (!button) return;
      if (button.hasAttribute("data-ws-thread"))
        selectThread(button.dataset.wsThread);
      else if (button.hasAttribute("data-ws-new")) addThread();
      else if (button.hasAttribute("data-ws-file")) {
        state.file = Number(button.dataset.wsFile);
        openReview("changes", button);
      } else if (button.hasAttribute("data-ws-check")) {
        state.check = button.dataset.wsCheck;
        renderInspector();
      } else if (button.hasAttribute("data-ws-tab"))
        selectTab(button.dataset.wsTab);
      else if (button.hasAttribute("data-ws-open-tab"))
        openReview(button.dataset.wsOpenTab, button);
      else if (button.hasAttribute("data-ws-review-toggle"))
        openReview(state.tab, button);
      else if (button.hasAttribute("data-ws-review-close")) closeReview();
      else if (button.hasAttribute("data-ws-sidebar")) {
        state.navReturn = button;
        $("[data-ws-nav-dialog]").showModal();
      } else if (button.hasAttribute("data-ws-nav-close")) {
        closeNav();
      } else if (button.hasAttribute("data-ws-prompt"))
        send(button.dataset.wsPrompt);
      else if (button.hasAttribute("data-ws-run-checks")) send("Run checks");
      else if (button.hasAttribute("data-ws-mode")) {
        state.mode = button.dataset.wsMode;
        $$("[data-ws-mode]").forEach((b) =>
          b.setAttribute("aria-pressed", String(b === button)),
        );
      } else if (button.hasAttribute("data-ws-stop")) stopJob();
      else if (button.hasAttribute("data-ws-approve")) {
        const t = current();
        if (t.merged || t.phase === "running") return;
        t.approved = !t.approved;
        event(t, t.approved ? "Approved by you" : "Approval revoked");
        renderState();
      } else if (button.hasAttribute("data-ws-merge")) requestMerge();
      else if (button.hasAttribute("data-ws-confirm-merge")) merge();
      else if (button.hasAttribute("data-ws-cancel-merge")) {
        $("[data-ws-confirm]").close();
        state.mergeReturn?.focus({ preventScroll: true });
      } else if (button.hasAttribute("data-ws-copy-path")) {
        const ok = await S.copy(current().files[state.file].path);
        $("[data-ws-status]").textContent = ok
          ? "File path copied"
          : "Copy failed";
      } else if (button.hasAttribute("data-ws-rename")) {
        $("#ws-name").value = current().title;
        $("[data-ws-name-dialog]").showModal();
        $("#ws-name").focus();
        $("#ws-name").select();
      } else if (button.hasAttribute("data-ws-name-cancel"))
        $("[data-ws-name-dialog]").close();
      else if (button.hasAttribute("data-ws-reset")) {
        stopJob(false);
        state.threads = fixtures.map(createThread);
        state.selected = "search";
        state.file = 0;
        state.check = "tests";
        state.tab = "pr";
        $("[data-ws-prompt-input]").value = "";
        selectTab("pr");
        renderMessages();
        renderState();
      }
    });
    $("[data-ws-composer]").addEventListener("submit", (e) => {
      e.preventDefault();
      send($("[data-ws-prompt-input]").value);
    });
    $("[data-ws-prompt-input]").addEventListener("input", (e) => {
      current().draft = e.target.value;
      e.target.style.height = "auto";
      e.target.style.height = Math.min(e.target.scrollHeight, 150) + "px";
    });
    $("[data-ws-prompt-input]").addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
        e.preventDefault();
        send(e.target.value);
      }
    });
    $("[data-ws-name-form]").addEventListener("submit", (e) => {
      e.preventDefault();
      const title = $("#ws-name").value.trim();
      if (!title) return;
      current().title = title;
      $("[data-ws-name-dialog]").close();
      renderState();
    });
    page.addEventListener("keydown", (e) => {
      const dialog = e.target.closest("dialog[open]");
      if (dialog) {
        e.stopPropagation();
        if (e.key === "Tab") {
          const controls = [
            ...dialog.querySelectorAll(
              'button:not(:disabled),input:not(:disabled),textarea:not(:disabled),[href],[tabindex="0"]',
            ),
          ].filter(
            (el) => el.getClientRects().length && !el.closest("[inert]"),
          );
          const first = controls[0],
            last = controls.at(-1);
          if (e.shiftKey && document.activeElement === first) {
            e.preventDefault();
            last?.focus();
          } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault();
            first?.focus();
          }
        }
      }
      const tab = e.target.closest("[data-ws-tab]");
      if (tab && ["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
        e.preventDefault();
        const tabs = $$("[data-ws-tab]"),
          i = tabs.indexOf(tab),
          next =
            e.key === "Home"
              ? 0
              : e.key === "End"
                ? tabs.length - 1
                : (i + (e.key === "ArrowRight" ? 1 : -1) + tabs.length) %
                  tabs.length;
        selectTab(tabs[next].dataset.wsTab, true);
      }
    });
    function leave() {
      state.active = false;
      stopJob(false);
      clearTimers();
      cancelAnimations();
      $$("dialog[open]").forEach((d) => d.close());
      if (review.parentElement === reviewDialog) app.append(review);
    }
    state.leave = leave;
    const settleMotion = () => {
      if (S.reduced()) cancelAnimations();
    };
    new MutationObserver(settleMotion).observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-motion"],
    });
    matchMedia("(prefers-reduced-motion:reduce)").addEventListener(
      "change",
      settleMotion,
    );
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) {
        stopJob(false);
        cancelAnimations();
      }
    });
    renderThreads();
    renderFiles();
    renderMessages();
    renderState();
  }
  S.page({
    id: "workspace",
    title: "Coding workspace",
    group: "Examples",
    icon: S.iconNames.includes("code") ? "code" : "cube",
    order: 80,
    count: 1,
    description: "Chat, diffs, pull requests, CI checks and merges.",
    render,
    mount,
    enter(root) {
      const state = instances.get(root);
      if (state) state.active = true;
    },
    leave(root) {
      instances.get(root)?.leave();
    },
  });
})();
