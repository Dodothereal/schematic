# Verification

The release is checked against the actual offline reference in isolated Chrome profiles.

- 466 browser assertions passed: 55 routing/layout/runtime, 27 interactions, 23 motion/palette refinements, 15 onboarding behavior checks, 42 onboarding motion checks, 133 library checks, 47 drag-sorting checks, 73 coding-workspace and integration checks, 23 live-gallery checks, and 28 Overview motion checks.
- Ten structural checks validate JavaScript and inline scripts, local asset links, the 153-token manifest, zero npm/runtime network dependencies, and byte-for-byte deterministic builds.
- Checks cover all eleven routes, state retention, Back/Forward, interrupted and unsupported View Transitions, keyboard controls, mobile focus and reduced motion.
- Interaction checks exercise retries, cancellation, validation, OTP paste, local profile save/reset, file selection, chart inspection, table sorting/filtering/selection and overlay cleanup.
- Refinement checks inspect intermediate line-drawing, checkbox and theme-reveal frames. The finished path and axes remain stable while a line draws. Both theme directions include the empty background.
- Onboarding checks confirm optional startup, four-step navigation, real palette changes, announced motion, focus containment/restoration, completion persistence, narrow layout and modal isolation.
- Normal-speed guide checks sample actual animation frames: forward/back travel, separated prose, progressive step fills, anchored dialog/footer/button bounds, zero-jump reversals, rapid navigation and interrupted dismissal. All four steps are checked at 390 x 844 and 320 x 568, including keyboard reachability and both explicit and system reduced motion.
- Expanded-library checks cover all sixteen new specimens, twenty recorded motion traces, catalog counts/search/filters/exact-example focus, the home playground, native keyboard activation, validation, retained state, and cleanup. All eleven routes are checked at 320px and 390px in dark Mono. Reversals sample displayed geometry; reduced motion applies final states immediately.
- Drag checks use trusted mouse, touch and keyboard input. They cover cross-group moves, sorting, edge scrolling, Escape and pointer cancellation, exact rollback, focus preservation, reduced motion and cleanup.
- Coding-workspace checks exercise highlighted source, real local editing and cursor retention, split-diff alignment and scroll synchronization, failing CI logs, assistant and manual repairs, regression after removing a repair, revision-aware approval, cancellation, merge gating, independent sessions and mobile Editor/Assistant/Checks views. All AI, Git and CI behavior is a local simulation; the edited TypeScript is not compiled or executed.
- Gallery checks cover real action progress and cancellation, chart tracing and inspection, sorting, checkboxes, tab state, counter frames, icon focus, reduced motion, cleanup and narrow layouts.
- Overview motion checks sample heading entry, search focus and typing, moving category selection, result entry/exit and layout changes. They cover rapid reversals, all seven categories, query clearing, retained previews, wrapped filters at 320px and 390px, reduced motion and route/visibility cleanup.
- The catalog contains 69 specimens across eleven routes. The shared registry contains 82 animated icons.
- Primary and destructive labels and caption tokens meet the tested 4.5:1 threshold in both appearances and the tested hue presets. This is not a full accessibility certification.
- Local browser verification used Chrome on macOS. The public CI workflow repeats the suite on Linux Chrome.

## Reproduce

Node 22.13+ and Chrome/Chromium are required. No npm installation is needed. Set `SCHEMATIC_CHROME` when automatic browser discovery needs an explicit executable.

```sh
npm run build
npm run check -- --strict
npm run test:browser
```

The harness creates its own browser profile, closes its browser process and removes the profile. It does not attach to user tabs. Local reports and screenshots are written to ignored `scratch/qa/`.

## Walkthrough media

- The video records real controls in the generated offline application.
- The walkthrough records the component browser, editor and CI console, drag sorting, guide and component collections.
- H.264 video, 1440 x 1008 at 30 fps; AAC narration at 48 kHz.
- Runtime: approximately 4 minutes 22 seconds; 16 seekable chapters and 64 caption cues.
- English captions and a written transcript accompany the recording.
- Encoded samples from the opening, coding workspace, dragging, charts and closing were inspected. Playback and chapter seeking were tested. The watch page embeds captions so they also load from a local file.
- Narration is synthetic, generated locally with the installed macOS Samantha voice. No third-party footage or user desktop was recorded.

## Artifact integrity

- `dist/schematic.html` — 884,110 bytes; SHA-256 `5dc372ae198cabc33a2a626969507047e25586c0eb181d5030d847d281bc116e`.
- `dist/schematic-mono.html` — 884,109 bytes; SHA-256 `1d0b79dade6062f85b0833656b9706cd1eff7648ce76c842b86e880a881ee992`.
- `docs/assets/schematic-walkthrough.mp4` — 20,165,969 bytes; SHA-256 `3fd6c24b20ddb7e2d0a4cd60be6903d25561b38601a8ec4099c77d9b1f61b91a`.

## Limits

Safari, Firefox, touch hardware and VoiceOver were not independently verified. Async actions, uploads and analytics are labelled local demonstrations. React/Next.js, SwiftUI and Rust adapters are planned; they are not implemented by this release.

GitHub Actions supplies the current remote build status. See the [CI workflow](https://github.com/Dodothereal/schematic/actions/workflows/ci.yml) and [Pages workflow](https://github.com/Dodothereal/schematic/actions/workflows/pages.yml).

The preceding hosted release was verified for route navigation, the Guide API,
caption loading, seeking/playback and runtime errors. The v0.4.1 local recording
and player were checked with 16 chapters and 64 loaded caption cues. The
inherited account-level custom domain currently serves the Pages site over HTTP;
direct HTTPS on that domain fails hostname verification. GitHub repository and
release download URLs use valid HTTPS.
