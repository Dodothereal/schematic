# Verification

The release is checked against the actual offline reference in isolated Chrome profiles.

- 111 browser assertions passed: 46 routing/layout/runtime, 27 interactions, 23 motion/palette refinements and 15 onboarding checks.
- Structural checks validate JavaScript and inline scripts, local asset links, the 153-token manifest, zero npm/runtime network dependencies, and byte-for-byte deterministic builds.
- Checks cover all eight routes, state retention, Back/Forward, interrupted and unsupported View Transitions, keyboard controls, mobile focus and reduced motion.
- Interaction checks exercise retries, cancellation, validation, OTP paste, local profile save/reset, file selection, chart inspection, table sorting/filtering/selection and overlay cleanup.
- Refinement checks inspect intermediate line-drawing, checkbox and theme-reveal frames. The finished path and axes remain stable while a line draws. Both theme directions include the empty background.
- Onboarding checks confirm optional startup, four-step navigation, real palette changes, announced motion, focus containment/restoration, completion persistence, narrow layout and modal isolation.
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
- H.264 video, 1440 x 1008 at 30 fps; AAC narration at 48 kHz.
- Runtime: approximately 3 minutes 22 seconds; 13 seekable chapters.
- English captions and a written transcript accompany the recording.
- Encoded samples from the opening, guide, charts and closing were inspected. Playback and chapter seeking were tested. The watch page embeds captions so they also load from a local file.
- Narration is synthetic, generated locally with the installed macOS Samantha voice. No third-party footage or user desktop was recorded.

## Artifact integrity

- `dist/schematic.html` — 535,764 bytes; SHA-256 `393d097ce4457ba001dfd9b419ee2e0c923c32ab84376c776a4f8fe14dedd13e`.
- `dist/schematic-mono.html` — 535,763 bytes; SHA-256 `75b4eace3d0cd8637db0e09c2c6e747852ac1ff8ff279829f52643a85c4bd58f`.
- `docs/assets/schematic-walkthrough.mp4` — 15,439,994 bytes; SHA-256 `d8fe271e635ba477192e9410ac35bde7c08a17bd1bb42be51393117acd28331d`.

## Limits

Safari, Firefox, touch hardware and VoiceOver were not independently verified. Async actions, uploads and analytics are labelled local demonstrations. React/Next.js, SwiftUI and Rust adapters are planned; they are not implemented by this release.

GitHub Actions supplies the current remote build status. See the [CI workflow](https://github.com/Dodothereal/schematic/actions/workflows/ci.yml) and [Pages workflow](https://github.com/Dodothereal/schematic/actions/workflows/pages.yml).
