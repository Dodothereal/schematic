//! Record only Schematic's native framebuffer. Requires FFmpeg in PATH.
//! Usage: cargo run --release --example record_walkthrough -- output.mp4
use eframe::egui::{self, Context, Event, UserData, ViewportCommand};
use schematic_native::{
    app::SchematicApp,
    runtime::{MotionPreference, Preferences, Route},
};
use std::{
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
};
struct Recorder {
    app: SchematicApp,
    output: PathBuf,
    index: u64,
    pending: bool,
    encoder: Option<(Child, ChildStdin)>,
    scene: usize,
}
impl Recorder {
    fn frame(&mut self, ctx: &Context) {
        for event in ctx.input(|i| i.events.clone()) {
            if let Event::Screenshot {
                user_data, image, ..
            } = event
                && user_data
                    .data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<u64>())
                    .is_some()
            {
                if self.encoder.is_none() {
                    if let Some(parent) = self.output.parent() {
                        std::fs::create_dir_all(parent).expect("recording directory");
                    }
                    let mut child = Command::new("ffmpeg")
                        .args([
                            "-hide_banner",
                            "-loglevel",
                            "error",
                            "-y",
                            "-f",
                            "rawvideo",
                            "-pixel_format",
                            "rgba",
                            "-video_size",
                            &format!("{}x{}", image.width(), image.height()),
                            "-framerate",
                            "24",
                            "-i",
                            "-",
                            "-vf",
                            "scale=1440:-2:flags=lanczos",
                            "-c:v",
                            "libx264",
                            "-preset",
                            "fast",
                            "-crf",
                            "19",
                            "-pix_fmt",
                            "yuv420p",
                            "-movflags",
                            "+faststart",
                        ])
                        .arg(&self.output)
                        .stdin(Stdio::piped())
                        .spawn()
                        .expect("FFmpeg executable");
                    let input = child.stdin.take().expect("FFmpeg input");
                    self.encoder = Some((child, input));
                }
                let bytes: Vec<_> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                self.encoder
                    .as_mut()
                    .unwrap()
                    .1
                    .write_all(&bytes)
                    .expect("record native frame");
                self.index += 1;
                self.pending = false;
                if self.index >= 24 * 60 {
                    let (child, input) = self.encoder.take().unwrap();
                    drop(input);
                    let mut child = child;
                    assert!(child.wait().expect("FFmpeg completion").success());
                    eprintln!("Recorded {}", self.output.display());
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                    return;
                }
            }
        }
        let time = self.index as f64 / 24.0;
        ctx.input_mut(|i| i.time = time);
        let scene = [
            4.0, 6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0, 44.0, 46.0, 50.0,
            56.0,
        ]
        .iter()
        .position(|end| time < *end)
        .unwrap_or(16);
        if scene != self.scene {
            self.scene = scene;
            self.app.rt.now = time;
            match scene {
                1 => self.app.guide.open(&mut self.app.rt, None, None),
                2..=4 => self.app.guide.change_step(scene - 1, &mut self.app.rt),
                5 => {
                    self.app.guide.close();
                    self.app.rt.navigate(Route::Foundations);
                }
                6 => self.app.rt.navigate(Route::Motion),
                7 => self.app.rt.navigate(Route::Icons),
                8 => self.app.rt.navigate(Route::Components),
                9 => self.app.rt.navigate(Route::Primitives),
                10 => self.app.rt.navigate(Route::Navigation),
                11 => self.app.rt.navigate(Route::Forms),
                12 => self.app.rt.jump(Route::Data, "data-0"),
                13 => {
                    self.app.rt.set_palette(true);
                    self.app.rt.set_theme(true);
                }
                14 => self.app.rt.navigate(Route::Overlays),
                15 => self.app.rt.navigate(Route::Workspace),
                16 => {
                    self.app.rt.navigate(Route::Overview);
                    self.app.rt.set_palette(false);
                    self.app.rt.set_theme(false);
                }
                _ => {}
            }
        }
        self.app.render(ctx);
        if !self.pending {
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::new(self.index)));
            self.pending = true;
        }
        ctx.request_repaint();
    }
}
impl eframe::App for Recorder {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.frame(ctx);
    }
}
fn main() -> eframe::Result {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("output.mp4 path");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 1000.0])
            .with_app_id("dev.schematic.recorder"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Schematic native recording",
        options,
        Box::new(move |cc| {
            let mut app = SchematicApp::new(cc);
            app.rt.prefs = Preferences {
                motion: MotionPreference::Full,
                ..Default::default()
            };
            Ok(Box::new(Recorder {
                app,
                output,
                index: 0,
                pending: false,
                encoder: None,
                scene: 0,
            }))
        }),
    )
}
