# Verification

The release is checked against the actual offline reference in isolated Chrome profiles.

- 288 browser assertions passed: 52 routing/layout/runtime, 27 interactions, 23 motion/palette refinements, 15 onboarding behavior checks, 42 onboarding motion checks and 129 expanded-library checks.
- Structural checks validate JavaScript and inline scripts, local asset links, the 153-token manifest, zero npm/runtime network dependencies, and byte-for-byte deterministic builds.
- Checks cover all ten routes, state retention, Back/Forward, interrupted and unsupported View Transitions, keyboard controls, mobile focus and reduced motion.
- Interaction checks exercise retries, cancellation, validation, OTP paste, local profile save/reset, file selection, chart inspection, table sorting/filtering/selection and overlay cleanup.
- Refinement checks inspect intermediate line-drawing, checkbox and theme-reveal frames. The finished path and axes remain stable while a line draws. Both theme directions include the empty background.
- Onboarding checks confirm optional startup, four-step navigation, real palette changes, announced motion, focus containment/restoration, completion persistence, narrow layout and modal isolation.
- Normal-speed guide checks sample actual animation frames: forward/back travel, separated prose, progressive step fills, anchored dialog/footer/button bounds, zero-jump reversals, rapid navigation and interrupted dismissal. All four steps are checked at 390 x 844 and 320 x 568, including keyboard reachability and both explicit and system reduced motion.
- Expanded-library checks cover all sixteen new specimens, twenty recorded motion traces, catalog counts/search/filters/exact-example focus, the home playground, native keyboard activation, validation, retained state, and cleanup. All ten routes are checked at 320px and 390px in dark Mono. Reversals sample displayed geometry; reduced motion applies final states immediately.
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
- The refreshed v0.2 walkthrough includes the redesigned home, the refined guide and all sixteen new specimens in Primitives and Navigation.
- H.264 video, 1440 x 1008 at 30 fps; AAC narration at 48 kHz.
- Runtime: approximately 4 minutes 1 second; 15 seekable chapters and 61 caption cues.
- English captions and a written transcript accompany the recording.
- Encoded samples from the opening, guide, charts and closing were inspected. Playback and chapter seeking were tested. The watch page embeds captions so they also load from a local file.
- Narration is synthetic, generated locally with the installed macOS Samantha voice. No third-party footage or user desktop was recorded.

## Artifact integrity

- `dist/schematic.html` — 718,479 bytes; SHA-256 `7e9eb2a96b90c38050b7982f6ade0c157db0bb18a3871147e8197e015ccc606d`.
- `dist/schematic-mono.html` — 718,478 bytes; SHA-256 `06140ac41ef2b363af60f2e771f02c170d6f5ade147d0f1fb4000e8435a98f3a`.
- `docs/assets/schematic-walkthrough.mp4` — 19,938,130 bytes; SHA-256 `b404f7361a9b1022534cd14a773ba26fc4d6f1272da2f7eeb38f2ab8d07e1fb5`.

## Limits

Safari, Firefox, touch hardware and VoiceOver were not independently verified. Async actions, uploads and analytics are labelled local demonstrations. React/Next.js, SwiftUI and Rust adapters are planned; they are not implemented by this release.

GitHub Actions supplies the current remote build status. See the [CI workflow](https://github.com/Dodothereal/schematic/actions/workflows/ci.yml) and [Pages workflow](https://github.com/Dodothereal/schematic/actions/workflows/pages.yml).

The preceding hosted release was verified for route navigation, the Guide API,
caption loading, seeking/playback and runtime errors. The v0.2 local recording
and player were checked with 15 chapters and 61 loaded caption cues. The
inherited account-level custom domain currently serves the Pages site over HTTP;
direct HTTPS on that domain fails hostname verification. GitHub repository and
release download URLs use valid HTTPS.
