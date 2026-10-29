//! Native coding workspace. Buffers, revisions, CI simulation and presentation are Rust.
//! Fixture TypeScript is inert editor text. No shell, network or code execution is used.
use crate::{
    motion::Curve,
    runtime::{Page, Runtime},
    tokens::Tokens,
};
use eframe::egui::{
    self, Color32, FontId, Id, Key, Layout, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    time::{SystemTime, UNIX_EPOCH},
};

const SEARCH_SOURCE: &str = r###"import { createSearchStore } from './search-store';
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
}"###;

const SEARCH_TESTS: &str = r###"import { afterEach, describe, expect, it, vi } from 'vitest';
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
});"###;

const SORT_BEFORE: &str = r###"import type { Task } from './types';

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
}"###;

const SORT_AFTER: &str = r###"import type { Task } from './types';

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
}"###;

const CLIENT_AFTER: &str = r###"const RETRY_LIMIT = 3;
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
}"###;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Passed,
    Failed,
    Queued,
    Running,
    Stale,
    Cancelled,
}
impl Status {
    fn key(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Stale => "stale",
            Self::Cancelled => "cancelled",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Stale => "Not run",
            Self::Cancelled => "Cancelled",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Passed => "check-circle",
            Self::Failed => "x",
            Self::Running => "loader",
            Self::Cancelled => "stop",
            _ => "clock",
        }
    }
    fn pending(self) -> bool {
        matches!(self, Self::Running | Self::Queued)
    }
}
#[derive(Clone)]
struct File {
    path: String,
    before: Vec<String>,
    after: Vec<String>,
}
fn lines(s: &str) -> Vec<String> {
    s.split('\n').map(str::to_owned).collect()
}
fn patch(path: &str, before: &str, after: &str) -> File {
    File {
        path: path.into(),
        before: lines(before),
        after: lines(after),
    }
}
#[derive(Clone)]
struct Tool {
    kind: &'static str,
    label: String,
    status: Status,
}
fn tool(kind: &'static str, label: impl Into<String>, status: Status) -> Tool {
    Tool {
        kind,
        label: label.into(),
        status,
    }
}
#[derive(Clone)]
struct Message {
    user: bool,
    text: String,
    tools: Vec<Tool>,
    streaming: bool,
    stopped: bool,
}
fn message(user: bool, text: impl Into<String>, tools: Vec<Tool>) -> Message {
    Message {
        user,
        text: text.into(),
        tools,
        streaming: false,
        stopped: false,
    }
}
#[derive(Clone)]
struct Activity {
    text: String,
    time: String,
}
#[derive(Clone)]
struct Thread {
    id: String,
    title: String,
    branch: String,
    number: u32,
    files: Vec<File>,
    draft: String,
    file: usize,
    editor_mode: String,
    editing: bool,
    approved_revision: Option<u64>,
    checks_revision: Option<u64>,
    merged: bool,
    running: bool,
    revision: u64,
    draft_pr: bool,
    checks: [Status; 3],
    events: Vec<Activity>,
    messages: Vec<Message>,
}
impl Thread {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: &str,
        title: &str,
        branch: &str,
        number: u32,
        failed: bool,
        request: &str,
        summary: &str,
        files: Vec<File>,
    ) -> Self {
        let tools = vec![
            tool("read", format!("Read {}", files[0].path), Status::Passed),
            tool(
                "edit",
                format!("Updated {} files", files.len()),
                Status::Passed,
            ),
            tool(
                "test",
                "Run checks",
                if failed {
                    Status::Failed
                } else {
                    Status::Passed
                },
            ),
        ];
        Self {
            id: id.into(),
            title: title.into(),
            branch: branch.into(),
            number,
            files,
            draft: String::new(),
            file: 0,
            editor_mode: "source".into(),
            editing: false,
            approved_revision: None,
            checks_revision: Some(0),
            merged: false,
            running: false,
            revision: 0,
            draft_pr: false,
            checks: [
                Status::Passed,
                if failed {
                    Status::Failed
                } else {
                    Status::Passed
                },
                Status::Passed,
            ],
            events: vec![
                Activity {
                    text: format!("Opened pull request #{number}"),
                    time: "09:41".into(),
                },
                Activity {
                    text: if failed {
                        "Unit tests failed"
                    } else {
                        "All checks passed"
                    }
                    .into(),
                    time: "09:42".into(),
                },
            ],
            messages: vec![
                message(true, request, vec![]),
                message(false, summary, tools),
            ],
        }
    }
    fn passed(&self) -> bool {
        self.checks.iter().all(|s| *s == Status::Passed)
    }
    fn approved(&self) -> bool {
        self.approved_revision == Some(self.revision)
    }
    fn can_merge(&self) -> bool {
        self.passed()
            && self.approved()
            && self.checks_revision == Some(self.revision)
            && !self.merged
            && !self.running
    }
    fn repaired(&self) -> bool {
        if self.id != "search" && !self.id.starts_with("new-") {
            return true;
        }
        let Some(file) = self.files.iter().find(|f| f.path == "src/search.ts") else {
            return false;
        };
        let body = file.after.join("\n");
        // Same source-level assertion as the reference. Every check reads this buffer anew.
        body.match_indices("if (!isShortcut || event.repeat) return;")
            .any(|(at, guard)| {
                body[at + guard.len()..]
                    .trim_start()
                    .strip_prefix("event.preventDefault();")
                    .is_some_and(|rest| rest.trim_start().starts_with("openSearch();"))
            })
    }
    fn invalidate(&mut self) {
        self.approved_revision = None;
        self.checks_revision = None;
        self.revision += 1;
        self.checks = [Status::Stale; 3];
    }
    fn record(&mut self, text: impl Into<String>) {
        self.events.push(Activity {
            text: text.into(),
            time: clock(),
        });
    }
    fn summary(&self) -> &'static str {
        if self.merged {
            "Merged into main"
        } else if self.running {
            "Checks running"
        } else if self.checks.contains(&Status::Failed) {
            "1 check failed"
        } else if self.passed() {
            if self.approved() {
                "Ready to merge"
            } else {
                "3 checks passed"
            }
        } else if self.checks.contains(&Status::Cancelled) {
            "Checks cancelled"
        } else {
            "Checks not run"
        }
    }
}
fn fixtures() -> Vec<Thread> {
    let search_before=SEARCH_SOURCE.replace("    const modifier = event.metaKey || event.ctrlKey;\n    const isShortcut = modifier && event.key.toLowerCase() === 'k';\n    if (!isShortcut || event.repeat) return;","    if (event.key !== 'k') return;");
    vec![
        Thread::new(
            "search",
            "Search shortcut",
            "fix/search-shortcut",
            128,
            true,
            "Fix Cmd+K so search opens without triggering the browser shortcut.",
            "The modifier guard and regression tests are in place. One test is failing: the shortcut still reaches the browser. The failing assertion is in the CI console.",
            vec![
                patch("src/search.ts", &search_before, SEARCH_SOURCE),
                patch(
                    "src/search.test.ts",
                    &SEARCH_TESTS.replace("    expect(event.defaultPrevented).toBe(true);\n", ""),
                    SEARCH_TESTS,
                ),
                patch(
                    "CHANGELOG.md",
                    "## Unreleased\n\n### Fixed\n",
                    "## Unreleased\n\n### Fixed\n- Search accepts Cmd+K and Ctrl+K.\n- Unmodified keys no longer open search.",
                ),
            ],
        ),
        Thread::new(
            "sorting",
            "List drag handles",
            "feat/drag-handles",
            129,
            false,
            "Add pointer and keyboard sorting with an Escape rollback.",
            "Drag handles preserve the original order until drop. Escape restores the snapshot and focus. Pointer, keyboard and cancellation checks passed.",
            vec![
                patch("src/task-list.ts", SORT_BEFORE, SORT_AFTER),
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
        ),
        Thread::new(
            "retry",
            "API retry backoff",
            "fix/retry-backoff",
            130,
            false,
            "Limit API retries and support cancellation during backoff.",
            "Requests retry at most three times. The AbortSignal reaches both the request and the backoff timer. All checks passed.",
            vec![
                patch(
                    "src/client.ts",
                    "export async function load(url: string) {\n  return fetch(url);\n}",
                    CLIENT_AFTER,
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
        ),
    ]
}
fn clock() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let offset = std::fs::read("/etc/localtime")
        .ok()
        .and_then(|bytes| timezone_offset(&bytes, seconds));
    let local = seconds + offset.unwrap_or(0) as i64;
    format!(
        "{:02}:{:02}{}",
        local.div_euclid(3600).rem_euclid(24),
        local.div_euclid(60).rem_euclid(60),
        if offset.is_some() { "" } else { " UTC" }
    )
}
/// Read the host's TZif transition table without a platform binding or subprocess.
fn timezone_offset(bytes: &[u8], timestamp: i64) -> Option<i32> {
    fn number(bytes: &[u8], at: usize) -> Option<usize> {
        Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize)
    }
    fn counts(bytes: &[u8], header: usize) -> Option<[usize; 6]> {
        if bytes.get(header..header + 4)? != b"TZif" {
            return None;
        }
        let mut result = [0; 6];
        for (i, n) in result.iter_mut().enumerate() {
            *n = number(bytes, header + 20 + i * 4)?;
        }
        Some(result)
    }
    let mut header = 0;
    let mut width = 4usize;
    let mut c = counts(bytes, header)?;
    if matches!(bytes.get(4), Some(b'2' | b'3' | b'4')) {
        let length = c[3]
            .checked_mul(5)?
            .checked_add(c[4].checked_mul(6)?)?
            .checked_add(c[5])?
            .checked_add(c[2].checked_mul(8)?)?
            .checked_add(c[1])?
            .checked_add(c[0])?;
        header = 44usize.checked_add(length)?;
        c = counts(bytes, header)?;
        width = 8;
    }
    let start = header.checked_add(44)?;
    let types = start.checked_add(c[3].checked_mul(width)?)?;
    let info = types.checked_add(c[3])?;
    let mut kind = 0usize;
    for i in 0..c[3] {
        let at = start.checked_add(i.checked_mul(width)?)?;
        let transition = if width == 8 {
            i64::from_be_bytes(bytes.get(at..at + 8)?.try_into().ok()?)
        } else {
            i32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as i64
        };
        if transition > timestamp {
            break;
        }
        kind = *bytes.get(types + i)? as usize;
    }
    if kind >= c[4] {
        return None;
    }
    let at = info.checked_add(kind.checked_mul(6)?)?;
    Some(i32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

struct Check {
    id: &'static str,
    title: &'static str,
    command: &'static str,
    duration: &'static str,
    success: &'static [&'static str],
}
const CHECKS: [Check; 3] = [
    Check {
        id: "types",
        title: "TypeScript",
        command: "pnpm typecheck",
        duration: "0.8s",
        success: &[
            "> tsc --noEmit",
            "No type errors.",
            "Process exited with code 0.",
        ],
    },
    Check {
        id: "tests",
        title: "Unit tests",
        command: "pnpm test",
        duration: "1.4s",
        success: &[
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
    Check {
        id: "build",
        title: "Build",
        command: "pnpm build",
        duration: "0.6s",
        success: &[
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

#[derive(Clone, Debug)]
struct DiffLine {
    kind: &'static str,
    old: Option<usize>,
    next: Option<usize>,
    text: String,
}
fn diff_lines(file: &File) -> Vec<DiffLine> {
    let (a, b) = (&file.before, &file.after);
    let mut table = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i][j] = if a[i] == b[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut result = vec![];
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            i += 1;
            j += 1;
            result.push(DiffLine {
                kind: "context",
                old: Some(i),
                next: Some(j),
                text: a[i - 1].clone(),
            });
        } else if i < a.len() && (j == b.len() || table[i + 1][j] >= table[i][j + 1]) {
            i += 1;
            result.push(DiffLine {
                kind: "removed",
                old: Some(i),
                next: None,
                text: a[i - 1].clone(),
            });
        } else {
            j += 1;
            result.push(DiffLine {
                kind: "added",
                old: None,
                next: Some(j),
                text: b[j - 1].clone(),
            });
        }
    }
    result
}
fn split_rows(file: &File) -> Vec<[Option<DiffLine>; 2]> {
    let lines = diff_lines(file);
    let mut rows = vec![];
    let mut i = 0;
    while i < lines.len() {
        if lines[i].kind == "context" {
            rows.push([Some(lines[i].clone()), Some(lines[i].clone())]);
            i += 1;
            continue;
        }
        let (mut removed, mut added) = (vec![], vec![]);
        while i < lines.len() && lines[i].kind != "context" {
            if lines[i].kind == "removed" {
                removed.push(lines[i].clone())
            } else {
                added.push(lines[i].clone())
            }
            i += 1;
        }
        for k in 0..removed.len().max(added.len()) {
            rows.push([removed.get(k).cloned(), added.get(k).cloned()]);
        }
    }
    rows
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum JobKind {
    Plan,
    Checks,
    Fix,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Streaming,
    Reading,
    Check(usize),
}
#[derive(Clone, Debug)]
struct Job {
    thread: String,
    message: usize,
    kind: JobKind,
    stage: Stage,
    response: Vec<char>,
    cursor: usize,
    due: f64,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum View {
    Editor,
    Chat,
    Checks,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dialog {
    Review,
    Sessions,
    Files,
    Rename,
    ConfirmMerge,
}
#[derive(Clone)]
struct ModalState {
    kind: Dialog,
    return_to: Option<Id>,
    initial: bool,
}
#[derive(Clone, Copy)]
struct KeyboardCapture {
    editor: Option<Id>,
    dialog: bool,
}
/// Let the application shell defer library shortcuts to the editor and dialogs.
/// The reference stops those key events at the workspace boundary.
pub fn captures_keyboard(ctx: &egui::Context) -> bool {
    let capture = ctx.data(|d| d.get_temp::<KeyboardCapture>(Id::new("ws-keyboard-capture")));
    capture.is_some_and(|c| c.dialog || c.editor.is_some_and(|id| ctx.memory(|m| m.has_focus(id))))
}
#[derive(Default)]
struct EditorCache {
    key: (String, usize, u64),
    rows: Vec<[Option<DiffLine>; 2]>,
}
/// Kept alive by the shell. Domain transitions never execute the displayed source.
pub struct State {
    threads: Vec<Thread>,
    selected: String,
    check: usize,
    mode_plan: bool,
    activity_tab: bool,
    job: Option<Job>,
    serial: u32,
    active: bool,
    view: View,
    console_open: bool,
    explorer_open: bool,
    chat_open: bool,
    composer_ime: bool,
    tree_open: bool,
    dialogs: Vec<ModalState>,
    modal_controls: Vec<Id>,
    rename: String,
    pending_focus: Option<Id>,
    restore_focus: Option<Id>,
    focus_retry: u8,
    focus_targets: HashMap<Id, (Option<Dialog>, bool)>,
    owned_focus: HashSet<Id>,
    drawn_dialog: Option<Dialog>,
    editor_focus: bool,
    editor_scroll: Option<f32>,
    cursor: (usize, usize),
    stopped_status: bool,
    editor_transition: Option<f64>,
    diff_cache: EditorCache,
    diff_scroll: f32,
    diff_horizontal: [f32; 2],
    log_scroll_reset: bool,
    clear_editor_history: bool,
    controls: HashMap<String, Rect>,
    source_scroll: HashMap<(String, usize), Vec2>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            threads: fixtures(),
            selected: "search".into(),
            check: 1,
            mode_plan: false,
            activity_tab: false,
            job: None,
            serial: 0,
            active: false,
            view: View::Editor,
            console_open: true,
            explorer_open: true,
            chat_open: true,
            composer_ime: false,
            tree_open: true,
            dialogs: vec![],
            modal_controls: vec![],
            rename: String::new(),
            pending_focus: None,
            restore_focus: None,
            focus_retry: 0,
            focus_targets: HashMap::new(),
            owned_focus: HashSet::new(),
            drawn_dialog: None,
            editor_focus: false,
            editor_scroll: Some(232.),
            cursor: (27, 5),
            stopped_status: false,
            editor_transition: None,
            diff_cache: EditorCache::default(),
            diff_scroll: 0.,
            diff_horizontal: [0.; 2],
            log_scroll_reset: false,
            clear_editor_history: false,
            controls: HashMap::new(),
            source_scroll: HashMap::new(),
        }
    }
}
impl State {
    fn track_focus(&mut self, response: &Response) {
        self.owned_focus.insert(response.id);
        self.focus_targets
            .insert(response.id, (self.drawn_dialog, response.enabled()));
    }
    fn focus_available(&self, id: Id) -> bool {
        self.focus_targets
            .get(&id)
            .is_some_and(|(scope, enabled)| *enabled && *scope == self.drawn_dialog)
    }
    fn clear_owned_focus(&self, ctx: &egui::Context) {
        if ctx
            .memory(|m| m.focused())
            .is_some_and(|id| self.owned_focus.contains(&id))
        {
            crate::runtime::restore_focus(ctx, None);
        }
    }
    fn settle_focus(&mut self, ctx: &egui::Context, rt: &Runtime, restore: Option<Id>) {
        let requested = self.pending_focus.take().or(restore);
        if let Some(id) = requested {
            if self.focus_available(id) {
                ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                crate::runtime::restore_focus(ctx, Some(id));
                self.focus_retry = 0;
                rt.repaint();
            } else if self.focus_retry < 2 {
                // The target may belong to the parent modal or next mobile view.
                // Never publish its focus before that frame actually draws it.
                self.restore_focus = Some(id);
                self.focus_retry += 1;
                rt.repaint();
            } else {
                self.focus_retry = 0;
            }
        }
        if let Some(id) = ctx
            .memory(|m| m.focused())
            .filter(|id| self.owned_focus.contains(id))
            && !self.focus_available(id)
        {
            crate::runtime::restore_focus(ctx, None);
        }
        if ctx.memory(|m| m.focused()).is_none()
            && (requested.is_some() || self.drawn_dialog.is_some())
        {
            let fallback = if self.drawn_dialog.is_some() {
                self.modal_controls
                    .iter()
                    .copied()
                    .find(|id| self.focus_available(*id))
            } else {
                [
                    "ws-sidebar",
                    "ws-review-toggle",
                    "ws-editor-mode-source",
                    "ws-prompt-input",
                ]
                .into_iter()
                .map(Id::new)
                .find(|id| self.focus_available(*id))
            };
            if let Some(id) = fallback {
                crate::runtime::restore_focus(ctx, Some(id));
                rt.repaint();
            }
        }
    }
    fn current(&self) -> &Thread {
        self.threads
            .iter()
            .find(|t| t.id == self.selected)
            .expect("selected session")
    }
    fn current_mut(&mut self) -> &mut Thread {
        self.threads
            .iter_mut()
            .find(|t| t.id == self.selected)
            .expect("selected session")
    }
    fn edit_buffer(&mut self, file: usize, source: &str) -> bool {
        let t = self.current_mut();
        if t.merged || t.running || file >= t.files.len() {
            return false;
        }
        let after = lines(source);
        if t.files[file].after == after {
            return false;
        }
        t.files[file].after = after;
        t.invalidate();
        self.stopped_status = false;
        true
    }
    fn approve(&mut self) {
        let t = self.current_mut();
        if t.running || t.merged {
            return;
        }
        let approved = !t.approved();
        t.approved_revision = approved.then_some(t.revision);
        t.record(if approved {
            "Approved by you"
        } else {
            "Approval revoked"
        });
        self.stopped_status = false;
    }
    fn merge(&mut self) -> bool {
        let t = self.current_mut();
        if !t.can_merge() {
            return false;
        }
        t.merged = true;
        t.record(format!(
            "Merged #{} into main · {}",
            t.number,
            format!("a{}c72e", t.number)
                .chars()
                .take(7)
                .collect::<String>()
        ));
        t.messages.push(message(
            false,
            format!("Merged #{} into main.", t.number),
            vec![tool("edit", "Merge commit created", Status::Passed)],
        ));
        self.stopped_status = false;
        true
    }
    fn select_thread(&mut self, id: &str) {
        if id == self.selected || !self.threads.iter().any(|t| t.id == id) {
            return;
        }
        self.stop(false);
        self.selected = id.into();
        self.check = 1;
        self.stopped_status = false;
        self.editor_transition = None;
        self.editor_scroll = None;
        self.log_scroll_reset = true;
        self.editor_focus = false;
    }
    fn add_thread(&mut self) {
        self.stop(false);
        self.serial += 1;
        let mut t = fixtures().remove(0);
        t.id = format!("new-{}", self.serial);
        t.title = "New session".into();
        t.branch = format!("work/session-{}", self.serial);
        t.number = 130 + self.serial;
        t.messages.clear();
        t.events = vec![Activity {
            text: "Draft session created".into(),
            time: clock(),
        }];
        t.draft_pr = true;
        t.checks = [Status::Stale; 3];
        t.checks_revision = None;
        let id = t.id.clone();
        self.threads.insert(0, t);
        self.select_thread(&id);
        self.view = View::Chat;
        self.dialogs.clear();
        self.pending_focus = Some(Id::new("ws-prompt-input"));
    }
    fn send(&mut self, text: &str, now: f64) -> bool {
        let text = text.trim();
        if text.is_empty() || self.current().running || self.current().merged {
            return false;
        }
        let first = text
            .split(|c: char| !c.is_alphanumeric())
            .next()
            .unwrap_or("")
            .to_lowercase();
        let lower = text.to_lowercase();
        let kind = if self.mode_plan
            || ["explain", "summarize", "summarise", "why", "what"].contains(&first.as_str())
        {
            JobKind::Plan
        } else if ["run", "rerun", "retry"].contains(&first.as_str())
            && !lower.contains("fix")
            && !lower.contains("repair")
        {
            JobKind::Checks
        } else {
            JobKind::Fix
        };
        let t = self.current_mut();
        if t.title == "New session" {
            t.title = if text.chars().count() > 38 {
                text.chars().take(35).collect::<String>() + "…"
            } else {
                text.into()
            };
        }
        t.messages.push(message(true, text, vec![]));
        t.draft.clear();
        t.editing = false;
        let response=match kind {
            JobKind::Plan=>format!("The shortcut guard is in {}. {}",t.files[0].path,if t.repaired(){"Checks and approval are required before merging."}else{"The handler opens search without cancelling the keyboard event. Add event.preventDefault() immediately before openSearch(), then rerun CI."}),
            JobKind::Checks=>"Running TypeScript, unit tests and the production build against the current working tree.".into(),
            JobKind::Fix=>format!("I’ll update {}, then rerun all three CI checks.",t.files[0].path),
        };
        let mut m = message(false, "", vec![]);
        m.streaming = true;
        t.messages.push(m);
        t.running = true;
        if kind != JobKind::Plan {
            t.checks = [Status::Queued; 3];
            t.checks_revision = None;
            if kind == JobKind::Fix {
                t.approved_revision = None
            }
        }
        self.job = Some(Job {
            thread: t.id.clone(),
            message: t.messages.len() - 1,
            kind,
            stage: Stage::Streaming,
            response: response.chars().collect(),
            cursor: 0,
            due: now + 0.060,
        });
        self.stopped_status = false;
        true
    }
    /// Advance real-time local work. Motion preferences are deliberately absent.
    fn tick(&mut self, now: f64) {
        let Some(mut job) = self.job.take() else {
            return;
        };
        let Some(index) = self.threads.iter().position(|t| t.id == job.thread) else {
            return;
        };
        let t = &mut self.threads[index];
        let mut finished = false;
        while !finished && now + 1e-9 >= job.due {
            match job.stage {
                Stage::Streaming => {
                    job.cursor = (job.cursor + 12).min(job.response.len());
                    t.messages[job.message].text = job.response[..job.cursor].iter().collect();
                    if job.cursor < job.response.len() {
                        job.due += 0.024;
                    } else {
                        t.messages[job.message].streaming = false;
                        if job.kind == JobKind::Plan {
                            finished = true;
                        } else {
                            t.messages[job.message].tools = vec![tool(
                                "read",
                                format!("Read {}", t.files[0].path),
                                if job.kind == JobKind::Checks {
                                    Status::Passed
                                } else {
                                    Status::Running
                                },
                            )];
                            job.stage = Stage::Reading;
                            job.due += if job.kind == JobKind::Checks {
                                0.140
                            } else {
                                0.420
                            };
                        }
                    }
                }
                Stage::Reading => {
                    t.messages[job.message].tools[0].status = Status::Passed;
                    if job.kind == JobKind::Fix {
                        if !t.repaired()
                            && let Some(file) =
                                t.files.iter_mut().find(|f| f.path == "src/search.ts")
                            && let Some(at) = file
                                .after
                                .iter()
                                .position(|line| line.trim() == "openSearch();")
                        {
                            file.after.insert(at, "    event.preventDefault();".into());
                        }
                        t.invalidate();
                        t.draft_pr = false;
                        t.checks = [Status::Queued; 3];
                        t.messages[job.message].tools.push(tool(
                            "edit",
                            format!("Applied patch to {}", t.files[0].path),
                            Status::Passed,
                        ));
                        t.record("Applied file changes");
                    }
                    t.messages[job.message].tools.push(tool(
                        "test",
                        "Run CI checks",
                        Status::Running,
                    ));
                    t.checks[0] = Status::Running;
                    job.stage = Stage::Check(0);
                    job.due += 0.440;
                }
                Stage::Check(i) => {
                    t.checks[i] = if i == 1 && !t.repaired() {
                        Status::Failed
                    } else {
                        Status::Passed
                    };
                    if i == 2 {
                        t.checks_revision = Some(t.revision);
                        let passed = t.passed();
                        if let Some(tool) = t.messages[job.message].tools.last_mut() {
                            tool.status = if passed {
                                Status::Passed
                            } else {
                                Status::Failed
                            };
                        }
                        finished = true;
                    } else {
                        t.checks[i + 1] = Status::Running;
                        job.stage = Stage::Check(i + 1);
                        job.due += if i + 1 == 1 { 0.720 } else { 0.440 };
                    }
                }
            }
        }
        if finished {
            t.running = false;
            t.messages[job.message].streaming = false;
            if job.kind != JobKind::Plan {
                t.messages[job.message].text = if t.passed() {
                    "All three checks passed. The patch is ready for review."
                } else {
                    "Unit tests failed. The assertion and source location are in the CI console."
                }
                .into();
            }
            t.record(if job.kind == JobKind::Plan {
                "Reviewed changes"
            } else if t.passed() {
                "All checks passed"
            } else {
                "Unit tests failed"
            });
        } else {
            self.job = Some(job);
        }
    }
    fn stop(&mut self, announce: bool) {
        let Some(job) = self.job.take() else { return };
        if let Some(t) = self.threads.iter_mut().find(|t| t.id == job.thread) {
            t.running = false;
            let m = &mut t.messages[job.message];
            m.streaming = false;
            m.stopped = true;
            for tool in &mut m.tools {
                if tool.status.pending() {
                    tool.status = Status::Cancelled;
                }
            }
            if job.kind != JobKind::Plan {
                for status in &mut t.checks {
                    if status.pending() {
                        *status = Status::Cancelled;
                    }
                }
                t.checks_revision = None;
            }
            t.record("Stopped task");
        }
        self.stopped_status = announce;
    }
    fn reset(&mut self) {
        self.stop(false);
        self.threads = fixtures();
        self.selected = "search".into();
        self.check = 1;
        self.activity_tab = false;
        self.dialogs.clear();
        self.view = View::Editor;
        self.console_open = true;
        self.stopped_status = false;
        self.editor_scroll = Some(232.);
        self.editor_focus = false;
        self.editor_transition = None;
        self.diff_cache = EditorCache::default();
        self.source_scroll.clear();
        self.cursor = (27, 5);
        self.clear_editor_history = true;
        self.pending_focus = None;
        self.restore_focus = None;
        self.focus_retry = 0;
    }
    fn choose_file(&mut self, index: usize, now: f64) {
        if index >= self.current().files.len() {
            return;
        }
        self.current_mut().file = index;
        self.view = View::Editor;
        self.editor_transition = Some(now);
        self.editor_scroll = None;
        self.editor_focus = self.current().editing;
    }
    fn set_editor_mode(&mut self, diff: bool, now: f64) {
        let t = self.current_mut();
        t.editing = false;
        t.editor_mode = if diff { "diff" } else { "source" }.into();
        self.editor_transition = Some(now);
    }
    fn toggle_edit(&mut self) {
        let t = self.current_mut();
        if t.merged || t.running {
            return;
        }
        t.editor_mode = "source".into();
        t.editing = !t.editing;
        self.editor_focus = t.editing;
        if self.editor_focus {
            let source = self.current().files[self.current().file].after.join("\n");
            let at = source.find("    openSearch();").unwrap_or(0);
            self.editor_scroll =
                Some((source[..at].split('\n').count().saturating_sub(8) as f32) * 22.);
        } else {
            self.pending_focus = Some(Id::new("ws-edit"));
        }
    }
    fn finish_edit(&mut self) {
        self.current_mut().editing = false;
        self.editor_focus = false;
        self.pending_focus = Some(Id::new("ws-edit"));
    }
    fn open_dialog(&mut self, kind: Dialog, return_to: Option<Id>) {
        if self.dialogs.last().is_some_and(|d| d.kind == kind) {
            return;
        }
        if kind == Dialog::Rename {
            self.rename = self.current().title.clone();
        }
        self.pending_focus = None;
        self.restore_focus = None;
        self.focus_retry = 0;
        self.dialogs.push(ModalState {
            kind,
            return_to,
            initial: true,
        });
        self.modal_controls.clear();
    }
    fn close_dialog(&mut self, restore: bool) {
        self.pending_focus = None;
        self.restore_focus = None;
        self.focus_retry = 0;
        if let Some(dialog) = self.dialogs.pop()
            && restore
        {
            // The previous modal layer blocks its parent for this frame.
            // Restore on the following frame after that layer is released.
            self.restore_focus = dialog.return_to;
        }
        self.modal_controls.clear();
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokenKind {
    Plain,
    Comment,
    String,
    Keyword,
    Number,
    Type,
    Function,
}
fn tokenize(source: &str) -> Vec<(Range<usize>, TokenKind)> {
    let b = source.as_bytes();
    let mut tokens = vec![];
    let mut at = 0;
    while at < b.len() {
        let start = at;
        let mut kind = TokenKind::Plain;
        if source[at..].starts_with("//") {
            at = source[at..].find('\n').map_or(b.len(), |n| at + n);
            kind = TokenKind::Comment;
        } else if source[at..].starts_with("/*") {
            at = source[at + 2..].find("*/").map_or(b.len(), |n| at + n + 4);
            kind = TokenKind::Comment;
        } else if matches!(b[at], b'\'' | b'"') {
            let quote = b[at];
            at += 1;
            kind = TokenKind::String;
            while at < b.len() {
                let c = source[at..].chars().next().unwrap();
                at += c.len_utf8();
                if c == '\\' {
                    if at < b.len() {
                        at += source[at..].chars().next().unwrap().len_utf8();
                    }
                } else if c as u32 == quote as u32 {
                    break;
                }
            }
        } else if b[at].is_ascii_alphabetic() || matches!(b[at], b'_' | b'$') {
            at += 1;
            while at < b.len() && (b[at].is_ascii_alphanumeric() || matches!(b[at], b'_' | b'$')) {
                at += 1;
            }
            let word = &source[start..at];
            kind = if [
                "import",
                "from",
                "export",
                "interface",
                "type",
                "const",
                "let",
                "return",
                "function",
                "if",
                "else",
                "for",
                "new",
                "throw",
                "async",
                "await",
                "void",
                "true",
                "false",
                "null",
                "undefined",
                "as",
                "get",
            ]
            .contains(&word)
            {
                TokenKind::Keyword
            } else if b[start].is_ascii_uppercase() {
                TokenKind::Type
            } else if source[at..].trim_start().starts_with('(') {
                TokenKind::Function
            } else {
                TokenKind::Plain
            };
        } else if b[at].is_ascii_digit() {
            at += 1;
            while at < b.len() && b[at].is_ascii_digit() {
                at += 1
            }
            if at + 1 < b.len() && b[at] == b'.' && b[at + 1].is_ascii_digit() {
                at += 1;
                while at < b.len() && b[at].is_ascii_digit() {
                    at += 1
                }
            }
            kind = TokenKind::Number;
        } else {
            at += source[at..].chars().next().unwrap().len_utf8();
        }
        tokens.push((start..at, kind));
    }
    tokens
}
fn syntax_job(source: &str, t: &Tokens, size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    for (range, kind) in tokenize(source) {
        let color = match kind {
            TokenKind::Plain => t.text,
            TokenKind::Comment => t.text3,
            TokenKind::String => t.accent_text,
            TokenKind::Keyword => t.info_text,
            TokenKind::Number => t.warning_text,
            TokenKind::Type => t.text2,
            TokenKind::Function => t.text,
        };
        let format = egui::TextFormat {
            font_id: FontId::monospace(size),
            color,
            line_height: Some(22.),
            italics: kind == TokenKind::Comment,
            ..Default::default()
        };
        job.append(&source[range], 0., format);
    }
    if source.is_empty() {
        job.append(
            "",
            0.,
            egui::TextFormat {
                font_id: FontId::monospace(size),
                line_height: Some(22.),
                color: t.text,
                ..Default::default()
            },
        );
    }
    job
}
fn status_color(status: Status, t: &Tokens) -> Color32 {
    match status {
        Status::Passed => t.success_text,
        Status::Failed => t.danger_text,
        Status::Running => t.accent_text,
        _ => t.text3,
    }
}
fn icon_progress(rt: &mut Runtime, id: impl std::hash::Hash, name: &str, engaged: bool) -> f32 {
    if name == "loader" {
        if rt.reduced() {
            0.
        } else {
            rt.repaint();
            ((rt.now / (0.8 * rt.motion_scale() as f64)).fract() as f32 * (800. / 1200.))
                .max(0.00001)
        }
    } else {
        rt.icon_progress(id, engaged)
    }
}
fn child(ui: &mut Ui, rect: Rect, id: impl std::hash::Hash) -> Ui {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(ui.clip_rect().intersect(rect));
    child.spacing_mut().item_spacing = Vec2::new(6., 4.);
    child.spacing_mut().interact_size.y = 24.;
    child
}
fn separator(ui: &Ui, a: Pos2, b: Pos2, color: Color32) {
    ui.painter().line_segment([a, b], Stroke::new(1., color));
}
#[derive(Clone, Copy)]
struct Control<'a> {
    id: &'a str,
    label: &'a str,
    accessible_name: &'a str,
    icon: Option<&'a str>,
    selected: bool,
    selection: bool,
    enabled: bool,
    width: Option<f32>,
    height: f32,
    tab: bool,
}
impl<'a> Control<'a> {
    fn new(id: &'a str, label: &'a str, icon: Option<&'a str>) -> Self {
        Self {
            id,
            label,
            accessible_name: label,
            icon,
            selected: false,
            selection: false,
            enabled: true,
            width: None,
            height: 28.,
            tab: false,
        }
    }
    fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self.selection = true;
        self
    }
    fn accessible_name(mut self, name: &'a str) -> Self {
        self.accessible_name = name;
        self
    }
    fn enabled(mut self, v: bool) -> Self {
        self.enabled = v;
        self
    }
    fn width(mut self, v: f32) -> Self {
        self.width = Some(v);
        self
    }
    fn height(mut self, v: f32) -> Self {
        self.height = v;
        self
    }
    fn tab(mut self) -> Self {
        self.tab = true;
        self
    }
}
impl State {
    fn control(&mut self, ui: &mut Ui, rt: &mut Runtime, c: Control<'_>) -> Response {
        let tokens = rt.tokens.clone();
        let font = FontId::proportional(12.);
        let galley = ui
            .painter()
            .layout_no_wrap(c.label.into(), font.clone(), tokens.text);
        let width = c
            .width
            .unwrap_or(galley.size().x + 16. + if c.icon.is_some() { 22. } else { 0. })
            .min(ui.available_width())
            .max(1.);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, c.height), Sense::hover());
        let sense = if c.tab && !c.selected {
            Sense::CLICK
        } else {
            Sense::click()
        };
        let response = if c.enabled {
            ui.interact(rect, Id::new(c.id), sense)
        } else {
            let disabled = ui.new_child(egui::UiBuilder::new().max_rect(rect).disabled());
            disabled.interact(rect, Id::new(c.id), Sense::hover())
        };
        response.widget_info(|| {
            if c.selection || c.tab {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    response.enabled(),
                    c.selected,
                    c.accessible_name,
                )
            } else {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    response.enabled(),
                    c.accessible_name,
                )
            }
        });
        if response.has_focus() && !response.interact_rect.contains_rect(response.rect) {
            response.scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
        }
        if c.tab {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(egui::accesskit::Role::Tab);
                node.clear_toggled();
                node.set_selected(c.selected);
                node.add_action(egui::accesskit::Action::Focus);
            });
        }
        let hover = rt.animate(
            ("ws-hover", c.id),
            if response.hovered() || response.has_focus() {
                1.
            } else {
                0.
            },
            0.14,
            Curve::Out,
        );
        let bg = if c.selected {
            tokens.accent_soft
        } else {
            tokens.surface3.gamma_multiply(hover)
        };
        if c.selected || hover > 0.001 {
            ui.painter().rect_filled(rect, 3. * tokens.radius, bg);
        }
        if response.has_focus() {
            ui.painter().rect_stroke(
                rect.shrink(1.),
                3. * tokens.radius,
                Stroke::new(1.5, tokens.accent),
                egui::StrokeKind::Inside,
            );
        }
        let color = if !c.enabled {
            tokens.text4
        } else if c.selected {
            tokens.accent_text
        } else {
            tokens.text2
        };
        let clip = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let mut x = rect.left() + 8.;
        if let Some(icon) = c.icon {
            crate::icons::paint(
                &clip,
                icon,
                Rect::from_center_size(Pos2::new(x + 7., rect.center().y), Vec2::splat(14.)),
                color,
                1.6,
                icon_progress(
                    rt,
                    ("ws-icon", c.id),
                    icon,
                    response.hovered() || response.has_focus(),
                ),
            );
            x += 22.;
        }
        if !c.label.is_empty() {
            clip.text(
                Pos2::new(x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                c.label,
                font,
                color,
            );
        }
        self.controls.insert(c.id.into(), rect);
        self.track_focus(&response);
        rt.record(c.id, &response);
        response
    }
    fn icon_control(
        &mut self,
        ui: &mut Ui,
        rt: &mut Runtime,
        id: &str,
        icon: &str,
        label: &str,
    ) -> Response {
        let response = self.control(
            ui,
            rt,
            Control::new(id, "", Some(icon))
                .accessible_name(label)
                .width(32.)
                .height(32.),
        );
        response.on_hover_text(label)
    }
    fn header(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect, mobile: bool) {
        let t = self.current().clone();
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface2);
        separator(
            ui,
            rect.left_bottom(),
            rect.right_bottom(),
            tokens.border_strong,
        );
        let mut bar = child(ui, rect.shrink2(Vec2::new(8., 4.)), "ws-topbar");
        bar.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
            if mobile {
                let r = self.icon_control(ui, rt, "ws-sidebar", "folder", "Open project files");
                if r.clicked() {
                    self.open_dialog(Dialog::Files, Some(r.id));
                }
            }
            ui.label(
                RichText::new("atlas / storefront")
                    .size(if mobile { 11. } else { 12. })
                    .color(tokens.text),
            );
            if !mobile {
                let branch = if t.merged { "main" } else { &t.branch };
                let r = self.control(
                    ui,
                    rt,
                    Control::new("ws-sessions", branch, Some("git-branch"))
                        .accessible_name("Switch coding session")
                        .width(if rect.width() < 1100. { 190. } else { 220. }),
                );
                if r.clicked() {
                    self.open_dialog(Dialog::Sessions, Some(r.id));
                }
            }
            ui.label(RichText::new("Local demo").size(10.).color(tokens.text3));
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if !mobile {
                    let r = self.icon_control(
                        ui,
                        rt,
                        "ws-chat-toggle",
                        "chat",
                        "Toggle assistant panel",
                    );
                    if r.clicked() {
                        self.chat_open = !self.chat_open;
                    }
                }
                let label = if mobile {
                    format!("#{}", t.number)
                } else {
                    format!("Pull request #{}", t.number)
                };
                let r = self.control(
                    ui,
                    rt,
                    Control::new("ws-review-toggle", &label, Some("git-pull-request"))
                        .accessible_name(&format!("Pull request #{}", t.number)),
                );
                let status = if t.merged || t.passed() {
                    tokens.success_text
                } else if t.checks.contains(&Status::Failed) {
                    tokens.danger_text
                } else {
                    tokens.text4
                };
                ui.painter()
                    .circle_filled(r.rect.right_center() - Vec2::new(3., 0.), 2.5, status);
                if r.clicked() {
                    self.activity_tab = false;
                    self.open_dialog(Dialog::Review, Some(r.id));
                }
            });
        });
    }
    fn mobile_views(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect) {
        ui.painter().rect_filled(rect, 0., rt.tokens.surface2);
        let mut bar = child(ui, rect, "ws-mobile-views");
        let width = (rect.width() - 12.) / 3.;
        bar.horizontal(|ui| {
            for (view, id, label, icon) in [
                (View::Editor, "ws-view-editor", "Editor", "code"),
                (View::Chat, "ws-view-chat", "Assistant", "chat"),
                (View::Checks, "ws-view-checks", "Checks", "terminal"),
            ] {
                let selected = self.view == view;
                let r = self.control(
                    ui,
                    rt,
                    Control::new(id, label, Some(icon))
                        .tab()
                        .selected(selected)
                        .width(width)
                        .height(rect.height()),
                );
                if selected {
                    separator(
                        ui,
                        r.rect.left_bottom(),
                        r.rect.right_bottom(),
                        rt.tokens.accent,
                    );
                }
                if view == View::Checks && self.current().checks.contains(&Status::Failed) {
                    let badge = Rect::from_center_size(
                        r.rect.right_center() - Vec2::new(12., 0.),
                        Vec2::splat(16.),
                    );
                    ui.painter().rect_filled(badge, 8., rt.tokens.danger_soft);
                    ui.painter().text(
                        badge.center(),
                        egui::Align2::CENTER_CENTER,
                        "1",
                        FontId::proportional(10.),
                        rt.tokens.danger_text,
                    );
                }
                if r.clicked() {
                    self.view = view;
                    self.pending_focus = Some(r.id);
                    if view == View::Checks {
                        self.console_open = true;
                    }
                }
                if let Some(next) = self.tab_navigation(
                    ui,
                    &r,
                    id,
                    &["ws-view-editor", "ws-view-chat", "ws-view-checks"],
                ) {
                    self.view = [View::Editor, View::Chat, View::Checks][next];
                    if self.view == View::Checks {
                        self.console_open = true;
                    }
                }
            }
        });
    }
    fn tab_navigation(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        id: &str,
        tabs: &[&str],
    ) -> Option<usize> {
        if !response.has_focus() {
            return None;
        }
        let mut target = None;
        ui.input_mut(|i| {
            let index = tabs.iter().position(|x| *x == id).unwrap_or(0);
            if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
                target = Some((index + 1) % tabs.len())
            }
            if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
                target = Some((index + tabs.len() - 1) % tabs.len())
            }
            if i.consume_key(egui::Modifiers::NONE, Key::Home) {
                target = Some(0)
            }
            if i.consume_key(egui::Modifiers::NONE, Key::End) {
                target = Some(tabs.len() - 1)
            }
        });
        if let Some(index) = target {
            self.pending_focus = Some(Id::new(tabs[index]));
            return Some(index);
        }
        None
    }
    fn activity(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.bg_sunken);
        separator(
            ui,
            rect.right_top(),
            rect.right_bottom(),
            tokens.border_strong,
        );
        let mut tools = child(ui, rect.shrink2(Vec2::new(4., 8.)), "ws-activity");
        let r = self.icon_control(
            &mut tools,
            rt,
            "ws-explorer-toggle",
            "folder",
            "Toggle file explorer",
        );
        if r.clicked() {
            self.explorer_open = !self.explorer_open;
        }
        let r = self.icon_control(
            &mut tools,
            rt,
            "ws-open-changes",
            "file-diff",
            "Review changes",
        );
        if r.clicked() {
            self.view = View::Editor;
            self.set_editor_mode(true, rt.now);
        }
        let r = self.icon_control(
            &mut tools,
            rt,
            "ws-activity-sessions",
            "git-branch",
            "Coding sessions",
        );
        if r.clicked() {
            self.open_dialog(Dialog::Sessions, Some(r.id));
        }
        let r = self.icon_control(
            &mut tools,
            rt,
            "ws-console-toggle",
            "terminal",
            "Toggle CI console",
        );
        if r.clicked() {
            self.console_open = !self.console_open;
        }
        tools.add_space((rect.height() - 220.).max(0.));
        if self
            .icon_control(&mut tools, rt, "ws-reset", "refresh", "Reset local demo")
            .clicked()
        {
            self.reset();
        }
    }
    fn files(&mut self, ui: &mut Ui, rt: &mut Runtime, modal: bool) {
        let t = self.current().clone();
        let r = self.control(
            ui,
            rt,
            Control::new(
                if modal {
                    "ws-mobile-tree"
                } else {
                    "ws-tree-toggle"
                },
                "STOREFRONT",
                Some(if self.tree_open {
                    "chevron-down"
                } else {
                    "chevron-right"
                }),
            )
            .width(ui.available_width()),
        );
        if r.clicked() {
            self.tree_open = !self.tree_open;
        }
        if self.tree_open {
            ui.label(RichText::new("  src").size(11.).color(rt.tokens.text3));
            let ids = (0..t.files.len())
                .map(|i| format!("ws-file-{i}"))
                .collect::<Vec<_>>();
            for (i, file) in t.files.iter().enumerate() {
                let name = file.path.rsplit('/').next().unwrap_or(&file.path);
                let label = format!(
                    "{}  {name}   M",
                    if file.path.ends_with(".ts") {
                        "TS"
                    } else if file.path.ends_with(".css") {
                        "#"
                    } else {
                        "MD"
                    }
                );
                let id = if modal {
                    format!("ws-mobile-file-{i}")
                } else {
                    format!("ws-explorer-file-{i}")
                };
                let r = self.control(
                    ui,
                    rt,
                    Control::new(&id, &label, None)
                        .selected(t.file == i)
                        .width(ui.available_width())
                        .height(30.),
                );
                if r.clicked() {
                    self.choose_file(i, rt.now);
                    if modal {
                        self.close_dialog(false);
                        self.restore_focus = Some(Id::new(&ids[i]));
                    }
                }
                if modal {
                    self.modal_controls.push(r.id);
                }
            }
        }
    }
    fn explorer(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface2);
        separator(
            ui,
            rect.right_top(),
            rect.right_bottom(),
            tokens.border_strong,
        );
        let mut panel = child(ui, rect.shrink2(Vec2::new(8., 8.)), "ws-explorer");
        panel.horizontal(|ui| {
            ui.label(RichText::new("EXPLORER").size(10.).color(tokens.text3));
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let r = self.icon_control(
                    ui,
                    rt,
                    "ws-explorer-sessions",
                    "git-branch",
                    "Switch session",
                );
                if r.clicked() {
                    self.open_dialog(Dialog::Sessions, Some(r.id));
                }
            });
        });
        let bottom = Rect::from_min_max(
            Pos2::new(
                rect.left() + 8.,
                (rect.bottom() - 116.).max(rect.top() + 90.),
            ),
            rect.right_bottom() - Vec2::splat(8.),
        );
        let file_top = panel.cursor().min;
        let mut file_ui = child(
            &mut panel,
            Rect::from_min_max(file_top, Pos2::new(rect.right() - 8., bottom.top() - 6.)),
            "ws-file-list",
        );
        egui::ScrollArea::vertical()
            .id_salt("ws-explorer-scroll")
            .show(&mut file_ui, |ui| self.files(ui, rt, false));
        let mut footer = child(ui, bottom, "ws-explorer-footer");
        let t = self.current().clone();
        footer.label(RichText::new("SESSION").size(10.).color(tokens.text3));
        let r = self.control(
            &mut footer,
            rt,
            Control::new("ws-session-summary", &t.title, Some("git-branch")).width(bottom.width()),
        );
        if r.clicked() {
            self.open_dialog(Dialog::Sessions, Some(r.id));
        }
        footer.label(RichText::new(t.summary()).size(10.).color(tokens.text3));
        if self
            .control(
                &mut footer,
                rt,
                Control::new("ws-new", "New session", Some("plus")).width(bottom.width()),
            )
            .clicked()
        {
            self.add_thread();
        }
    }
}
fn byte_at_char(s: &str, index: usize) -> usize {
    s.char_indices().nth(index).map_or(s.len(), |(i, _)| i)
}
fn indent_buffer(source: &mut String, selection: Range<usize>, outdent: bool) -> Range<usize> {
    let a = byte_at_char(source, selection.start);
    let b = byte_at_char(source, selection.end);
    if outdent {
        let start = source[..a].rfind('\n').map_or(0, |n| n + 1);
        let indent = source[start..]
            .chars()
            .take(2)
            .take_while(|c| *c == ' ')
            .count();
        source.replace_range(start..start + indent, "");
        selection.start.saturating_sub(indent)..selection.end.saturating_sub(indent)
    } else {
        source.replace_range(a..b, "  ");
        selection.start + 2..selection.start + 2
    }
}
impl State {
    fn editor(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface);
        let t = self.current().clone();
        let tabs = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 40.));
        ui.painter().rect_filled(tabs, 0., tokens.surface2);
        let mut tab_ui = child(ui, tabs, "ws-file-tabs");
        egui::ScrollArea::horizontal()
            .id_salt("ws-file-tabs-scroll")
            .auto_shrink([false, false])
            .show(&mut tab_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.;
                    let ids = (0..t.files.len())
                        .map(|i| format!("ws-file-{i}"))
                        .collect::<Vec<_>>();
                    for (i, file) in t.files.iter().enumerate() {
                        let label = format!(
                            "{}  {}  M",
                            if file.path.ends_with(".ts") {
                                "TS"
                            } else if file.path.ends_with(".css") {
                                "#"
                            } else {
                                "MD"
                            },
                            file.path.rsplit('/').next().unwrap_or(&file.path)
                        );
                        let r = self.control(
                            ui,
                            rt,
                            Control::new(&ids[i], &label, None)
                                .tab()
                                .selected(i == t.file)
                                .width(148.)
                                .height(40.),
                        );
                        if i == t.file {
                            separator(ui, r.rect.left_top(), r.rect.right_top(), tokens.accent);
                        }
                        let mut choose = r.clicked();
                        if r.has_focus() {
                            let next = ui.input_mut(|input| {
                                if input.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
                                    Some((i + 1) % t.files.len())
                                } else if input.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
                                    Some((i + t.files.len() - 1) % t.files.len())
                                } else if input.consume_key(egui::Modifiers::NONE, Key::Home) {
                                    Some(0)
                                } else if input.consume_key(egui::Modifiers::NONE, Key::End) {
                                    Some(t.files.len() - 1)
                                } else {
                                    None
                                }
                            });
                            if let Some(next) = next {
                                self.choose_file(next, rt.now);
                                self.pending_focus = Some(Id::new(&ids[next]));
                                choose = false;
                            }
                        }
                        if choose {
                            self.choose_file(i, rt.now);
                            self.pending_focus = Some(r.id);
                        }
                    }
                });
            });
        let toolbar = Rect::from_min_size(
            Pos2::new(rect.left(), tabs.bottom()),
            Vec2::new(rect.width(), 34.),
        );
        separator(
            ui,
            toolbar.left_bottom(),
            toolbar.right_bottom(),
            tokens.border_soft,
        );
        let mut tools = child(ui, toolbar.shrink2(Vec2::new(6., 3.)), "ws-editor-toolbar");
        let narrow = rect.width() < 500.;
        tools.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
            let path = if narrow {
                t.files[t.file].path.rsplit('/').next().unwrap_or("").into()
            } else {
                t.files[t.file].path.replace('/', " / ")
            };
            let right_width = if narrow { 184. } else { 214. };
            let label_rect = ui
                .allocate_exact_size(
                    Vec2::new((toolbar.width() - right_width - 12.).max(30.), 28.),
                    Sense::hover(),
                )
                .0;
            ui.painter().with_clip_rect(label_rect).text(
                label_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                path,
                FontId::proportional(if narrow { 10. } else { 11. }),
                tokens.text3,
            );
            let current_diff = self.current().editor_mode == "diff";
            if self
                .control(
                    ui,
                    rt,
                    Control::new("ws-editor-mode-source", "Source", None)
                        .selected(!current_diff)
                        .width(if narrow { 48. } else { 56. }),
                )
                .clicked()
            {
                self.set_editor_mode(false, rt.now);
            }
            if self
                .control(
                    ui,
                    rt,
                    Control::new("ws-editor-mode-diff", "Split diff", None)
                        .selected(current_diff)
                        .width(if narrow { 58. } else { 66. }),
                )
                .clicked()
            {
                self.set_editor_mode(true, rt.now);
            }
            let t = self.current();
            let editing = t.editing;
            let enabled = !t.merged && !t.running;
            if self
                .control(
                    ui,
                    rt,
                    Control::new(
                        "ws-edit",
                        if editing { "Done" } else { "Edit" },
                        Some(if editing { "check" } else { "pencil" }),
                    )
                    .enabled(enabled)
                    .width(if narrow { 62. } else { 70. }),
                )
                .clicked()
            {
                self.toggle_edit();
            }
        });
        let footer = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - 24.), rect.max);
        separator(
            ui,
            footer.left_top(),
            footer.right_top(),
            tokens.border_soft,
        );
        let mut bottom = child(ui, footer.shrink2(Vec2::new(8., 2.)), "ws-editor-footer");
        let t = self.current();
        let note = if t.merged {
            "Merged · read only"
        } else if t.editing {
            if narrow {
                "Esc / Ctrl+S to finish"
            } else {
                "Editing local buffer · Esc or Ctrl/⌘ S to finish"
            }
        } else if t.editor_mode == "diff" {
            "main ↔ working tree"
        } else {
            "Working tree"
        };
        bottom.horizontal(|ui| {
            ui.label(RichText::new(note).size(10.).color(tokens.text3));
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let t = self.current();
                let f = &t.files[t.file];
                if !narrow {
                    ui.label(
                        RichText::new(if f.path.ends_with(".ts") {
                            "TypeScript"
                        } else if f.path.ends_with(".css") {
                            "CSS"
                        } else {
                            "Markdown"
                        })
                        .size(10.)
                        .color(tokens.text3),
                    );
                    ui.label(RichText::new("UTF-8").size(10.).color(tokens.text3));
                }
                ui.label(
                    RichText::new(format!("{} lines", f.after.len()))
                        .size(10.)
                        .color(tokens.text3),
                );
            });
        });
        let mut content = Rect::from_min_max(
            Pos2::new(rect.left(), toolbar.bottom()),
            Pos2::new(rect.right(), footer.top()),
        );
        let progress = self
            .editor_transition
            .map_or(1., |start| rt.progress(start, 0.18));
        if progress < 1. {
            rt.repaint();
            content = content.translate(Vec2::new(
                5. * (1. - crate::motion::ease(Curve::Out, progress)),
                0.,
            ));
        } else {
            self.editor_transition = None;
        }
        let mut body = child(ui, content, "ws-editor-content");
        body.set_opacity(0.4 + 0.6 * crate::motion::ease(Curve::Out, progress));
        if self.current().editor_mode == "diff" {
            self.split_diff(&mut body, rt);
        } else {
            self.source_editor(&mut body, rt);
        }
    }
    fn source_editor(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let t = self.current().clone();
        let file = t.file;
        let original = t.files[file].after.join("\n");
        let mut source = original.clone();
        let id = Id::new(("ws-code-input", &t.id, file));
        let scroll_key = (t.id.clone(), file);
        let editing = t.editing && !t.merged && !t.running;
        let diagnostic = t.id == "search" && !t.repaired() && !editing;
        let tokens = rt.tokens.clone();
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
        if self.clear_editor_history {
            state.clear_undoer();
            state.clone().store(ui.ctx(), id);
            self.clear_editor_history = false;
        }
        if self.editor_focus && editing {
            let at = source.find("    openSearch();").unwrap_or(0);
            let char_at = source[..at].chars().count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(char_at),
                )));
            state.clone().store(ui.ctx(), id);
            ui.memory_mut(|m| m.request_focus(id));
            // A release on Edit is outside the textarea. Restore after the
            // complete frame so the native editor does not blur on that release.
            self.pending_focus = Some(id);
            self.editor_focus = false;
        }
        let mut finish = false;
        let mut indentation = None;
        if editing && ui.memory(|m| m.has_focus(id)) && self.dialogs.is_empty() {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, Key::Escape)
                    || i.consume_key(egui::Modifiers::COMMAND, Key::S)
                    || i.consume_key(egui::Modifiers::CTRL, Key::S)
                {
                    finish = true;
                }
                if i.consume_key(egui::Modifiers::SHIFT, Key::Tab) {
                    indentation = Some(true)
                } else if i.consume_key(egui::Modifiers::NONE, Key::Tab) {
                    indentation = Some(false)
                }
            });
        }
        if let Some(outdent) = indentation {
            let selection = state
                .cursor
                .char_range()
                .map_or(0..0, |r| r.as_sorted_char_range());
            let selection = indent_buffer(&mut source, selection, outdent);
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(selection.start),
                    egui::text::CCursor::new(selection.end),
                )));
            state.store(ui.ctx(), id);
        }
        let viewport = ui.available_rect_before_wrap();
        let diagnostic_background = ui.painter().with_clip_rect(viewport).add(egui::Shape::Noop);
        let font_size = if rt.context().content_rect().width() <= 760. {
            12.
        } else {
            13.
        };
        let glyph_width = ui
            .painter()
            .layout_no_wrap(" ".into(), FontId::monospace(font_size), tokens.text)
            .size()
            .x;
        let mut scroll = egui::ScrollArea::both()
            .id_salt(("ws-source", &t.id, file))
            .auto_shrink([false, false]);
        if let Some(y) = self.editor_scroll.take() {
            scroll = scroll.vertical_scroll_offset(y);
        }
        let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
            ui.fonts_mut(|f| f.layout_job(syntax_job(buffer.as_str(), &tokens, font_size)))
        };
        let longest = source
            .split('\n')
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as f32
            * glyph_width;
        let mut editor_output = None;
        let output = scroll.show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(12.);
            ui.horizontal_top(|ui| {
                ui.allocate_exact_size(Vec2::new(54., 1.), Sense::hover());
                let builder = egui::TextEdit::multiline(&mut source)
                    .id(id)
                    .code_editor()
                    .frame(false)
                    .margin(egui::Margin::same(0))
                    .desired_width(longest.max((viewport.width() - 64.).max(1.)))
                    .desired_rows(1)
                    .layouter(&mut layouter);
                let output = if editing {
                    builder.show(ui)
                } else {
                    drop(builder);
                    let mut readonly = source.as_str();
                    egui::TextEdit::multiline(&mut readonly)
                        .id(id)
                        .font(FontId::monospace(font_size))
                        .frame(false)
                        .margin(egui::Margin::same(0))
                        .desired_width(longest.max((viewport.width() - 64.).max(1.)))
                        .desired_rows(1)
                        .layouter(&mut layouter)
                        .show(ui)
                };
                rt.record(
                    if editing {
                        "ws-code-input"
                    } else {
                        "ws-source"
                    },
                    &output.response,
                );
                self.track_focus(&output.response);
                self.controls.insert(
                    if editing {
                        "ws-code-input"
                    } else {
                        "ws-source"
                    }
                    .into(),
                    output.response.rect.intersect(viewport),
                );
                editor_output = Some(output);
            });
            ui.add_space(32.);
        });
        self.source_scroll.insert(scroll_key, output.state.offset);
        if let Some(output) = editor_output {
            let painter = ui.painter().with_clip_rect(viewport);
            painter.rect_filled(
                Rect::from_min_size(viewport.min, Vec2::new(48., viewport.height())),
                0.,
                tokens.bg_sunken,
            );
            for (i, line) in source.split('\n').enumerate() {
                let y = output.galley_pos.y + i as f32 * 22. + 11.;
                if y >= viewport.top() - 11. && y <= viewport.bottom() + 11. {
                    let marked = diagnostic && line.trim() == "openSearch();";
                    if marked {
                        let row = Rect::from_min_size(
                            Pos2::new(viewport.left(), y - 11.),
                            Vec2::new(viewport.width(), 22.),
                        );
                        painter.set(
                            diagnostic_background,
                            egui::Shape::rect_filled(row, 0., tokens.danger_soft),
                        );
                        painter.rect_filled(
                            Rect::from_min_size(row.min, Vec2::new(48., 22.)),
                            0.,
                            tokens.danger_soft,
                        );
                        painter.line_segment(
                            [
                                row.left_top() + Vec2::new(1., 0.),
                                row.left_bottom() + Vec2::new(1., 0.),
                            ],
                            Stroke::new(2., tokens.danger_text),
                        );
                        let start = output.galley_pos.x
                            + line.chars().take_while(|c| c.is_whitespace()).count() as f32
                                * glyph_width;
                        let end = output.galley_pos.x + line.chars().count() as f32 * glyph_width;
                        let points = (0..=((end - start) / 2.).ceil() as usize)
                            .map(|n| {
                                Pos2::new(
                                    start + 2. * n as f32,
                                    y + 7. + if n % 2 == 0 { 0. } else { 1.5 },
                                )
                            })
                            .collect::<Vec<_>>();
                        painter.add(egui::Shape::line(
                            points,
                            Stroke::new(1., tokens.danger_text),
                        ));
                    }
                    painter.text(
                        Pos2::new(viewport.left() + 38., y),
                        egui::Align2::RIGHT_CENTER,
                        (i + 1).to_string(),
                        FontId::monospace(11.),
                        if marked {
                            tokens.danger_text
                        } else {
                            tokens.text4
                        },
                    );
                }
            }
            if let Some(range) = output.cursor_range {
                let prefix = source.chars().take(range.primary.index).collect::<String>();
                self.cursor = (
                    prefix.split('\n').count(),
                    prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
                );
            }
        }
        if source != original && editing {
            self.edit_buffer(file, &source);
        }
        if finish {
            self.finish_edit();
        }
    }
    fn split_diff(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let t = self.current().clone();
        let key = (t.id.clone(), t.file, t.revision);
        if self.diff_cache.key != key {
            self.diff_cache.rows = split_rows(&t.files[t.file]);
            self.diff_cache.key = key;
        }
        let row_count = self.diff_cache.rows.len();
        let tokens = rt.tokens.clone();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            let half = ui.available_width() / 2.;
            for label in [
                "main · Original".to_owned(),
                format!("{} · Modified", t.branch),
            ] {
                let rect = ui
                    .allocate_exact_size(Vec2::new(half, 28.), Sense::hover())
                    .0;
                ui.painter().rect_filled(rect, 0., tokens.surface2);
                ui.painter().with_clip_rect(rect).text(
                    rect.left_center() + Vec2::new(8., 0.),
                    egui::Align2::LEFT_CENTER,
                    label,
                    FontId::monospace(10.),
                    tokens.text3,
                );
            }
        });
        let view = ui.available_rect_before_wrap();
        let mut vertical = None;
        let mut horizontal_request = [None, None];
        let mut horizontal = self.diff_horizontal;
        for side in 0..2 {
            let name = if side == 0 {
                "ws-diff-old"
            } else {
                "ws-diff-new"
            };
            let pane = Rect::from_min_max(
                Pos2::new(
                    if side == 0 {
                        view.left()
                    } else {
                        view.center().x
                    },
                    view.top(),
                ),
                Pos2::new(
                    if side == 0 {
                        view.center().x
                    } else {
                        view.right()
                    },
                    view.bottom(),
                ),
            );
            let response = ui.interact(pane, Id::new(name), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Other,
                    ui.is_enabled(),
                    if side == 0 {
                        "Original file"
                    } else {
                        "Modified file"
                    },
                )
            });
            if response.clicked() {
                response.request_focus();
            }
            if response.has_focus() {
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    )
                });
                ui.input_mut(|i| {
                    let mut y = self.diff_scroll;
                    if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                        y += 22.;
                        vertical = Some(y);
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                        y -= 22.;
                        vertical = Some(y.max(0.));
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::PageDown) {
                        y += view.height() * 0.8;
                        vertical = Some(y);
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::PageUp) {
                        y -= view.height() * 0.8;
                        vertical = Some(y.max(0.));
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::Home) {
                        vertical = Some(0.);
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::End) {
                        vertical = Some(row_count as f32 * 22.);
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
                        horizontal_request[side] = Some(horizontal[side] + 40.);
                    }
                    if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
                        horizontal_request[side] = Some((horizontal[side] - 40.).max(0.));
                    }
                });
            }
            self.controls.insert(name.into(), pane);
            self.track_focus(&response);
            rt.record(name, &response);
        }
        let rows = &self.diff_cache.rows;
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt(("ws-diff-vertical", &t.id, t.file))
            .auto_shrink([false, false]);
        if let Some(y) = vertical {
            scroll = scroll.vertical_scroll_offset(y);
        }
        let output = scroll.show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.columns(2, |columns| {
                for (side, column) in columns.iter_mut().enumerate() {
                    let width = column.available_width();
                    let mut scroll = egui::ScrollArea::horizontal()
                        .id_salt(("ws-diff-horizontal", &t.id, t.file, side))
                        .auto_shrink([false, true]);
                    if let Some(x) = horizontal_request[side] {
                        scroll = scroll.horizontal_scroll_offset(x);
                    }
                    let output = scroll.show(column, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        for pair in rows {
                            let line = pair[side].as_ref();
                            let number = line
                                .and_then(|l| if side == 0 { l.old } else { l.next })
                                .map_or(String::new(), |n| n.to_string());
                            let sign = line.map_or("", |l| match l.kind {
                                "added" => "+",
                                "removed" => "−",
                                _ => "",
                            });
                            let code =
                                line.map_or(" ", |l| if l.text.is_empty() { " " } else { &l.text });
                            let mut job = syntax_job(code, &tokens, 12.);
                            let prefix = format!("{number:>4} {sign:1} ");
                            let mut combined = egui::text::LayoutJob::default();
                            combined.append(
                                &prefix,
                                0.,
                                egui::TextFormat {
                                    font_id: FontId::monospace(12.),
                                    color: tokens.text4,
                                    line_height: Some(22.),
                                    ..Default::default()
                                },
                            );
                            let offset = combined.text.len();
                            combined.text.push_str(&job.text);
                            for mut section in job.sections.drain(..) {
                                section.byte_range.start += offset;
                                section.byte_range.end += offset;
                                combined.sections.push(section);
                            }
                            combined.wrap.max_width = f32::INFINITY;
                            let galley = ui.fonts_mut(|f| f.layout_job(combined));
                            let row = ui
                                .allocate_exact_size(
                                    Vec2::new(width.max(galley.size().x + 8.), 22.),
                                    Sense::hover(),
                                )
                                .0;
                            let bg = match line.map(|l| l.kind) {
                                Some("added") => tokens.success_soft,
                                Some("removed") => tokens.danger_soft,
                                None => tokens.bg_sunken,
                                _ => tokens.surface,
                            };
                            ui.painter().rect_filled(row, 0., bg);
                            ui.painter()
                                .galley(row.min + Vec2::new(2., 0.), galley, tokens.text);
                        }
                    });
                    horizontal[side] = output.state.offset.x;
                }
            });
        });
        self.diff_scroll = output.state.offset.y;
        self.diff_horizontal = horizontal;
        separator(
            ui,
            Pos2::new(view.center().x, view.top()),
            Pos2::new(view.center().x, view.bottom()),
            tokens.border_strong,
        );
        rt.anchor("ws-diff", "Split diff", view);
    }
}
impl State {
    fn log_lines(&self) -> Vec<String> {
        let t = self.current();
        let c = &CHECKS[self.check];
        let mut result = vec![format!("$ {}", c.command)];
        match t.checks[self.check] {
            Status::Stale => {
                result.push("Working tree changed. Rerun all checks for this revision.".into())
            }
            Status::Queued => result.push("Waiting for previous check…".into()),
            Status::Running => {
                result.extend(c.success.iter().take(2).map(|s| s.to_string()));
                result.push("Running…".into());
            }
            Status::Cancelled => {
                result.push("Run cancelled. No result recorded for this revision.".into())
            }
            Status::Failed => result.extend(
                [
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
                ]
                .map(str::to_owned),
            ),
            Status::Passed => result.extend(c.success.iter().map(|s| s.to_string())),
        }
        result
    }
    fn console(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect, mobile: bool) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.bg_sunken);
        separator(ui, rect.left_top(), rect.right_top(), tokens.border_strong);
        let heading = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 36.));
        let mut head = child(ui, heading.shrink2(Vec2::new(6., 3.)), "ws-console-heading");
        head.horizontal(|ui| {
            ui.label(
                RichText::new("CI checks")
                    .size(12.)
                    .strong()
                    .color(tokens.text),
            );
            let t = self.current();
            let summary = if t.running {
                "Running".into()
            } else {
                format!(
                    "{} / 3 passed",
                    t.checks.iter().filter(|s| **s == Status::Passed).count()
                )
            };
            ui.label(RichText::new(summary).size(10.).color(tokens.text3));
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if self
                    .icon_control(
                        ui,
                        rt,
                        "ws-console-close",
                        "chevron-down",
                        "Collapse CI console",
                    )
                    .clicked()
                {
                    if mobile {
                        self.view = View::Editor
                    } else {
                        self.console_open = false;
                        self.pending_focus = Some(Id::new("ws-console-toggle"));
                    }
                }
                let enabled = !self.current().running && !self.current().merged;
                if self
                    .control(
                        ui,
                        rt,
                        Control::new(
                            "ws-run-checks",
                            if mobile { "Rerun" } else { "Rerun all" },
                            Some("refresh"),
                        )
                        .enabled(enabled),
                    )
                    .clicked()
                {
                    self.send("Run checks", rt.now);
                }
            });
        });
        let body = Rect::from_min_max(Pos2::new(rect.left(), heading.bottom()), rect.max);
        let width = if mobile {
            108.
        } else if rect.width() < 700. {
            140.
        } else {
            178.
        };
        let list_rect = Rect::from_min_max(body.min, Pos2::new(body.left() + width, body.bottom()));
        let mut list = child(ui, list_rect.shrink2(Vec2::new(4., 4.)), "ws-checks-list");
        let t = self.current().clone();
        for (i, c) in CHECKS.iter().enumerate() {
            let id = format!("ws-check-{}", c.id);
            let status = t.checks[i];
            let r = self.control(
                &mut list,
                rt,
                Control::new(&id, c.title, Some(status.icon()))
                    .selected(i == self.check)
                    .width(width - 8.)
                    .height(34.),
            );
            let details = if mobile {
                status.label().to_owned()
            } else {
                format!(
                    "{}   {}",
                    status.label(),
                    if matches!(status, Status::Passed | Status::Failed) {
                        c.duration
                    } else {
                        ""
                    }
                )
            };
            list.label(
                RichText::new(details)
                    .size(10.)
                    .color(status_color(status, &tokens)),
            );
            if r.clicked() {
                self.check = i;
                self.log_scroll_reset = true;
            }
        }
        separator(
            ui,
            list_rect.right_top(),
            list_rect.right_bottom(),
            tokens.border_strong,
        );
        let log_rect = Rect::from_min_max(
            Pos2::new(list_rect.right() + 8., body.top() + 4.),
            body.max - Vec2::new(4., 4.),
        );
        let mut log_ui = child(ui, log_rect, "ws-log-pane");
        log_ui.horizontal(|ui| {
            ui.label(
                RichText::new(CHECKS[self.check].command)
                    .monospace()
                    .size(11.)
                    .color(tokens.text2),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let status = self.current().checks[self.check];
                ui.label(
                    RichText::new(match status {
                        Status::Passed => "exit 0",
                        Status::Failed => "exit 1",
                        _ => status.label(),
                    })
                    .monospace()
                    .size(10.)
                    .color(status_color(status, &tokens)),
                );
            });
        });
        let log = self.log_lines().join("\n");
        let mut readonly = log.as_str();
        let mut layouter = |ui: &Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
            let mut job = egui::text::LayoutJob::default();
            for line in buffer.as_str().split_inclusive('\n') {
                let color = if ["FAIL", "AssertionError", "Expected:", "Received:", "code 1"]
                    .iter()
                    .any(|s| line.contains(s))
                {
                    tokens.danger_text
                } else if ["PASS", "passed", "code 0"]
                    .iter()
                    .any(|s| line.contains(s))
                {
                    tokens.success_text
                } else {
                    tokens.text2
                };
                job.append(
                    line,
                    0.,
                    egui::TextFormat {
                        font_id: FontId::monospace(if mobile { 10. } else { 11. }),
                        color,
                        line_height: Some(20.),
                        ..Default::default()
                    },
                );
            }
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let mut scroll = egui::ScrollArea::both()
            .id_salt(("ws-log-scroll", &self.selected, self.check))
            .auto_shrink([false, false]);
        if self.log_scroll_reset {
            scroll = scroll.vertical_scroll_offset(0.);
            self.log_scroll_reset = false;
        }
        scroll.show(&mut log_ui, |ui| {
            let r = egui::TextEdit::multiline(&mut readonly)
                .id(Id::new(("ws-log", &self.selected, self.check)))
                .frame(false)
                .margin(0)
                .desired_width(f32::INFINITY)
                .desired_rows(1)
                .layouter(&mut layouter)
                .show(ui)
                .response;
            rt.record("ws-log", &r);
            self.track_focus(&r);
        });
        rt.anchor("ws-console", "CI console", rect);
    }
    fn open_changes(&mut self, now: f64) {
        self.dialogs.clear();
        self.view = View::Editor;
        self.set_editor_mode(true, now);
        self.pending_focus = Some(Id::new("ws-editor-mode-diff"));
    }
    fn open_checks(&mut self, mobile: bool) {
        self.dialogs.clear();
        self.console_open = true;
        if mobile {
            self.view = View::Checks;
        }
        self.pending_focus = Some(Id::new(format!("ws-check-{}", CHECKS[self.check].id)));
    }
    fn chat(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect, mobile: bool) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface2);
        separator(
            ui,
            rect.left_top(),
            rect.left_bottom(),
            tokens.border_strong,
        );
        let header = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 40.));
        let mut head = child(ui, header.shrink2(Vec2::new(12., 4.)), "ws-chat-heading");
        head.horizontal(|ui| {
            ui.label(
                RichText::new("Assistant")
                    .strong()
                    .size(12.)
                    .color(tokens.text),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let r = self.icon_control(ui, rt, "ws-rename", "pencil", "Rename coding session");
                if r.clicked() {
                    self.open_dialog(Dialog::Rename, Some(r.id));
                }
            });
        });
        separator(
            ui,
            header.left_bottom(),
            header.right_bottom(),
            tokens.border_soft,
        );
        let bottom_height = if rect.height() < 440. { 142. } else { 180. };
        let bottom = Rect::from_min_max(
            Pos2::new(
                rect.left() + 12.,
                (rect.bottom() - bottom_height).max(header.bottom() + 30.),
            ),
            rect.max - Vec2::new(12., 8.),
        );
        let messages = Rect::from_min_max(
            Pos2::new(rect.left() + 12., header.bottom() + 8.),
            Pos2::new(rect.right() - 12., bottom.top() - 8.),
        );
        let mut conversation = child(ui, messages, "ws-messages");
        let t = self.current().clone();
        egui::ScrollArea::vertical()
            .id_salt(("ws-messages-scroll", &t.id))
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(&mut conversation, |ui| {
                ui.spacing_mut().item_spacing.y = 6.;
                for (i, m) in t.messages.iter().enumerate() {
                    ui.push_id((&t.id, i), |ui| {
                        ui.horizontal(|ui| {
                            let (icon_rect, _) =
                                ui.allocate_exact_size(Vec2::splat(18.), Sense::hover());
                            crate::icons::paint(
                                ui.painter(),
                                if m.user { "user" } else { "code" },
                                icon_rect.shrink(2.),
                                tokens.text2,
                                1.6,
                                0.,
                            );
                            ui.label(
                                RichText::new(if m.user { "You" } else { "Assistant" })
                                    .strong()
                                    .size(11.)
                                    .color(tokens.text),
                            );
                            if m.streaming {
                                ui.label(
                                    RichText::new("Working").size(10.).color(tokens.accent_text),
                                );
                            }
                        });
                        ui.add(
                            egui::Label::new(RichText::new(&m.text).size(12.).color(tokens.text2))
                                .wrap(),
                        );
                        for (j, tool) in m.tools.iter().enumerate() {
                            let id = Id::new(("ws-tool", &t.id, i, j));
                            let (toggle, _, _) =
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    id,
                                    false,
                                )
                                .show_header(ui, |ui| {
                                    let (icon_rect, _) =
                                        ui.allocate_exact_size(Vec2::splat(14.), Sense::hover());
                                    let progress = icon_progress(
                                        rt,
                                        ("ws-tool-status", &t.id, i, j, tool.status.key()),
                                        tool.status.icon(),
                                        true,
                                    );
                                    crate::icons::paint(
                                        ui.painter(),
                                        tool.status.icon(),
                                        icon_rect,
                                        status_color(tool.status, &tokens),
                                        1.6,
                                        progress,
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&tool.label)
                                                .size(10.)
                                                .color(tokens.text2),
                                        )
                                        .truncate(),
                                    );
                                })
                                .body(|ui| {
                                    if tool.kind == "read" {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}\n{} lines",
                                                t.files[0].path,
                                                t.files[0].after.len()
                                            ))
                                            .monospace()
                                            .size(10.)
                                            .color(tokens.text3),
                                        );
                                    } else {
                                        let id = format!("ws-tool-open-{i}-{j}");
                                        let changes = tool.kind == "edit";
                                        if self
                                            .control(
                                                ui,
                                                rt,
                                                Control::new(
                                                    &id,
                                                    if changes {
                                                        "Open split diff"
                                                    } else {
                                                        "Open CI console"
                                                    },
                                                    Some("arrow-right"),
                                                )
                                                .width(ui.available_width()),
                                            )
                                            .clicked()
                                        {
                                            if changes {
                                                self.open_changes(rt.now);
                                            } else {
                                                self.open_checks(mobile);
                                            }
                                        }
                                    }
                                });
                            self.track_focus(&toggle);
                            rt.record(&format!("ws-tool-{i}-{j}"), &toggle);
                        }
                        if m.stopped {
                            ui.label(RichText::new("Stopped").size(10.).color(tokens.text3));
                        }
                        ui.add_space(12.);
                    });
                }
            });
        let mut composer = child(ui, bottom, "ws-chat-bottom");
        let merged = self.current().merged;
        let running = self.current().running;
        if !merged && !running {
            composer.horizontal_wrapped(|ui| {
                if self
                    .control(
                        ui,
                        rt,
                        Control::new("ws-prompt-fix", "Fix failed check", Some("code")).height(24.),
                    )
                    .clicked()
                {
                    self.send("Fix the failed check", rt.now);
                }
                if self
                    .control(
                        ui,
                        rt,
                        Control::new("ws-prompt-explain", "Explain changes", None).height(24.),
                    )
                    .clicked()
                {
                    self.send("Explain these changes", rt.now);
                }
            });
        } else {
            composer.add_space(28.);
        }
        let mut draft = self.current().draft.clone();
        let id = Id::new("ws-prompt-input");
        let composer_focused = composer.memory(|m| m.has_focus(id));
        if composer_focused {
            composer.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Ime(egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_)) => {
                            self.composer_ime = true
                        }
                        egui::Event::Ime(egui::ImeEvent::Disabled | egui::ImeEvent::Commit(_)) => {
                            self.composer_ime = false
                        }
                        _ => {}
                    }
                }
            });
        } else {
            self.composer_ime = false;
        }
        let ime_event = composer.input(|i| {
            i.events
                .iter()
                .any(|event| matches!(event, egui::Event::Ime(_)))
        });
        let send_enter = composer_focused && self.dialogs.is_empty() && !self.composer_ime && !ime_event && composer.input_mut(|i|{
            let mut send=false;
            i.events.retain(|event|{
                if matches!(event,egui::Event::Key{key:Key::Enter,pressed:true,modifiers,..} if !modifiers.shift){send=true;false}else{true}
            });send
        });
        let output = composer.add_enabled(
            !merged,
            egui::TextEdit::multiline(&mut draft)
                .id(id)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text("Ask for a code change…")
                .char_limit(2000)
                .font(FontId::proportional(12.)),
        );
        rt.record("ws-prompt-input", &output);
        self.track_focus(&output);
        self.controls.insert("ws-prompt-input".into(), output.rect);
        if output.changed() {
            self.current_mut().draft = draft.clone();
        }
        composer.horizontal(|ui| {
            if self
                .control(
                    ui,
                    rt,
                    Control::new("ws-mode-build", "Build", None)
                        .selected(!self.mode_plan)
                        .width(52.),
                )
                .clicked()
            {
                self.mode_plan = false;
            }
            if self
                .control(
                    ui,
                    rt,
                    Control::new("ws-mode-plan", "Plan", None)
                        .selected(self.mode_plan)
                        .width(48.),
                )
                .clicked()
            {
                self.mode_plan = true;
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if self.current().running {
                    if self
                        .icon_control(ui, rt, "ws-stop", "stop", "Stop generation")
                        .clicked()
                    {
                        self.stop(true);
                    }
                } else if self
                    .control(
                        ui,
                        rt,
                        Control::new("ws-send", "Send", Some("arrow-up"))
                            .enabled(!merged)
                            .width(66.),
                    )
                    .clicked()
                {
                    self.send(&draft, rt.now);
                }
            });
        });
        if send_enter {
            self.send(&draft, rt.now);
        }
        rt.anchor("ws-chat", "Coding assistant", rect);
    }
}
impl State {
    fn modal_control(&mut self, ui: &mut Ui, rt: &mut Runtime, c: Control<'_>) -> Response {
        let enabled = c.enabled && (!c.tab || c.selected);
        let response = self.control(ui, rt, c);
        if enabled {
            self.modal_controls.push(response.id);
        }
        response
    }
    fn modal_focus(&mut self, _ui: &mut Ui, initial: bool, first: Option<Id>) {
        if initial
            && self.dialogs.last().map(|d| d.kind) == self.drawn_dialog
            && let Some(id) = first.filter(|id| self.focus_available(*id)).or_else(|| {
                self.modal_controls
                    .iter()
                    .copied()
                    .find(|id| self.focus_available(*id))
            })
        {
            self.pending_focus = Some(id);
        }
    }
    fn dialogs(&mut self, ctx: &egui::Context, rt: &mut Runtime, mobile: bool) {
        let Some(dialog) = self.dialogs.last().cloned() else {
            return;
        };
        let focused = ctx.memory(|m| m.focused());
        let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        if escape {
            self.close_dialog(true);
            return;
        }
        if !self.modal_controls.is_empty() {
            let backward = ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, Key::Tab));
            let forward = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Tab));
            if forward || backward {
                let n = self.modal_controls.len();
                let i = focused.and_then(|id| self.modal_controls.iter().position(|x| *x == id));
                let next = if backward {
                    i.map_or(n - 1, |i| (i + n - 1) % n)
                } else {
                    i.map_or(0, |i| (i + 1) % n)
                };
                ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                self.pending_focus = Some(self.modal_controls[next]);
            }
        }
        self.modal_controls.clear();
        let tokens = rt.tokens.clone();
        let width = (ctx.content_rect().width() - 32.).clamp(
            248.,
            if dialog.kind == Dialog::Review {
                620.
            } else {
                440.
            },
        );
        let max_height = (ctx.content_rect().height() - 96.).max(160.);
        let frame = egui::Frame::new()
            .fill(tokens.surface)
            .stroke(Stroke::new(1., tokens.border_strong))
            .corner_radius(10. * tokens.radius)
            .inner_margin(16.);
        self.drawn_dialog = Some(dialog.kind);
        let modal = egui::Modal::new(Id::new(("ws-dialog", dialog.kind as u8)))
            .backdrop_color(tokens.overlay)
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(width - 32.);
                let title = match dialog.kind {
                    Dialog::Review => format!("Pull request #{}", self.current().number),
                    Dialog::Sessions => "Coding sessions".into(),
                    Dialog::Files => "Project files".into(),
                    Dialog::Rename => "Rename session".into(),
                    Dialog::ConfirmMerge => "Merge pull request".into(),
                };
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(title).size(16.).color(tokens.text));
                    if !matches!(dialog.kind, Dialog::Rename | Dialog::ConfirmMerge) {
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let (id, label) = match dialog.kind {
                                Dialog::Review => ("ws-review-close", "Close pull request"),
                                Dialog::Sessions => ("ws-nav-close", "Close sessions"),
                                _ => ("ws-files-close", "Close project files"),
                            };
                            if self
                                .modal_control(
                                    ui,
                                    rt,
                                    Control::new(id, "", Some("x"))
                                        .accessible_name(label)
                                        .width(28.),
                                )
                                .on_hover_text(label)
                                .clicked()
                            {
                                self.close_dialog(true);
                            }
                        });
                    }
                });
                ui.add_space(10.);
                match dialog.kind {
                    Dialog::Review => {
                        ui.horizontal(|ui| {
                            for (index, id, label) in [
                                (0, "ws-tab-pr", "Summary"),
                                (1, "ws-tab-activity", "Activity"),
                            ] {
                                let selected = self.activity_tab == (index == 1);
                                let r = self.modal_control(
                                    ui,
                                    rt,
                                    Control::new(id, label, None).tab().selected(selected),
                                );
                                if r.clicked() {
                                    self.activity_tab = index == 1;
                                    self.pending_focus = Some(r.id);
                                }
                                if let Some(next) = self.tab_navigation(
                                    ui,
                                    &r,
                                    id,
                                    &["ws-tab-pr", "ws-tab-activity"],
                                ) {
                                    self.activity_tab = next == 1;
                                }
                            }
                        });
                        ui.separator();
                        let t = self.current().clone();
                        egui::ScrollArea::vertical()
                            .id_salt(("ws-review-content", self.activity_tab))
                            .max_height(max_height - 100.)
                            .show(ui, |ui| {
                                if self.activity_tab {
                                    for event in &t.events {
                                        ui.horizontal_top(|ui| {
                                            let (rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(18.),
                                                Sense::hover(),
                                            );
                                            let icon = if event.text.contains("Merged") {
                                                "git-merge"
                                            } else if event.text.contains("passed")
                                                || event.text.contains("Approved")
                                            {
                                                "check-circle"
                                            } else {
                                                "git-commit"
                                            };
                                            crate::icons::paint(
                                                ui.painter(),
                                                icon,
                                                rect.shrink(2.),
                                                tokens.text3,
                                                1.6,
                                                0.,
                                            );
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(&event.text)
                                                        .size(12.)
                                                        .color(tokens.text2),
                                                );
                                                ui.label(
                                                    RichText::new(&event.time)
                                                        .size(10.)
                                                        .color(tokens.text3),
                                                );
                                            });
                                        });
                                        ui.add_space(8.);
                                    }
                                } else {
                                    ui.label(
                                        RichText::new(if t.merged {
                                            "Merged"
                                        } else if t.draft_pr {
                                            "Draft"
                                        } else {
                                            "Open"
                                        })
                                        .size(12.)
                                        .strong()
                                        .color(
                                            if t.merged {
                                                tokens.success_text
                                            } else {
                                                tokens.accent_text
                                            },
                                        ),
                                    );
                                    ui.heading(
                                        RichText::new(&t.title).size(18.).color(tokens.text),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("{}  →  main", t.branch))
                                                .monospace()
                                                .size(11.)
                                                .color(tokens.text3),
                                        )
                                        .wrap(),
                                    );
                                    let deltas =
                                        t.files.iter().flat_map(diff_lines).collect::<Vec<_>>();
                                    let adds = deltas.iter().filter(|l| l.kind == "added").count();
                                    let deletes =
                                        deltas.iter().filter(|l| l.kind == "removed").count();
                                    ui.horizontal(|ui| {
                                        if self
                                            .modal_control(
                                                ui,
                                                rt,
                                                Control::new(
                                                    "ws-review-changes",
                                                    &format!("{} changed files", t.files.len()),
                                                    Some("file-diff"),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            self.open_changes(rt.now);
                                        }
                                        ui.label(
                                            RichText::new(format!("+{adds}"))
                                                .size(12.)
                                                .color(tokens.success_text),
                                        );
                                        ui.label(
                                            RichText::new(format!("−{deletes}"))
                                                .size(12.)
                                                .color(tokens.danger_text),
                                        );
                                    });
                                    ui.add_space(8.);
                                    ui.separator();
                                    if self
                                        .modal_control(
                                            ui,
                                            rt,
                                            Control::new(
                                                "ws-review-checks",
                                                t.summary(),
                                                Some(if t.passed() {
                                                    "check-circle"
                                                } else if t.checks.contains(&Status::Failed) {
                                                    "x"
                                                } else {
                                                    "clock"
                                                }),
                                            )
                                            .width(ui.available_width())
                                            .height(38.),
                                        )
                                        .clicked()
                                    {
                                        self.open_checks(mobile);
                                    }
                                    ui.label(
                                        RichText::new(if t.approved() {
                                            "Approved by you"
                                        } else {
                                            "Review required"
                                        })
                                        .size(12.)
                                        .color(tokens.text2),
                                    );
                                    ui.separator();
                                    ui.add_space(10.);
                                    ui.horizontal_wrapped(|ui| {
                                        if self
                                            .modal_control(
                                                ui,
                                                rt,
                                                Control::new(
                                                    "ws-approve",
                                                    if t.approved() {
                                                        "Revoke approval"
                                                    } else {
                                                        "Approve changes"
                                                    },
                                                    Some(if t.approved() {
                                                        "check"
                                                    } else {
                                                        "user"
                                                    }),
                                                )
                                                .enabled(!t.merged && !t.running)
                                                .height(34.),
                                            )
                                            .clicked()
                                        {
                                            self.approve();
                                        }
                                        let enabled = self.current().can_merge();
                                        let r = self.modal_control(
                                            ui,
                                            rt,
                                            Control::new(
                                                "ws-merge",
                                                if t.merged {
                                                    "Merged"
                                                } else {
                                                    "Merge pull request"
                                                },
                                                Some("git-merge"),
                                            )
                                            .enabled(enabled)
                                            .height(34.),
                                        );
                                        if r.clicked() {
                                            self.open_dialog(Dialog::ConfirmMerge, Some(r.id));
                                        }
                                    });
                                }
                            });
                        self.modal_focus(ui, dialog.initial, Some(Id::new("ws-review-close")));
                    }
                    Dialog::Sessions => {
                        let threads = self.threads.clone();
                        egui::ScrollArea::vertical()
                            .id_salt("ws-thread-list")
                            .max_height(max_height - 130.)
                            .show(ui, |ui| {
                                for t in threads {
                                    let id = format!("ws-thread-{}", t.id);
                                    let label = format!(
                                        "{}   {}",
                                        t.title,
                                        if t.merged {
                                            "Merged".into()
                                        } else {
                                            format!("#{}", t.number)
                                        }
                                    );
                                    let r = self.modal_control(
                                        ui,
                                        rt,
                                        Control::new(
                                            &id,
                                            &label,
                                            Some(if t.merged { "git-merge" } else { "git-branch" }),
                                        )
                                        .selected(t.id == self.selected)
                                        .width(ui.available_width())
                                        .height(34.),
                                    );
                                    ui.label(
                                        RichText::new(&t.branch)
                                            .monospace()
                                            .size(10.)
                                            .color(tokens.text3),
                                    );
                                    ui.add_space(4.);
                                    if r.clicked() {
                                        let same = t.id == self.selected;
                                        self.select_thread(&t.id);
                                        self.close_dialog(same);
                                        if !same {
                                            self.pending_focus = Some(Id::new(match self.view {
                                                View::Editor => "ws-editor-mode-source",
                                                View::Chat => "ws-prompt-input",
                                                View::Checks => "ws-check-tests",
                                            }));
                                        }
                                    }
                                }
                            });
                        if self
                            .modal_control(
                                ui,
                                rt,
                                Control::new("ws-nav-new", "New session", Some("plus"))
                                    .width(ui.available_width())
                                    .height(34.),
                            )
                            .clicked()
                        {
                            self.add_thread();
                        }
                        self.modal_focus(
                            ui,
                            dialog.initial,
                            Some(Id::new(format!("ws-thread-{}", self.selected))),
                        );
                    }
                    Dialog::Files => {
                        egui::ScrollArea::vertical()
                            .id_salt("ws-mobile-files")
                            .max_height(max_height - 140.)
                            .show(ui, |ui| self.files(ui, rt, true));
                        let r = self.modal_control(
                            ui,
                            rt,
                            Control::new(
                                "ws-files-sessions",
                                "Switch coding session",
                                Some("git-branch"),
                            )
                            .width(ui.available_width())
                            .height(34.),
                        );
                        if r.clicked() {
                            self.close_dialog(false);
                            self.open_dialog(Dialog::Sessions, Some(Id::new("ws-sidebar")));
                        }
                        self.modal_focus(ui, dialog.initial, Some(Id::new("ws-mobile-file-0")));
                    }
                    Dialog::Rename => {
                        let submit = ui.memory(|m| m.has_focus(Id::new("ws-name")))
                            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
                        ui.label(RichText::new("Session name").size(12.).color(tokens.text2));
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.rename)
                                .id(Id::new("ws-name"))
                                .desired_width(f32::INFINITY)
                                .char_limit(60),
                        );
                        self.modal_controls.push(r.id);
                        rt.record("ws-name", &r);
                        self.track_focus(&r);
                        self.controls.insert("ws-name".into(), r.rect);
                        ui.add_space(10.);
                        let mut save = submit;
                        ui.horizontal(|ui| {
                            if self
                                .modal_control(
                                    ui,
                                    rt,
                                    Control::new("ws-name-cancel", "Cancel", None).height(34.),
                                )
                                .clicked()
                            {
                                self.close_dialog(true);
                            }
                            if self
                                .modal_control(
                                    ui,
                                    rt,
                                    Control::new("ws-name-save", "Save", Some("check"))
                                        .enabled(!self.rename.trim().is_empty())
                                        .height(34.),
                                )
                                .clicked()
                            {
                                save = true;
                            }
                        });
                        if save && !self.rename.trim().is_empty() {
                            self.current_mut().title = self.rename.trim().into();
                            self.close_dialog(true);
                        }
                        if dialog.initial {
                            let id = Id::new("ws-name");
                            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id)
                                .unwrap_or_default();
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::two(
                                    egui::text::CCursor::new(0),
                                    egui::text::CCursor::new(self.rename.chars().count()),
                                )));
                            state.store(ui.ctx(), id);
                        }
                        self.modal_focus(ui, dialog.initial, Some(Id::new("ws-name")));
                    }
                    Dialog::ConfirmMerge => {
                        let t = self.current().clone();
                        ui.label(
                            RichText::new(format!("#{} {}", t.number, t.title))
                                .strong()
                                .size(14.)
                                .color(tokens.text),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!("{}  →  main", t.branch))
                                    .monospace()
                                    .size(11.)
                                    .color(tokens.text3),
                            )
                            .wrap(),
                        );
                        ui.label(
                            RichText::new("3 checks passed · Approved by you")
                                .size(12.)
                                .color(tokens.text2),
                        );
                        ui.add_space(12.);
                        ui.horizontal_wrapped(|ui| {
                            if self
                                .modal_control(
                                    ui,
                                    rt,
                                    Control::new("ws-cancel-merge", "Cancel", None).height(34.),
                                )
                                .clicked()
                            {
                                self.close_dialog(true);
                            }
                            if self
                                .modal_control(
                                    ui,
                                    rt,
                                    Control::new("ws-confirm-merge", "Merge", Some("git-merge"))
                                        .enabled(t.can_merge())
                                        .height(34.),
                                )
                                .clicked()
                                && self.merge()
                            {
                                self.close_dialog(false);
                                self.pending_focus = Some(Id::new("ws-review-close"));
                            }
                        });
                        self.modal_focus(ui, dialog.initial, Some(Id::new("ws-cancel-merge")));
                    }
                }
            });
        rt.anchor("ws-dialog", "Workspace dialog", modal.response.rect);
        let ready = self
            .modal_controls
            .iter()
            .any(|id| self.focus_available(*id));
        if let Some(current) = self.dialogs.last_mut()
            && current.kind == dialog.kind
            && ready
        {
            current.initial = false;
        }
        if !ready {
            rt.repaint();
        }
    }
    fn statusbar(&mut self, ui: &mut Ui, rt: &mut Runtime, rect: Rect, mobile: bool) {
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface2);
        separator(ui, rect.left_top(), rect.right_top(), tokens.border_strong);
        let mut bar = child(ui, rect.shrink2(Vec2::new(8., 0.)), "ws-statusbar");
        let t = self.current().clone();
        bar.horizontal(|ui| {
            if !mobile {
                ui.label(
                    RichText::new(if t.merged { "main" } else { &t.branch })
                        .monospace()
                        .size(10.)
                        .color(tokens.text3),
                );
            }
            let label = if self.stopped_status {
                "Stopped"
            } else {
                t.summary()
            };
            if self
                .control(
                    ui,
                    rt,
                    Control::new("ws-status", label, Some("terminal")).height(24.),
                )
                .clicked()
            {
                self.open_checks(mobile);
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if !mobile {
                    ui.label(RichText::new("Local changes").size(10.).color(tokens.text3));
                }
                ui.label(
                    RichText::new(format!("Ln {}, Col {}", self.cursor.0, self.cursor.1))
                        .monospace()
                        .size(10.)
                        .color(tokens.text3),
                );
            });
        });
    }
}
impl Page for State {
    fn enter(&mut self, _rt: &mut Runtime) {
        self.active = true;
    }
    fn leave(&mut self, rt: &mut Runtime) {
        self.clear_owned_focus(rt.context());
        self.active = false;
        self.stop(false);
        self.dialogs.clear();
        self.modal_controls.clear();
        self.pending_focus = None;
        self.restore_focus = None;
        self.focus_retry = 0;
        self.focus_targets.clear();
        self.drawn_dialog = None;
        self.editor_transition = None;
        rt.context()
            .data_mut(|d| d.remove::<KeyboardCapture>(Id::new("ws-keyboard-capture")));
    }
    fn show(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let restore_focus = self.restore_focus.take();
        self.controls.clear();
        self.focus_targets.clear();
        self.drawn_dialog = None;
        let hidden = ui.ctx().input(|i| i.viewport().minimized.unwrap_or(false));
        if hidden {
            self.clear_owned_focus(ui.ctx());
            self.pending_focus = None;
            self.restore_focus = None;
            self.focus_retry = 0;
            self.stop(false);
            self.editor_transition = None;
            ui.ctx()
                .data_mut(|d| d.remove::<KeyboardCapture>(Id::new("ws-keyboard-capture")));
            return;
        }
        self.tick(rt.now);
        let size = ui.available_size();
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let tokens = rt.tokens.clone();
        ui.painter().rect_filled(rect, 0., tokens.surface);
        let mobile = rect.width() <= 760.;
        let header = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 40.));
        self.header(ui, rt, header, mobile);
        let mut top = header.bottom();
        if mobile {
            let views =
                Rect::from_min_size(Pos2::new(rect.left(), top), Vec2::new(rect.width(), 40.));
            self.mobile_views(ui, rt, views);
            top = views.bottom();
        }
        let footer = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - 24.), rect.max);
        let body = Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), footer.top()),
        );
        if mobile {
            match self.view {
                View::Editor => self.editor(ui, rt, body),
                View::Chat => self.chat(ui, rt, body, true),
                View::Checks => self.console(ui, rt, body, true),
            }
        } else {
            let narrow = rect.width() <= 1100.;
            let activity_width = if narrow { 40. } else { 48. };
            let activity = Rect::from_min_max(
                body.min,
                Pos2::new(body.left() + activity_width, body.bottom()),
            );
            self.activity(ui, rt, activity);
            let mut left = activity.right();
            if self.explorer_open {
                let width = if narrow { 168. } else { 204. };
                let explorer = Rect::from_min_max(
                    Pos2::new(left, body.top()),
                    Pos2::new(left + width, body.bottom()),
                );
                self.explorer(ui, rt, explorer);
                left += width;
            }
            let mut right = body.right();
            if self.chat_open {
                let width = if narrow { 280. } else { 324. };
                right -= width;
                let chat = Rect::from_min_max(Pos2::new(right, body.top()), body.max);
                self.chat(ui, rt, chat, false);
            }
            let center = Rect::from_min_max(
                Pos2::new(left, body.top()),
                Pos2::new(right.max(left + 1.), body.bottom()),
            );
            let console_fraction = rt.animate(
                "ws-console-open",
                if self.console_open { 1. } else { 0. },
                0.32,
                Curve::InOut,
            );
            let console_height = (rt.context().content_rect().height() * 0.24)
                .clamp(188., 224.)
                .min((body.height() - 180.).max(0.))
                * console_fraction;
            let editor = Rect::from_min_max(
                center.min,
                Pos2::new(center.right(), center.bottom() - console_height),
            );
            self.editor(ui, rt, editor);
            if console_height > 0.5 {
                let console =
                    Rect::from_min_max(Pos2::new(center.left(), editor.bottom()), center.max);
                let mut clipped = child(ui, console, "ws-console-wrap");
                let full = Rect::from_min_max(
                    console.min,
                    Pos2::new(
                        console.right(),
                        console.top()
                            + if console_fraction > 0. {
                                console_height / console_fraction
                            } else {
                                0.
                            },
                    ),
                );
                if !self.console_open {
                    clipped.disable();
                    clipped.set_opacity(1.);
                }
                self.console(&mut clipped, rt, full, false);
            }
            rt.anchor("ws-center", "Code editor", center);
        }
        self.statusbar(ui, rt, footer, mobile);
        let ctx = ui.ctx().clone();
        self.dialogs(&ctx, rt, mobile);
        self.settle_focus(&ctx, rt, restore_focus);
        if self.restore_focus.is_some() {
            rt.repaint();
        }
        let t = self.current();
        ctx.data_mut(|d| {
            d.insert_temp(
                Id::new("ws-keyboard-capture"),
                KeyboardCapture {
                    editor: t.editing.then(|| Id::new(("ws-code-input", &t.id, t.file))),
                    dialog: !self.dialogs.is_empty(),
                },
            )
        });
        if let Some(job) = &self.job {
            rt.repaint_after((job.due - rt.now).max(0.001));
        }
        rt.anchor("workspace", "Coding workspace", rect);
    }
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::runtime::{MotionPreference, Preferences};
    fn run(s: &mut State, text: &str, now: f64) {
        assert!(s.send(text, now));
        s.tick(now + 20.);
        assert!(s.job.is_none());
        assert!(!s.current().running);
    }
    fn repaired_source(s: &State) -> String {
        s.current().files[0].after.join("\n").replace(
            "    openSearch();",
            "    event.preventDefault();\n    openSearch();",
        )
    }
    #[test]
    fn fixtures_keep_substantive_sources_and_initial_failure() {
        let s = State::default();
        assert_eq!(s.threads.len(), 3);
        assert_eq!(s.current().files[0].after.len(), 63);
        assert!(!s.current().repaired());
        assert_eq!(
            s.current().checks,
            [Status::Passed, Status::Failed, Status::Passed]
        );
        assert!(!s.current().can_merge());
        assert!(
            s.log_lines()
                .join("\n")
                .contains("src/search.test.ts:31:36")
        );
    }
    #[test]
    fn rerunning_does_not_repair_source() {
        let mut s = State::default();
        let source = s.current().files[0].after.clone();
        run(&mut s, "Run checks", 0.);
        assert_eq!(s.current().files[0].after, source);
        assert_eq!(
            s.current().checks,
            [Status::Passed, Status::Failed, Status::Passed]
        );
        assert!(s.log_lines().join("\n").contains("defaultPrevented"));
    }
    #[test]
    fn manual_repair_passes_and_regression_revokes_revision_approval() {
        let mut s = State::default();
        s.approve();
        let repaired = repaired_source(&s);
        assert!(s.edit_buffer(0, &repaired));
        assert_eq!(s.current().checks, [Status::Stale; 3]);
        assert!(!s.current().approved());
        assert!(s.current().repaired());
        run(&mut s, "Run checks", 0.);
        assert!(s.current().passed());
        assert!(!s.current().can_merge());
        s.approve();
        assert!(s.current().can_merge());
        let revision = s.current().revision;
        run(&mut s, "Rerun checks", 30.);
        assert_eq!(s.current().revision, revision);
        assert!(s.current().approved());
        assert!(s.current().can_merge());
        let broken = s.current().files[0]
            .after
            .join("\n")
            .replace("    event.preventDefault();\n", "");
        assert!(s.edit_buffer(0, &broken));
        assert_eq!(s.current().revision, revision + 1);
        assert_eq!(s.current().checks, [Status::Stale; 3]);
        assert!(!s.current().approved());
        assert!(!s.current().can_merge());
        run(&mut s, "Run checks", 60.);
        assert_eq!(s.current().checks[1], Status::Failed);
        assert!(!s.current().can_merge());
    }
    #[test]
    fn assistant_repair_changes_only_modified_buffer_and_lcs_diff() {
        let mut s = State::default();
        let before = s.current().files[0].before.clone();
        run(&mut s, "Fix the failed check", 0.);
        assert!(s.current().repaired());
        assert!(s.current().passed());
        assert_eq!(s.current().files[0].before, before);
        let diff = diff_lines(&s.current().files[0]);
        let added = diff
            .iter()
            .filter(|l| l.text.contains("event.preventDefault();"))
            .collect::<Vec<_>>();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].kind, "added");
        assert!(added[0].old.is_none());
        assert!(added[0].next.is_some());
    }
    #[test]
    fn incomplete_source_patch_still_fails_source_assertion() {
        let mut s = State::default();
        let bad = SEARCH_SOURCE.replace(
            "if (!isShortcut || event.repeat) return;",
            "if (!isShortcut) return;",
        );
        s.edit_buffer(0, &bad);
        run(&mut s, "Fix failed check", 0.);
        assert!(
            s.current().files[0]
                .after
                .join("\n")
                .contains("event.preventDefault();")
        );
        assert_eq!(s.current().checks[1], Status::Failed);
        assert!(!s.current().repaired());
    }
    #[test]
    fn streaming_and_ci_use_real_elapsed_time() {
        let mut s = State::default();
        assert!(s.send("Run checks", 100.));
        s.tick(100.059);
        assert_eq!(s.current().messages.last().unwrap().text, "");
        s.tick(100.060);
        assert_eq!(
            s.current().messages.last().unwrap().text.chars().count(),
            12
        );
        assert_eq!(s.current().checks, [Status::Queued; 3]);
        s.tick(100.084);
        assert_eq!(
            s.current().messages.last().unwrap().text.chars().count(),
            24
        );
        let mut rt = Runtime::new(egui::Context::default(), Preferences::default());
        rt.set_motion(MotionPreference::Reduced);
        rt.now = 100.1;
        s.tick(rt.now);
        assert!(s.current().running);
        rt.set_motion(MotionPreference::Slow);
        rt.now = 103.;
        s.tick(rt.now);
        assert!(!s.current().running);
    }
    #[test]
    fn stop_cancels_checks_and_never_completes_later() {
        for now in [0.1, 0.5, 0.8, 1.4] {
            let mut s = State::default();
            s.send("Run checks", 0.);
            s.tick(now);
            s.stop(true);
            assert!(s.job.is_none());
            assert!(!s.current().running);
            assert!(s.stopped_status);
            assert!(s.current().checks.contains(&Status::Cancelled));
            let checks = s.current().checks;
            s.tick(1000.);
            assert_eq!(s.current().checks, checks);
            assert!(s.current().messages.last().unwrap().stopped);
        }
    }
    #[test]
    fn planning_and_cancelled_planning_preserve_approval_and_ci() {
        let mut s = State::default();
        s.select_thread("sorting");
        s.approve();
        let rev = s.current().revision;
        run(&mut s, "Explain these changes", 0.);
        assert_eq!(s.current().revision, rev);
        assert!(s.current().approved());
        assert!(s.current().can_merge());
        s.mode_plan = true;
        s.send("Change a file", 40.);
        s.tick(40.1);
        s.stop(false);
        assert!(s.current().approved());
        assert!(s.current().passed());
        assert!(s.current().can_merge());
    }
    #[test]
    fn sessions_retain_drafts_files_buffers_and_independent_approval() {
        let mut s = State::default();
        s.current_mut().draft = "Keep this draft".into();
        s.current_mut().file = 1;
        s.approve();
        s.send("Run checks", 0.);
        s.tick(0.1);
        s.select_thread("sorting");
        assert_eq!(s.current().files[0].path, "src/task-list.ts");
        assert!(!s.current().approved());
        s.current_mut().draft = "Sorting draft".into();
        s.select_thread("search");
        // Sending clears the draft, as in the reference; independent later drafts survive switching.
        assert_eq!(s.current().file, 1);
        assert!(s.current().checks.contains(&Status::Cancelled));
        s.current_mut().draft = "Another draft".into();
        s.select_thread("sorting");
        assert_eq!(s.current().draft, "Sorting draft");
        s.select_thread("search");
        assert_eq!(s.current().draft, "Another draft");
    }
    #[test]
    fn merge_requires_latest_checks_approval_and_idle_and_freezes_source() {
        let mut s = State::default();
        s.approve();
        assert!(!s.merge());
        run(&mut s, "Fix failed check", 0.);
        assert!(!s.merge());
        s.approve();
        assert!(s.current().can_merge());
        s.send("Run checks", 30.);
        assert!(!s.merge());
        s.tick(40.);
        assert!(s.current().can_merge());
        assert!(s.merge());
        assert!(!s.merge());
        assert!(!s.send("Fix again", 50.));
        assert!(!s.edit_buffer(0, "changed"));
        assert!(
            s.current()
                .events
                .iter()
                .any(|e| e.text.contains("Merged #128"))
        );
        assert!(
            s.current()
                .messages
                .last()
                .unwrap()
                .text
                .contains("Merged #128")
        );
    }
    #[test]
    fn new_session_draft_and_reset_are_independent() {
        let mut s = State::default();
        s.add_thread();
        assert_eq!(s.current().id, "new-1");
        assert!(s.current().draft_pr);
        assert_eq!(s.current().checks, [Status::Stale; 3]);
        assert!(!s.current().can_merge());
        run(&mut s, "Fix shortcut", 0.);
        assert_eq!(s.current().title, "Fix shortcut");
        assert!(!s.current().draft_pr);
        assert!(s.current().passed());
        s.reset();
        assert_eq!(s.current().id, "search");
        assert!(!s.current().repaired());
        assert_eq!(s.threads.len(), 3);
    }
    #[test]
    fn split_diff_pairs_replacements_and_context_without_fake_lines() {
        let file = patch("a", "a\nb\nc\nd", "a\nx\ny\nc\nd");
        let rows = split_rows(&file);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0].as_ref().unwrap().kind, "context");
        assert_eq!(rows[1][0].as_ref().unwrap().text, "b");
        assert_eq!(rows[1][1].as_ref().unwrap().text, "x");
        assert!(rows[2][0].is_none());
        assert_eq!(rows[2][1].as_ref().unwrap().text, "y");
        assert_eq!(rows.last().unwrap()[1].as_ref().unwrap().next, Some(5));
    }
    #[test]
    fn indentation_and_highlighting_handle_unicode_and_literal_markup() {
        let mut source = "λx\n    openSearch();".to_owned();
        assert_eq!(indent_buffer(&mut source, 3..3, false), 5..5);
        assert!(source.starts_with("λx\n      openSearch"));
        assert_eq!(indent_buffer(&mut source, 9..9, true), 7..7);
        assert!(source.starts_with("λx\n    openSearch"));
        let sample =
            "// <script> remains text\nconst message = 'Grünwald \\'λ';\n/* multi\nline */";
        let tokens = tokenize(sample);
        let reconstructed = tokens
            .iter()
            .map(|(r, _)| &sample[r.clone()])
            .collect::<String>();
        assert_eq!(reconstructed, sample);
        assert!(tokens.iter().any(|(_, k)| *k == TokenKind::Comment));
        assert!(tokens.iter().any(|(_, k)| *k == TokenKind::Keyword));
        assert!(tokens.iter().any(|(_, k)| *k == TokenKind::String));
    }
    #[test]
    fn leave_cancels_work_dialogs_and_pending_focus_without_losing_buffers() {
        let mut s = State::default();
        let source = repaired_source(&s);
        s.edit_buffer(0, &source);
        s.send("Run checks", 0.);
        s.tick(0.1);
        s.open_dialog(Dialog::Review, Some(Id::new("ws-review-toggle")));
        let mut rt = Runtime::new(egui::Context::default(), Preferences::default());
        s.leave(&mut rt);
        assert!(s.job.is_none());
        assert!(s.dialogs.is_empty());
        assert!(!s.active);
        assert!(s.current().repaired());
        assert!(s.current().checks.contains(&Status::Cancelled));
        s.enter(&mut rt);
        s.tick(100.);
        assert!(s.current().checks.contains(&Status::Cancelled));
        assert!(s.active);
    }
    fn native_frame(
        s: &mut State,
        ctx: &egui::Context,
        rt: &mut Runtime,
        size: Vec2,
        time: f64,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.enable_accesskit();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| {
            rt.begin_frame();
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| s.show(ui, rt));
        });
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native accessibility tree");
        assert!(
            update.nodes.iter().any(|(id, _)| *id == update.focus),
            "workspace focus {:?} is absent from this frame's node list at {time}; dialogs={:?}, view={:?}",
            update.focus,
            s.dialogs.iter().map(|d| d.kind).collect::<Vec<_>>(),
            s.view
        );
        output
    }
    #[test]
    fn native_responsive_layout_contains_controls_at_desktop_and_mobile_widths() {
        for width in [320., 390., 760., 1100., 1440.] {
            let ctx = egui::Context::default();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            rt.set_motion(MotionPreference::Reduced);
            let mut s = State::default();
            s.enter(&mut rt);
            let output = native_frame(&mut s, &ctx, &mut rt, Vec2::new(width, 844.), 0., vec![]);
            assert!(!output.shapes.is_empty());
            assert!(s.controls.contains_key("ws-edit"));
            for id in [
                "ws-review-toggle",
                "ws-edit",
                "ws-editor-mode-source",
                "ws-editor-mode-diff",
            ] {
                let r = s.controls[id];
                assert!(
                    r.left() >= -0.5 && r.right() <= width + 0.5,
                    "{width}: {id}: {r:?}"
                );
            }
            if width <= 760. {
                assert!(!s.controls.contains_key("ws-prompt-input"));
                s.view = View::Chat;
                native_frame(&mut s, &ctx, &mut rt, Vec2::new(width, 844.), 0.1, vec![]);
                assert!(s.controls.contains_key("ws-prompt-input"));
                assert!(!s.controls.contains_key("ws-edit"));
            }
        }
    }

    fn native_tap(
        s: &mut State,
        ctx: &egui::Context,
        rt: &mut Runtime,
        size: Vec2,
        time: f64,
        id: &str,
    ) {
        let pos = s
            .controls
            .get(id)
            .unwrap_or_else(|| panic!("missing {id}"))
            .center();
        native_frame(
            s,
            ctx,
            rt,
            size,
            time,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        native_frame(
            s,
            ctx,
            rt,
            size,
            time + 0.01,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        native_frame(s, ctx, rt, size, time + 0.02, vec![]);
        // A newly-created native modal first measures its Area in an invisible
        // sizing pass; validate that frame too, then allow its visible pass.
        native_frame(s, ctx, rt, size, time + 0.03, vec![]);
    }
    fn native_key(
        s: &mut State,
        ctx: &egui::Context,
        rt: &mut Runtime,
        size: Vec2,
        time: f64,
        key: Key,
        modifiers: egui::Modifiers,
    ) {
        native_frame(
            s,
            ctx,
            rt,
            size,
            time,
            vec![
                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                },
            ],
        );
        native_frame(s, ctx, rt, size, time + 0.01, vec![]);
    }
    #[test]
    fn native_editor_input_manually_repairs_and_finishes_with_keyboard_focus() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-edit");
        assert!(s.current().editing);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new(("ws-code-input", "search", 0usize)))
        );
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            vec![egui::Event::Text("    event.preventDefault();\n".into())],
        );
        assert!(s.current().repaired());
        assert_eq!(s.current().checks, [Status::Stale; 3]);
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::S,
            egui::Modifiers::CTRL,
        );
        assert!(!s.current().editing);
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-edit")));
        native_tap(&mut s, &ctx, &mut rt, size, 0.4, "ws-run-checks");
        native_frame(&mut s, &ctx, &mut rt, size, 4., vec![]);
        assert!(s.current().passed());
    }
    #[test]
    fn native_review_approval_focus_wrap_merge_cancel_and_confirmation() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-review-toggle");
        assert_eq!(s.dialogs.last().unwrap().kind, Dialog::Review);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-review-close"))
        );
        assert!(
            !rt.records
                .iter()
                .find(|r| r.id == "ws-merge")
                .unwrap()
                .enabled
        );
        native_tap(&mut s, &ctx, &mut rt, size, 0.2, "ws-approve");
        assert!(s.current().approved());
        assert!(!s.current().can_merge());
        ctx.memory_mut(|m| m.request_focus(Id::new("ws-approve")));
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::Tab,
            egui::Modifiers::NONE,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-review-close"))
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.4,
            Key::Escape,
            egui::Modifiers::NONE,
        );
        assert!(s.dialogs.is_empty());
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-review-toggle"))
        );
        run(&mut s, "Fix failed check", 1.);
        s.approve();
        native_frame(&mut s, &ctx, &mut rt, size, 22., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 22.1, "ws-review-toggle");
        native_tap(&mut s, &ctx, &mut rt, size, 22.2, "ws-merge");
        assert_eq!(s.dialogs.last().unwrap().kind, Dialog::ConfirmMerge);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-cancel-merge"))
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            22.3,
            Key::Tab,
            egui::Modifiers::SHIFT,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-confirm-merge"))
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            22.4,
            Key::Escape,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.dialogs.last().unwrap().kind, Dialog::Review);
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-merge")));
        assert!(!s.current().merged);
        native_tap(&mut s, &ctx, &mut rt, size, 22.5, "ws-merge");
        native_tap(&mut s, &ctx, &mut rt, size, 22.6, "ws-confirm-merge");
        assert!(s.current().merged);
    }
    #[test]
    fn native_mobile_view_keyboard_files_and_rename_keep_drafts() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(390., 844.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        ctx.memory_mut(|m| m.request_focus(Id::new("ws-view-editor")));
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.1,
            Key::ArrowRight,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.view, View::Chat);
        native_tap(&mut s, &ctx, &mut rt, size, 0.2, "ws-prompt-input");
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            vec![egui::Event::Text("Mobile draft".into())],
        );
        assert_eq!(s.current().draft, "Mobile draft");
        native_tap(&mut s, &ctx, &mut rt, size, 0.4, "ws-rename");
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.5,
            vec![egui::Event::Text("Keyboard repair".into())],
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.6,
            Key::Enter,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.current().title, "Keyboard repair");
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-rename")));
        native_tap(&mut s, &ctx, &mut rt, size, 0.7, "ws-view-editor");
        native_tap(&mut s, &ctx, &mut rt, size, 0.8, "ws-sidebar");
        native_tap(&mut s, &ctx, &mut rt, size, 0.9, "ws-mobile-file-1");
        assert_eq!(s.current().file, 1);
        assert!(s.dialogs.is_empty());
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-file-1")));
        native_tap(&mut s, &ctx, &mut rt, size, 1., "ws-view-chat");
        assert_eq!(s.current().draft, "Mobile draft");
    }

    #[test]
    fn native_diff_panes_share_vertical_scroll_and_keep_independent_horizontal_offsets() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        rt.set_motion(MotionPreference::Reduced);
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        s.set_editor_mode(true, 0.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        let old = s.controls["ws-diff-old"];
        let new = s.controls["ws-diff-new"];
        assert!((old.width() - new.width()).abs() < 1.);
        ctx.memory_mut(|m| m.request_focus(Id::new("ws-diff-new")));
        native_frame(&mut s, &ctx, &mut rt, size, 0.1, vec![]);
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            Key::PageDown,
            egui::Modifiers::NONE,
        );
        assert!(s.diff_scroll > 100.);
        let vertical = s.diff_scroll;
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::ArrowRight,
            egui::Modifiers::NONE,
        );
        assert!(s.diff_horizontal[1] > 0.);
        assert_eq!(s.diff_horizontal[0], 0.);
        assert_eq!(s.diff_scroll, vertical);
        ctx.memory_mut(|m| m.request_focus(Id::new("ws-diff-old")));
        native_frame(&mut s, &ctx, &mut rt, size, 0.4, vec![]);
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.5,
            Key::Home,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.diff_scroll, 0.);
    }
    #[test]
    fn native_console_transition_reverses_from_current_geometry_and_reduced_motion_settles() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        rt.set_motion(MotionPreference::Full);
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        let original = s.controls["ws-source"].height();
        s.console_open = false;
        native_frame(&mut s, &ctx, &mut rt, size, 0.1, vec![]);
        native_frame(&mut s, &ctx, &mut rt, size, 0.22, vec![]);
        let partial = s.controls["ws-source"].height();
        assert!(partial > original + 1. && partial < original + 279.);
        s.console_open = true;
        native_frame(&mut s, &ctx, &mut rt, size, 0.23, vec![]);
        let reversed = s.controls["ws-source"].height();
        assert!((reversed - partial).abs() < 35.);
        native_frame(&mut s, &ctx, &mut rt, size, 0.7, vec![]);
        assert!((s.controls["ws-source"].height() - original).abs() < 1.);
        s.choose_file(1, 0.7);
        native_frame(&mut s, &ctx, &mut rt, size, 0.75, vec![]);
        assert!(s.editor_transition.is_some());
        rt.set_motion(MotionPreference::Reduced);
        s.console_open = false;
        native_frame(&mut s, &ctx, &mut rt, size, 0.76, vec![]);
        assert!(s.editor_transition.is_none());
        assert!(!s.controls.contains_key("ws-run-checks"));
    }
    #[test]
    fn native_composer_shift_enter_inserts_line_while_enter_sends() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.view = View::Chat;
        let size = Vec2::new(390., 844.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-prompt-input");
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            vec![egui::Event::Text("Explain".into())],
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::Enter,
            egui::Modifiers::SHIFT,
        );
        assert!(s.job.is_none());
        assert!(s.current().draft.contains('\n'));
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.4,
            vec![egui::Event::Text("these changes".into())],
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.5,
            Key::Enter,
            egui::Modifiers::NONE,
        );
        assert!(s.job.is_some());
        assert_eq!(s.job.as_ref().unwrap().kind, JobKind::Plan);
        assert!(s.current().draft.is_empty());
    }
    #[test]
    fn native_file_tabs_and_reverse_dialog_focus_use_selected_controls() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-file-1");
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            Key::ArrowRight,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.current().file, 2);
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-file-2")));
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::Home,
            egui::Modifiers::NONE,
        );
        assert_eq!(s.current().file, 0);
        native_tap(&mut s, &ctx, &mut rt, size, 0.4, "ws-review-toggle");
        assert!(captures_keyboard(&ctx));
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.5,
            Key::Tab,
            egui::Modifiers::SHIFT,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-approve")));
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.6,
            Key::Escape,
            egui::Modifiers::NONE,
        );
        assert!(!captures_keyboard(&ctx));
    }
    #[test]
    fn minimized_viewport_cancels_without_advancing_queued_work() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.send("Run checks", 0.);
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 1000.))),
            time: Some(5.),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .minimized = Some(true);
        let _ = ctx.run(input, |ctx| {
            rt.begin_frame();
            egui::CentralPanel::default().show(ctx, |ui| s.show(ui, &mut rt));
        });
        assert!(s.job.is_none());
        assert_eq!(s.current().checks, [Status::Cancelled; 3]);
        assert!(!captures_keyboard(&ctx));
    }

    #[test]
    fn timezone_transition_lookup_uses_offsets_and_rejects_malformed_tables() {
        let mut tz = vec![0u8; 44];
        tz[..4].copy_from_slice(b"TZif");
        tz[32..36].copy_from_slice(&2u32.to_be_bytes());
        tz[36..40].copy_from_slice(&2u32.to_be_bytes());
        tz[40..44].copy_from_slice(&9u32.to_be_bytes());
        tz.extend_from_slice(&0i32.to_be_bytes());
        tz.extend_from_slice(&3600i32.to_be_bytes());
        tz.extend_from_slice(&[0, 1]);
        tz.extend_from_slice(&3600i32.to_be_bytes());
        tz.extend_from_slice(&[0, 0]);
        tz.extend_from_slice(&7200i32.to_be_bytes());
        tz.extend_from_slice(&[1, 4]);
        tz.extend_from_slice(b"CET\0CEST\0");
        assert_eq!(timezone_offset(&tz, 100), Some(3600));
        assert_eq!(timezone_offset(&tz, 3600), Some(7200));
        assert_eq!(timezone_offset(b"not a timezone", 0), None);
        assert_eq!(timezone_offset(&tz[..45], 100), None);
        if let Ok(host) = std::fs::read("/etc/localtime") {
            assert!(timezone_offset(&host, 1_790_000_000).is_some());
        }
    }
    #[test]
    fn native_ime_composition_does_not_send_the_composer_message() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.view = View::Chat;
        let size = Vec2::new(390., 844.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-prompt-input");
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            vec![
                egui::Event::Ime(egui::ImeEvent::Enabled),
                egui::Event::Ime(egui::ImeEvent::Preedit("検索".into())),
            ],
        );
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.3,
            Key::Enter,
            egui::Modifiers::NONE,
        );
        assert!(s.job.is_none());
        native_frame(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.4,
            vec![
                egui::Event::Ime(egui::ImeEvent::Commit("検索".into())),
                egui::Event::Ime(egui::ImeEvent::Disabled),
            ],
        );
        assert!(s.job.is_none());
    }

    fn ax_node<'a>(output: &'a egui::FullOutput, id: &str) -> &'a egui::accesskit::Node {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native accessibility tree")
            .nodes
            .iter()
            .find(|(node_id, _)| node_id.0 == Id::new(id).value())
            .map(|(_, node)| node)
            .unwrap_or_else(|| panic!("missing accessibility node {id}"))
    }
    fn assert_action_node(output: &egui::FullOutput, id: &str, label: &str) {
        let node = ax_node(output, id);
        assert_eq!(node.role(), egui::accesskit::Role::Button, "{id}");
        assert_eq!(node.label(), Some(label), "{id}");
        assert_eq!(
            node.toggled(),
            None,
            "action {id} must not announce toggle off"
        );
        assert_eq!(
            node.is_selected(),
            None,
            "action {id} must not announce selection"
        );
    }
    #[test]
    fn native_accessibility_actions_are_named_buttons_and_real_tabs_are_selected_tabs() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        let output = native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        for (id, label) in [
            ("ws-edit", "Edit"),
            ("ws-run-checks", "Rerun all"),
            ("ws-review-toggle", "Pull request #128"),
            ("ws-reset", "Reset local demo"),
            ("ws-rename", "Rename coding session"),
            ("ws-new", "New session"),
            ("ws-sessions", "Switch coding session"),
            ("ws-send", "Send"),
            ("ws-console-close", "Collapse CI console"),
        ] {
            assert_action_node(&output, id, label);
        }
        for (id, selected) in [("ws-file-0", true), ("ws-file-1", false)] {
            let node = ax_node(&output, id);
            assert_eq!(node.role(), egui::accesskit::Role::Tab);
            assert_eq!(node.is_selected(), Some(selected));
            assert_eq!(node.toggled(), None);
        }
        assert_eq!(
            ax_node(&output, "ws-check-tests").toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert_eq!(
            ax_node(&output, "ws-mode-build").toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert_eq!(
            ax_node(&output, "ws-mode-plan").toggled(),
            Some(egui::accesskit::Toggled::False)
        );
    }
    #[test]
    fn native_accessibility_all_dialog_close_controls_have_literal_action_names() {
        for (kind, id, label) in [
            (Dialog::Review, "ws-review-close", "Close pull request"),
            (Dialog::Sessions, "ws-nav-close", "Close sessions"),
            (Dialog::Files, "ws-files-close", "Close project files"),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            let mut s = State::default();
            let size = Vec2::new(1440., 1000.);
            s.open_dialog(kind, Some(Id::new("ws-review-toggle")));
            let _ = native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
            let output = native_frame(&mut s, &ctx, &mut rt, size, 0.01, vec![]);
            assert_action_node(&output, id, label);
            if kind == Dialog::Review {
                assert_action_node(&output, "ws-approve", "Approve changes");
                assert_action_node(&output, "ws-merge", "Merge pull request");
                assert!(ax_node(&output, "ws-merge").is_disabled());
                let tab = ax_node(&output, "ws-tab-pr");
                assert_eq!(tab.role(), egui::accesskit::Role::Tab);
                assert_eq!(tab.is_selected(), Some(true));
                assert_eq!(tab.toggled(), None);
            }
            native_tap(&mut s, &ctx, &mut rt, size, 0.1, id);
            assert!(s.dialogs.is_empty(), "{label} must still close its dialog");
        }
    }
    #[test]
    fn native_accessibility_mobile_pr_trigger_keeps_its_full_name() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(390., 844.);
        let output = native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        assert_action_node(&output, "ws-review-toggle", "Pull request #128");
        assert_action_node(&output, "ws-sidebar", "Open project files");
        for (id, selected) in [
            ("ws-view-editor", true),
            ("ws-view-chat", false),
            ("ws-view-checks", false),
        ] {
            let node = ax_node(&output, id);
            assert_eq!(node.role(), egui::accesskit::Role::Tab);
            assert_eq!(node.is_selected(), Some(selected));
            assert_eq!(node.toggled(), None);
        }
    }

    #[test]
    fn native_dialog_restore_rejects_missing_and_removed_origins_on_every_frame() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(1440., 1000.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        s.open_dialog(Dialog::Review, Some(Id::new("removed-origin")));
        native_frame(&mut s, &ctx, &mut rt, size, 0.1, vec![]);
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.2,
            Key::Escape,
            egui::Modifiers::NONE,
        );
        for i in 0..4 {
            native_frame(&mut s, &ctx, &mut rt, size, 0.3 + i as f64 * 0.1, vec![]);
        }
        assert_ne!(ctx.memory(|m| m.focused()), Some(Id::new("removed-origin")));
        assert!(s.restore_focus.is_none());
        native_tap(&mut s, &ctx, &mut rt, size, 0.8, "ws-rename");
        s.chat_open = false;
        native_key(
            &mut s,
            &ctx,
            &mut rt,
            size,
            0.9,
            Key::Escape,
            egui::Modifiers::NONE,
        );
        for i in 0..4 {
            native_frame(&mut s, &ctx, &mut rt, size, 1. + i as f64 * 0.1, vec![]);
        }
        assert_ne!(ctx.memory(|m| m.focused()), Some(Id::new("ws-rename")));
        assert!(s.restore_focus.is_none());
    }
    #[test]
    fn native_mobile_sessions_and_new_session_never_focus_hidden_desktop_controls() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        let size = Vec2::new(390., 844.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-sidebar");
        native_tap(&mut s, &ctx, &mut rt, size, 0.2, "ws-files-sessions");
        native_tap(&mut s, &ctx, &mut rt, size, 0.3, "ws-thread-search");
        assert!(s.dialogs.is_empty());
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-sidebar")));
        native_tap(&mut s, &ctx, &mut rt, size, 0.4, "ws-sidebar");
        native_tap(&mut s, &ctx, &mut rt, size, 0.5, "ws-files-sessions");
        native_tap(&mut s, &ctx, &mut rt, size, 0.6, "ws-nav-new");
        assert_eq!(s.view, View::Chat);
        assert_eq!(s.current().id, "new-1");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-prompt-input"))
        );
        native_tap(&mut s, &ctx, &mut rt, size, 0.7, "ws-sidebar");
        native_tap(&mut s, &ctx, &mut rt, size, 0.8, "ws-files-sessions");
        native_tap(&mut s, &ctx, &mut rt, size, 0.9, "ws-thread-sorting");
        assert_eq!(s.current().id, "sorting");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ws-prompt-input"))
        );
    }
    #[test]
    fn native_files_dialog_with_collapsed_tree_focuses_an_existing_close_control() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.tree_open = false;
        let size = Vec2::new(390., 844.);
        native_frame(&mut s, &ctx, &mut rt, size, 0., vec![]);
        native_tap(&mut s, &ctx, &mut rt, size, 0.1, "ws-sidebar");
        assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-files-close")));
        native_tap(&mut s, &ctx, &mut rt, size, 0.2, "ws-files-close");
        assert!(s.dialogs.is_empty());
    }
    fn app_frame(
        app: &mut crate::app::SchematicApp,
        ctx: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.enable_accesskit();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1440., 1000.))),
            time: Some(time),
            events,
            focused: true,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| app.render(ctx));
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("integrated native accessibility tree");
        assert!(
            tree.nodes.iter().any(|(id, _)| *id == tree.focus),
            "integrated workspace focus {:?} is absent at {time} on {:?}",
            tree.focus,
            app.rt.route
        );
        output
    }
    fn app_tap(
        app: &mut crate::app::SchematicApp,
        ctx: &egui::Context,
        time: f64,
        id: &str,
    ) -> egui::FullOutput {
        app_frame(app, ctx, time - 0.009, vec![]);
        let record = app
            .rt
            .records
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("missing integrated control {id}"));
        let widget_id = record.widget_id;
        let response = ctx
            .read_response(widget_id)
            .expect("recorded native widget response");
        let visible = response.rect.intersect(response.interact_rect);
        if !visible.is_positive() || visible.height() + 0.5 < response.rect.height() {
            // Native dialogs can scroll their content. Match a user's scroll
            // before using a trusted pointer, rather than clicking clipped bounds.
            app_frame(
                app,
                ctx,
                time - 0.007,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::ScrollIntoView,
                        target: widget_id.value().into(),
                        data: None,
                    },
                )],
            );
            app_frame(app, ctx, time - 0.004, vec![]);
            // The offset is committed at the end of a layout pass. Wait for
            // the following visible geometry, checking focus on every pass.
            for pass in 0..4 {
                app_frame(app, ctx, time - 0.003 + pass as f64 * 0.0005, vec![]);
                let response = ctx
                    .read_response(widget_id)
                    .expect("scroll target response");
                let visible = response.rect.intersect(response.interact_rect);
                if visible.is_positive() && visible.height() + 0.5 >= response.rect.height() {
                    break;
                }
            }
        }
        let response = ctx
            .read_response(widget_id)
            .expect("settled native widget response");
        let target = response.rect.intersect(response.interact_rect);
        assert!(
            target.is_positive(),
            "native action {id} must be visible: bounds={:?}, interact={:?}",
            response.rect,
            response.interact_rect
        );
        let pos = target.center();
        app_frame(
            app,
            ctx,
            time,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        app_frame(
            app,
            ctx,
            time + 0.01,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        app_frame(app, ctx, time + 0.02, vec![]);
        app_frame(app, ctx, time + 0.03, vec![])
    }
    #[test]
    fn native_app_approval_merge_confirmation_cancel_close_reopen_and_merge_keep_valid_focus() {
        use crate::runtime::Route;
        for motion in [MotionPreference::Full, MotionPreference::Reduced] {
            let ctx = egui::Context::default();
            let mut prefs = Preferences::default();
            prefs.motion = motion;
            let mut app = crate::app::SchematicApp::with_context(ctx.clone(), prefs);
            app.rt.navigate(Route::Workspace);
            app_frame(&mut app, &ctx, 0., vec![]);
            app_frame(&mut app, &ctx, 0.1, vec![]);
            app_frame(&mut app, &ctx, 0.5, vec![]);
            app_tap(&mut app, &ctx, 0.6, "ws-prompt-fix");
            let output = app_frame(&mut app, &ctx, 5., vec![]);
            assert_eq!(
                ax_node(&output, "ws-status").label(),
                Some("3 checks passed"),
                "the integrated repair action must actually complete before review"
            );
            app_tap(&mut app, &ctx, 5.1, "ws-review-toggle");
            let output = app_tap(&mut app, &ctx, 5.2, "ws-approve");
            assert_eq!(
                ax_node(&output, "ws-approve").label(),
                Some("Revoke approval"),
                "the integrated review must record its actual approval"
            );
            assert!(
                app.rt
                    .records
                    .iter()
                    .find(|r| r.id == "ws-merge")
                    .unwrap()
                    .enabled
            );
            app_tap(&mut app, &ctx, 5.3, "ws-merge");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(Id::new("ws-cancel-merge"))
            );
            app_tap(&mut app, &ctx, 5.4, "ws-cancel-merge");
            assert_eq!(ctx.memory(|m| m.focused()), Some(Id::new("ws-merge")));
            app_tap(&mut app, &ctx, 5.5, "ws-review-close");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(Id::new("ws-review-toggle"))
            );
            app_tap(&mut app, &ctx, 5.6, "ws-review-toggle");
            app_tap(&mut app, &ctx, 5.7, "ws-merge");
            let output = app_tap(&mut app, &ctx, 5.8, "ws-confirm-merge");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(Id::new("ws-review-close"))
            );
            assert_eq!(
                ax_node(&output, "ws-status").label(),
                Some("Merged into main")
            );
            app_tap(&mut app, &ctx, 5.9, "ws-review-close");
            let output = app_tap(&mut app, &ctx, 6., "ws-review-toggle");
            assert!(ax_node(&output, "ws-merge").is_disabled());
            app.rt.navigate(Route::Overview);
            app_frame(&mut app, &ctx, 6.1, vec![]);
            app_frame(&mut app, &ctx, 6.2, vec![]);
            assert!(!captures_keyboard(&ctx));
            app.rt.navigate(Route::Workspace);
            app_frame(&mut app, &ctx, 6.3, vec![]);
            let output = app_frame(&mut app, &ctx, 6.4, vec![]);
            assert!(ax_node(&output, "ws-edit").is_disabled());
        }
    }
}
