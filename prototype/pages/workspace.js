/* Offline IDE simulation. Code, conversations and CI state stay in this page. */
(() => {
  const S = window.Schematic,
    instances = new WeakMap(),
    esc = S.esc;
  const ic = (name, size = 16) =>
    S.icon(S.iconNames.includes(name) ? name : "code", { size });
  const patch = (path, before, after = before) => ({
    path,
    before: before.split("\n"),
    after: after.split("\n"),
  });
  const searchSource = `import { createSearchStore } from './search-store';
import type { SearchResult } from './types';

export interface SearchOptions {
  root: HTMLElement;
  onSelect: (result: SearchResult) => void;
}

export function createSearchController(options: SearchOptions) {
  const { root, onSelect } = options;
  const store = createSearchStore();
  const input = root.querySelector<HTMLInputElement>('[data-search-input]');
  const panel = root.querySelector<HTMLElement>('[data-search-panel]');
  let previousFocus: HTMLElement | null = null;

  function openSearch() {
    previousFocus = document.activeElement as HTMLElement;
    store.open();
    panel?.removeAttribute('hidden');
    input?.focus();
  }

  function handleShortcut(event: KeyboardEvent) {
    const modifier = event.metaKey || event.ctrlKey;
    const isShortcut = modifier && event.key.toLowerCase() === 'k';
    if (!isShortcut || event.repeat) return;
    openSearch();
  }

  function closeSearch() {
    store.close();
    panel?.setAttribute('hidden', '');
    previousFocus?.focus();
  }

  function handleInput() {
    store.setQuery(input?.value ?? '');
  }

  function selectResult(result: SearchResult) {
    onSelect(result);
    closeSearch();
  }

  function dispose() {
    document.removeEventListener('keydown', handleShortcut);
    input?.removeEventListener('input', handleInput);
    store.dispose();
  }

  document.addEventListener('keydown', handleShortcut);
  input?.addEventListener('input', handleInput);

  return {
    open: openSearch,
    close: closeSearch,
    select: selectResult,
    dispose,
    get state() {
      return store.snapshot();
    },
  };
}`;
  const searchBefore = searchSource.replace(
    "    const modifier = event.metaKey || event.ctrlKey;\n    const isShortcut = modifier && event.key.toLowerCase() === 'k';\n    if (!isShortcut || event.repeat) return;",
    "    if (event.key !== 'k') return;",
  );
  const searchTests = `import { afterEach, describe, expect, it, vi } from 'vitest';
import { createSearchController } from './search';

function setup() {
  const root = document.createElement('section');
  root.innerHTML = '<div data-search-panel hidden><input data-search-input></div>';
  document.body.append(root);
  const controller = createSearchController({ root, onSelect: vi.fn() });
  return { root, controller };
}

function press(key: string, init: KeyboardEventInit = {}) {
  const event = new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  document.dispatchEvent(event);
  return event;
}

afterEach(() => {
  document.body.replaceChildren();
});

describe('search keyboard shortcut', () => {
  it('prevents the browser shortcut on macOS', () => {
    const { controller } = setup();
    const event = press('k', { metaKey: true });
    expect(event.defaultPrevented).toBe(true);
    expect(controller.state.isOpen).toBe(true);
    controller.dispose();
  });

  it('supports the control key on Windows', () => {
    const { controller } = setup();
    press('k', { ctrlKey: true });
    expect(controller.state.isOpen).toBe(true);
    controller.dispose();
  });

  it('ignores unmodified keys', () => {
    const { controller } = setup();
    press('k');
    expect(controller.state.isOpen).toBe(false);
    controller.dispose();
  });
});`;
  const fixtures = [
    {
      id: "search",
      title: "Search shortcut",
      branch: "fix/search-shortcut",
      number: 128,
      failed: true,
      request:
        "Fix Cmd+K so search opens without triggering the browser shortcut.",
      summary:
        "The modifier guard and regression tests are in place. One test is failing: the shortcut still reaches the browser. The failing assertion is in the CI console.",
      files: [
        patch("src/search.ts", searchBefore, searchSource),
        patch(
          "src/search.test.ts",
          searchTests.replace(
            "    expect(event.defaultPrevented).toBe(true);\n",
            "",
          ),
          searchTests,
        ),
        patch(
          "CHANGELOG.md",
          "## Unreleased\n\n### Fixed\n",
          "## Unreleased\n\n### Fixed\n- Search accepts Cmd+K and Ctrl+K.\n- Unmodified keys no longer open search.",
        ),
      ],
    },
    {
      id: "sorting",
      title: "List drag handles",
      branch: "feat/drag-handles",
      number: 129,
      failed: false,
      request: "Add pointer and keyboard sorting with an Escape rollback.",
      summary:
        "Drag handles preserve the original order until drop. Escape restores the snapshot and focus. Pointer, keyboard and cancellation checks passed.",
      files: [
        patch(
          "src/task-list.ts",
          `import type { Task } from './types';

export function createTaskList(initial: Task[]) {
  let tasks = [...initial];
  return {
    all: () => [...tasks],
    move(id: string, to: number) {
      const from = tasks.findIndex(task => task.id === id);
      if (from < 0) return;
      const [task] = tasks.splice(from, 1);
      tasks.splice(to, 0, task);
    },
  };
}`,
          `import type { Task } from './types';

type DragSession = {
  id: string;
  snapshot: Task[];
  origin: HTMLElement;
};

export function createTaskList(initial: Task[]) {
  let tasks = [...initial];
  let session: DragSession | null = null;

  function move(id: string, to: number) {
    const from = tasks.findIndex(task => task.id === id);
    if (from < 0) return;
    const target = Math.max(0, Math.min(to, tasks.length - 1));
    const [task] = tasks.splice(from, 1);
    tasks.splice(target, 0, task);
  }

  function begin(id: string, origin: HTMLElement) {
    if (session) cancel();
    session = { id, snapshot: [...tasks], origin };
    origin.setAttribute('aria-pressed', 'true');
  }

  function drop() {
    if (!session) return;
    session.origin.setAttribute('aria-pressed', 'false');
    session.origin.focus();
    session = null;
  }

  function cancel() {
    if (!session) return;
    tasks = session.snapshot;
    drop();
  }

  function handleKey(event: KeyboardEvent) {
    if (!session) return;
    if (event.key === 'Escape') cancel();
    if (event.key === 'Enter') drop();
    const delta = event.key === 'ArrowDown' ? 1 : -1;
    if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
    event.preventDefault();
    const index = tasks.findIndex(task => task.id === session?.id);
    move(session.id, index + delta);
  }

  return { all: () => [...tasks], move, begin, drop, cancel, handleKey };
}`,
        ),
        patch(
          "src/task-list.css",
          ".task {\n  display: flex;\n}",
          ".task {\n  display: flex;\n  align-items: center;\n}\n\n.drag-handle {\n  touch-action: none;\n  cursor: grab;\n}\n\n.drag-handle:active {\n  cursor: grabbing;\n}",
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
      request: "Limit API retries and support cancellation during backoff.",
      summary:
        "Requests retry at most three times. The AbortSignal reaches both the request and the backoff timer. All checks passed.",
      files: [
        patch(
          "src/client.ts",
          "export async function load(url: string) {\n  return fetch(url);\n}",
          `const RETRY_LIMIT = 3;
const BASE_DELAY = 200;

function wait(duration: number, signal?: AbortSignal) {
  return new Promise<void>((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason);
      return;
    }
    const timer = setTimeout(finish, duration);
    function finish() {
      signal?.removeEventListener('abort', cancel);
      resolve();
    }
    function cancel() {
      clearTimeout(timer);
      signal?.removeEventListener('abort', cancel);
      reject(signal?.reason);
    }
    signal?.addEventListener('abort', cancel, { once: true });
  });
}

export async function load(url: string, signal?: AbortSignal) {
  for (let attempt = 0; attempt < RETRY_LIMIT; attempt++) {
    const response = await fetch(url, { signal });
    if (response.ok) return response;
    const retryable = response.status === 429 || response.status >= 500;
    if (!retryable || attempt === RETRY_LIMIT - 1) {
      throw new Error('Request failed: ' + response.status);
    }
    await wait(BASE_DELAY * 2 ** attempt, signal);
  }
  throw new Error('Retry limit exceeded');
}`,
        ),
        patch(
          "src/client.test.ts",
          "describe('client', () => {});",
          "describe('client', () => {\n  it('stops after three attempts', retryLimit);\n  it('backs off between requests', backoff);\n  it('cancels pending retries', cancellation);\n});",
        ),
        patch(
          "README.md",
          "## API client",
          "## API client\n\nRequests retry at most three times.\nPass an AbortSignal to cancel a request or pending retry.",
        ),
      ],
    },
  ];
  const checks = [
    {
      id: "types",
      title: "TypeScript",
      command: "pnpm typecheck",
      duration: "0.8s",
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
      duration: "1.4s",
      success: [
        "> vitest run",
        " RUN  v3.2.4 /atlas/storefront",
        "",
        " PASS src/search.test.ts (3 tests)",
        " PASS src/task-list.test.ts (12 tests)",
        " PASS src/client.test.ts (9 tests)",
        "",
        " Test Files  3 passed (3)",
        "      Tests  24 passed (24)",
        "   Duration  1.41s",
        "Process exited with code 0.",
      ],
    },
    {
      id: "build",
      title: "Build",
      command: "pnpm build",
      duration: "0.6s",
      success: [
        "> vite build",
        "vite v6.3.5 building for production...",
        "32 modules transformed.",
        "dist/index.html                  1.42 kB",
        "dist/assets/index-C3xq7.js      148.12 kB",
        "Build completed in 612ms.",
        "Process exited with code 0.",
      ],
    },
  ];
  const statusIcon = (status) =>
    ({
      passed: "check-circle",
      failed: "x",
      running: "loader",
      cancelled: "stop",
    })[status] || "clock";
  const statusLabel = (status) =>
    ({
      passed: "Passed",
      failed: "Failed",
      queued: "Queued",
      running: "Running",
      stale: "Not run",
      cancelled: "Cancelled",
    })[status] || status;
  const createThread = (f) => ({
    ...f,
    files: f.files.map((file) => ({
      ...file,
      before: [...file.before],
      after: [...file.after],
    })),
    draft: "",
    file: 0,
    editorMode: "source",
    editing: false,
    approved: false,
    merged: false,
    phase: "idle",
    revision: 0,
    checks: Object.fromEntries(
      checks.map((c) => [
        c.id,
        f.failed && c.id === "tests" ? "failed" : "passed",
      ]),
    ),
    events: [
      { text: `Opened pull request #${f.number}`, time: "09:41" },
      {
        text: f.failed ? "Unit tests failed" : "All checks passed",
        time: "09:42",
      },
    ],
    messages: [
      { role: "user", text: f.request },
      {
        role: "assistant",
        text: f.summary,
        tools: [
          { kind: "read", label: `Read ${f.files[0].path}`, status: "passed" },
          {
            kind: "edit",
            label: `Updated ${f.files.length} files`,
            status: "passed",
          },
          {
            kind: "test",
            label: "Run checks",
            status: f.failed ? "failed" : "passed",
          },
        ],
      },
    ],
  });
  // Tokenize text only; fixture code is never executed.
  function highlight(source) {
    const pattern =
      /(\/\/[^\n]*|\/\*[\s\S]*?\*\/)|('(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*")|\b(import|from|export|interface|type|const|let|return|function|if|else|for|new|throw|async|await|void|true|false|null|undefined|as|get)\b|\b(\d+(?:\.\d+)?)\b|\b([A-Z][A-Za-z0-9_]*)\b|\b([a-zA-Z_$][\w$]*)(?=\s*\()/g;
    let html = "",
      at = 0;
    for (const match of source.matchAll(pattern)) {
      html += esc(source.slice(at, match.index));
      const kind = match[1]
        ? "comment"
        : match[2]
          ? "string"
          : match[3]
            ? "keyword"
            : match[4]
              ? "number"
              : match[5]
                ? "type"
                : "function";
      html += `<span class="ws-token-${kind}">${esc(match[0])}</span>`;
      at = match.index + match[0].length;
    }
    return html + esc(source.slice(at));
  }
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
      if (i < a.length && j < b.length && a[i] === b[j])
        lines.push({ kind: "context", old: ++i, next: ++j, text: a[i - 1] });
      else if (
        i < a.length &&
        (j === b.length || table[i + 1][j] >= table[i][j + 1])
      )
        lines.push({ kind: "removed", old: ++i, next: "", text: a[i - 1] });
      else lines.push({ kind: "added", old: "", next: ++j, text: b[j - 1] });
    }
    return lines;
  }
  function splitRows(file) {
    const lines = diffLines(file),
      rows = [];
    for (let i = 0; i < lines.length; ) {
      if (lines[i].kind === "context") {
        rows.push([lines[i], lines[i]]);
        i++;
        continue;
      }
      const removed = [],
        added = [];
      while (i < lines.length && lines[i].kind !== "context") {
        (lines[i].kind === "removed" ? removed : added).push(lines[i++]);
      }
      for (let k = 0; k < Math.max(removed.length, added.length); k++)
        rows.push([removed[k], added[k]]);
    }
    return rows;
  }
  const render =
    () => `<div class="pg-workspace"><div class="demo ws-app" data-ws-app data-view="editor" data-console="open">
    <header class="ws-topbar"><h1 class="demo-title ws-sr">Coding workspace</h1><span class="demo-desc ws-sr">Code editor, split diffs, CI console and local coding chat.</span><button class="ws-icon-button ws-mobile-only" data-ws-sidebar aria-label="Open project files">${ic("folder")}</button><div class="ws-project">${ic("code", 18)}<span>atlas <span class="ws-muted">/</span> <strong>storefront</strong></span></div><button class="ws-branch-button" data-ws-sessions aria-label="Switch coding session"><span data-ws-branch>${ic("git-branch", 13)} fix/search-shortcut</span>${ic("chevron-down", 12)}</button><span class="ws-local">Local demo</span><span class="ws-topbar-space"></span><button class="ws-pr-button" data-ws-review-toggle>${ic("git-pull-request", 14)}<span>Pull request <span data-ws-pr-number>#128</span></span><span class="ws-pr-indicator" data-ws-pr-indicator></span></button><button class="ws-icon-button ws-chat-toggle" data-ws-chat-toggle aria-pressed="true" aria-label="Toggle assistant panel">${ic("chat")}</button></header>
    <div class="ws-mobile-views" role="tablist" aria-label="Workspace views">${[
      ["editor", "code", "Editor"],
      ["chat", "chat", "Assistant"],
      ["checks", "terminal", "Checks"],
    ]
      .map(
        ([id, icon, label], i) =>
          `<button role="tab" data-ws-view="${id}" aria-selected="${i === 0}" tabindex="${i === 0 ? 0 : -1}">${ic(icon, 14)}${label}<span ${id === "checks" ? "data-ws-mobile-check-count" : ""}>${id === "checks" ? "1" : ""}</span></button>`,
      )
      .join("")}</div>
    <div class="ws-body"><nav class="ws-activity" aria-label="Workspace tools"><button class="ws-icon-button is-active" data-ws-explorer-toggle aria-label="Toggle file explorer" aria-pressed="true">${ic("folder", 20)}</button><button class="ws-icon-button" data-ws-open-tab="changes" aria-label="Review changes">${ic("file-diff", 20)}<span class="ws-activity-count" data-ws-file-total>3</span></button><button class="ws-icon-button" data-ws-sessions aria-label="Coding sessions">${ic("git-branch", 20)}</button><button class="ws-icon-button" data-ws-console-toggle aria-label="Toggle CI console" aria-expanded="true">${ic("terminal", 20)}</button><span class="ws-activity-space"></span><button class="ws-icon-button" data-ws-reset aria-label="Reset local demo">${ic("refresh", 18)}</button></nav>
    <aside class="ws-explorer" data-ws-explorer aria-label="Project explorer"><div class="ws-pane-label"><strong>Explorer</strong><button class="ws-icon-button" data-ws-sessions aria-label="Switch session">${ic("git-branch", 14)}</button></div><button class="ws-project-root" data-ws-tree-toggle aria-expanded="true">${ic("chevron-down", 12)}<strong>STOREFRONT</strong></button><div data-ws-files></div><div class="ws-explorer-bottom"><span class="ws-pane-label">Session</span><button class="ws-session-summary" data-ws-sessions><strong data-ws-title>Search shortcut</strong><small data-ws-session-state>1 check failed</small>${ic("chevron-down", 12)}</button><button class="ws-new-session" data-ws-new>${ic("plus", 14)} New session</button></div></aside>
    <main class="ws-center"><section class="ws-editor-pane" aria-label="Code editor"><div class="ws-file-tabs" role="tablist" aria-label="Open files" data-ws-file-tabs></div><div class="ws-editor-toolbar"><span class="ws-breadcrumb" data-ws-file-path>src / search.ts</span><div class="ws-editor-modes" role="group" aria-label="Editor mode"><button data-ws-editor-mode="source" aria-pressed="true">Source</button><button data-ws-editor-mode="diff" aria-pressed="false">Split diff</button></div><button class="ws-edit-button" data-ws-edit aria-pressed="false">${ic("pencil", 12)} Edit</button></div><div class="ws-editor-content" data-ws-editor-content><div class="ws-source" data-ws-source tabindex="0" role="region" aria-label="File source"></div><div class="ws-edit-area" data-ws-edit-area hidden><pre class="ws-edit-gutter" aria-hidden="true" data-ws-edit-gutter></pre><div class="ws-edit-stack"><pre class="ws-edit-highlight" aria-hidden="true" data-ws-highlight></pre><label class="ws-sr" for="ws-code-input">File contents</label><textarea id="ws-code-input" class="ws-code-input" data-ws-code-input aria-describedby="ws-code-help" spellcheck="false" autocapitalize="off" autocomplete="off" wrap="off"></textarea></div></div><div class="ws-split-diff" data-ws-diff hidden></div></div><div class="ws-editor-footer"><span id="ws-code-help" data-ws-editor-note>Working tree</span><span data-ws-line-count>63 lines</span><span>UTF-8</span><span data-ws-language>TypeScript</span></div></section>
    <section class="ws-console-wrap" data-ws-console-wrap aria-label="CI console"><div class="ws-console-clip"><div class="ws-console"><header class="ws-console-heading"><strong>${ic("terminal", 14)} CI checks</strong><span data-ws-check-summary>2 passed, 1 failed</span><button data-ws-run-checks class="ws-console-rerun">${ic("refresh", 12)} Rerun all</button><button class="ws-icon-button" data-ws-console-toggle aria-label="Collapse CI console" aria-expanded="true">${ic("chevron-down", 14)}</button></header><div class="ws-console-body"><div class="ws-checks-list" aria-label="Checks" data-ws-checks></div><section class="ws-log-pane" aria-label="Selected check output"><div class="ws-log-heading"><code data-ws-log-command>pnpm test</code><span data-ws-log-exit>exit 1</span></div><pre class="ws-log" data-ws-log tabindex="0"></pre></section></div></div></div></section></main>
    <aside class="ws-chat" aria-label="Coding assistant"><header class="ws-chat-heading"><strong>${ic("chat", 14)} Assistant</strong><button class="ws-icon-button" data-ws-rename aria-label="Rename coding session">${ic("pencil", 13)}</button></header><div class="ws-messages" data-ws-messages role="log" aria-label="Conversation messages" aria-live="off"></div><div class="ws-chat-bottom"><div class="ws-suggestions" data-ws-suggestions><button data-ws-prompt="Fix the failed check">${ic("code", 12)} Fix failed check</button><button data-ws-prompt="Explain these changes">Explain changes</button></div><form class="ws-composer" data-ws-composer><label class="ws-sr" for="ws-prompt">Message assistant</label><textarea id="ws-prompt" data-ws-prompt-input rows="3" placeholder="Ask for a code change…" maxlength="2000"></textarea><div class="ws-composer-bar"><div class="ws-mode" role="group" aria-label="Assistant mode"><button type="button" data-ws-mode="build" aria-pressed="true">Build</button><button type="button" data-ws-mode="plan" aria-pressed="false">Plan</button></div><span class="ws-shortcut">Enter</span><button type="submit" class="ws-send" data-ws-send aria-label="Send message">${ic("arrow-up", 16)}</button><button type="button" class="ws-send ws-stop" data-ws-stop aria-label="Stop generation" hidden>${ic("stop", 16)}</button></div></form></div></aside></div>
    <footer class="ws-statusbar"><span data-ws-status-branch>${ic("git-branch", 12)} fix/search-shortcut</span><button data-ws-open-tab="checks">${ic("terminal", 12)}<span data-ws-status>1 check failed</span></button><span class="ws-status-space"></span><span data-ws-cursor>Ln 27, Col 5</span><span>Local changes</span></footer>
    </div>
    <dialog class="ws-review-dialog" data-ws-panel-dialog aria-labelledby="ws-review-title"><header class="ws-dialog-header"><div>${ic("git-pull-request", 18)}<h2 id="ws-review-title">Pull request <span data-ws-dialog-number>#128</span></h2></div><button class="ws-icon-button" data-ws-review-close aria-label="Close pull request">${ic("x")}</button></header><div class="ws-review-tabs" role="tablist" aria-label="Pull request panels"><button role="tab" data-ws-tab="pr" aria-selected="true" tabindex="0">Summary</button><button role="tab" data-ws-tab="activity" aria-selected="false" tabindex="-1">Activity</button></div><div class="ws-review-content" data-ws-review-content role="tabpanel" aria-label="Pull request summary"></div></dialog>
    <dialog class="ws-sessions-dialog" data-ws-nav-dialog aria-labelledby="ws-sessions-title"><header class="ws-dialog-header"><h2 id="ws-sessions-title">Coding sessions</h2><button class="ws-icon-button" data-ws-nav-close aria-label="Close sessions">${ic("x")}</button></header><div data-ws-threads></div><button class="ws-new-session" data-ws-new>${ic("plus", 14)} New session</button></dialog>
    <dialog class="ws-files-dialog" data-ws-files-dialog aria-labelledby="ws-files-title"><header class="ws-dialog-header"><h2 id="ws-files-title">Project files</h2><button class="ws-icon-button" data-ws-files-close aria-label="Close project files">${ic("x")}</button></header><div data-ws-mobile-files></div><button class="ws-new-session" data-ws-sessions>${ic("git-branch", 14)} Switch coding session</button></dialog>
    <dialog class="ws-confirm-dialog" data-ws-confirm aria-labelledby="ws-confirm-title"><h2 id="ws-confirm-title">Merge pull request</h2><div data-ws-merge-summary></div><div class="ws-dialog-actions"><button class="btn btn-secondary" data-ws-cancel-merge>Cancel</button><button class="btn btn-primary" data-ws-confirm-merge>${ic("git-merge", 16)} Merge</button></div></dialog>
    <dialog class="ws-confirm-dialog" data-ws-name-dialog aria-labelledby="ws-name-title"><form data-ws-name-form><h2 id="ws-name-title">Rename session</h2><label class="ws-sr" for="ws-name">Session name</label><input id="ws-name" maxlength="60" required><div class="ws-dialog-actions"><button type="button" class="btn btn-secondary" data-ws-name-cancel>Cancel</button><button type="submit" class="btn btn-primary">Save</button></div></form></dialog>
  </div>`;

  function mount(root) {
    const page = root.querySelector(".pg-workspace"),
      $ = (q) => page.querySelector(q),
      $$ = (q) => [...page.querySelectorAll(q)],
      app = $("[data-ws-app]");
    const state = {
      active: false,
      threads: fixtures.map(createThread),
      selected: "search",
      check: "tests",
      mode: "build",
      tab: "pr",
      job: null,
      timers: new Set(),
      animations: new Map(),
      serial: 0,
      view: "editor",
      console: true,
      dialogReturns: new Map(),
    };
    instances.set(root, state);
    const current = () => state.threads.find((t) => t.id === state.selected);
    const clock = () =>
      new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    const event = (t, text) => t.events.push({ text, time: clock() });
    const passed = (t) => checks.every((c) => t.checks[c.id] === "passed");
    const canMerge = (t) =>
      passed(t) && t.approved && !t.merged && t.phase === "idle";
    function animate(el) {
      state.animations.get(el)?.cancel();
      if (!state.active || S.reduced() || document.hidden) return;
      const animation = el.animate(
        [
          { opacity: 0.4, transform: "translateX(5px)" },
          { opacity: 1, transform: "none" },
        ],
        { duration: S.dur(180), easing: S.curve.out },
      );
      state.animations.set(el, animation);
      animation.finished
        .catch(() => {})
        .finally(() => {
          if (state.animations.get(el) === animation)
            state.animations.delete(el);
        });
    }
    function settleMotion() {
      state.animations.forEach((a) => a.cancel());
      state.animations.clear();
    }
    const later = (fn, ms) => {
      const id = setTimeout(() => {
        state.timers.delete(id);
        if (state.active && !document.hidden) fn();
      }, ms);
      state.timers.add(id);
    };
    const clearTimers = () => {
      state.timers.forEach(clearTimeout);
      state.timers.clear();
    };
    function openDialog(selector, trigger, first) {
      const dialog = $(selector);
      if (dialog.open) return;
      state.dialogReturns.set(dialog, trigger || document.activeElement);
      dialog.showModal();
      (
        dialog.querySelector(first || "button:not(:disabled),input") || dialog
      ).focus();
      animate(dialog);
    }
    function closeDialog(selector, restore = true) {
      const dialog = typeof selector === "string" ? $(selector) : selector;
      if (!dialog.open) return;
      dialog.close();
      const target = state.dialogReturns.get(dialog);
      state.dialogReturns.delete(dialog);
      if (restore && target?.isConnected && target.getClientRects().length)
        target.focus({ preventScroll: true });
    }
    function summary(t) {
      if (t.merged) return "Merged into main";
      if (t.phase === "running") return "Checks running";
      const statuses = Object.values(t.checks);
      if (statuses.includes("failed")) return "1 check failed";
      if (passed(t)) return t.approved ? "Ready to merge" : "3 checks passed";
      if (statuses.includes("cancelled")) return "Checks cancelled";
      return "Checks not run";
    }
    function renderThreads() {
      const html = state.threads
        .map(
          (t) =>
            `<button class="ws-thread" data-ws-thread="${t.id}" aria-current="${t.id === state.selected}">${ic(t.merged ? "git-merge" : "git-branch", 16)}<span><strong>${esc(t.title)}</strong><small>${esc(t.branch)}</small></span><span class="ws-thread-state" data-status="${Object.values(t.checks).includes("failed") ? "failed" : t.merged ? "passed" : "queued"}">${t.merged ? "Merged" : "#" + t.number}</span></button>`,
        )
        .join("");
      if (state.threadHTML === html) return;
      const focus = document.activeElement?.dataset.wsThread;
      state.threadHTML = html;
      $("[data-ws-threads]").innerHTML = html;
      if (focus)
        $(`[data-ws-thread="${focus}"]`)?.focus({ preventScroll: true });
    }
    function renderFiles() {
      const t = current(),
        file = t.files[t.file];
      const tree = `<div class="ws-folder-row">${ic("chevron-down", 12)}${ic("folder", 14)} src</div>${t.files.map((f, i) => `<button class="ws-file-row ${f.path.startsWith("src/") ? "is-nested" : ""}" data-ws-file="${i}" aria-current="${i === t.file}"><span class="ws-file-kind">${f.path.endsWith(".ts") ? "TS" : f.path.endsWith(".css") ? "#" : "MD"}</span><span>${esc(f.path.split("/").at(-1))}</span><small>M</small></button>`).join("")}`;
      const tabs = t.files
        .map(
          (f, i) =>
            `<button class="ws-file-tab" role="tab" data-ws-file="${i}" aria-selected="${i === t.file}" tabindex="${i === t.file ? 0 : -1}"><span class="ws-file-kind">${f.path.endsWith(".ts") ? "TS" : f.path.endsWith(".css") ? "#" : "MD"}</span><span>${esc(f.path.split("/").at(-1))}</span><small>M</small></button>`,
        )
        .join("");
      const signature = t.id + ":" + t.file;
      if (state.fileHTML !== signature) {
        const focused = document.activeElement?.dataset.wsFile;
        state.fileHTML = signature;
        $("[data-ws-files]").innerHTML = tree;
        $("[data-ws-mobile-files]").innerHTML = tree;
        $("[data-ws-file-tabs]").innerHTML = tabs;
        if (focused !== undefined)
          $("[data-ws-file-tabs]")
            .querySelector(`[data-ws-file="${t.file}"]`)
            ?.focus({ preventScroll: true });
      }
      $("[data-ws-file-total]").textContent = t.files.length;
      $("[data-ws-file-path]").textContent = file.path.replaceAll("/", " / ");
      $("[data-ws-line-count]").textContent = `${file.after.length} lines`;
      $("[data-ws-language]").textContent = file.path.endsWith(".ts")
        ? "TypeScript"
        : file.path.endsWith(".css")
          ? "CSS"
          : "Markdown";
    }
    function sourceHTML(file) {
      return file.after
        .map(
          (text, i) =>
            `<div class="ws-code-row" ${current().id === "search" && !hasRepair(current()) && text.trim() === "openSearch();" ? 'data-diagnostic="true"' : ""}><span class="ws-line-number" aria-hidden="true">${i + 1}</span><code>${highlight(text) || " "}</code></div>`,
        )
        .join("");
    }
    function renderEditor() {
      const t = current(),
        file = t.files[t.file],
        source = file.after.join("\n");
      const signature =
        t.id +
        ":" +
        t.file +
        ":" +
        t.revision +
        ":" +
        t.editorMode +
        ":" +
        t.editing;
      if (state.editorHTML === signature) return;
      state.editorHTML = signature;
      renderFiles();
      const sourceView = $("[data-ws-source]"),
        diff = $("[data-ws-diff]"),
        editArea = $("[data-ws-edit-area]"),
        input = $("[data-ws-code-input]");
      sourceView.hidden = t.editorMode !== "source" || t.editing;
      diff.hidden = t.editorMode !== "diff";
      editArea.hidden = t.editorMode !== "source" || !t.editing;
      app.dataset.editorMode = t.editorMode;
      $$("[data-ws-editor-mode]").forEach((b) =>
        b.setAttribute(
          "aria-pressed",
          String(b.dataset.wsEditorMode === t.editorMode),
        ),
      );
      $("[data-ws-edit]").setAttribute("aria-pressed", String(t.editing));
      $("[data-ws-edit]").innerHTML =
        ic(t.editing ? "check" : "pencil", 12) +
        (t.editing ? " Done" : " Edit");
      $("[data-ws-edit]").disabled = t.merged || t.phase === "running";
      $("[data-ws-editor-note]").textContent = t.merged
        ? "Merged · read only"
        : t.editing
          ? "Editing local buffer · Esc or Ctrl/⌘ S to finish"
          : t.editorMode === "diff"
            ? "main ↔ working tree"
            : "Working tree";
      if (t.editorMode === "source" && !t.editing)
        sourceView.innerHTML = sourceHTML(file);
      if (t.editing) {
        if (input.value !== source) input.value = source;
        renderEditable();
      }
      if (t.editorMode === "diff") {
        const cell = (line, side) =>
          `<div class="ws-diff-cell ${line ? "ws-diff-" + line.kind : "ws-diff-empty"}"><span class="ws-line-number" aria-hidden="true">${line ? (side === "old" ? line.old : line.next) : ""}</span><span class="ws-diff-sign" aria-hidden="true">${line?.kind === "removed" ? "−" : line?.kind === "added" ? "+" : ""}</span><code>${line ? highlight(line.text) || " " : " "}</code></div>`;
        const rows = splitRows(file);
        diff.innerHTML = `<div class="ws-diff-labels"><span>${ic("git-branch", 12)} main <small>Original</small></span><span>${ic("pencil", 12)} ${esc(t.branch)} <small>Modified</small></span></div><div class="ws-diff-scroll">${["old", "new"].map((side, index) => `<div class="ws-diff-side" data-ws-diff-side="${side}" tabindex="0" role="region" aria-label="${index ? "Modified" : "Original"} file"><div class="ws-diff-lines">${rows.map((pair) => cell(pair[index], index ? "next" : "old")).join("")}</div></div>`).join("")}</div>`;
        const sides = [...diff.querySelectorAll("[data-ws-diff-side]")];
        sides.forEach((side, index) =>
          side.addEventListener("scroll", () => {
            const other = sides[1 - index];
            if (other.scrollTop !== side.scrollTop)
              other.scrollTop = side.scrollTop;
          }),
        );
      }
    }
    function renderEditable() {
      const input = $("[data-ws-code-input]");
      $("[data-ws-highlight]").innerHTML = highlight(input.value) + "\n";
      $("[data-ws-edit-gutter]").textContent =
        input.value
          .split("\n")
          .map((_, i) => i + 1)
          .join("\n") + "\n";
      syncEditorScroll();
    }
    function syncEditorScroll() {
      const input = $("[data-ws-code-input]");
      $("[data-ws-highlight]").scrollTop = input.scrollTop;
      $("[data-ws-highlight]").scrollLeft = input.scrollLeft;
      $("[data-ws-edit-gutter]").scrollTop = input.scrollTop;
    }
    function hasRepair(t) {
      if (t.id !== "search" && !t.id.startsWith("new-")) return true;
      const body =
        t.files.find((f) => f.path === "src/search.ts")?.after.join("\n") || "";
      return /if \(!isShortcut \|\| event\.repeat\) return;\s*event\.preventDefault\(\);\s*openSearch\(\);/.test(
        body,
      );
    }
    function invalidate(t) {
      t.approved = false;
      t.revision++;
      checks.forEach((c) => {
        t.checks[c.id] = "stale";
      });
    }
    function applyEdit() {
      const t = current();
      if (t.merged || t.phase === "running") return;
      const input = $("[data-ws-code-input]");
      t.files[t.file].after = input.value.split("\n");
      invalidate(t);
      renderEditable();
      renderFiles();
      renderChrome();
      renderChecks();
      renderReview();
    }
    function toolHTML(tool, index) {
      return `<details class="ws-tool" data-ws-tool="${index}" data-status="${tool.status}"><summary>${ic(tool.kind === "read" ? "document" : tool.kind === "edit" ? "file-diff" : "terminal", 13)}<span>${esc(tool.label)}</span><span class="ws-tool-state">${ic(statusIcon(tool.status), 12)}</span>${ic("chevron-down", 10)}</summary><div class="ws-tool-output">${tool.kind === "read" ? `<code>${esc(current().files[0].path)}\n${current().files[0].after.length} lines</code>` : `<button data-ws-open-tab="${tool.kind === "edit" ? "changes" : "checks"}">${tool.kind === "edit" ? "Open split diff" : "Open CI console"} ${ic("arrow-right", 12)}</button>`}</div></details>`;
    }
    function renderMessages(follow = true) {
      const view = $("[data-ws-messages]"),
        nearBottom =
          view.scrollHeight - view.scrollTop - view.clientHeight < 80;
      const keyFor = (el) =>
        el.closest("[data-ws-message]").dataset.wsMessage +
        ":" +
        el.dataset.wsTool;
      const opened = new Set(
        [...view.querySelectorAll("details[open]")].map(keyFor),
      );
      const active = document.activeElement?.closest("[data-ws-tool]"),
        focusKey = active && view.contains(active) ? keyFor(active) : null;
      view.innerHTML = current()
        .messages.map(
          (m, i) =>
            `<article class="ws-message ws-message-${m.role}" data-ws-message="${i}"><header><span class="ws-message-avatar">${ic(m.role === "user" ? "user" : "code", 12)}</span><strong>${m.role === "user" ? "You" : "Assistant"}</strong>${m.streaming ? '<span class="ws-streaming-label">Working</span>' : ""}</header><p data-ws-message-text>${esc(m.text)}</p>${m.tools?.map(toolHTML).join("") || ""}${m.stopped ? '<span class="ws-muted ws-stopped">Stopped</span>' : ""}</article>`,
        )
        .join("");
      view.querySelectorAll("[data-ws-tool]").forEach((el) => {
        const key = keyFor(el);
        el.open = opened.has(key);
        if (key === focusKey)
          el.querySelector("summary").focus({ preventScroll: true });
      });
      state.toolStatuses ||= new Map();
      current().messages.forEach((m, i) =>
        m.tools?.forEach((tool, j) => {
          const key = current().id + ":" + i + ":" + j,
            old = state.toolStatuses.get(key);
          state.toolStatuses.set(key, tool.status);
          if (old !== undefined && old !== tool.status && state.active)
            S.playIcon(
              view.querySelector(
                `[data-ws-message="${i}"] [data-ws-tool="${j}"] .ws-tool-state .ic`,
              ),
            );
        }),
      );
      if (follow && nearBottom) view.scrollTop = view.scrollHeight;
    }
    function logLines(t, id) {
      const c = checks.find((c) => c.id === id),
        status = t.checks[id];
      if (status === "stale")
        return [
          "$ " + c.command,
          "Working tree changed. Rerun all checks for this revision.",
        ];
      if (status === "queued")
        return ["$ " + c.command, "Waiting for previous check…"];
      if (status === "running")
        return ["$ " + c.command, ...c.success.slice(0, 2), "Running…"];
      if (status === "cancelled")
        return [
          "$ " + c.command,
          "Run cancelled. No result recorded for this revision.",
        ];
      if (status === "failed")
        return [
          "$ " + c.command,
          "> vitest run",
          " RUN  v3.2.4 /atlas/storefront",
          "",
          " FAIL  src/search.test.ts > search keyboard shortcut",
          "       prevents the browser shortcut on macOS",
          "",
          " AssertionError: expected false to be true",
          " > src/search.test.ts:31:36",
          "   30 | const event = press('k', { metaKey: true });",
          "   31 | expect(event.defaultPrevented).toBe(true);",
          "      |                                ^",
          " Expected: true",
          " Received: false",
          "",
          " Tests  1 failed | 23 passed (24)",
          "Process exited with code 1.",
        ];
      return ["$ " + c.command, ...c.success];
    }
    function renderChecks() {
      const t = current(),
        focus = document.activeElement?.dataset.wsCheck,
        oldScroll = $("[data-ws-log]").scrollTop;
      const html = checks
        .map(
          (c) =>
            `<button class="ws-check-row" data-ws-check="${c.id}" data-status="${t.checks[c.id]}" aria-pressed="${state.check === c.id}">${ic(statusIcon(t.checks[c.id]), 13)}<span><strong>${c.title}</strong><small>${statusLabel(t.checks[c.id])}</small></span><time>${["failed", "passed"].includes(t.checks[c.id]) ? c.duration : ""}</time></button>`,
        )
        .join("");
      if (state.checkHTML !== html) {
        state.checkHTML = html;
        $("[data-ws-checks]").innerHTML = html;
        if (focus)
          $(`[data-ws-check="${focus}"]`)?.focus({ preventScroll: true });
      }
      $("[data-ws-check-summary]").textContent =
        t.phase === "running"
          ? "Running"
          : `${Object.values(t.checks).filter((s) => s === "passed").length} / 3 passed`;
      $("[data-ws-log-command]").textContent = checks.find(
        (c) => c.id === state.check,
      ).command;
      $("[data-ws-log-exit]").textContent =
        t.checks[state.check] === "failed"
          ? "exit 1"
          : t.checks[state.check] === "passed"
            ? "exit 0"
            : statusLabel(t.checks[state.check]);
      $("[data-ws-log-exit]").dataset.status = t.checks[state.check];
      const log = logLines(t, state.check)
        .map(
          (line) =>
            `<span class="${/FAIL|AssertionError|Expected:|Received:|code 1/.test(line) ? "is-error" : /PASS|passed|code 0/.test(line) ? "is-success" : ""}">${esc(line) || " "}</span>`,
        )
        .join("");
      if (state.logHTML !== log) {
        state.logHTML = log;
        $("[data-ws-log]").innerHTML = log;
        $("[data-ws-log]").scrollTop = oldScroll;
      }
      $("[data-ws-run-checks]").disabled = t.phase === "running" || t.merged;
    }
    function renderReview() {
      const t = current(),
        panel = $("[data-ws-review-content]"),
        focus = panel.contains(document.activeElement)
          ? document.activeElement.dataset.wsFocus
          : null,
        scroll = panel.scrollTop;
      const deltas = t.files.flatMap(diffLines),
        adds = deltas.filter((l) => l.kind === "added").length,
        deletes = deltas.filter((l) => l.kind === "removed").length;
      $("[data-ws-dialog-number]").textContent = "#" + t.number;
      const html =
        state.tab === "activity"
          ? `<ol class="ws-timeline">${t.events.map((e) => `<li>${ic(/Merged/.test(e.text) ? "git-merge" : /passed|Approved/.test(e.text) ? "check-circle" : "git-commit", 14)}<div><p>${esc(e.text)}</p><time>${esc(e.time)}</time></div></li>`).join("")}</ol>`
          : `<div class="ws-pr-state">${ic(t.merged ? "git-merge" : "git-pull-request", 16)}${t.merged ? "Merged" : t.draftPR ? "Draft" : "Open"}</div><h3>${esc(t.title)}</h3><div class="ws-branch-pair"><code>${esc(t.branch)}</code>${ic("arrow-right", 12)}<code>main</code></div><div class="ws-pr-change-count"><button data-ws-open-tab="changes">${t.files.length} changed files</button><span class="ws-added">+${adds}</span><span class="ws-deleted">−${deletes}</span></div><div class="ws-pr-requirements"><button data-ws-open-tab="checks">${ic(passed(t) ? "check-circle" : Object.values(t.checks).includes("failed") ? "x" : "clock", 16)}<span>${summary(t)}</span>${ic("chevron-right", 12)}</button><div>${ic(t.approved ? "check-circle" : "user", 16)}<span>${t.approved ? "Approved by you" : "Review required"}</span></div></div><div class="ws-pr-actions"><button class="btn btn-secondary" data-ws-approve data-ws-focus="approve" ${t.merged || t.phase === "running" ? "disabled" : ""}>${ic(t.approved ? "check" : "user", 14)}${t.approved ? "Revoke approval" : "Approve changes"}</button><button class="btn btn-primary" data-ws-merge data-ws-focus="merge" ${canMerge(t) ? "" : "disabled"}>${ic("git-merge", 14)}${t.merged ? "Merged" : "Merge pull request"}</button></div>`;
      if (state.reviewHTML !== html) {
        state.reviewHTML = html;
        panel.innerHTML = html;
        panel.scrollTop = scroll;
        if (focus)
          panel
            .querySelector(`[data-ws-focus="${focus}"]`)
            ?.focus({ preventScroll: true });
      }
    }
    function renderChrome() {
      const t = current();
      renderThreads();
      $("[data-ws-title]").textContent = t.title;
      $("[data-ws-pr-number]").textContent = "#" + t.number;
      const branch =
        ic("git-branch", 12) + " " + esc(t.merged ? "main" : t.branch);
      $("[data-ws-branch]").innerHTML = branch;
      $("[data-ws-status-branch]").innerHTML = branch;
      $("[data-ws-session-state]").textContent = summary(t);
      $("[data-ws-status]").textContent = summary(t);
      $("[data-ws-pr-indicator]").dataset.status = t.merged
        ? "merged"
        : passed(t)
          ? "passed"
          : Object.values(t.checks).includes("failed")
            ? "failed"
            : "queued";
      $("[data-ws-mobile-check-count]").textContent = Object.values(
        t.checks,
      ).includes("failed")
        ? "1"
        : "";
      $("[data-ws-send]").hidden = t.phase === "running";
      $("[data-ws-stop]").hidden = t.phase !== "running";
      $("[data-ws-send]").disabled = t.merged;
      $("[data-ws-prompt-input]").disabled = t.merged;
      $("[data-ws-code-input]").readOnly = t.merged || t.phase === "running";
      $("[data-ws-suggestions]").hidden = t.merged || t.phase === "running";
      $("[data-ws-edit]").disabled = t.merged || t.phase === "running";
    }
    function renderState() {
      renderChrome();
      renderEditor();
      renderChecks();
      renderReview();
    }
    function setView(view, focus = false) {
      state.view = view;
      app.dataset.view = view;
      $$("[data-ws-view]").forEach((b) => {
        const active = b.dataset.wsView === view;
        b.setAttribute("aria-selected", String(active));
        b.tabIndex = active ? 0 : -1;
      });
      if (focus) $(`[data-ws-view="${view}"]`).focus();
    }
    function setConsole(open) {
      if (
        !open &&
        $("[data-ws-console-wrap]").contains(document.activeElement)
      ) {
        const trigger = $$(".ws-activity [data-ws-console-toggle]").find(
          (el) => el.getClientRects().length,
        );
        trigger?.focus({ preventScroll: true });
      }
      state.console = open;
      app.dataset.console = open ? "open" : "closed";
      $$("[data-ws-console-toggle]").forEach((b) =>
        b.setAttribute("aria-expanded", String(open)),
      );
      $("[data-ws-console-wrap]").inert = !open && state.view !== "checks";
    }
    function editorMode(mode) {
      const t = current();
      t.editorMode = mode;
      t.editing = false;
      renderEditor();
      animate($("[data-ws-editor-content]"));
    }
    function openPanel(tab, trigger) {
      if (tab === "changes") {
        closeDialog("[data-ws-panel-dialog]", false);
        setView("editor");
        editorMode("diff");
        $('[data-ws-editor-mode="diff"]').focus({ preventScroll: true });
      } else if (tab === "checks") {
        closeDialog("[data-ws-panel-dialog]", false);
        setConsole(true);
        if (matchMedia("(max-width:760px)").matches) setView("checks");
        $(`[data-ws-check="${state.check}"]`).focus({ preventScroll: true });
      } else {
        state.tab = tab;
        $$("[data-ws-tab]").forEach((b) => {
          const active = b.dataset.wsTab === tab;
          b.setAttribute("aria-selected", String(active));
          b.tabIndex = active ? 0 : -1;
        });
        renderReview();
        openDialog("[data-ws-panel-dialog]", trigger, "[data-ws-review-close]");
      }
    }
    function stopJob(announce = true) {
      if (!state.job) return;
      const job = state.job,
        t = state.threads.find((t) => t.id === job.thread),
        m = t.messages[job.message];
      clearTimers();
      state.job = null;
      t.phase = "idle";
      m.streaming = false;
      m.stopped = true;
      m.tools.forEach((tool) => {
        if (["running", "queued"].includes(tool.status))
          tool.status = "cancelled";
      });
      if (job.kind !== "plan")
        checks.forEach((c) => {
          if (["running", "queued"].includes(t.checks[c.id]))
            t.checks[c.id] = "cancelled";
        });
      event(t, "Stopped task");
      renderMessages(false);
      renderState();
      if (announce) $("[data-ws-status]").textContent = "Stopped";
    }
    function selectThread(id) {
      closeDialog("[data-ws-nav-dialog]", false);
      if (id === state.selected) {
        $("[data-ws-sessions]").focus({ preventScroll: true });
        return;
      }
      current().draft = $("[data-ws-prompt-input]").value;
      stopJob(false);
      state.selected = id;
      state.check = "tests";
      $("[data-ws-prompt-input]").value = current().draft;
      renderMessages();
      renderState();
      $("[data-ws-messages]").scrollTop = $("[data-ws-messages]").scrollHeight;
      $('[data-ws-editor-mode="source"]').focus({ preventScroll: true });
    }
    function addThread() {
      stopJob(false);
      const id = "new-" + ++state.serial,
        t = createThread({
          ...fixtures[0],
          id,
          title: "New session",
          branch: "work/session-" + state.serial,
          number: 130 + state.serial,
          failed: true,
        });
      t.messages = [];
      t.draftPR = true;
      t.events = [{ text: "Draft session created", time: clock() }];
      checks.forEach((c) => (t.checks[c.id] = "stale"));
      state.threads.unshift(t);
      selectThread(id);
      setView("chat");
      $("[data-ws-prompt-input]").focus();
    }
    function send(text) {
      const t = current();
      text = text.trim();
      if (!text || t.phase === "running" || t.merged) return;
      if (t.title === "New session")
        t.title = text.length > 38 ? text.slice(0, 35) + "…" : text;
      const plan =
          state.mode === "plan" ||
          /^(explain|summari[sz]e|why|what)\b/i.test(text),
        runOnly =
          /^(run|rerun|retry)\b/i.test(text) && !/fix|repair/i.test(text);
      t.messages.push({ role: "user", text });
      $("[data-ws-prompt-input]").value = "";
      t.draft = "";
      t.editing = false;
      const response = plan
        ? `The shortcut guard is in ${t.files[0].path}. ${hasRepair(t) ? "Checks and approval are required before merging." : "The handler opens search without cancelling the keyboard event. Add event.preventDefault() immediately before openSearch(), then rerun CI."}`
        : runOnly
          ? "Running TypeScript, unit tests and the production build against the current working tree."
          : `I’ll update ${t.files[0].path}, then rerun all three CI checks.`;
      const message = {
        role: "assistant",
        text: "",
        streaming: true,
        tools: [],
      };
      t.messages.push(message);
      t.phase = "running";
      if (!plan) {
        if (!runOnly) t.approved = false;
        checks.forEach((c) => (t.checks[c.id] = "queued"));
      }
      const job = (state.job = {
        thread: t.id,
        message: t.messages.length - 1,
        kind: plan ? "plan" : runOnly ? "checks" : "fix",
      });
      renderMessages();
      renderState();
      $("[data-ws-messages]").scrollTop = $("[data-ws-messages]").scrollHeight;
      let cursor = 0;
      function finish() {
        if (state.job !== job) return;
        t.phase = "idle";
        state.job = null;
        message.streaming = false;
        if (!plan)
          message.text = passed(t)
            ? "All three checks passed. The patch is ready for review."
            : "Unit tests failed. The assertion and source location are in the CI console.";
        event(
          t,
          plan
            ? "Reviewed changes"
            : passed(t)
              ? "All checks passed"
              : "Unit tests failed",
        );
        renderMessages();
        renderState();
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
              c.id === "tests" && !hasRepair(t) ? "failed" : "passed";
            renderState();
            runCheck(i + 1);
          },
          c.id === "tests" ? 720 : 440,
        );
      }
      function work() {
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
              const file = t.files.find((f) => f.path === "src/search.ts");
              if (file && !hasRepair(t)) {
                const at = file.after.findIndex(
                  (line) => line.trim() === "openSearch();",
                );
                if (at >= 0)
                  file.after.splice(at, 0, "    event.preventDefault();");
              }
              invalidate(t);
              t.draftPR = false;
              checks.forEach((c) => (t.checks[c.id] = "queued"));
              message.tools.push({
                kind: "edit",
                label: "Applied patch to " + t.files[0].path,
                status: "passed",
              });
              event(t, "Applied file changes");
            }
            message.tools.push({
              kind: "test",
              label: "Run CI checks",
              status: "running",
            });
            renderEditor();
            runCheck(0);
          },
          runOnly ? 140 : 420,
        );
      }
      function stream() {
        if (state.job !== job) return;
        cursor = Math.min(response.length, cursor + 12);
        message.text = response.slice(0, cursor);
        const node = $(
          `[data-ws-message="${job.message}"] [data-ws-message-text]`,
        );
        if (node) node.textContent = message.text;
        if (cursor < response.length) later(stream, 24);
        else {
          message.streaming = false;
          if (plan) finish();
          else work();
        }
      }
      later(stream, 60);
    }
    function requestMerge(trigger) {
      const t = current();
      if (!canMerge(t)) return;
      $("[data-ws-merge-summary]").innerHTML =
        `<p><strong>#${t.number} ${esc(t.title)}</strong></p><p><code>${esc(t.branch)}</code> ${ic("arrow-right", 12)} <code>main</code></p><p>3 checks passed · Approved by you</p>`;
      openDialog("[data-ws-confirm]", trigger, "[data-ws-cancel-merge]");
    }
    function merge() {
      const t = current();
      if (!canMerge(t)) return;
      t.merged = true;
      event(
        t,
        `Merged #${t.number} into main · ${("a" + t.number + "c72e").slice(0, 7)}`,
      );
      closeDialog("[data-ws-confirm]", false);
      t.messages.push({
        role: "assistant",
        text: `Merged #${t.number} into main.`,
        tools: [
          { kind: "edit", label: "Merge commit created", status: "passed" },
        ],
      });
      renderMessages();
      renderState();
      $("[data-ws-review-close]").focus({ preventScroll: true });
      S.playIcon($("[data-ws-review-toggle] .ic"));
    }
    page.addEventListener("click", (e) => {
      const b = e.target.closest("button");
      if (!b) return;
      if (b.hasAttribute("data-ws-file")) {
        const t = current();
        t.file = Number(b.dataset.wsFile);
        renderEditor();
        setView("editor");
        closeDialog("[data-ws-files-dialog]", false);
        $("[data-ws-file-tabs]")
          .querySelector(`[data-ws-file="${t.file}"]`)
          .focus({ preventScroll: true });
        animate($("[data-ws-editor-content]"));
      } else if (b.hasAttribute("data-ws-editor-mode"))
        editorMode(b.dataset.wsEditorMode);
      else if (b.hasAttribute("data-ws-edit")) {
        const t = current();
        if (t.phase === "running" || t.merged) return;
        t.editorMode = "source";
        t.editing = !t.editing;
        renderEditor();
        if (t.editing) {
          const input = $("[data-ws-code-input]");
          input.focus();
          const at = input.value.indexOf("    openSearch();");
          input.setSelectionRange(Math.max(0, at), Math.max(0, at));
          input.scrollTop = Math.max(
            0,
            (input.value.slice(0, Math.max(0, at)).split("\n").length - 8) * 22,
          );
        } else $("[data-ws-edit]").focus();
      } else if (b.hasAttribute("data-ws-check")) {
        state.check = b.dataset.wsCheck;
        state.logHTML = null;
        $("[data-ws-log]").scrollTop = 0;
        renderChecks();
      } else if (b.hasAttribute("data-ws-console-toggle")) {
        if (state.view === "checks" && matchMedia("(max-width:760px)").matches)
          setView("editor");
        else setConsole(!state.console);
      } else if (b.hasAttribute("data-ws-view")) {
        setView(b.dataset.wsView);
        if (b.dataset.wsView === "checks") setConsole(true);
      } else if (b.hasAttribute("data-ws-open-tab"))
        openPanel(b.dataset.wsOpenTab, b);
      else if (b.hasAttribute("data-ws-review-toggle")) openPanel("pr", b);
      else if (b.hasAttribute("data-ws-tab")) {
        state.tab = b.dataset.wsTab;
        $$("[data-ws-tab]").forEach((tab) => {
          tab.setAttribute("aria-selected", String(tab === b));
          tab.tabIndex = tab === b ? 0 : -1;
        });
        renderReview();
        animate($("[data-ws-review-content]"));
      } else if (b.hasAttribute("data-ws-review-close"))
        closeDialog("[data-ws-panel-dialog]");
      else if (b.hasAttribute("data-ws-sessions")) {
        closeDialog("[data-ws-files-dialog]", false);
        openDialog(
          "[data-ws-nav-dialog]",
          b,
          '[data-ws-thread="' + state.selected + '"]',
        );
      } else if (b.hasAttribute("data-ws-thread"))
        selectThread(b.dataset.wsThread);
      else if (b.hasAttribute("data-ws-new")) addThread();
      else if (b.hasAttribute("data-ws-nav-close"))
        closeDialog("[data-ws-nav-dialog]");
      else if (b.hasAttribute("data-ws-sidebar"))
        openDialog("[data-ws-files-dialog]", b, "[data-ws-file]");
      else if (b.hasAttribute("data-ws-files-close"))
        closeDialog("[data-ws-files-dialog]");
      else if (b.hasAttribute("data-ws-explorer-toggle")) {
        app.classList.toggle("ws-explorer-hidden");
        b.setAttribute(
          "aria-pressed",
          String(!app.classList.contains("ws-explorer-hidden")),
        );
      } else if (b.hasAttribute("data-ws-tree-toggle")) {
        const open = b.getAttribute("aria-expanded") !== "true";
        b.setAttribute("aria-expanded", String(open));
        $("[data-ws-files]").hidden = !open;
      } else if (b.hasAttribute("data-ws-chat-toggle")) {
        app.classList.toggle("ws-chat-hidden");
        b.setAttribute(
          "aria-pressed",
          String(!app.classList.contains("ws-chat-hidden")),
        );
      } else if (b.hasAttribute("data-ws-prompt")) send(b.dataset.wsPrompt);
      else if (b.hasAttribute("data-ws-run-checks")) send("Run checks");
      else if (b.hasAttribute("data-ws-mode")) {
        state.mode = b.dataset.wsMode;
        $$("[data-ws-mode]").forEach((el) =>
          el.setAttribute("aria-pressed", String(el === b)),
        );
      } else if (b.hasAttribute("data-ws-stop")) stopJob();
      else if (b.hasAttribute("data-ws-approve")) {
        const t = current();
        if (t.merged || t.phase === "running") return;
        t.approved = !t.approved;
        event(t, t.approved ? "Approved by you" : "Approval revoked");
        renderState();
      } else if (b.hasAttribute("data-ws-merge")) requestMerge(b);
      else if (b.hasAttribute("data-ws-confirm-merge")) merge();
      else if (b.hasAttribute("data-ws-cancel-merge"))
        closeDialog("[data-ws-confirm]");
      else if (b.hasAttribute("data-ws-rename")) {
        $("#ws-name").value = current().title;
        openDialog("[data-ws-name-dialog]", b, "#ws-name");
        $("#ws-name").select();
      } else if (b.hasAttribute("data-ws-name-cancel"))
        closeDialog("[data-ws-name-dialog]");
      else if (b.hasAttribute("data-ws-reset")) {
        stopJob(false);
        state.threads = fixtures.map(createThread);
        state.selected = "search";
        state.check = "tests";
        state.tab = "pr";
        $("[data-ws-prompt-input]").value = "";
        renderMessages();
        renderState();
        setView("editor");
        setConsole(true);
        $("[data-ws-source]").scrollTop = 12 + 10 * 22;
      }
    });
    $("[data-ws-composer]").addEventListener("submit", (e) => {
      e.preventDefault();
      send($("[data-ws-prompt-input]").value);
    });
    $("[data-ws-prompt-input]").addEventListener("input", (e) => {
      current().draft = e.target.value;
    });
    $("[data-ws-prompt-input]").addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
        e.preventDefault();
        send(e.target.value);
      }
    });
    $("[data-ws-code-input]").addEventListener("input", applyEdit);
    $("[data-ws-code-input]").addEventListener("scroll", syncEditorScroll);
    const updateCursor = () => {
      const input = $("[data-ws-code-input]"),
        text = input.value.slice(0, input.selectionStart),
        lines = text.split("\n");
      $("[data-ws-cursor]").textContent =
        `Ln ${lines.length}, Col ${lines.at(-1).length + 1}`;
    };
    ["keyup", "click", "input", "select"].forEach((name) =>
      $("[data-ws-code-input]").addEventListener(name, updateCursor),
    );
    $("[data-ws-code-input]").addEventListener("keydown", (e) => {
      const input = e.target;
      if (
        e.key === "Escape" ||
        ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s")
      ) {
        e.preventDefault();
        current().editing = false;
        renderEditor();
        $("[data-ws-edit]").focus();
      }
      if (e.key === "Tab" && !input.readOnly) {
        e.preventDefault();
        const a = input.selectionStart,
          b = input.selectionEnd;
        if (e.shiftKey) {
          const start = input.value.lastIndexOf("\n", a - 1) + 1;
          const indent =
            input.value.slice(start).match(/^ {1,2}/)?.[0].length || 0;
          input.setRangeText("", start, start + indent, "preserve");
        } else input.setRangeText("  ", a, b, "end");
        applyEdit();
      }
      e.stopPropagation();
    });
    $("[data-ws-name-form]").addEventListener("submit", (e) => {
      e.preventDefault();
      const title = $("#ws-name").value.trim();
      if (!title) return;
      current().title = title;
      closeDialog("[data-ws-name-dialog]");
      renderState();
    });
    $$("dialog").forEach((dialog) =>
      dialog.addEventListener("cancel", (e) => {
        e.preventDefault();
        closeDialog(dialog);
      }),
    );
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
      const tab = e.target.closest('[role="tab"]');
      if (tab && ["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
        e.preventDefault();
        const tabs = [
            ...tab.closest('[role="tablist"]').querySelectorAll('[role="tab"]'),
          ],
          i = tabs.indexOf(tab),
          next =
            e.key === "Home"
              ? 0
              : e.key === "End"
                ? tabs.length - 1
                : (i + (e.key === "ArrowRight" ? 1 : -1) + tabs.length) %
                  tabs.length;
        tabs[next].click();
        (tab.hasAttribute("data-ws-file")
          ? $("[data-ws-file-tabs]").querySelector(
              `[data-ws-file="${current().file}"]`,
            )
          : tabs[next]
        ).focus({ preventScroll: true });
      }
    });
    state.leave = () => {
      state.active = false;
      stopJob(false);
      clearTimers();
      settleMotion();
      $$("dialog[open]").forEach((d) => closeDialog(d, false));
    };
    new MutationObserver(() => {
      if (S.reduced()) settleMotion();
    }).observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-motion"],
    });
    matchMedia("(prefers-reduced-motion:reduce)").addEventListener(
      "change",
      () => {
        if (S.reduced()) settleMotion();
      },
    );
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) {
        stopJob(false);
        settleMotion();
      }
    });
    renderMessages();
    renderState();
    $("[data-ws-source]").scrollTop = 12 + 10 * 22;
  }
  S.page({
    id: "workspace",
    title: "Coding workspace",
    group: "Examples",
    icon: "code",
    order: 80,
    count: 1,
    description: "Editor, split diffs, CI logs and coding assistant.",
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
