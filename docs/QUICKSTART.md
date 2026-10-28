# Start with Schematic

Schematic is a working HTML design reference. Try it, then bring the parts you
need into your project.

## Open it in a minute

1. Download the standalone `schematic.html`, or extract the source archive and
   find `dist/schematic.html`.
2. Open that file in a current browser. It works offline: no installation,
   server, account or CDN.
3. Select **Guide** in the top bar for a short optional tour. It never opens
   automatically. Close it anytime; Guide always starts again at the beginning.

`dist/schematic-mono.html` starts with Mono when no palette preference is saved.
For source work, open `prototype/index.html` and keep its sibling folders
together. Use a current browser for native dialogs, OKLCH colors and modern CSS.

## Make it yours

- **Palette:** open **Tune tokens** (the sliders control) and choose Color or
  Mono. Color uses warm paper / graphite with a forest accent; Mono makes the
  interface grayscale while charts retain distinct series colors. Choosing an
  accent hue switches back to Color. The Guide's palette buttons apply the same
  setting to the real app.
- **Theme:** the theme control switches light / dark independently.
- **Motion:** Tune tokens offers Full, Slow-mo x4 and Reduced. Your system's
  reduced-motion setting applies unless you explicitly choose Full. Try a small
  motion sample in Guide, or inspect timing, curves and springs in Motion Lab.
- **Preferences:** choices are saved locally when browser storage is available.
  Storage for local files varies by browser; moving a file may reset preferences.

## Find the right example

Use the sidebar, or **Menu** on a narrow window. **Commands** or **Cmd/Ctrl-K**
searches pages and settings. Overview has a live interaction playground and a
searchable catalog of the actual specimens. Press **/** there to reach search;
choose a result to open and focus the working example. Filter by family or use
Show more to explore the collection.

Start with Foundations for tokens and typography, then explore Motion Lab,
Icons, Components, Primitives, Navigation, Forms & Inputs, Data & Charts, and
Overlays & Feedback. Navigation preserves page state and scroll position.

Use Tab to reach controls, Enter or Space to activate buttons, and Escape to
dismiss dialogs. Guide provides Back / Next, step progress and a Finish button.
Its frame and navigation stay anchored while complete steps travel forward or
back. You can reverse direction mid-transition or reopen while it is closing;
the motion continues from the displayed frame. Reduced motion changes steps
immediately.
Closing it early does not mark it complete; Finish hides the first-visit nudge.
Palette changes remain saved even if you close early. Clearing browser site
storage resets preferences and completion. Uploads, async actions and live
charts are local simulations with no backend.

## Reuse the actual reference

Work from the source archive rather than extracting code from the bundled HTML:

| Source | What to reuse |
| --- | --- |
| `prototype/css/tokens.css` | Semantic colors, light / dark and Mono palettes, spacing, type, radii, motion durations and curves. |
| `prototype/css/base.css`, `prototype/css/ui.css` | Layout, typography and controls, including showcase shell rules. Select what your product needs. |
| `prototype/js/icons.js`, `prototype/css/icons.css` | SVG registry and its motion styles. |
| `prototype/js/core.js` | `window.Schematic` helpers, easing, tweens, segmented controls and popovers. Load tokens and shared CSS first. |
| `prototype/js/feedback.js` | Toasts and tooltips, with core and shared styles; initialize via `Schematic._initFeedback()`. |
| `prototype/pages/*.js`, `prototype/css/pages/*.css` | Concrete examples to adapt, each with a scoped stylesheet. |

Use semantic variables rather than copying color values:

```css
.your-surface {
  background: var(--surface);
  color: var(--text);
  padding: var(--s-4);
  border-radius: var(--r-lg);
}
```

The showcase supplies routing and boot wiring through `shell.js` and `app.js`;
individual page files are not drop-in widgets.
Read [the page contract](../prototype/CONTRACT.md) for registration and
`render` / `mount` / `enter` / `leave` responsibilities, and
[the design guide](../DESIGN_GUIDE.md) for interaction rules. Match the source's
script order when extending the showcase. Rebuild with Node 22.13 or newer:

```sh
node tools/bundle.mjs
```

HTML / CSS / JavaScript is implemented. React / Next.js, SwiftUI and Rust
adapters are planned. Components, lifecycle integration, native rendering and
accessibility still need implementation and runtime QA; browser colors and
animation APIs require mapping for native targets. Product composition and
backend behavior remain your application's responsibility.

For testing, call `Schematic.onboarding.open()` and
`await Schematic.onboarding.close()` after boot. Opening starts at step one,
preserves the route and returns focus on close. Only Finish writes
`schematic:guide-completed`; `open()` returns `false` if another dialog is open.
