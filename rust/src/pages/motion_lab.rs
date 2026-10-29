//! Easing and spring workbenches. All drawing and timing are native Rust.
use crate::{
    easing::Spring,
    icons,
    motion::{self, Curve},
    runtime::{Page, Route, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{self, Align2, FontId, Rect, Sense, Shape, Stroke, StrokeKind, pos2, vec2};

const CURVES: [(Curve, &str, &str); 9] = [
    (Curve::Out, "Respond", "--ease-out"),
    (Curve::OutExpo, "Enter", "--ease-out-expo"),
    (Curve::In, "Exit", "--ease-in"),
    (Curve::InOut, "Travel", "--ease-in-out"),
    (Curve::Overshoot, "Confirm", "--ease-overshoot"),
    (Curve::SpringSmooth, "Smooth spring", "--spring-smooth"),
    (Curve::SpringSnappy, "Snappy spring", "--spring-snappy"),
    (Curve::SpringBouncy, "Bouncy spring", "--spring-bouncy"),
    (Curve::Linear, "Linear reference", "linear"),
];
const ROUTES: [Route; 11] = [
    Route::Overview,
    Route::Foundations,
    Route::Motion,
    Route::Icons,
    Route::Components,
    Route::Primitives,
    Route::Navigation,
    Route::Forms,
    Route::Data,
    Route::Overlays,
    Route::Workspace,
];

fn range(ui: &mut egui::Ui, rt: &Runtime, slider: egui::Slider<'_>, width: f32) -> egui::Response {
    ui.scope(|ui| {
        ui.spacing_mut().slider_width = width;
        let visuals = ui.visuals_mut();
        visuals.widgets.inactive.bg_fill = rt.tokens.surface3;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, rt.tokens.border_strong);
        visuals.widgets.hovered.bg_fill = rt.tokens.accent;
        visuals.widgets.active.bg_fill = rt.tokens.accent;
        visuals.selection.bg_fill = rt.tokens.accent;
        ui.add(slider.trailing_fill(true))
    })
    .inner
}

pub struct State {
    pub curve_index: usize,
    pub duration_ms: u32,
    pub progress: f32,
    curve_start: Option<f64>,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
    pub spring: Spring,
    spring_start: Option<f64>,
    spring_progress: f64,
    list_start: Option<f64>,
    continuity: usize,
    continuity_start: Option<f64>,
    continuity_direction: f32,
    graph_curve: usize,
    graph_samples: Vec<f32>,
    cache: Vec<(Route, String)>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            curve_index: 0,
            duration_ms: 600,
            progress: 0.0,
            curve_start: None,
            stiffness: 300.0,
            damping: 26.0,
            mass: 1.0,
            spring: Spring::simulate(300.0, 26.0, 1.0),
            spring_start: None,
            spring_progress: 0.0,
            list_start: None,
            continuity: 0,
            continuity_start: None,
            continuity_direction: 1.0,
            graph_curve: usize::MAX,
            graph_samples: Vec::new(),
            cache: Vec::new(),
        }
    }
}
impl State {
    pub fn scrub(&mut self, t: f32) {
        self.curve_start = None;
        self.progress = t.clamp(0.0, 1.0);
    }
    pub fn replay_curve(&mut self, now: f64) {
        self.progress = 0.0;
        self.curve_start = Some(now);
    }
    pub fn replay_spring(&mut self, now: f64) {
        self.spring = Spring::simulate(self.stiffness, self.damping, self.mass);
        self.spring_start = Some(now);
        self.spring_progress = 0.0;
    }
    pub fn curve_value(&self) -> f32 {
        motion::ease(CURVES[self.curve_index].0, self.progress)
    }
    pub fn spring_export(&self) -> String {
        let samples = self
            .spring
            .values
            .iter()
            .map(|v| {
                let value = format!("{v:.4}").parse::<f64>().unwrap_or(0.0);
                value.to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "transition: transform {}ms linear({samples});\n/* Multiply duration by your motion preference; suppress spatial motion when reduced. */",
            self.spring.duration_ms
        )
    }
    fn graph(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        if self.graph_curve != self.curve_index {
            self.graph_curve = self.curve_index;
            self.graph_samples = (0..=120)
                .map(|i| motion::ease(CURVES[self.curve_index].0, i as f32 / 120.0))
                .collect();
        }
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            vec2(width, (width * 220.0 / 360.0).min(330.0)),
            Sense::hover(),
        );
        rt.record("motion-graph", &response);
        let map = |x: f32, y: f32| {
            pos2(
                rect.left() + x / 360.0 * rect.width(),
                rect.top() + y / 220.0 * rect.height(),
            )
        };
        let p = ui.painter();
        for y in [40.0, 110.0, 180.0] {
            p.line_segment(
                [map(30.0, y), map(330.0, y)],
                Stroke::new(1.0_f32, rt.tokens.border_soft),
            );
        }
        for x in [30.0, 180.0, 330.0] {
            p.line_segment(
                [map(x, 40.0), map(x, 180.0)],
                Stroke::new(1.0_f32, rt.tokens.border_soft),
            );
        }
        let a = map(30.0, 180.0);
        let b = map(330.0, 40.0);
        p.add(Shape::dashed_line(
            &[a, b],
            Stroke::new(1.0_f32, rt.tokens.border_strong),
            3.0,
            5.0,
        ));
        p.add(Shape::line(
            self.graph_samples
                .iter()
                .enumerate()
                .map(|(i, v)| map(30.0 + i as f32 * 2.5, 180.0 - v * 140.0))
                .collect(),
            Stroke::new(2.5_f32, rt.tokens.accent),
        ));
        p.circle(
            map(
                30.0 + self.progress * 300.0,
                180.0 - self.curve_value() * 140.0,
            ),
            5.0,
            rt.tokens.surface,
            Stroke::new(3.0_f32, rt.tokens.accent),
        );
        for (point, text) in [
            (map(30.0, 205.0), "0"),
            (map(300.0, 205.0), "time →"),
            (map(8.0, 44.0), "1"),
            (map(8.0, 184.0), "0"),
        ] {
            p.text(
                point,
                Align2::LEFT_CENTER,
                text,
                FontId::monospace(11.0),
                rt.tokens.text3,
            );
        }
    }
    fn curve_controls(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.label("Curve");
        let before = self.curve_index;
        egui::ComboBox::from_id_salt("motion-curve")
            .selected_text(CURVES[self.curve_index].1)
            .width(ui.available_width().min(240.0))
            .show_ui(ui, |ui| {
                for (i, (_, label, _)) in CURVES.iter().enumerate() {
                    let r = ui.selectable_value(&mut self.curve_index, i, *label);
                    rt.record(&format!("motion-curve-{i}"), &r);
                }
            });
        ui.add_space(12.0);
        ui.label(format!("Duration {}ms", self.duration_ms));
        let r = range(
            ui,
            rt,
            egui::Slider::new(&mut self.duration_ms, 150..=1400)
                .step_by(50.0)
                .suffix("ms"),
            160.0,
        );
        rt.record("motion-duration", &r);
        if before != self.curve_index || r.drag_stopped() || (r.changed() && !r.dragged()) {
            self.replay_curve(rt.now);
        }
        if ui::button(
            ui,
            rt,
            "motion-play",
            "Replay curve",
            Some("play"),
            ButtonKind::Primary,
        )
        .clicked()
        {
            self.replay_curve(rt.now)
        }
        ui.monospace(CURVES[self.curve_index].2);
    }
    fn tick(&mut self, rt: &mut Runtime) {
        if let Some(start) = self.curve_start {
            self.progress = rt.progress(start, self.duration_ms as f32 / 1000.0);
            if self.progress >= 1.0 {
                self.curve_start = None
            } else {
                rt.repaint()
            }
        }
        if let Some(start) = self.spring_start {
            let p = rt.progress(start, self.spring.duration_ms as f32 / 1000.0);
            self.spring_progress = self.spring.at(p as f64);
            if p >= 1.0 {
                self.spring_start = None
            } else {
                rt.repaint()
            }
        }
    }
    fn inspect_cache(&mut self, rt: &Runtime) {
        self.cache = ROUTES
            .iter()
            .map(|route| {
                let status = if rt.mounted.contains(route) || *route == rt.route {
                    "Mounted".into()
                } else {
                    "Prepared".into()
                };
                (*route, status)
            })
            .collect();
    }
}

