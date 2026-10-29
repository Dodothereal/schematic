# Verification

The release is checked against the actual offline reference in isolated Chrome profiles.

- 439 browser assertions passed: 55 routing/layout/runtime, 27 interactions, 23 motion/palette refinements, 15 onboarding behavior checks, 42 onboarding motion checks, 133 library checks, 47 drag-sorting checks, 68 coding-workspace checks and 29 assembly/integration checks.
- Ten structural checks validate JavaScript and inline scripts, local asset links, the 153-token manifest, zero npm/runtime network dependencies, and byte-for-byte deterministic builds.
- Checks cover all eleven routes, state retention, Back/Forward, interrupted and unsupported View Transitions, keyboard controls, mobile focus and reduced motion.
- Interaction checks exercise retries, cancellation, validation, OTP paste, local profile save/reset, file selection, chart inspection, table sorting/filtering/selection and overlay cleanup.
- Refinement checks inspect intermediate line-drawing, checkbox and theme-reveal frames. The finished path and axes remain stable while a line draws. Both theme directions include the empty background.
- Onboarding checks confirm optional startup, four-step navigation, real palette changes, announced motion, focus containment/restoration, completion persistence, narrow layout and modal isolation.
- Normal-speed guide checks sample actual animation frames: forward/back travel, separated prose, progressive step fills, anchored dialog/footer/button bounds, zero-jump reversals, rapid navigation and interrupted dismissal. All four steps are checked at 390 x 844 and 320 x 568, including keyboard reachability and both explicit and system reduced motion.
- Expanded-library checks cover all sixteen new specimens, twenty recorded motion traces, catalog counts/search/filters/exact-example focus, the home playground, native keyboard activation, validation, retained state, and cleanup. All eleven routes are checked at 320px and 390px in dark Mono. Reversals sample displayed geometry; reduced motion applies final states immediately.
- Drag checks use trusted mouse, touch and keyboard input. They cover cross-group moves, sorting, edge scrolling, Escape and pointer cancellation, exact rollback, focus preservation, reduced motion and cleanup.
- Coding-workspace checks exercise highlighted source, real local editing and cursor retention, split-diff alignment and scroll synchronization, failing CI logs, assistant and manual repairs, regression after removing a repair, revision-aware approval, cancellation, merge gating, independent sessions and mobile Editor/Assistant/Checks views. All AI, Git and CI behavior is a local simulation; the edited TypeScript is not compiled or executed.
- Assembly checks inspect intermediate reassembly and chart-tracing frames, wireframe view, actual radius changes, keyboard selection, retained state, navigation cleanup and full-size mobile components. Integration checks cover the workspace's full-width layout and the library navigation's inertness, focus containment and restoration.
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
- The v0.4 walkthrough records the component assembly, rebuilt editor and CI console, drag sorting, guide and component collections.
- H.264 video, 1440 x 1008 at 30 fps; AAC narration at 48 kHz.
- Runtime: approximately 4 minutes 18 seconds; 16 seekable chapters and 64 caption cues.
- English captions and a written transcript accompany the recording.
- Encoded samples from the opening, coding workspace, dragging, charts and closing were inspected. Playback and chapter seeking were tested. The watch page embeds captions so they also load from a local file.
- Narration is synthetic, generated locally with the installed macOS Samantha voice. No third-party footage or user desktop was recorded.

## Artifact integrity

- `dist/schematic.html` — 873,991 bytes; SHA-256 `799a231968423746a2b73c9c3238f2a3c2644f4f58f4b307979f41896cae2411`.
- `dist/schematic-mono.html` — 873,990 bytes; SHA-256 `ae98b64264a23deea8cadf1f80e8a8ef616726ec75284e6bd2b943d8c69e894a`.
- `docs/assets/schematic-walkthrough.mp4` — 20,873,206 bytes; SHA-256 `35c1686e6c944d11d93e8addd8cb5f3d8f1c019a7c3ce9048d302746525ebf86`.

## Limits

Safari, Firefox, touch hardware and VoiceOver were not independently verified. Async actions, uploads and analytics are labelled local demonstrations. React/Next.js, SwiftUI and Rust adapters are planned; they are not implemented by this release.

GitHub Actions supplies the current remote build status. See the [CI workflow](https://github.com/Dodothereal/schematic/actions/workflows/ci.yml) and [Pages workflow](https://github.com/Dodothereal/schematic/actions/workflows/pages.yml).

The preceding hosted release was verified for route navigation, the Guide API,
caption loading, seeking/playback and runtime errors. The v0.4 local recording
and player were checked with 16 chapters and 64 loaded caption cues. The
inherited account-level custom domain currently serves the Pages site over HTTP;
direct HTTPS on that domain fails hostname verification. GitHub repository and
release download URLs use valid HTTPS.
