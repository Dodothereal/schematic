# Native parity inventory

This is an implementation map, not a claim of pixel identity or complete
platform certification. The native application draws with Rust and egui. The
HTML/CSS/JavaScript reference supplies behavior and design intent, not a runtime
renderer or embedded implementation.

## Route mapping

The 69 entries in [`src/catalog.rs`](src/catalog.rs) are distributed across ten
content routes. Overview is the eleventh route and aggregates those same
entries; it adds no extra catalog specimens. Counts do not include every
control, dialog, icon or supporting section on a page.

| Route | Native page | Catalog IDs | Count | Supporting native behavior |
| --- | --- | --- | ---: | --- |
| Overview | [overview.rs](src/pages/overview.rs) | Aggregates all IDs below | 0 additional | Search, categories, working previews, commands and navigation |
| Foundations | [foundations.rs](src/pages/foundations.rs) | `foundations-0` through `foundations-1` | 2 | Color/type/surface specimens, theme/radius choices, resolved CSS/JSON exports |
| Motion Lab | [motion_lab.rs](src/pages/motion_lab.rs) | `motion-0` through `motion-3` | 4 | Nine curves, scrubbing, spring simulation/export, actual retained-page cache inspection |
| Icons | [icons_page.rs](src/pages/icons_page.rs) | `icons-0` through `icons-1` | 2 | Search 82 icons, size/stroke controls, wave/replay and snippets |
| Components | [components.rs](src/pages/components.rs) | `components-0` through `components-12` | 13 | Pending/retry states, fixed-time confirmation, keyboard controls and retained choices |
| Primitives | [primitives.rs](src/pages/primitives.rs) | `primitives-0` through `primitives-7` | 8 | Disclosure, pointer/keyboard sorting, cross-group pinning and reversible motion |
| Navigation | [navigation.rs](src/pages/navigation.rs) | `navigation-0` through `navigation-7` | 8 | Menus, collection filtering, tree traversal, pagination, wizard validation and carousel |
| Forms & Inputs | [forms.rs](src/pages/forms.rs) | `forms-0` through `forms-10` | 11 | Validation, OTP paste/focus, bounded ranges, local file simulation and composed profile |
| Data & Charts | [data.rs](src/pages/data.rs) | `data-0` through `data-4` | 5 | Seeded KPIs, range changes, visible-only traces, linked inspection, customer table and CSV |
| Overlays & Feedback | [overlays.rs](src/pages/overlays.rs) | `overlays-0` through `overlays-14` | 15 | Focus-managed surfaces, dismissal/drag, nested menus, toasts and timed loading |
| Coding workspace | [workspace.rs](src/pages/workspace.rs) | `workspace-0` | 1 | Editable buffers, split diffs, local chat/checks/review/merge simulation |
| Total | | | 69 | |

All routes use the shared semantic tokens, light/dark and Color/Mono preferences,
native input and motion runtime. Page state remains in memory across navigation.
Preferences persist separately; this does not imply that all demo state survives
an application restart. Clipboard and file exports use native OS integrations.

## Catalog specimens

IDs and titles below follow the catalog. Their presence identifies the native
source to inspect; it does not assert that every interaction has passed manual
acceptance testing.