impl Page for State {
    fn enter(&mut self, rt: &mut Runtime) {
        self.replay_curve(rt.now);
        self.replay_spring(rt.now);
        self.list_start = Some(rt.now);
        self.inspect_cache(rt);
    }
    fn leave(&mut self, _rt: &mut Runtime) {
        self.curve_start = None;
        self.spring_start = None;
        self.list_start = None;
        self.continuity_start = None;
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let viewport_width = (ui.clip_rect().right() - ui.cursor().min.x - 16.0).max(100.0);
        ui.set_max_width(ui.available_width().min(viewport_width));
        self.tick(rt);
        ui::page_head(
            ui,
            rt,
            "wave",
            "Foundations / 02",
            "Motion Lab",
            "Compare easing curves and tune spring parameters.",
        );
        ui::section(ui, rt, "motion-curves", "01", "Easing curves", "");
        ui::demo(
            ui,
            rt,
            "motion-workbench",
            "Easing workbench",
            "An out-curve responds quickly and settles gently. A spring carries momentum through the destination.",
            0.0,
            |ui, rt| {
                if ui.available_width() > 650.0 {
                    ui.columns(2, |columns| {
                        self.graph(&mut columns[0], rt);
                        self.curve_controls(&mut columns[1], rt);
                    });
                } else {
                    self.graph(ui, rt);
                    self.curve_controls(ui, rt);
                }
                ui.add_space(16.0);
                let (track, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
                rt.record("motion-track", &response);
                let p = ui.painter().with_clip_rect(track);
                p.rect_filled(track, 12.0 * rt.prefs.radius, rt.tokens.surface2);
                let goal = Rect::from_min_size(
                    pos2(track.right() - 44.0, track.center().y - 20.0),
                    vec2(40.0, 40.0),
                );
                p.rect_stroke(
                    goal,
                    8.0,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                    StrokeKind::Inside,
                );
                let travel = Rect::from_min_size(
                    pos2(
                        track.left() + 4.0 + (track.width() - 48.0).max(0.0) * self.curve_value(),
                        track.center().y - 20.0,
                    ),
                    vec2(40.0, 40.0),
                );
                p.rect_filled(travel, 8.0 * rt.prefs.radius, rt.tokens.accent);
                icons::paint(
                    &p,
                    "cube",
                    travel.shrink(10.0),
                    rt.tokens.accent_contrast,
                    1.6,
                    0.0,
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label("Scrub time");
                    let mut scrub = (self.progress * 1000.0).round() as u32;
                    let width = (ui.available_width() - 230.0).max(100.0);
                    let r = range(
                        ui,
                        rt,
                        egui::Slider::new(&mut scrub, 0..=1000).show_value(false),
                        width,
                    );
                    rt.record("motion-scrub", &r);
                    if r.changed() {
                        self.scrub(scrub as f32 / 1000.0)
                    }
                    ui.monospace(format!(
                        "{}% → {}%",
                        (self.progress * 100.0).round(),
                        (self.curve_value() * 100.0).round()
                    ));
                });
            },
        );
        ui::section(ui, rt, "motion-springs", "02", "Springs", "");
        ui::demo(
            ui,
            rt,
            "motion-spring",
            "Spring bench",
            "Change mass, stiffness or damping, then copy the sampled physical curve.",
            0.0,
            |ui, rt| {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 160.0), Sense::hover());
                rt.record("motion-spring-stage", &response);
                let p = ui.painter().with_clip_rect(rect);
                p.rect_filled(rect, 12.0 * rt.prefs.radius, rt.tokens.surface2);
                p.line_segment(
                    [
                        pos2(rect.left() + 16.0, rect.center().y),
                        pos2(rect.right() - 16.0, rect.center().y),
                    ],
                    Stroke::new(1.0_f32, rt.tokens.border_strong),
                );
                let goal = Rect::from_center_size(
                    pos2(rect.right() - 40.0, rect.center().y),
                    vec2(48.0, 48.0),
                );
                p.rect_stroke(
                    goal,
                    12.0 * rt.prefs.radius,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                    StrokeKind::Inside,
                );
                let object = Rect::from_min_size(
                    pos2(
                        rect.left() + 16.0 + (rect.width() - 80.0) * self.spring_progress as f32,
                        rect.center().y - 24.0,
                    ),
                    vec2(48.0, 48.0),
                );
                p.rect(
                    object,
                    12.0 * rt.prefs.radius,
                    rt.tokens.surface,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                    StrokeKind::Inside,
                );
                icons::paint(
                    &p,
                    "layers",
                    object.shrink(10.0),
                    rt.tokens.accent_text,
                    1.6,
                    0.0,
                );
                let mut changed = false;
                ui.horizontal_wrapped(|ui| {
                    for (name, value, range, step) in [
                        ("Stiffness", &mut self.stiffness, 80.0..=600.0, 10.0),
                        ("Damping", &mut self.damping, 8.0..=60.0, 1.0),
                        ("Mass", &mut self.mass, 0.5..=2.5, 0.1),
                    ] {
                        ui.vertical(|ui| {
                            ui.label(name);
                            let r = self::range(
                                ui,
                                rt,
                                egui::Slider::new(value, range).step_by(step),
                                140.0,
                            );
                            rt.record(&format!("motion-{}", name.to_lowercase()), &r);
                            changed |= r.changed();
                        });
                    }
                });
                if changed {
                    self.replay_spring(rt.now)
                }
                ui.horizontal_wrapped(|ui| {
                    ui::muted(
                        ui,
                        rt,
                        &format!(
                            "Simulation: {}ms{}",
                            self.spring.duration_ms,
                            if rt.reduced() {
                                " / reduced motion"
                            } else {
                                ""
                            }
                        ),
                    );
                    if ui::button(
                        ui,
                        rt,
                        "motion-spring-replay",
                        "Replay",
                        Some("refresh"),
                        ButtonKind::Secondary,
                    )
                    .clicked()
                    {
                        self.replay_spring(rt.now)
                    }
                    if ui::button(
                        ui,
                        rt,
                        "motion-spring-copy",
                        "Copy curve",
                        Some("copy"),
                        ButtonKind::Soft,
                    )
                    .clicked()
                    {
                        let ok = rt.copy(ui.ctx(), self.spring_export());
                        rt.toast(
                            if ok {
                                "Spring curve copied"
                            } else {
                                "Clipboard unavailable"
                            },
                            "",
                            if ok { "success" } else { "warning" },
                        );
                    }
                });
            },
        );
        ui::section(ui, rt, "motion-choreography", "03", "Sequences", "");
        ui::demo(
            ui,
            rt,
            "motion-list",
            "Staggered arrival",
            "Replay the staggered list.",
            0.0,
            |ui, rt| {
                if ui::button(
                    ui,
                    rt,
                    "motion-list-replay",
                    "Replay list",
                    Some("refresh"),
                    ButtonKind::Ghost,
                )
                .clicked()
                {
                    self.list_start = Some(rt.now)
                }
                let mut finished = true;
                for (i, (icon, title, desc)) in [
                    ("document", "Read the brief", "Requirements aligned"),
                    ("droplet", "Apply the language", "Tokens connected"),
                    ("check-circle", "Review the details", "Ready to build"),
                ]
                .iter()
                .enumerate()
                {
                    let progress = if let Some(start) = self.list_start {
                        rt.progress(start + i as f64 * 0.032 * rt.motion_scale() as f64, 0.46)
                    } else {
                        1.0
                    };
                    finished &= progress >= 1.0;
                    let eased = motion::ease(Curve::OutExpo, progress);
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 68.0), Sense::hover());
                    let rect = rect.translate(vec2(
                        0.0,
                        if rt.reduced() {
                            0.0
                        } else {
                            12.0 * (1.0 - eased)
                        },
                    ));
                    let mut painter = ui.painter().clone();
                    painter.set_opacity(eased);
                    painter.rect(
                        rect,
                        12.0 * rt.prefs.radius,
                        rt.tokens.surface,
                        Stroke::new(1.0_f32, rt.tokens.border),
                        StrokeKind::Inside,
                    );
                    let icon_rect = Rect::from_min_size(
                        rect.left_center() + vec2(12.0, -16.0),
                        vec2(32.0, 32.0),
                    );
                    painter.rect_filled(icon_rect, 8.0, rt.tokens.accent_soft);
                    icons::paint(
                        &painter,
                        icon,
                        icon_rect.shrink(7.0),
                        rt.tokens.accent_text,
                        1.6,
                        0.0,
                    );
                    painter.text(
                        rect.left_top() + vec2(56.0, 24.0),
                        Align2::LEFT_CENTER,
                        *title,
                        FontId::proportional(14.0),
                        rt.tokens.text,
                    );
                    painter.text(
                        rect.left_top() + vec2(56.0, 44.0),
                        Align2::LEFT_CENTER,
                        *desc,
                        FontId::proportional(12.0),
                        rt.tokens.text2,
                    );
                    painter.text(
                        rect.right_center() - vec2(16.0, 0.0),
                        Align2::RIGHT_CENTER,
                        format!("0{}", i + 1),
                        FontId::monospace(12.0),
                        rt.tokens.success_text,
                    );
                    ui.add_space(8.0);
                }
                if finished {
                    self.list_start = None
                } else {
                    rt.repaint()
                }
            },
        );
        ui::demo(
            ui,
            rt,
            "motion-continuity",
            "State continuity",
            "A moving indicator and directional content keep the relationship visible.",
            0.0,
            |ui, rt| {
                let previous = self.continuity;
                if ui::segmented(
                    ui,
                    rt,
                    "motion-state",
                    &["Design", "Build", "Review"],
                    &mut self.continuity,
                ) {
                    self.continuity_direction = if self.continuity >= previous {
                        1.0
                    } else {
                        -1.0
                    };
                    self.continuity_start = Some(rt.now);
                }
                let (icon, title, desc) = [
                    (
                        "droplet",
                        "Start with a shared language.",
                        "Decisions become reusable tokens.",
                    ),
                    (
                        "cube",
                        "Compose familiar parts.",
                        "Components carry the same behaviour.",
                    ),
                    (
                        "check-circle",
                        "Check the whole experience.",
                        "Keyboard, motion and responsive layouts.",
                    ),
                ][self.continuity];
                let progress = self
                    .continuity_start
                    .map(|start| rt.progress(start, 0.32))
                    .unwrap_or(1.0);
                let p = motion::ease(Curve::Out, progress);
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 180.0), Sense::hover());
                let mut painter = ui.painter().clone();
                painter.set_opacity(p);
                let center = rect.center()
                    + vec2(
                        if rt.reduced() {
                            0.0
                        } else {
                            self.continuity_direction * 12.0 * (1.0 - p)
                        },
                        0.0,
                    );
                let icon_rect = Rect::from_center_size(center - vec2(0.0, 44.0), vec2(64.0, 64.0));
                painter.rect_filled(icon_rect, 16.0 * rt.prefs.radius, rt.tokens.accent_soft);
                icons::paint(
                    &painter,
                    icon,
                    icon_rect.shrink(18.0),
                    rt.tokens.accent_text,
                    1.6,
                    0.0,
                );
                painter.text(
                    center + vec2(0.0, 10.0),
                    Align2::CENTER_CENTER,
                    title,
                    FontId::proportional(16.0),
                    rt.tokens.text,
                );
                painter.text(
                    center + vec2(0.0, 36.0),
                    Align2::CENTER_CENTER,
                    desc,
                    FontId::proportional(12.0),
                    rt.tokens.text2,
                );
                if progress < 1.0 {
                    rt.repaint()
                } else {
                    self.continuity_start = None
                }
            },
        );
        ui::section(ui, rt, "motion-performance", "04", "Performance", "");
        ui::card(ui, rt, "motion-cache", |ui, rt| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Local page cache");
                if ui::button(
                    ui,
                    rt,
                    "motion-cache-refresh",
                    "Inspect",
                    Some("refresh"),
                    ButtonKind::Secondary,
                )
                .clicked()
                {
                    self.inspect_cache(rt)
                }
            });
            for (route, status) in &self.cache {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    icons::paint(ui.painter(), route.icon(), rect, rt.tokens.text2, 1.5, 0.0);
                    ui.label(route.title());
                    ui::badge(
                        ui,
                        rt,
                        status,
                        if status == "Mounted" {
                            "success"
                        } else {
                            "accent"
                        },
                    );
                });
                ui.separator();
            }
            ui::muted(
                ui,
                rt,
                "Pages retain native state in memory. No server request or network prefetch is required.",
            );
        });
        for (icon, title, desc) in [
            (
                "arrow-right",
                "Enter with out",
                "Let the first frame acknowledge intent. Use out or out-expo, and keep distance short.",
            ),
            (
                "logout",
                "Exit with in",
                "An exit should clear the way. Use an in-curve and roughly 30% less time.",
            ),
            (
                "eye",
                "Respect the preference",
                "Use the tune panel to inspect slow motion or reduce spatial motion. User action timings stay usable.",
            ),
        ] {
            ui::card(ui, rt, ("motion-advice", icon), |ui, rt| {
                ui.strong(title);
                ui::muted(ui, rt, desc);
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn scrub_cancels_running_curve_and_has_exact_endpoints() {
        let mut state = State::default();
        state.replay_curve(10.0);
        state.scrub(1.0);
        assert!(state.curve_start.is_none());
        assert_eq!(state.curve_value(), 1.0);
        state.scrub(-0.4);
        assert_eq!(state.progress, 0.0);
    }
    #[test]
    fn spring_export_tracks_last_simulation() {
        let mut state = State::default();
        state.damping = 8.0;
        state.replay_spring(0.0);
        assert_eq!(state.spring.values.len(), 81);
        assert!(
            state
                .spring_export()
                .contains(&format!("{}ms linear(", state.spring.duration_ms))
        );
        assert!(state.spring.values.iter().any(|v| *v > 1.0));
    }
    #[test]
    fn all_workbench_curves_reach_destination() {
        let mut state = State::default();
        for i in 0..9 {
            state.curve_index = i;
            state.scrub(1.0);
            assert_eq!(state.curve_value(), 1.0);
        }
    }
    #[test]
    fn native_workbenches_render_narrow_and_wide_in_all_appearances() {
        use crate::runtime::Preferences;
        for width in [390.0, 1200.0] {
            for dark in [false, true] {
                for mono in [false, true] {
                    let ctx = egui::Context::default();
                    let mut rt = Runtime::new(
                        ctx.clone(),
                        Preferences {
                            dark,
                            mono,
                            ..Default::default()
                        },
                    );
                    rt.route = Route::Motion;
                    rt.mounted.insert(Route::Foundations);
                    let mut state = State::default();
                    state.enter(&mut rt);
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(
                                egui::Pos2::ZERO,
                                vec2(width, 950.0),
                            )),
                            time: Some(0.3),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| state.show(ui, &mut rt));
                            });
                        },
                    );
                    assert!(
                        !ctx.tessellate(output.shapes, output.pixels_per_point)
                            .is_empty()
                    );
                    assert!(
                        state
                            .cache
                            .iter()
                            .any(|(route, status)| *route == Route::Foundations
                                && status == "Mounted")
                    );
                }
            }
        }
    }
    #[test]
    fn slow_motion_and_scrub_use_one_clock_and_leave_stops_playback() {
        use crate::runtime::{MotionPreference, Preferences};
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx, Preferences::default());
        rt.set_motion(MotionPreference::Slow);
        let mut state = State::default();
        state.enter(&mut rt);
        rt.now = 1.2;
        state.tick(&mut rt);
        assert!((state.progress - 0.5).abs() < 1e-6);
        state.scrub(0.25);
        rt.now = 2.0;
        state.tick(&mut rt);
        assert_eq!(state.progress, 0.25);
        state.leave(&mut rt);
        assert!(
            state.curve_start.is_none()
                && state.spring_start.is_none()
                && state.list_start.is_none()
        );
    }
}
