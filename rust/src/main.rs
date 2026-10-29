#![forbid(unsafe_code)]
fn main() -> eframe::Result {
    #[cfg(debug_assertions)]
    {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let path = std::env::temp_dir()
                .join(format!("schematic-native-panic-{}.log", std::process::id()));
            let _ = std::fs::write(
                path,
                format!("{info}\n{}", std::backtrace::Backtrace::force_capture()),
            );
            previous(info);
        }));
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 1000.0])
            .with_min_inner_size([320.0, 480.0])
            .with_app_id("dev.schematic.native"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Schematic",
        options,
        Box::new(|cc| Ok(Box::new(schematic_native::app::SchematicApp::new(cc)))),
    )
}