| Catalog ID | Reference title |
| --- | --- |
| `foundations-0` | Spacing scale |
| `foundations-1` | Control rhythm |
| `motion-0` | Easing workbench |
| `motion-1` | Spring bench |
| `motion-2` | Staggered arrival |
| `motion-3` | State continuity |
| `icons-0` | Animated icons |
| `icons-1` | Snippet |
| `components-0` | Button hierarchy |
| `components-1` | A size for the moment |
| `components-2` | Action button |
| `components-3` | Copy with certainty |
| `components-4` | A deliberate confirmation |
| `components-5` | A little appreciation |
| `components-6` | One action, a few alternatives |
| `components-7` | A toolbar that travels with you |
| `components-8` | Workspace preferences |
| `components-9` | Segmented, connected |
| `components-10` | Tabs |
| `components-11` | Labels that make room |
| `components-12` | People, with presence |
| `primitives-0` | Accordion |
| `primitives-1` | Collapsible |
| `primitives-2` | Pinned list |
| `primitives-3` | Sortable list |
| `primitives-4` | Text swap |
| `primitives-5` | Animated counter |
| `primitives-6` | Flip card |
| `primitives-7` | Expandable surface |
| `navigation-0` | Travelling navigation menu |
| `navigation-1` | Searchable collection picker |
| `navigation-2` | Living breadcrumb trail |
| `navigation-3` | Directional pagination |
| `navigation-4` | Keyboard file tree |
| `navigation-5` | Workspace action dock |
| `navigation-6` | Three-step workspace wizard |
| `navigation-7` | Carousel |
| `forms-0` | Input sizes |
| `forms-1` | States with something to say |
| `forms-2` | Validation, in context |
| `forms-3` | Auto-growing textarea |
| `forms-4` | Show the requirements |
| `forms-5` | One code, six small steps |
| `forms-6` | Let the platform help |
| `forms-7` | Every step accounted for |
| `forms-8` | A single point of control |
| `forms-9` | Two bounds, one interval |
| `forms-10` | Local files / simulated progress |
| `data-0` | Revenue line chart |
| `data-1` | Grouped and stacked bars |
| `data-2` | Animated ranking |
| `data-3` | Donut chart |
| `data-4` | Progress rings |
| `overlays-0` | Origin morph |
| `overlays-1` | Modal dialog |
| `overlays-2` | Destructive confirmation |
| `overlays-3` | Side drawer |
| `overlays-4` | Bottom sheet |
| `overlays-5` | Dropdown + submenu |
| `overlays-6` | Context menu |
| `overlays-7` | Intent hovercard |
| `overlays-8` | Tooltip toolbar |
| `overlays-9` | Toast tones |
| `overlays-10` | Promise feedback |
| `overlays-11` | Single accordion |
| `overlays-12` | Multiple accordion |
| `overlays-13` | Show-more disclosure |
| `overlays-14` | Skeleton → content |
| `workspace-0` | Coding workspace |

## Checked locally for native v0.1.1

- 170 library tests and seven whole-application integration tests pass on macOS
  Apple Silicon. Clippy passes with warnings denied, and formatting passes.
- All 69 catalog destinations resolve to live native specimen geometry.
- The guide is checked through interrupted steps and closure at 320, 390 and
  1440 logical pixels with AccessKit enabled and reachable footer controls.
- A native window completed repair, three passing CI checks, approval, merge
  confirmation and simulated merge. Command navigation and focus restoration
  were also operated in the native window.
- Native clipboard feedback reported success. The OS save dialog wrote a token
  JSON export that was read back and parsed.
- The framebuffer recording was inspected at intermediate theme-reveal frames,
  including the window corners. Both theme directions have regression coverage.

## Validation boundaries

- `cargo test --locked --all-targets` runs domain/state tests, native egui input
  tests and [`tests/parity.rs`](tests/parity.rs). The latter checks the 69 unique
  catalog IDs, route rendering/tessellation at 1440, 390 and 320 logical-pixel
  widths, four appearances, and selected shell navigation/command flows. The
  route matrix uses reduced motion; individual motion tests cover other timing
  paths. This is not exhaustive input or frame coverage of all 69 specimens.
- Headless egui tests exercise Rust layout, painting output, focus and state.
  They do not launch a platform window or verify an OS compositor, display
  scaling, GPU driver, native save dialog or clipboard service.
- Native macOS windows and framebuffer captures have been inspected during
  development on Apple Silicon. Representative captures and tested state
  transitions do not establish pixel identity, exact timing equivalence or
  complete manual accessibility coverage. Guide and merge/review accessibility
  and focus flows have been checked on macOS; this is scoped evidence, not
  certification of every VoiceOver or other assistive-technology path.
- The macOS packager verifies bundle structure, host architecture, ad-hoc code
  signing and ZIP extraction. Those checks do not establish Developer ID trust,
  notarization, support for every macOS version, Intel runtime behavior or a
  universal binary.
- Linux CI compiles/lints and runs headless tests; macOS CI builds and packages
  the runner's architecture. Adding a workflow is not evidence that a remote CI
  run has passed. Neither job certifies desktop behavior on another OS. Windows
  native validation is not claimed.
- Test totals and outcomes can change while runtime work continues. Consult the
  actual command output and CI run for the revision being assessed. A passing
  build or a catalog entry alone is not proof of interaction parity.

Read [README.md](README.md) for build, capture, package and reuse commands.
The native walkthrough is a native framebuffer recording, not a browser capture
or an exhaustive acceptance test. Native release tags remain separate from the
web release and must not replace its latest-HTML download designation.
