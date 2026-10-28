# Schematic — Page Contract

Every page in the prototype is a self-contained pair of files:

```
prototype/pages/<id>.js        registers the page
prototype/css/pages/<id>.css   page-scoped styles (every selector starts with .pg-<id>)
```

Nothing else may be edited by a page author. If you need a new *shared* primitive
(something two pages would use), write it inside your page first and flag it in
your report — it gets promoted into `core.js` / `ui.css` during review.

## 1. Registering

```js
(() => {
  const S = window.Schematic;
  S.page({
    id: "components",            // route: #/components
    title: "Components",
    group: "Build",              // sidebar group: "Start" | "Foundations" | "Build"
    icon: "cube",                // any name from js/icons.js (S.iconNames)
    order: 40,                   // sidebar order
    count: 24,                   // optional sidebar counter
    description: "…",            // one line, used on the overview + command palette
    render: () => `<div class="pg-components">…</div>`,   // pure HTML string, cheap
    mount: (root) => {},         // once, first time shown. Wire up events here.
    enter: (root) => {},         // every time the page becomes visible (replay entrances)
    leave: (root) => {},         // every time it's hidden (pause loops, timers)
  });
})();
```

* `render()` may run **long before** the page is visible (intent / idle prerender).
  It must not measure layout or start timers. Do that in `mount`/`enter`.
* Pages are **kept alive** — DOM and state persist between visits.
* Root markup: `<div class="pg-<id>"> S.pageHead(...) + S.section(...)* </div>`.

## 2. Markup helpers (always use them — this is what keeps pages identical)

```js
S.pageHead({ icon, eyebrow, title, lead, meta: [badgeHtml…] })
S.section({ id, num: "01", title, desc, body })        // feeds the "On this page" rail
S.demo({ title, desc, body, specs: ["ease-out", "220ms"], stage: "is-plain|is-col|is-start", replay: true })
S.icon(name, { size: 18 })                              // animated SVG string
```

Layout: `.grid.cols-2|cols-3|cols-4`, `.span-2`, `.span-all`, `.stack`, `.row`, `.row-between`
(set `style="--gap:var(--s-2)"` to change gaps).

Existing classes you should reuse, never re-implement:
`.btn .btn-primary|secondary|ghost|soft|danger .btn-sm|lg|xl .btn-icon .btn-block`,
`.badge .badge-accent|success|warning|danger|info .badge-dot`, `.kbd`, `.kbd-group`,
`.card`, `.spot` (cursor-tracking border light), `.seg/.seg-btn` (+ `S.segmented`),
`.pop` (+ `S.popover`), `.range`, `.reveal` (stagger children with `style="--i:n"`),
`.t-h1 .t-h2 .t-h3 .t-lead .t-small .t-caption .t-eyebrow .t-mono .t-num .muted .faint`.

## 3. Runtime API (`window.Schematic`, alias `S`)

| API | Purpose |
|---|---|
| `S.toast({title, desc, tone, icon, action:{label,onClick}, duration})` | stacked toasts |
| `data-tip="Label" data-tip-kbd="⌘ C" data-tip-side="bottom"` | tooltips (no JS) |
| `S.tween({from,to,duration,ease,delay,onUpdate})` | rAF tween; `from/to` can be arrays |
| `S.countTo(el, value, {format})` · `S.roll(el, "1,284")` | count-up · odometer digits |
| `S.flip(elements, mutate, {stagger})` | FLIP layout animation (reorder, filter) |
| `S.glide(container, {items})` | one hover highlight that travels between items |
| `S.segmented(el, {value,onChange})` · `S.popover(trigger, panel)` | controls |
| `S.ease.*` / `S.curve.*` | JS easing fns / CSS strings: out, outExpo, in, inOut, overshoot, springSmooth, springSnappy, springBouncy |
| `S.dur(ms)` · `S.wait(ms)` · `S.reduced()` | respect the global motion scale |
| `S.playIcon(svg)` | play an icon's animation programmatically |
| `S.navigate(id)` · `S.scrollTo(y)` | routing, curved scroll |

Demos with `replay: true` receive a bubbling `replay` CustomEvent on the `.demo` element.

## 4. Motion rules (non-negotiable)

1. **Never** a hover that just lifts (`translateY(-2px)`) or grows (`scale(1.05)`) a button.
   Hover *reveals*: light, border, icon verb, extra info. Press compresses (`scale(.97)`).
2. Durations & curves only from tokens: `var(--d-fast|base|moderate|slow|slower)`,
   `var(--ease-out|ease-out-expo|ease-in|ease-in-out|ease-overshoot)`,
   `var(--spring-smooth|snappy|bouncy)` with `var(--d-spring-*)`.
   In JS use `S.dur()`, `S.ease.*`, `S.curve.*`. Never a raw `ms` that ignores `--motion-scale`.
3. Enter with *out* curves, exit with *in* curves and ~30% faster, move on-screen with *in-out*.
4. Animate `transform`, `opacity`, `filter`, `clip-path`. Height only via grid `0fr→1fr`
   or measured px. Never `top/left/margin` for motion.
5. Interruptible: every animation must cope with being reversed mid-flight
   (prefer CSS transitions over keyframes for state; keyframes for one-shots).
6. Every interactive element has a `:focus-visible` state (global ring exists) and is
   keyboard operable. Use real `<button>`, `<input>`, `role`/`aria-*`.
7. Spacing only from `--s-*`, radius from `--r-*`, colour from semantic tokens. No hex.
8. Must look right in **light and dark** and with every accent hue (tune panel).
