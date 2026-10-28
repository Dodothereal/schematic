# Design and interaction guide

The HTML prototype is the visual and behavioral reference. A new framework
adapter should match it deliberately, rather than translate every CSS effect
literally.

## Foundations

Default to warm paper and graphite surfaces with a restrained forest accent.
Headlines use plain text colour. Mono is an equally supported interface palette;
data series retain independent colours so charts remain distinguishable.

- Consume semantic colours: background, surface, text, border, accent and status.
  Both themes live in `prototype/css/tokens.css`; chart series have separate
  tokens. Retune the accent through `--accent-h` instead of inventing colours.
- Use the 4px spacing scale and its documented half steps. Align controls by
  their shared height tokens. Use the same inset borders and radius multiplier.
- Keep typography on the declared scale. Prefer local system fonts for instant
  rendering, readable supporting text and tabular figures for changing data.
- A status must be identifiable by text or an icon in addition to colour.

## Motion

An animation should explain an action, a relationship, a change or feedback.
Hover reveals a border, light, useful detail or an icon's verb. Do not make a
button lift or grow just to demonstrate interactivity. A press may compress and
release with a small spring.

| Intent | Curve | Timing |
| --- | --- | --- |
| Respond / enter | out / out-expo | 140–460ms by distance |
| Leave / dismiss | in | About 30% shorter than entry |
| Travel within a screen | in-out | 220–460ms |
| Selection / physical confirmation | sampled spring | Natural spring duration |
| Small sequential group | out-expo | 32ms stagger, bounded group |

Use CSS transitions for reversible state, one-shot keyframes for entrances,
and cancel prior JavaScript tweens when user intent changes. Animate transform
and opacity first. Measure layout outside frame loops. Height transitions need
grid expansion or explicit measured endpoints. Hidden pages must stop recurring
work. Respect reduced motion and page visibility. Slow-motion inspection should
scale visual timing, never shorten hold-to-confirm or other user-action windows.

Draw a chart's finished geometry instead of growing the line from a zero-valued
shape. Delay below-fold drawing until the actual plot comes into view. On range
changes, cancel the old trace and draw the new one. Keep axes and paths out of
the frame loop; only the stroke reveal, drawing head and area curtain change.
Hover and keyboard focus link a graphical mark to its values. Preserve readable
labels when dimming other series, and provide a click-to-pin option where useful.

## Access and performance

Use native buttons, inputs, labels and dialogs. Maintain visible keyboard focus,
logical tab order, Escape dismissal and focus restoration. A custom listbox
needs arrow keys, selection state, a clear label and the matching ARIA pattern.
Give charts a textual summary and keyboard access to the same data shown on hover.

Preserve form and scroll state during navigation. Prepare likely destinations
on user intent and schedule optional work while idle. Do not fetch every remote
resource indiscriminately; the HTML prototype already ships every route locally.
Keep route changes usable when View Transitions are unavailable. Label simulated
async behavior and never turn an operation failure into success feedback.

## Adapter boundary

| Platform | Reusable contract | Work still required |
| --- | --- | --- |
| HTML / CSS / JavaScript | Working reference | Product-specific composition and backend |
| React / Next.js | CSS tokens, SVG paths, motion semantics | Components, effects/cleanup, router integration and framework QA |
| SwiftUI | Semantic colours, spacing/type scale and interaction intent | Native assets, Dynamic Type, focus/VoiceOver and spring mapping |
| Rust web UI / Tauri | CSS/SVG contract and page lifecycle intent | Framework wrappers, lifecycle cleanup and host integration |
| Rust native UI | Semantic token names and behavior | Native layout, rendering, accessibility and motion implementation |

CSS OKLCH values, sampled `linear()` curves and Web Animations are not native
Swift or Rust APIs. Convert colour spaces, map springs using measured behavior,
and validate each port in its actual runtime before calling it supported.
