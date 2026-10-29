# Native walkthrough

[Watch or download the recording](assets/native-walkthrough.mp4).
The one-minute video captures only the native Rust application's framebuffer.
It has no audio. The following descriptions also serve as its transcript.
[WebVTT captions](assets/native-walkthrough.vtt) are available separately.

| Time | Native interface |
| --- | --- |
| 00:00 | Overview: search, categories and working component previews. |
| 00:04 | Guide: browse the library, choose Color or Mono, preview motion and reuse Rust components. |
| 00:12 | Foundations: semantic color tokens, typography, spacing, surfaces and exports. |
| 00:16 | Motion Lab: easing curves, scrubbing, physical spring controls and choreography. |
| 00:20 | Icons: 82 vector icons with individual animation, search, sizing, stroke and copy controls. |
| 00:24 | Components: actions, menus, tabs, choices and pending/retry states. |
| 00:28 | Primitives: disclosures, sorting, pinning, changing text, counters and cards. |
| 00:32 | Navigation: menus, searchable collection picker, tree, pagination, wizard and carousel. |
| 00:36 | Forms: validation, OTP, password requirements, ranges and local file selection. |
| 00:40 | Data: a native revenue plot with final-geometry tracing and linked values. |
| 00:44 | Appearance: Mono and the native circular dark-theme reveal, including the background. |
| 00:46 | Overlays: origin-aware surfaces, dialogs, drawers, sheets, menus and feedback. |
| 00:50 | Coding workspace: editable source, split diffs, simulated CI checks and logs, chat and review. |
| 00:56 | Return to Overview and reveal the light Color palette. |

The recording is a tour, not a demonstration of every action. The workspace
repair, check, approval and merge workflow is a local simulation. It never
contacts an AI provider, Git host or CI service.

To regenerate it from the repository root with FFmpeg installed:

```sh
cargo run --release --manifest-path rust/Cargo.toml \
  --example record_walkthrough -- rust/docs/assets/native-walkthrough.mp4
```

The example drives retained application state on a fixed frame clock, requests
native framebuffer captures and sends RGBA frames to FFmpeg. It does not capture
the desktop, an OS title bar or another application.
