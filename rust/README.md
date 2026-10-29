# Schematic native Rust

The native application uses Rust and eframe/egui 0.33.3. Rust owns layout,
painting, state, input and animation. It opens an OS window with the Glow/OpenGL
renderer; it does not load the website, a WebView, HTML/CSS, a JavaScript runtime
or a Wasm bundle. The icon module converts its path data into egui primitives;
there is no SVG document renderer.

The [HTML reference](../README.md) remains a separate implementation.
[PARITY.md](PARITY.md) maps the eleven native routes and 69 catalog specimens
and records what validation does, and does not, establish.

[Watch the native walkthrough](docs/assets/native-walkthrough.mp4) or read its
[chapters and transcript](docs/WALKTHROUGH.md). The
60-second tour covers the native routes, Guide and appearances. Its frames are
captured by the Rust [recording example](examples/record_walkthrough.rs), then
encoded by FFmpeg; it does not record a browser. The video is a separate
documentation asset and is not required to run the app.

## Build and run

Run these commands from the repository root. Install a current stable Rust
toolchain with Cargo; development checks have used Rust 1.96.0. Dependencies are
resolved by the committed `rust/Cargo.lock`. The first build can download crates.

On macOS, install Xcode Command Line Tools if they are missing:

```sh
xcode-select --install
```

Build and open the native window:

```sh
cargo run --manifest-path rust/Cargo.toml --locked
```

For the optimized binary:

```sh
cargo build --manifest-path rust/Cargo.toml --locked --release
./rust/target/release/schematic-native
```

Cargo's output location changes if `CARGO_TARGET_DIR` or a target is configured.
No npm command, web server, browser or JavaScript renderer participates in the
native build or application.

Linux build prerequisites extend the
[eframe 0.33.3 native instructions](https://github.com/emilk/egui/tree/0.33.3/crates/eframe)
for this crate's X11, Wayland and OpenGL features. The Ubuntu CI recipe is:

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends -y \
  pkg-config libssl-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-0 \
  libwayland-dev libx11-dev libxi-dev libxcursor-dev libxrandr-dev \
  libxinerama-dev libgl1-mesa-dev libegl1-mesa-dev libudev-dev
```

A Linux desktop run also needs a working X11 or Wayland session, OpenGL support
and an appropriate desktop portal for file dialogs. The Linux recipe and CI
checks are not a claim that Linux desktop behavior has been verified. No Windows
native build or runtime verification is claimed here.

## Use the application

Use the sidebar or Commands to choose a route. Cmd/Ctrl-K opens Commands. Guide
is optional. Tune changes theme, Color/Mono, accent hue, radius and motion.
Preferences persist between launches; individual page state is retained while
the process runs. Leaving a page stops its pending or recurring demo work.

On macOS, Cmd-[ goes Back and Cmd-] goes Forward through route history.
Alt-Left and Alt-Right are also wired as Back/Forward shortcuts.

Clipboard actions use the native clipboard and report its result. Exports use
an OS save dialog and write only the chosen file. Forms, generated analytics,
uploads, coding chat, CI and merge actions are local simulations. They do not
contact an AI, Git host, analytics service or upload backend.

## Validate changes

```sh
rustup component add rustfmt clippy
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml --locked --all-targets
```

CI treats Clippy warnings as errors. Tests include pure state/math and native
egui input, focus and tessellation checks. Headless egui tests do not create an OS window. They cannot
certify display output, screen-reader operation, native dialogs or clipboard
availability. Test counts and results belong to the checkout being tested;
this document does not freeze a passing count while the implementation changes.

For a real native framebuffer capture on a desktop with a display:

```sh
SCHEMATIC_NATIVE_ROUTE=data \
SCHEMATIC_NATIVE_CAPTURE="${TMPDIR:-/tmp}/schematic-data.png" \
SCHEMATIC_NATIVE_CAPTURE_QUIT=1 \
  cargo run --manifest-path rust/Cargo.toml --locked
```

Capture mode selects the route, uses default preferences, writes a PNG and a
JSON file of recorded controls, then exits when requested. Set
`SCHEMATIC_NATIVE_DARK=1` and/or `SCHEMATIC_NATIVE_MONO=1` for the other
appearances. Inspect the image and interact with the application as well as
running tests. Keep captures out of source control.

## Package a macOS app

The script works from any working directory when invoked by its path. From the
repository root on macOS:

```sh
./rust/scripts/package-macos.sh
open rust/dist/Schematic.app
```

The script derives the crate,
version, host target and Cargo output directory. It builds the release binary
for the Rust host architecture and creates:

```text
rust/dist/Schematic.app
rust/dist/schematic-native-<version>-<host-target>.zip
rust/dist/schematic-native-<version>-<host-target>.zip.sha256
```

The default packaging build cache is `rust/target/package-macos`; an explicit
`CARGO_TARGET_DIR` overrides it. Existing Cargo compiler flags are retained,
and checkout/home compiler paths are remapped in the release build. The
packager checks for unbundled dynamic libraries, ad-hoc signs the app, verifies
its signature, extracts the ZIP and verifies the extracted executable and
signature. It converts the native-vector-generated `assets/icon.png` to
`AppIcon.icns` with `sips` and `iconutil`; all resized icon files stay inside the
ignored staging directory. The ZIP contains the actual app bundle, including
its display name, icon, executable modes, the license and these native docs.
Build output stays ignored by Git.

The package contains one architecture, not a universal binary. Its minimum
macOS plist value comes from the executable's Mach-O load command, not a tested
OS compatibility matrix. Ad-hoc signing is not Developer ID signing or
notarization and does not establish Gatekeeper distribution trust. The script
does not change security policy, remove quarantine attributes or install the
app into an Applications directory.

Native releases use the separate tag `native-v0.1.0`. A maintainer creating that
release with `gh release create` must pass `--latest=false`, so the web release
`v0.4.2` and its latest-HTML download links keep their existing meaning. The
native CI workflow only uploads run artifacts; it does not create a release,
retag the web version or change GitHub's latest-release designation.

## Reuse native components

Start with [PORTING.md](PORTING.md), then inspect the corresponding source:

| Native source | Responsibility |
| --- | --- |
| `src/tokens.rs` | Semantic colors and appearance conversion |
| `src/motion.rs`, `src/easing.rs` | Curves, interruptible tweens, spring samples |
| `src/ui.rs`, `src/icons.rs` | Native controls and vector painting |
| `src/runtime.rs` | Page lifecycle, preferences, feedback, clipboard and saves |
| `src/feedback.rs`, `src/sortable.rs` | Surfaces, notifications and drag transactions |
| `src/pages/` | Retained state and complete working examples |

Each page exposes `State: Default` and implements `runtime::Page`. Its `show`
method draws into an egui `Ui`; `enter` and `leave` start and cancel work. Keep
state transitions testable without a window, use real time for user-action
deadlines, and request frames only for visible finite motion. Register new
pages in `runtime::Route` and `pages::create`; add catalog entries deliberately.

These pages are examples integrated with Schematic's runtime, not a published
drop-in widget package. For reuse in a product, carry over the relevant tokens,
painting, state and lifecycle code, then validate that product's keyboard,
appearance, motion and platform integration.

The [native workflow](../.github/workflows/native.yml) is configured for Linux
formatting, Clippy and tests, plus macOS tests and host-architecture packaging.
Successful packaging jobs upload CI artifacts; the workflow does not publish a
release or certify native runtime parity. A remote run must be checked separately.
