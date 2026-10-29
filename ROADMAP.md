# Where Schematic goes next

The project includes a web reference and a native Rust application. Further work should make it
easier to use in a real product while keeping its motion and visual consistency.

## 1. From showcase to reusable recipes

Add a copyable implementation beside each specimen: markup, scoped styles,
required tokens, behavior and cleanup. Provide a small application starter with
a responsive shell, real empty/error/loading states and a complete settings flow.
Keep the existing showcase as the reference rather than turning it into a framework.

## 2. Motion quality across browsers

Profile chart drawing, theme reveals and large tables on slower hardware.
Add Safari and Firefox interaction coverage, touch testing and VoiceOver checks.
Record frame timing during representative transitions; keep the result and test
conditions with each performance claim. Use a simple visual regression baseline
for the four appearance/palette combinations and reduced motion.

## 3. One measured framework adapter

Build a small React/Next.js adapter first: tokens, animated icons, buttons, forms,
dialogs and the application shell. Match behavior against the HTML specimens,
including focus restoration and interrupted animations. Publish the adapter only
after it has a real example and framework/browser checks.

## 4. Native parity

The Rust implementation uses eframe/egui, native vector drawing, semantic OKLCH
conversion and sampled spring curves. Its [parity ledger](rust/PARITY.md) records
the implemented behavior and validation boundaries. Next, verify physical touch,
screen readers and desktop rendering on additional operating systems. Map the
same contracts into SwiftUI when that adapter is started.
