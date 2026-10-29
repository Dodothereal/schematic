//! Native icon gallery: stable identities, interruptible geometry, native vector
//! choreography, and clipboard feedback tied to the actual write result.
use crate::{
    icons,
    motion::{self, Curve},
    runtime::{Page, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{self, *};
use std::collections::{HashMap, HashSet};

const SIZES: [f32; 4] = [16., 20., 24., 32.];
const COPY_SECONDS: f64 = 2.2;
const VERBS: [(&str, &str, &str); 6] = [
    (
        "bell",
        "Rings",
        "The body swings from the hook; the clapper follows.",
    ),
    (
        "trash",
        "Lid lifts",
        "The lid opens on its right hinge, then settles.",
    ),
    (
        "send",
        "Flies off",
        "The arrow exits up-right and returns from the opposite corner.",
    ),
    (
        "download",
        "Drops in",
        "The arrow drops into the tray; the tray absorbs the impact.",
    ),
    (
        "copy",
        "Duplicates",
        "The front sheet slides away from the back sheet.",
    ),
    (
        "eye",
        "Blinks",
        "The eye blinks, then the pupil looks around.",
    ),
];

// These are the reference's actual search categories. Additional registry icons
// remain searchable by name and by Other, rather than acquiring guessed tags.
fn category(name: &str) -> &'static str {
    if [
        "arrow-right",
        "arrow-left",
        "arrow-up-right",
        "chevron-right",
        "chevron-down",
        "chevrons-up-down",
        "download",
        "upload",
        "refresh",
        "logout",
        "external",
    ]
    .contains(&name)
    {
        "Arrows"
    } else if [
        "check", "x", "plus", "minus", "search", "trash", "copy", "pencil", "send", "link",
        "filter", "sliders", "play", "pause", "more", "menu", "command", "cursor",
    ]
    .contains(&name)
    {
        "Actions"
    } else if [
        "bell", "home", "mail", "lock", "eye", "heart", "star", "bookmark", "calendar", "clock",
        "document", "folder", "chat", "user", "users", "globe", "gauge", "droplet", "palette",
        "ruler", "type", "cube", "layers", "window", "input", "sidebar", "code", "grid",
    ]
    .contains(&name)
    {
        "Objects"
    } else if ["chart-bar", "trending", "wave", "pie"].contains(&name) {
        "Data"
    } else if [
        "info",
        "alert",
        "check-circle",
        "x-circle",
        "bolt",
        "sparkles",
        "sun",
        "moon",
        "loader",
    ]
    .contains(&name)
    {
        "Status"
    } else {
        "Other"
    }
}

#[derive(Clone, Copy, Debug)]
struct Visual {
    rect: Rect,
    opacity: f32,
}
#[derive(Clone, Debug)]
struct Travel {
    from: Visual,
    to: Visual,
    start: f64,
    seconds: f32,
    leaving: bool,
    bounds_width: f32,
}
impl Travel {
    fn value(&self, now: f64) -> Visual {
        let p = if self.seconds <= 0. {
            1.
        } else {
            ((now - self.start) / self.seconds as f64).clamp(0., 1.) as f32
        };
        let p = motion::ease(
            if self.leaving {
                Curve::In
            } else {
                Curve::SpringSnappy
            },
            p,
        );
        let rect = Rect::from_min_max(
            self.from.rect.min.lerp(self.to.rect.min, p),
            self.from.rect.max.lerp(self.to.rect.max, p),
        );
        // Resize constrains the actual painted/hit-tested tile, not just its
        // clip rectangle or QA record. Spring overshoot also stays in bounds.
        let width = rect.width().clamp(0., self.bounds_width.max(0.));
        let left = rect.left().clamp(0., (self.bounds_width - width).max(0.));
        Visual {
            rect: Rect::from_min_size(pos2(left, rect.top()), vec2(width, rect.height())),
            opacity: egui::lerp(self.from.opacity..=self.to.opacity, p).clamp(0., 1.),
        }
    }
    fn moving(&self, now: f64) -> bool {
        self.start + self.seconds as f64 > now
            && (self.from.rect != self.to.rect || self.from.opacity != self.to.opacity)
    }
    fn retarget(&mut self, to: Visual, now: f64, seconds: f32, leaving: bool) {
        if self.to.rect == to.rect && self.to.opacity == to.opacity && self.leaving == leaving {
            return;
        }
        self.from = self.value(now);
        self.to = to;
        self.start = now;
        self.seconds = seconds;
        self.leaving = leaving;
    }
    fn settle(&mut self) {
        self.from = self.to;
        self.seconds = 0.;
    }
}
#[derive(Clone)]
struct PaintSnapshot {
    shapes: Vec<egui::epaint::ClippedShape>,
    opacity: f32,
}
#[derive(Clone, Debug, Default)]
struct VerbClock {
    start: Option<f64>,
    scale: f32,
    engaged: bool,
}
impl VerbClock {
    fn trigger(&mut self, now: f64, scale: f32) {
        if scale <= 0. {
            self.start = None;
            return;
        }
        if self
            .start
            .is_none_or(|s| now - s >= 1.2 * self.scale as f64)
        {
            self.start = Some(now);
            self.scale = scale;
        }
    }
    fn progress(
        &mut self,
        now: f64,
        engaged: bool,
        activate: bool,
        visible: bool,
        scale: f32,
    ) -> (f32, bool) {
        if !visible || scale <= 0. {
            self.start = None;
            self.engaged = false;
            return (0., false);
        }
        if activate || (engaged && !self.engaged) {
            self.trigger(now, scale);
        }
        self.engaged = engaged;
        if let Some(start) = self.start {
            let p = ((now - start) / (1.2 * self.scale.max(0.001)) as f64) as f32;
            if p >= 1. {
                self.start = None;
            } else {
                return (p.max(0.00001), true);
            }
        }
        (0., false)
    }
}
#[derive(Clone, Debug)]
struct Wave {
    starts: HashMap<&'static str, f64>,
    seconds: f64,
}
impl Wave {
    fn deadline(&self) -> f64 {
        self.starts
            .values()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            + self.seconds
    }
}

pub struct State {
    pub query: String,
    pub size: usize,
    pub stroke: f32,
    wave: Option<Wave>,
    copied: Option<(String, f64)>,
    status: String,
    tiles: HashMap<&'static str, Travel>,
    snapshots: HashMap<&'static str, PaintSnapshot>,
    clocks: HashMap<String, VerbClock>,
    height: Option<(f32, f32, f64, f32)>,
    active: bool,
    focus_search: bool,
    pending_focus: Option<&'static str>,
    count_from: f32,
    count_to: usize,
    count_started: f64,
    icon_size: f32,
    last_names: Vec<&'static str>,
    #[cfg(test)]
    copy_writer: Option<fn(&str) -> bool>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new(),
            size: 2,
            stroke: 1.6,
            wave: None,
            copied: None,
            status: String::new(),
            tiles: HashMap::new(),
            snapshots: HashMap::new(),
            clocks: HashMap::new(),
            height: None,
            active: false,
            focus_search: false,
            pending_focus: None,
            count_from: 82.,
            count_to: 82,
            count_started: 0.,
            icon_size: 24.,
            last_names: Vec::new(),
            #[cfg(test)]
            copy_writer: None,
        }
    }
}
impl State {
    pub fn filtered(&self) -> Vec<&'static str> {
        let query = self.query.trim().to_lowercase();
        icons::names()
            .iter()
            .copied()
            .filter(|name| {
                query.is_empty()
                    || name.contains(&query)
                    || category(name).to_lowercase().contains(&query)
            })
            .collect()
    }
    pub fn snippet(&self, name: &str) -> String {
        format!(
            "schematic_native::icons::paint(painter, \"{name}\", rect, color, {:.1}, progress);",
            self.stroke.clamp(1., 2.4)
        )
    }
    /// Success state is created only after the clipboard writer confirms success.
    /// The UI passes Runtime::copy; tests use a bounded in-memory writer.
    fn copy_with(&mut self, name: &str, now: f64, write: impl FnOnce(&str) -> bool) -> bool {
        let snippet = self.snippet(name);
        if write(&snippet) {
            self.copied = Some((name.into(), now));
            self.status = format!("Copied {name}.");
            true
        } else {
            self.copied = None;
            self.status = format!("Clipboard unavailable: {name}.");
            false
        }
    }
    fn copy_icon(&mut self, name: &str, rt: &Runtime) -> bool {
        #[cfg(test)]
        if let Some(write) = self.copy_writer {
            return self.copy_with(name, rt.now, write);
        }
        let ctx = rt.context().clone();
        self.copy_with(name, rt.now, |s| rt.copy(&ctx, s))
    }
    fn copy_visual(&self, name: &str, now: f64, reduced: bool) -> Option<(f32, f32)> {
        let (copied, start) = self.copied.as_ref()?;
        let age = now - *start;
        if copied != name || !(0. ..COPY_SECONDS).contains(&age) {
            return None;
        }
        if reduced {
            return Some((1., 0.));
        }
        let enter = motion::ease(Curve::Out, (age / 0.15).clamp(0., 1.) as f32);
        let exit = motion::ease(Curve::In, ((age - 1.8) / 0.4).clamp(0., 1.) as f32);
        Some((enter * (1. - exit), 4. * (1. - enter)))
    }
    fn columns(width: f32, viewport: f32) -> usize {
        if viewport <= 480. {
            3
        } else {
            ((width + 8.) / 120.).floor().max(1.) as usize
        }
    }
    fn targets(
        &self,
        width: f32,
        viewport: f32,
        names: &[&'static str],
    ) -> (Vec<(&'static str, Rect)>, f32, usize) {
        let cols = Self::columns(width, viewport);
        let tile_width = ((width - 8. * (cols - 1) as f32) / cols as f32).max(1.);
        let tile_height = if viewport <= 480. { 96. } else { 112. };
        let rects = names
            .iter()
            .enumerate()
            .map(|(i, &name)| {
                (
                    name,
                    Rect::from_min_size(
                        pos2(
                            (i % cols) as f32 * (tile_width + 8.),
                            (i / cols) as f32 * (tile_height + 8.),
                        ),
                        vec2(tile_width, tile_height),
                    ),
                )
            })
            .collect();
        let rows = names.len().div_ceil(cols);
        let height = if rows == 0 {
            112.
        } else {
            rows as f32 * (tile_height + 8.) - 8.
        };
        (rects, height, cols)
    }
    fn reconcile(&mut self, targets: &[(&'static str, Rect)], now: f64, scale: f32, width: f32) {
        let visible: HashSet<_> = targets.iter().map(|&(name, _)| name).collect();
        let first = self.tiles.is_empty();
        for tile in self.tiles.values_mut() {
            if (tile.bounds_width - width).abs() > 0.001 {
                let displayed = tile.value(now);
                tile.seconds = (tile.start + tile.seconds as f64 - now).max(0.) as f32;
                tile.from = displayed;
                tile.start = now;
                tile.bounds_width = width;
            }
        }
        for (&name, tile) in &mut self.tiles {
            if !visible.contains(name) && !tile.leaving {
                let before = tile.value(now);
                tile.retarget(
                    Visual {
                        rect: before.rect.translate(vec2(0., -4.)),
                        opacity: 0.,
                    },
                    now,
                    0.12 * scale,
                    true,
                );
            }
        }
        for &(name, rect) in targets {
            let target = Visual { rect, opacity: 1. };
            if let Some(tile) = self.tiles.get_mut(name) {
                tile.retarget(target, now, 0.47 * scale, false);
            } else {
                let from = if first || scale == 0. {
                    target
                } else {
                    Visual {
                        rect: Rect::from_center_size(rect.center(), rect.size() * 0.94),
                        opacity: 0.,
                    }
                };
                self.tiles.insert(
                    name,
                    Travel {
                        from,
                        to: target,
                        start: now,
                        seconds: if first { 0. } else { 0.47 * scale },
                        leaving: false,
                        bounds_width: width,
                    },
                );
            }
        }
        self.tiles.retain(|_, t| !t.leaving || t.moving(now));
        if let Some(wave) = &mut self.wave {
            wave.starts.retain(|name, _| visible.contains(name));
            if wave.starts.is_empty() {
                self.wave = None;
            }
        }
        self.clocks.retain(|key, _| {
            !key.starts_with("tile:") || visible.iter().any(|name| key == &format!("tile:{name}"))
        });
        self.snapshots
            .retain(|name, _| self.tiles.contains_key(name));
    }
    fn start_wave(&mut self, now: f64, scale: f32, names: &[&'static str]) {
        if scale <= 0. {
            self.wave = None;
            self.status = "Animations are reduced.".into();
            return;
        }
        let starts = names
            .iter()
            .filter(|&&name| name != "loader")
            .filter_map(|&name| {
                self.tiles.get(name).map(|t| {
                    let r = t.value(now).rect;
                    (
                        name,
                        now + (r.left() as f64 * 0.6 + r.top() as f64 * 0.9) / 1000. * scale as f64,
                    )
                })
            })
            .collect();
        self.wave = Some(Wave {
            starts,
            seconds: 1.2 * scale as f64,
        });
        self.status = "Playing icons.".into();
    }
    fn settle(&mut self) {
        self.wave = None;
        self.clocks.clear();
        self.tiles.retain(|_, t| !t.leaving);
        for t in self.tiles.values_mut() {
            t.settle();
        }
        if let Some((_, to, _, _)) = self.height {
            self.height = Some((to, to, 0., 0.));
        }
        self.icon_size = SIZES[self.size.min(3)];
    }
    fn grid_height(&mut self, target: f32, now: f64, scale: f32) -> f32 {
        let mut current = self
            .height
            .map(|(from, to, start, seconds)| {
                let p = if seconds == 0. {
                    1.
                } else {
                    ((now - start) / seconds as f64).clamp(0., 1.) as f32
                };
                egui::lerp(from..=to, motion::ease(Curve::Out, p))
            })
            .unwrap_or(target);
        if self
            .height
            .is_none_or(|(_, to, _, _)| (to - target).abs() > 0.1)
        {
            self.height = Some((current, target, now, 0.36 * scale));
        }
        if scale == 0. {
            current = target;
            self.height = Some((target, target, now, 0.));
        }
        current
    }
    fn stop(&mut self) {
        self.settle();
        self.copied = None;
        self.pending_focus = None;
        self.status.clear();
    }
}

fn tile_id(name: &str) -> Id {
    Id::new(("native-icons-tile", name))
}
fn live(response: &Response) {
    response.ctx.accesskit_node_builder(response.id, |n| {
        n.set_live(egui::accesskit::Live::Polite);
        n.set_live_atomic();
    });
}
fn ring(ui: &Ui, rt: &Runtime, r: &Response, radius: f32) {
    if r.has_focus() {
        ui.painter().rect_stroke(
            r.rect.shrink(1.),
            ui::rounded(radius * rt.tokens.radius),
            Stroke::new(2., rt.tokens.accent_ring),
            StrokeKind::Inside,
        );
    }
}
fn bounded(ui: &mut Ui) {
    let left = ui.cursor().min.x;
    let clip = ui.clip_rect();
    let remaining = (clip.right() - left).max(1.);
    if ui.available_width() > remaining {
        ui.set_max_width((remaining - (left - clip.left()).clamp(0., 56.)).max(1.));
    }
}
fn copy_status(ui: &mut Ui, rt: &mut Runtime, text: &str) {
    let r = ui.label(RichText::new(text).size(11.).color(rt.tokens.text3));
    r.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    live(&r);
    rt.record("icons-copy-status", &r);
}
fn stroke_control(ui: &mut Ui, rt: &mut Runtime, stroke: &mut f32) {
    ui.spacing_mut().slider_width = 80.;
    let response = ui.add(
        Slider::new(stroke, 1.0..=2.4)
            .step_by(0.1)
            .fixed_decimals(1)
            .text("Stroke"),
    );
    response.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::Slider,
            true,
            format!("Stroke width {stroke:.1}"),
        )
    });
    rt.record("icons-stroke", &response);
}
fn wave_control(ui: &mut Ui, rt: &mut Runtime) -> bool {
    let response = ui::button(
        ui,
        rt,
        "icons-wave",
        "Play all",
        Some("play"),
        ButtonKind::Secondary,
    );
    rt.record("icons-wave", &response);
    response.clicked()
}

impl State {
    fn toolbar(&mut self, ui: &mut Ui, rt: &mut Runtime) -> bool {
        let width = ui.available_width();
        let mut play = false;
        let search_width = if width >= 780. {
            (width - 555.).max(180.)
        } else {
            width
        };
        let (rect, _) = ui.allocate_exact_size(vec2(search_width, 40.), Sense::hover());
        ui.painter().rect(
            rect,
            ui::rounded(6. * rt.tokens.radius),
            rt.tokens.surface,
            Stroke::new(1., rt.tokens.border),
            StrokeKind::Inside,
        );
        let mut input_ui = ui.new_child(
            UiBuilder::new()
                .id_salt("icons-search-ui")
                .max_rect(rect.shrink2(vec2(36., 8.))),
        );
        let input = input_ui.add(
            TextEdit::singleline(&mut self.query)
                .id(Id::new("native-icons-search"))
                .hint_text("Search icons…")
                .font(FontId::proportional(13.))
                .frame(false)
                .desired_width((rect.width() - 92.).max(1.)),
        );
        input.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Search icons"));
        rt.record("icons-search", &input);
        if self.focus_search {
            input.request_focus();
            self.focus_search = false;
        }
        icons::paint(
            ui.painter(),
            "search",
            Rect::from_center_size(rect.left_center() + vec2(20., 0.), Vec2::splat(16.)),
            if input.has_focus() {
                rt.tokens.accent_text
            } else {
                rt.tokens.text3
            },
            1.5,
            0.,
        );
        if input.has_focus() {
            ui.painter().rect_stroke(
                rect,
                ui::rounded(6. * rt.tokens.radius),
                Stroke::new(2., rt.tokens.accent_ring),
                StrokeKind::Inside,
            );
        }
        let count = self.filtered().len();
        if count != self.count_to {
            let p = if rt.reduced() {
                1.
            } else {
                ((rt.now - self.count_started) / 0.28).clamp(0., 1.) as f32
            };
            self.count_from = egui::lerp(
                self.count_from..=self.count_to as f32,
                motion::ease(Curve::Out, p),
            );
            self.count_to = count;
            self.count_started = rt.now;
        }
        let p = if rt.reduced() {
            1.
        } else {
            ((rt.now - self.count_started) / 0.28).clamp(0., 1.) as f32
        };
        let value = egui::lerp(self.count_from..=count as f32, motion::ease(Curve::Out, p));
        ui.painter().text(
            rect.right_center() - vec2(12., 0.),
            Align2::RIGHT_CENTER,
            format!("{:.0}", value),
            FontId::monospace(11.),
            rt.tokens.text3,
        );
        let status = ui.interact(
            Rect::from_min_size(rect.right_top() - vec2(40., 0.), vec2(40., rect.height())),
            Id::new("native-icons-count"),
            Sense::hover(),
        );
        status.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Label, true, format!("{count} icons found"))
        });
        live(&status);
        rt.record("icons-count", &status);
        if p < 1. && ui.is_rect_visible(rect) {
            rt.repaint();
        }
        let controls = |ui: &mut Ui, rt: &mut Runtime, this: &mut Self| -> bool {
            let mut wave = false;
            let width = ui.available_width();
            // A slider is a composite widget: native wrapping cannot predict
            // its label/value width from the track alone. Assign rows first.
            ui.horizontal(|ui| {
                ui::segmented(
                    ui,
                    rt,
                    "icons-size",
                    &["16", "20", "24", "32"],
                    &mut this.size,
                );
                if width >= 350. {
                    stroke_control(ui, rt, &mut this.stroke);
                }
                if width >= 520. {
                    wave |= wave_control(ui, rt);
                }
            });
            if width < 350. {
                ui.horizontal(|ui| {
                    stroke_control(ui, rt, &mut this.stroke);
                    if width >= 300. {
                        wave |= wave_control(ui, rt);
                    }
                });
            }
            if width < 520. && !(300.0..350.0).contains(&width) {
                wave |= wave_control(ui, rt);
            }
            wave
        };
        if width >= 780. {
            let r = Rect::from_min_size(
                rect.right_top() + vec2(12., 0.),
                vec2(width - rect.width() - 12., 40.),
            );
            let mut row = ui.new_child(UiBuilder::new().id_salt("icons-controls").max_rect(r));
            play |= controls(&mut row, rt, self);
        } else {
            ui.add_space(8.);
            play |= controls(ui, rt, self);
        }
        ui.add_space(12.);
        copy_status(ui, rt, &self.status);
        ui.add_space(12.);
        play
    }
    fn paint_tile(
        &mut self,
        ui: &mut Ui,
        rt: &mut Runtime,
        name: &'static str,
        rect: Rect,
        response: &Response,
        visible: bool,
    ) {
        let engaged = response.hovered() || response.has_focus();
        let hover = rt.animate(
            ("icons-tile-hover", name),
            if engaged { 1. } else { 0. },
            0.14,
            Curve::Out,
        );
        ui.painter().rect(
            rect,
            ui::rounded(10. * rt.tokens.radius),
            ui::mix(rt.tokens.surface, rt.tokens.accent_soft, hover),
            Stroke::new(
                1.,
                ui::mix(rt.tokens.border_soft, rt.tokens.accent_ring, hover),
            ),
            StrokeKind::Inside,
        );
        ring(ui, rt, response, 10.);
        let key = format!("tile:{name}");
        let (mut progress, moving) = self.clocks.entry(key).or_default().progress(
            rt.now,
            engaged,
            response.clicked(),
            visible,
            rt.motion_scale(),
        );
        if moving {
            rt.repaint();
        }
        if name == "loader" && !rt.reduced() && visible {
            progress = ((rt.now % (0.8 * rt.motion_scale() as f64))
                / (1.2 * rt.motion_scale() as f64)) as f32;
            rt.repaint();
        }
        let size = self.icon_size;
        icons::paint(
            ui.painter(),
            name,
            Rect::from_center_size(rect.center() - vec2(0., 13.), Vec2::splat(size)),
            ui::mix(rt.tokens.text2, rt.tokens.accent_text, hover),
            self.stroke,
            progress,
        );
        let mut job = egui::text::LayoutJob::simple(
            name.into(),
            FontId::monospace(10.),
            rt.tokens.text3,
            (rect.width() - 12.).max(1.),
        );
        job.wrap.max_rows = 2;
        job.wrap.break_anywhere = true;
        let galley = ui.painter().layout_job(job);
        ui.painter().galley(
            pos2(rect.center().x - galley.size().x / 2., rect.bottom() - 27.),
            galley,
            rt.tokens.text3,
        );
        if let Some((opacity, dy)) = self.copy_visual(name, rt.now, rt.reduced()) {
            let badge = Rect::from_min_max(
                pos2(rect.left() + 4., rect.bottom() - 18. + dy),
                pos2(rect.right() - 4., rect.bottom() - 4. + dy),
            );
            ui.painter().rect_filled(
                badge,
                ui::rounded(3. * rt.tokens.radius),
                rt.tokens.surface.gamma_multiply(opacity),
            );
            ui.painter().text(
                badge.center(),
                Align2::CENTER_CENTER,
                "Copied",
                ui::strong_font(ui.ctx(), 10.),
                rt.tokens.success_text.gamma_multiply(opacity),
            );
            let status = ui.interact(
                badge,
                Id::new(("native-icons-copied", name)),
                Sense::hover(),
            );
            status.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Label, true, format!("Copied {name}"))
            });
            live(&status);
            rt.record(&format!("icon-copied:{name}"), &status);
            if rt.reduced() {
                if let Some((_, start)) = &self.copied {
                    rt.repaint_after((start + COPY_SECONDS - rt.now).max(0.));
                }
            } else {
                rt.repaint();
            }
        }
    }
    fn spatial_key(
        &mut self,
        ui: &Ui,
        names: &[&'static str],
        index: usize,
        cols: usize,
        response: &Response,
    ) {
        if !response.has_focus() {
            return;
        }
        for key in [
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::ArrowUp,
            Key::ArrowDown,
            Key::Home,
            Key::End,
        ] {
            if !ui.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                continue;
            }
            let next = match key {
                Key::ArrowLeft => index.saturating_sub(1),
                Key::ArrowRight => (index + 1).min(names.len() - 1),
                Key::ArrowUp => index.saturating_sub(cols),
                Key::ArrowDown => (index + cols).min(names.len() - 1),
                Key::Home => 0,
                _ => names.len() - 1,
            };
            ui.memory_mut(|m| {
                m.move_focus(egui::FocusDirection::None);
                m.request_focus(tile_id(names[next]));
            });
            self.pending_focus = Some(names[next]);
        }
    }
    fn gallery(&mut self, ui: &mut Ui, rt: &mut Runtime, play: bool) {
        let names = self.filtered();
        let width = ui.available_width();
        let viewport = ui.ctx().content_rect().width();
        let (targets, height, cols) = self.targets(width, viewport, &names);
        if names != self.last_names {
            if let Some(focused) = ui.memory(|m| m.focused())
                && self.last_names.iter().any(|name| tile_id(name) == focused)
                && !names.iter().any(|name| tile_id(name) == focused)
            {
                ui.memory_mut(|m| m.request_focus(Id::new("native-icons-search")));
            }
            self.last_names = names.clone();
        }
        self.reconcile(&targets, rt.now, rt.motion_scale(), width);
        if play {
            self.start_wave(rt.now, rt.motion_scale(), &names);
        }
        let previous_height = self.grid_height(height, rt.now, rt.motion_scale());
        let (grid, _) =
            ui.allocate_exact_size(vec2(width, previous_height.max(0.)), Sense::hover());
        let clip = grid.expand(2.).intersect(ui.clip_rect());
        if let Some((from, to, start, seconds)) = self.height
            && from != to
            && rt.now < start + seconds as f64
            && ui.is_rect_visible(grid)
        {
            rt.repaint();
        }
        let ghosts: Vec<_> = self
            .tiles
            .iter()
            .filter(|(_, t)| t.leaving)
            .map(|(&name, t)| (name, t.value(rt.now)))
            .collect();
        for (name, visual) in ghosts {
            if let Some(snapshot) = self.snapshots.get(name) {
                let offset = grid.min.to_vec2() + visual.rect.min.to_vec2();
                let alpha = (visual.opacity / snapshot.opacity.max(0.001)).clamp(0., 1.);
                if clip.intersects(visual.rect.translate(grid.min.to_vec2())) {
                    for cached in &snapshot.shapes {
                        let mut shape = cached.shape.clone();
                        shape.transform(emath::TSTransform::from_translation(offset));
                        epaint::shape_transform::adjust_colors(&mut shape, move |c| {
                            *c = c.gamma_multiply(alpha)
                        });
                        ui.painter()
                            .with_clip_rect(cached.clip_rect.translate(offset).intersect(clip))
                            .add(shape);
                    }
                    rt.repaint();
                }
            }
        }
        self.icon_size = rt.animate(
            "native-icons-size-paint",
            SIZES[self.size.min(3)],
            0.26,
            Curve::Out,
        );
        for (i, &name) in names.iter().enumerate() {
            let visual = self.tiles[name].value(rt.now);
            let rect = visual.rect.translate(grid.min.to_vec2());
            let mut tile =
                ui.new_child(UiBuilder::new().id(("icons-tile-ui", name)).max_rect(rect));
            tile.set_clip_rect(clip.intersect(rect));
            tile.set_opacity(visual.opacity);
            let response = tile.interact(rect, tile_id(name), Sense::click());
            response.widget_info(|| {
                WidgetInfo::labeled(
                    WidgetType::Button,
                    tile.is_enabled(),
                    format!("Copy {name}"),
                )
            });
            tile.ctx().accesskit_node_builder(response.id, |node| {
                node.set_description(format!(
                    "{} icon. Copy Rust usage. Arrow keys move through the icon grid.",
                    category(name)
                ));
            });
            rt.record(&format!("icon-tile:{name}"), &response);
            if response.clicked() {
                response.request_focus();
                let snippet = self.snippet(name);
                let copied = self.copy_icon(name, rt);
                if copied {
                    rt.toast("Copied to clipboard", &snippet, "success");
                } else {
                    rt.toast("Clipboard unavailable", &snippet, "warning");
                }
            }
            self.spatial_key(&tile, &names, i, cols, &response);
            if self.pending_focus == Some(name) {
                response.request_focus();
                response.scroll_to_me(Some(Align::Center));
                self.pending_focus = None;
                rt.repaint();
            }
            let visible = clip.intersects(rect) && rect.is_positive();
            // Fire each captured wave cue once. Offscreen cues are discarded;
            // they cannot resume halfway through when the user scrolls back.
            if let Some(wave) = &mut self.wave
                && let Some(&start) = wave.starts.get(name)
            {
                if rt.now >= start {
                    wave.starts.remove(name);
                    if visible {
                        self.clocks
                            .entry(format!("tile:{name}"))
                            .or_default()
                            .trigger(rt.now, rt.motion_scale());
                    }
                } else if visible {
                    rt.repaint_after(start - rt.now);
                }
            }
            if visible {
                let layer = tile.layer_id();
                let start = tile
                    .ctx()
                    .graphics(|g| g.get(layer).map(|p| p.all_entries().len()).unwrap_or(0));
                self.paint_tile(&mut tile, rt, name, rect, &response, visible);
                let shapes = tile.ctx().graphics(|g| {
                    g.get(layer)
                        .map(|p| {
                            p.all_entries()
                                .skip(start)
                                .cloned()
                                .map(|mut s| {
                                    s.shape.transform(emath::TSTransform::from_translation(
                                        -rect.min.to_vec2(),
                                    ));
                                    s.clip_rect = s.clip_rect.translate(-rect.min.to_vec2());
                                    s
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                });
                self.snapshots.insert(
                    name,
                    PaintSnapshot {
                        shapes,
                        opacity: visual.opacity,
                    },
                );
                if self.tiles[name].moving(rt.now) {
                    rt.repaint();
                }
            } else if let Some(clock) = self.clocks.get_mut(&format!("tile:{name}")) {
                clock.start = None;
                clock.engaged = false;
            }
        }
        if names.is_empty() {
            let r = ui.interact(grid, Id::new("native-icons-empty"), Sense::hover());
            let message = format!("No icon matches “{}”", self.query);
            r.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &message));
            live(&r);
            rt.record("icons-empty", &r);
            ui.painter().text(
                grid.center() - vec2(0., 10.),
                Align2::CENTER_CENTER,
                message,
                FontId::proportional(13.),
                rt.tokens.text3,
            );
            ui.painter().text(
                grid.center() + vec2(0., 15.),
                Align2::CENTER_CENTER,
                "Try another name or category.",
                FontId::proportional(11.),
                rt.tokens.text3,
            );
        }
        if self
            .wave
            .as_ref()
            .is_some_and(|w| w.starts.is_empty() || rt.now >= w.deadline())
        {
            self.wave = None;
        }
        if self
            .copied
            .as_ref()
            .is_some_and(|(_, start)| rt.now - start >= COPY_SECONDS)
        {
            self.copied = None;
        }
    }
    fn play_button(
        &mut self,
        ui: &mut Ui,
        rt: &mut Runtime,
        id: &str,
        label: &str,
        name: &str,
        kind: ButtonKind,
    ) -> Response {
        let font = FontId::proportional(13.);
        let text_width = ui
            .painter()
            .layout_no_wrap(label.into(), font.clone(), rt.tokens.text)
            .size()
            .x;
        let (rect, response) = ui.allocate_exact_size(
            vec2(
                if label.is_empty() {
                    34.
                } else {
                    text_width + 50.
                },
                34.,
            ),
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                ui.is_enabled(),
                format!("Play {name} icon"),
            )
        });
        if response.clicked() {
            response.request_focus();
        }
        let engaged = response.hovered() || response.has_focus();
        let hover = rt.animate(
            (id, "hover"),
            if engaged { 1. } else { 0. },
            0.14,
            Curve::Out,
        );
        let (fill, color, border) = match kind {
            ButtonKind::Primary => (
                ui::mix(rt.tokens.accent, rt.tokens.accent_hover, hover),
                rt.tokens.accent_contrast,
                rt.tokens.accent,
            ),
            ButtonKind::Secondary => (
                ui::mix(rt.tokens.surface, rt.tokens.surface2, hover),
                rt.tokens.text,
                rt.tokens.border,
            ),
            _ => (
                rt.tokens.surface2.gamma_multiply(hover),
                rt.tokens.text2,
                Color32::TRANSPARENT,
            ),
        };
        ui.painter().rect(
            rect,
            ui::rounded(6. * rt.tokens.radius),
            fill,
            Stroke::new(1., border),
            StrokeKind::Inside,
        );
        ring(ui, rt, &response, 6.);
        let visible = ui.is_rect_visible(rect);
        let (p, moving) = self
            .clocks
            .entry(format!("usage:{id}"))
            .or_default()
            .progress(
                rt.now,
                engaged,
                response.clicked(),
                visible,
                rt.motion_scale(),
            );
        if moving {
            rt.repaint();
        }
        icons::paint(
            ui.painter(),
            name,
            Rect::from_center_size(
                if label.is_empty() {
                    rect.center()
                } else {
                    rect.left_center() + vec2(20., 0.)
                },
                Vec2::splat(16.),
            ),
            color,
            1.6,
            p,
        );
        if !label.is_empty() {
            ui.painter().text(
                rect.left_center() + vec2(38., 0.),
                Align2::LEFT_CENTER,
                label,
                font,
                color,
            );
        }
        rt.record(id, &response);
        response
    }
    fn verbs(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui::section(ui, rt, "ic-verbs", "02", "Animations", "");
        let width = ui.available_width();
        let cols = if width >= 800. {
            3
        } else if width >= 500. {
            2
        } else {
            1
        };
        let gap = 16.;
        let cell = (width - gap * (cols - 1) as f32) / cols as f32;
        for row in VERBS.chunks(cols) {
            let height = row
                .iter()
                .map(|(_, _, desc)| {
                    ui.painter()
                        .layout(
                            desc.to_string(),
                            FontId::proportional(12.),
                            rt.tokens.text2,
                            (cell - 32.).max(1.),
                        )
                        .size()
                        .y
                        + 172.
                })
                .fold(0., f32::max);
            let (allocation, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            for (i, &(name, title, desc)) in row.iter().enumerate() {
                let rect = Rect::from_min_size(
                    allocation.min + vec2(i as f32 * (cell + gap), 0.),
                    vec2(cell, height),
                );
                let response =
                    ui.interact(rect, Id::new(("native-icon-verb", name)), Sense::click());
                response.widget_info(|| {
                    WidgetInfo::labeled(
                        WidgetType::Button,
                        ui.is_enabled(),
                        format!("Play {name}: {title}"),
                    )
                });
                ui.ctx()
                    .accesskit_node_builder(response.id, |n| n.set_description(desc));
                if response.clicked() {
                    response.request_focus();
                }
                let engaged = response.hovered() || response.has_focus();
                let (p, moving) = self
                    .clocks
                    .entry(format!("verb:{name}"))
                    .or_default()
                    .progress(
                        rt.now,
                        engaged,
                        response.clicked(),
                        ui.is_rect_visible(rect),
                        rt.motion_scale(),
                    );
                if moving {
                    rt.repaint();
                }
                ui.painter().rect(
                    rect,
                    ui::rounded(12. * rt.tokens.radius),
                    rt.tokens.surface,
                    Stroke::new(
                        1.,
                        if engaged {
                            rt.tokens.accent_ring
                        } else {
                            rt.tokens.border
                        },
                    ),
                    StrokeKind::Inside,
                );
                ring(ui, rt, &response, 12.);
                let stage = Rect::from_min_size(rect.min + vec2(16., 16.), vec2(cell - 32., 96.));
                ui.painter().rect_filled(
                    stage,
                    ui::rounded(6. * rt.tokens.radius),
                    rt.tokens.surface2,
                );
                icons::paint(
                    ui.painter(),
                    name,
                    Rect::from_center_size(stage.center(), Vec2::splat(40.)),
                    rt.tokens.accent_text,
                    1.6,
                    p,
                );
                ui.painter().text(
                    rect.min + vec2(16., 126.),
                    Align2::LEFT_TOP,
                    title,
                    ui::strong_font(ui.ctx(), 16.),
                    rt.tokens.text,
                );
                let galley = ui.painter().layout(
                    desc.into(),
                    FontId::proportional(12.),
                    rt.tokens.text2,
                    (cell - 32.).max(1.),
                );
                ui.painter()
                    .galley(rect.min + vec2(16., 151.), galley, rt.tokens.text2);
                rt.record(&format!("icon-verb:{name}"), &response);
            }
            ui.add_space(12.);
        }
    }
    fn usage(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui::section(ui, rt, "ic-usage", "03", "Usage", "");
        ui::demo(
            ui,
            rt,
            "icons-0",
            "Animated icons",
            "Buttons, links and focusable controls play their icons on hover, focus and touch.",
            120.,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    self.play_button(ui, rt, "icon-send", "Send", "send", ButtonKind::Primary);
                    self.play_button(
                        ui,
                        rt,
                        "icon-export",
                        "Export",
                        "download",
                        ButtonKind::Secondary,
                    );
                    self.play_button(ui, rt, "icon-notifications", "", "bell", ButtonKind::Ghost)
                        .on_hover_text("Notifications");
                    self.play_button(ui, rt, "icon-delete", "", "trash", ButtonKind::Ghost)
                        .on_hover_text("Delete");
                });
            },
        );
        ui.add_space(12.);
        ui::demo(
            ui,
            rt,
            "icons-1",
            "Snippet",
            "Call the native painter with an icon name, bounds, color, stroke and progress.",
            100.,
            |ui, _| {
                ScrollArea::horizontal().id_salt("native-icon-snippet").show(ui,|ui| {ui.add(Label::new(RichText::new("schematic_native::icons::paint(\n    painter, \"bell\", rect, color, 1.6, progress,\n);").monospace().size(12.)).extend());});
            },
        );
    }
}
impl Page for State {
    fn show(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        if !self.active {
            self.enter(rt);
        }
        bounded(ui);
        ui.spacing_mut().item_spacing = vec2(8., 8.);
        if ui.input(|i| i.viewport().minimized.unwrap_or(false)) {
            self.stop();
            return;
        }
        self.size = self.size.min(3);
        self.stroke = if self.stroke.is_finite() {
            self.stroke.clamp(1., 2.4)
        } else {
            1.6
        };
        if rt.reduced() {
            self.settle();
        }
        ui::page_head(
            ui,
            rt,
            "sparkles",
            "Foundations",
            "Icons",
            "Search icons. Select one to copy its Rust usage.",
        );
        ui::section(ui, rt, "ic-gallery", "01", "Gallery", "");
        let play = self.toolbar(ui, rt);
        self.gallery(ui, rt, play);
        self.verbs(ui, rt);
        self.usage(ui, rt);
    }
    fn enter(&mut self, _: &mut Runtime) {
        self.active = true;
    }
    fn leave(&mut self, _: &mut Runtime) {
        self.active = false;
        self.stop();
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    fn frame(
        s: &mut State,
        rt: &mut Runtime,
        ctx: &Context,
        time: f64,
        width: f32,
        height: f32,
        events: Vec<Event>,
    ) -> FullOutput {
        let mut released = ctx.input(|i| {
            i.keys_down
                .iter()
                .map(|&key| Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                })
                .collect::<Vec<_>>()
        });
        released.extend(events);
        ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(width, height))),
                time: Some(time),
                events: released,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                CentralPanel::default()
                    .frame(Frame::new().inner_margin(16).fill(rt.tokens.bg))
                    .show(ctx, |ui| {
                        ScrollArea::vertical()
                            .id_salt("native-icons-test-scroll")
                            .show(ui, |ui| {
                                ui.set_max_width(1080.);
                                s.show(ui, rt);
                            });
                    });
            },
        )
    }
    fn rect(rt: &Runtime, id: &str) -> Rect {
        let r = rt
            .records
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("Missing {id}"));
        Rect::from_min_max(pos2(r.rect[0], r.rect[1]), pos2(r.rect[2], r.rect[3]))
    }
    fn key(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }
    #[test]
    fn icon_search_matches_exact_reference_names_and_categories() {
        let mut s = State::default();
        assert_eq!(s.filtered().len(), 82);
        s.query = " bell ".into();
        assert_eq!(s.filtered(), ["bell"]);
        s.query = "DaTa".into();
        assert_eq!(s.filtered(), ["chart-bar", "trending", "wave", "pie"]);
        s.query = "Arrows".into();
        assert_eq!(s.filtered().len(), 11);
        assert_eq!(category("arrow-down"), "Other");
        s.query = "Other".into();
        assert_eq!(s.filtered().len(), 12);
        s.query = "unmatched-915".into();
        assert!(s.filtered().is_empty());
    }
    #[test]
    fn filtering_resize_and_reversal_retarget_from_the_displayed_rect() {
        let mut s = State::default();
        let names = s.filtered();
        let (targets, _, _) = s.targets(800., 1440., &names);
        s.reconcile(&targets, 0., 1., 800.);
        s.query = "Objects".into();
        let (next, _, _) = s.targets(800., 1440., &s.filtered());
        s.reconcile(&next, 1., 1., 800.);
        let bell = s.tiles["bell"].value(1.1);
        assert!(s.tiles["bell"].moving(1.1));
        let (small, _, cols) = s.targets(288., 320., &s.filtered());
        assert_eq!(cols, 3);
        s.reconcile(&small, 1.1, 1., 288.);
        let resized = s.tiles["bell"].value(1.1).rect;
        let expected_left = bell.rect.left().clamp(0., 288. - bell.rect.width());
        assert!((resized.left() - expected_left).abs() < 0.001);
        assert!((resized.top() - bell.rect.top()).abs() < 0.001);
        assert!((resized.width() - bell.rect.width()).abs() < 0.001);
        assert!(resized.left() >= 0. && resized.right() <= 288.);
        s.query.clear();
        let (all, _, _) = s.targets(800., 1440., &s.filtered());
        let old = s.tiles["bell"].value(1.2);
        s.reconcile(&all, 1.2, 1., 800.);
        let reversed = s.tiles["bell"].value(1.2).rect;
        assert!(
            reversed.min.distance(old.rect.min) < 0.001
                && reversed.max.distance(old.rect.max) < 0.001
        );
        let finished = s.tiles["bell"].value(3.).rect;
        let expected = targets.iter().find(|(n, _)| *n == "bell").unwrap().1;
        assert!(
            finished.min.distance(expected.min) < 0.001
                && finished.max.distance(expected.max) < 0.001
        );
        assert_eq!(tile_id("bell"), tile_id("bell"));
        assert_ne!(tile_id("bell"), tile_id("send"));
    }
    #[test]
    fn copied_feedback_requires_a_successful_writer_and_is_visible_when_reduced() {
        let mut s = State::default();
        s.stroke = 2.;
        let mut received = String::new();
        assert!(s.copy_with("bell", 1., |text| {
            received = text.into();
            true
        }));
        assert!(received.contains("\"bell\""));
        assert!(received.contains("2.0"));
        assert_eq!(s.status, "Copied bell.");
        assert!(
            s.copy_visual("bell", 1.1, false)
                .is_some_and(|(a, y)| a > 0. && a < 1. && y > 0.)
        );
        assert_eq!(s.copy_visual("bell", 1.1, true), Some((1., 0.)));
        assert!(s.copy_visual("send", 1.2, false).is_none());
        assert!(s.copy_visual("bell", 3.21, false).is_none());
        assert!(!s.copy_with("bell", 4., |_| false));
        assert!(s.copied.is_none());
        assert_eq!(s.status, "Clipboard unavailable: bell.");
    }
    #[test]
    fn native_keyboard_copy_paints_and_announces_only_confirmed_success() {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State {
            query: "bell".into(),
            copy_writer: Some(|_| true),
            ..Default::default()
        };
        let _ = frame(&mut s, &mut rt, &ctx, 0., 1000., 1000., vec![]);
        ctx.memory_mut(|m| m.request_focus(tile_id("bell")));
        let _ = frame(
            &mut s,
            &mut rt,
            &ctx,
            1.,
            1000.,
            1000.,
            vec![key(Key::Enter)],
        );
        let out = frame(&mut s, &mut rt, &ctx, 1.2, 1000., 1000., vec![]);
        assert_eq!(
            s.copied.as_ref().map(|(name, _)| name.as_str()),
            Some("bell")
        );
        assert!(rt.records.iter().any(|r| r.id == "icon-copied:bell"));
        assert!(
            out.shapes
                .iter()
                .any(|s| matches!(&s.shape,Shape::Text(t) if t.galley.job.text=="Copied"))
        );
        let tree = out.platform_output.accesskit_update.unwrap();
        assert!(
            tree.nodes
                .iter()
                .any(|(_, n)| n.role() == egui::accesskit::Role::Label
                    && n.value() == Some("Copied bell")
                    && n.live() == Some(egui::accesskit::Live::Polite))
        );
        s.copy_writer = Some(|_| false);
        let _ = frame(
            &mut s,
            &mut rt,
            &ctx,
            1.3,
            1000.,
            1000.,
            vec![key(Key::Space)],
        );
        assert!(s.copied.is_none());
        assert!(!rt.records.iter().any(|r| r.id.starts_with("icon-copied:")));
        assert_eq!(s.status, "Clipboard unavailable: bell.");
        assert_eq!(rt.toasts.last().unwrap().title, "Clipboard unavailable");
    }
    #[test]
    fn wave_schedule_is_captured_once_and_hidden_names_are_removed() {
        let mut s = State::default();
        let names = s.filtered();
        let (targets, _, _) = s.targets(800., 1440., &names);
        s.reconcile(&targets, 0., 1., 800.);
        s.start_wave(1., 1., &names);
        let before = s.wave.as_ref().unwrap().starts.clone();
        let (small, _, _) = s.targets(288., 320., &names);
        s.reconcile(&small, 1.1, 1., 288.);
        assert_eq!(s.wave.as_ref().unwrap().starts, before);
        s.query = "bell".into();
        let (one, _, _) = s.targets(288., 320., &s.filtered());
        s.reconcile(&one, 1.2, 1., 288.);
        assert_eq!(s.wave.as_ref().unwrap().starts.len(), 1);
        assert!(s.wave.as_ref().unwrap().starts.contains_key("bell"));
        s.start_wave(2., 0., &["bell"]);
        assert!(s.wave.is_none());
    }
    #[test]
    fn semantic_verbs_do_not_restart_midflight_but_activation_replays_after_finish() {
        let mut c = VerbClock::default();
        let (first, running) = c.progress(1., true, false, true, 1.);
        assert!(first > 0. && running);
        let p = c.progress(1.3, true, true, true, 1.).0;
        assert!((p - 0.25).abs() < 0.001);
        assert_eq!(c.progress(2.3, true, false, true, 1.), (0., false));
        assert!(c.progress(2.4, true, true, true, 1.).1);
        assert_eq!(c.progress(2.5, true, false, false, 1.), (0., false));
        assert!(c.start.is_none());
    }
    #[test]
    fn native_layout_is_bounded_and_tiles_have_stable_accessible_names() {
        let ctx = Context::default();
        ctx.enable_accesskit();
        ctx.style_mut(|s| {
            s.spacing.item_spacing = vec2(12., 12.);
            s.spacing.interact_size = vec2(34., 34.);
        });
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        for (i, width) in [1440., 1000., 390., 320.].into_iter().enumerate() {
            let out = frame(&mut s, &mut rt, &ctx, i as f64, width, 1100., vec![]);
            assert!(!ctx.tessellate(out.shapes, out.pixels_per_point).is_empty());
            for r in rt.records.iter().filter(|r| {
                r.id.starts_with("icon-tile:")
                    || r.id.starts_with("icon-verb:")
                    || r.id == "icons-search"
                    || r.id == "icons-stroke"
                    || r.id == "icons-wave"
            }) {
                assert!(
                    r.rect[0] >= -0.5 && r.rect[2] <= width + 0.5,
                    "overflow {width}: {} {:?}",
                    r.id,
                    r.rect
                );
            }
            let tree = out.platform_output.accesskit_update.unwrap();
            let labels: Vec<_> = tree.nodes.iter().filter_map(|(_, n)| n.label()).collect();
            assert!(labels.contains(&"Copy bell"));
            assert!(labels.contains(&"Copy send"));
            assert!(
                tree.nodes
                    .iter()
                    .any(|(_, n)| n.live() == Some(egui::accesskit::Live::Polite))
            );
        }
    }
    #[test]
    fn native_keyboard_grid_navigation_and_filter_focus_remain_on_real_icons() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let _ = frame(&mut s, &mut rt, &ctx, 0., 1440., 1600., vec![]);
        ctx.memory_mut(|m| m.request_focus(tile_id("arrow-right")));
        let _ = frame(
            &mut s,
            &mut rt,
            &ctx,
            1.,
            1440.,
            1600.,
            vec![key(Key::ArrowRight)],
        );
        assert!(ctx.memory(|m| m.has_focus(tile_id("arrow-left"))));
        let _ = frame(
            &mut s,
            &mut rt,
            &ctx,
            1.1,
            1440.,
            1600.,
            vec![key(Key::End)],
        );
        assert!(ctx.memory(|m| m.has_focus(tile_id(icons::names().last().unwrap()))));
        s.query = "bell".into();
        let _ = frame(&mut s, &mut rt, &ctx, 2., 1440., 1600., vec![]);
        assert!(ctx.memory(|m| m.has_focus(Id::new("native-icons-search"))));
        assert_eq!(s.filtered(), ["bell"]);
        let before = rect(&rt, "icon-tile:bell");
        let _ = frame(&mut s, &mut rt, &ctx, 2.2, 390., 1600., vec![]);
        assert!(s.tiles["bell"].moving(2.2));
        assert!((s.tiles["bell"].from.rect.width() - before.width()).abs() < 0.001);
    }
    #[test]
    fn route_leave_settles_owned_motion_and_retains_user_configuration() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let _ = frame(&mut s, &mut rt, &ctx, 0., 1440., 1000., vec![]);
        s.query = "bell".into();
        s.size = 3;
        s.stroke = 2.1;
        let _ = frame(&mut s, &mut rt, &ctx, 1., 1440., 1000., vec![]);
        s.copy_with("bell", 1.1, |_| true);
        s.start_wave(1.1, 1., &["bell"]);
        s.leave(&mut rt);
        assert!(s.wave.is_none() && s.copied.is_none() && s.clocks.is_empty());
        assert!(s.tiles.values().all(|t| !t.moving(1.1) && !t.leaving));
        s.enter(&mut rt);
        assert_eq!(s.query, "bell");
        assert_eq!(s.size, 3);
        assert_eq!(s.stroke, 2.1);
    }
}
