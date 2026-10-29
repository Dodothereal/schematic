//! Native component browser. All layout, vector painting, interaction state and
//! motion live here in Rust; the source website is only a behavioral reference.
use crate::{
    catalog::{ENTRIES, Entry},
    icons,
    motion::{self, Curve},
    runtime::{Page, Route, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Id, Key, Layout, Modifiers, Pos2, Rect, Response,
    RichText, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2,
    vec2,
};
use std::collections::{HashMap, HashSet};

const PRIORITY: [&str; 6] = [
    "components-2",
    "data-0",
    "primitives-3",
    "components-10",
    "icons-0",
    "primitives-5",
];
const CATEGORIES: [(&str, &str); 7] = [
    ("all", "Everything"),
    ("examples", "Apps"),
    ("controls", "Controls"),
    ("motion", "Motion"),
    ("data", "Data"),
    ("feedback", "Feedback"),
    ("foundations", "Foundations"),
];
const ICONS: [&str; 8] = [
    "send", "check", "bell", "trash", "lock", "folder", "heart", "download",
];
const TASKS: [&str; 3] = ["Design tokens", "Component API", "Keyboard review"];
const STUDY_TASKS: [&str; 3] = ["Define tokens", "Build components", "Test keyboard access"];
const FILES: [&str; 3] = ["tokens.css", "components.js", "README.md"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Ready,
    Running { started: f64 },
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub value: i32,
}

/// Geometry is cached on a range change, never reconstructed in animation frames.
#[derive(Clone, Debug)]
pub struct Revenue {
    pub range: usize,
    pub points: Vec<Point>,
    pub selected: Option<usize>,
    samples: Vec<Pos2>,
    lengths: Vec<f32>,
    length: f32,
    area: egui::Mesh,
    grid: Vec<[Pos2; 2]>,
}
impl Default for Revenue {
    fn default() -> Self {
        Self::new(7)
    }
}
impl Revenue {
    pub fn new(range: usize) -> Self {
        let range = if range == 30 { 30 } else { 7 };
        let base = [38, 53, 48, 68, 52, 76, 84];
        let mut points: Vec<Point> = (0..range)
            .map(|i| {
                let f = i as f64;
                Point {
                    x: 8. + i as f32 * 284. / (range - 1) as f32,
                    y: 0.,
                    value: if range == 7 {
                        base[i]
                    } else {
                        (48. + (f * 0.68).sin() * 13. + f * 0.65 + (f * 1.7).cos() * 9.).round()
                            as i32
                    },
                }
            })
            .collect();
        let min = points.iter().map(|p| p.value).min().unwrap() as f32 - 12.;
        let max = points.iter().map(|p| p.value).max().unwrap() as f32 + 10.;
        for p in &mut points {
            p.y = 105. - (p.value as f32 - min) / (max - min) * 91.;
        }
        let mut samples = vec![pos2(points[0].x, points[0].y)];
        for i in 1..range {
            let a = points[i.saturating_sub(2)];
            let b = points[i - 1];
            let c = points[i];
            let d = points[(i + 1).min(range - 1)];
            let p0 = pos2(b.x, b.y);
            let p1 = p0 + vec2(c.x - a.x, c.y - a.y) / 6.;
            let p3 = pos2(c.x, c.y);
            let p2 = p3 - vec2(d.x - b.x, d.y - b.y) / 6.;
            for step in 1..=24 {
                let t = step as f32 / 24.;
                let u = 1. - t;
                samples.push(pos2(
                    u * u * u * p0.x
                        + 3. * u * u * t * p1.x
                        + 3. * u * t * t * p2.x
                        + t * t * t * p3.x,
                    u * u * u * p0.y
                        + 3. * u * u * t * p1.y
                        + 3. * u * t * t * p2.y
                        + t * t * t * p3.y,
                ));
            }
        }
        let mut lengths = vec![0.];
        for pair in samples.windows(2) {
            lengths.push(lengths.last().unwrap() + pair[0].distance(pair[1]));
        }
        let length = *lengths.last().unwrap();
        let mut area = egui::Mesh::default();
        for pair in samples.windows(2) {
            let first = area.vertices.len() as u32;
            for pos in [
                pair[0],
                pair[1],
                pos2(pair[1].x, 116.),
                pos2(pair[0].x, 116.),
            ] {
                area.colored_vertex(pos, Color32::WHITE);
            }
            area.add_triangle(first, first + 1, first + 2);
            area.add_triangle(first, first + 2, first + 3);
        }
        let mut grid = Vec::new();
        for y in [25., 65., 105.] {
            for x in (8..292).step_by(6) {
                grid.push([pos2(x as f32, y), pos2((x + 2).min(292) as f32, y)]);
            }
        }
        Self {
            range,
            points,
            selected: None,
            samples,
            lengths,
            length,
            area,
            grid,
        }
    }
    pub fn total(&self) -> i32 {
        self.points.iter().map(|p| p.value * 10).sum()
    }
    pub fn value(&self) -> i32 {
        self.selected
            .and_then(|i| self.points.get(i))
            .map(|p| p.value * 10)
            .unwrap_or_else(|| self.total())
    }
    pub fn inspect_fraction(&mut self, fraction: f32) {
        self.selected = Some(
            (((fraction * 300. - 8.) / 284.) * (self.range - 1) as f32)
                .round()
                .clamp(0., (self.range - 1) as f32) as usize,
        );
    }
    pub fn key(&mut self, key: Key) -> bool {
        let last = self.range - 1;
        self.selected = Some(match key {
            Key::Home => 0,
            Key::End => last,
            Key::ArrowLeft => self.selected.unwrap_or(0).saturating_sub(1),
            Key::ArrowRight => (self.selected.unwrap_or(0) + 1).min(last),
            _ => return false,
        });
        true
    }
    pub fn caption(&self) -> String {
        self.selected
            .map(|i| format!("Day {} of {}", i + 1, self.range))
            .unwrap_or_else(|| format!("{} days", self.range))
    }
    fn traced_samples(&self, progress: f32) -> Vec<Pos2> {
        let distance = self.length * progress.clamp(0., 1.);
        let end = self
            .lengths
            .partition_point(|&d| d <= distance)
            .max(1)
            .min(self.samples.len());
        let mut samples = self.samples[..end].to_vec();
        if end < self.samples.len() {
            let a = self.lengths[end - 1];
            let b = self.lengths[end];
            samples.push(
                self.samples[end - 1]
                    .lerp(self.samples[end], ((distance - a) / (b - a)).clamp(0., 1.)),
            );
        }
        samples
    }
}

#[derive(Clone, Debug)]
struct Scalar {
    from: f32,
    to: f32,
    start: f64,
    seconds: f32,
    delay: f32,
}
impl Scalar {
    fn new(value: f32) -> Self {
        Self {
            from: value,
            to: value,
            start: 0.,
            seconds: 0.,
            delay: 0.,
        }
    }
    fn sample(&self, now: f64) -> f32 {
        let p = if self.seconds <= 0. {
            1.
        } else {
            ((now - self.start - self.delay as f64) / self.seconds as f64).clamp(0., 1.) as f32
        };
        self.from + (self.to - self.from) * motion::ease(Curve::OutExpo, p)
    }
    fn moving(&self, now: f64) -> bool {
        self.from != self.to && now < self.start + (self.seconds + self.delay) as f64
    }
    fn target(&mut self, to: f32, now: f64, seconds: f32, delay: f32) {
        if (to - self.to).abs() > 0.001 {
            self.from = self.sample(now);
            self.to = to;
            self.start = now;
            self.seconds = seconds;
            self.delay = delay;
        }
    }
    fn settle(&mut self) {
        self.from = self.to;
        self.seconds = 0.;
        self.delay = 0.;
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
    delay: f32,
    exiting: bool,
}
impl Travel {
    fn sample(&self, now: f64) -> Visual {
        let p = if self.seconds <= 0. {
            1.
        } else {
            ((now - self.start - self.delay as f64) / self.seconds as f64).clamp(0., 1.) as f32
        };
        let p = motion::ease(
            if self.exiting {
                Curve::In
            } else {
                Curve::OutExpo
            },
            p,
        );
        Visual {
            rect: Rect::from_min_max(
                self.from.rect.min.lerp(self.to.rect.min, p),
                self.from.rect.max.lerp(self.to.rect.max, p),
            ),
            opacity: egui::lerp(self.from.opacity..=self.to.opacity, p),
        }
    }
    fn moving(&self, now: f64) -> bool {
        now < self.start + (self.seconds + self.delay) as f64
            && ((self.from.rect != self.to.rect) || (self.from.opacity != self.to.opacity))
    }
    fn retarget(&mut self, to: Visual, now: f64, seconds: f32, delay: f32, exiting: bool) {
        if self.to.rect == to.rect && self.to.opacity == to.opacity && self.exiting == exiting {
            return;
        }
        self.from = self.sample(now);
        self.to = to;
        self.start = now;
        self.seconds = seconds;
        self.delay = delay;
        self.exiting = exiting;
    }
}
#[derive(Clone)]
struct PaintSnapshot {
    shapes: Vec<egui::epaint::ClippedShape>,
    opacity: f32,
}

/// Retained model for all overview controls. State methods are independent of a
/// window and are tested alongside headless egui input and layout.
pub struct State {
    pub query: String,
    pub category: String,
    pub limit: usize,
    pub action: Action,
    pub action_caption: String,
    pub revenue: Revenue,
    pub task_order: Vec<usize>,
    pub task_checks: [bool; 3],
    pub gallery_tab: usize,
    pub settings: [bool; 2],
    pub counter: i32,
    pub study: usize,
    pub study_tasks: [bool; 3],
    pub pins: [bool; 3],
    pub disclosures: [bool; 2],
    pub status: String,
    active: bool,
    entered: f64,
    search_started: Option<f64>,
    count_started: Option<f64>,
    chart_started: Option<f64>,
    chart_visible: bool,
    chart_hovered: bool,
    icon_name: String,
    icon_started: [Option<f64>; 8],
    icon_engaged: [bool; 8],
    tab_started: Option<f64>,
    study_started: Option<f64>,
    counter_digits: [Scalar; 3],
    travels: HashMap<usize, Travel>,
    snapshots: HashMap<usize, PaintSnapshot>,
    indicator: Option<Travel>,
    results_height: Scalar,
    last_query: String,
    last_category: String,
    last_limit: usize,
    last_count: usize,
    layout_width: f32,
    focus_search: bool,
    scroll_to_search: bool,
    sort: crate::sortable::Sortable,
}
impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new(),
            category: "all".into(),
            limit: 12,
            action: Action::Ready,
            action_caption: String::new(),
            revenue: Revenue::default(),
            task_order: vec![0, 1, 2],
            task_checks: [true, false, false],
            gallery_tab: 0,
            settings: [true, false],
            counter: 24,
            study: 0,
            study_tasks: [true, false, false],
            pins: [false; 3],
            disclosures: [false; 2],
            status: String::new(),
            active: false,
            entered: 0.,
            search_started: None,
            count_started: None,
            chart_started: None,
            chart_visible: false,
            chart_hovered: false,
            icon_name: "Hover or focus".into(),
            icon_started: [None; 8],
            icon_engaged: [false; 8],
            tab_started: None,
            study_started: None,
            counter_digits: [Scalar::new(0.), Scalar::new(2.), Scalar::new(4.)],
            travels: HashMap::new(),
            snapshots: HashMap::new(),
            indicator: None,
            results_height: Scalar::new(0.),
            last_query: String::new(),
            last_category: "all".into(),
            last_limit: 12,
            last_count: ENTRIES.len(),
            layout_width: 0.,
            focus_search: false,
            scroll_to_search: false,
            sort: crate::sortable::Sortable::new(vec![vec![0, 1, 2]]),
        }
    }
}

impl State {
    pub fn matches(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        let mut matches: Vec<_> = ENTRIES
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                (self.category == "all" || self.category == e.family)
                    && format!("{} {} {}", e.title, e.desc, e.collection)
                        .to_lowercase()
                        .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        matches.sort_by_key(|&i| {
            PRIORITY
                .iter()
                .position(|&id| id == ENTRIES[i].id)
                .unwrap_or(99)
        });
        matches
    }
    pub fn visible(&self) -> Vec<usize> {
        self.matches().into_iter().take(self.limit).collect()
    }
    pub fn set_query(&mut self, query: impl Into<String>, now: f64) {
        self.query = query.into();
        self.limit = 12;
        self.search_started = Some(now);
        self.cancel_filtered_action();
    }
    pub fn set_category(&mut self, category: &str) {
        if CATEGORIES.iter().any(|&(id, _)| id == category) {
            self.category = category.into();
            self.limit = 12;
            self.cancel_filtered_action();
        }
    }
    pub fn clear_query(&mut self, now: f64) {
        self.set_query("", now);
        self.focus_search = true;
    }
    pub fn clear_filters(&mut self) {
        self.query.clear();
        self.category = "all".into();
        self.limit = 12;
        self.focus_search = true;
    }
    pub fn show_more(&mut self) {
        self.limit += 24;
    }
    pub fn start_action(&mut self, now: f64) -> bool {
        if matches!(self.action, Action::Running { .. }) {
            return false;
        }
        self.action = Action::Running { started: now };
        self.action_caption = "Local preview".into();
        true
    }
    pub fn action_progress(&self, now: f64) -> f32 {
        match self.action {
            Action::Running { started } => ((now - started) / 1.3).clamp(0., 1.) as f32,
            Action::Complete => 1.,
            Action::Ready => 0.,
        }
    }
    pub fn tick_action(&mut self, now: f64, visible: bool) {
        if matches!(self.action, Action::Running { .. }) {
            if !visible {
                self.cancel_action();
            } else if self.action_progress(now) >= 1. {
                self.action = Action::Complete;
                self.action_caption = "Click to replay".into();
            }
        }
    }
    pub fn cancel_action(&mut self) {
        if matches!(self.action, Action::Running { .. }) {
            self.action = Action::Ready;
            self.action_caption = "Cancelled".into();
        }
    }
    fn cancel_filtered_action(&mut self) {
        if !self.visible().iter().any(|&i| ENTRIES[i].id == PRIORITY[0]) {
            self.cancel_action();
        }
    }
    pub fn set_range(&mut self, range: usize, now: f64) {
        self.revenue = Revenue::new(range);
        self.chart_started = Some(now);
    }
    pub fn step_counter(&mut self, step: i32, now: f64, scale: f32) {
        self.set_counter((self.counter + step).clamp(0, 999), now, scale);
    }
    pub fn set_counter(&mut self, value: i32, now: f64, scale: f32) {
        self.counter = value.clamp(0, 999);
        let digits = [
            self.counter / 100,
            (self.counter / 10) % 10,
            self.counter % 10,
        ];
        for (i, digit) in digits.iter().enumerate() {
            self.counter_digits[i].target(
                *digit as f32,
                now,
                0.64 * scale,
                (2 - i) as f32 * 0.028 * scale,
            );
        }
    }
    pub fn pinned_order(&self) -> Vec<usize> {
        (0..3)
            .filter(|&i| self.pins[i])
            .chain((0..3).filter(|&i| !self.pins[i]))
            .collect()
    }
    pub fn toggle_pin(&mut self, i: usize) {
        if let Some(pin) = self.pins.get_mut(i) {
            *pin = !*pin;
            self.status = if *pin {
                "File pinned."
            } else {
                "File unpinned."
            }
            .into();
        }
    }
    pub fn toggle_study_task(&mut self, i: usize) {
        if let Some(task) = self.study_tasks.get_mut(i) {
            *task = !*task;
            self.status = format!(
                "{} of 3 tasks completed.",
                self.study_tasks.iter().filter(|&&v| v).count()
            );
        }
    }
    pub fn reset_studies(&mut self) {
        self.study_tasks = [true, false, false];
        self.pins = [false; 3];
        self.disclosures = [false; 2];
        self.status = "Previews reset.".into();
    }
    pub fn columns(width: f32) -> usize {
        if width >= 1100. {
            3
        } else if width > 600. {
            2
        } else {
            1
        }
    }
    fn settle(&mut self) {
        self.travels.retain(|_, t| !t.exiting);
        for t in self.travels.values_mut() {
            t.from = t.to;
            t.seconds = 0.;
            t.delay = 0.;
        }
        if let Some(t) = &mut self.indicator {
            t.from = t.to;
            t.seconds = 0.;
            t.delay = 0.;
        }
        for d in &mut self.counter_digits {
            d.settle();
        }
        self.results_height.settle();
        self.search_started = None;
        self.count_started = None;
        self.chart_started = None;
        self.tab_started = None;
        self.study_started = None;
        self.icon_started = [None; 8];
        self.icon_engaged = [false; 8];
    }
    fn finite(&self, rt: &Runtime, start: Option<f64>, seconds: f32, delay: f32) -> f32 {
        if rt.reduced() {
            return 1.;
        }
        start
            .map(|s| {
                (((rt.now - s) / rt.motion_scale().max(0.001) as f64 - delay as f64)
                    / seconds as f64)
                    .clamp(0., 1.) as f32
            })
            .unwrap_or(1.)
    }
}

fn text(p: &egui::Painter, pos: Pos2, text: impl ToString, size: f32, color: Color32) {
    p.text(
        pos,
        Align2::LEFT_TOP,
        text,
        FontId::proportional(size),
        color,
    );
}
fn centered(p: &egui::Painter, pos: Pos2, text: impl ToString, size: f32, color: Color32) {
    p.text(
        pos,
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(size),
        color,
    );
}
fn strong_text(ui: &Ui, pos: Pos2, text: impl ToString, size: f32, color: Color32) {
    ui.painter().text(
        pos,
        Align2::LEFT_TOP,
        text,
        ui::strong_font(ui.ctx(), size),
        color,
    );
}
fn block_tabs(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: &str,
    labels: &[&str],
    selected: &mut usize,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.), Sense::hover());
    let each = (rect.width() - 6.) / labels.len() as f32;
    let mut changed = false;
    let gallery = id == "ov-gallery-tabs";
    ui.painter().rect(
        rect,
        rounded(rt, 7.),
        if gallery {
            rt.tokens.surface
        } else {
            rt.tokens.surface2
        },
        Stroke::new(1., rt.tokens.border_soft),
        StrokeKind::Inside,
    );
    let mut responses = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        let r = Rect::from_min_size(rect.min + vec2(3. + i as f32 * each, 3.), vec2(each, 30.));
        let response = control(ui, rt, (id, i), r, label, Some(*selected == i));
        if response.clicked() {
            changed |= *selected != i;
            *selected = i;
        }
        if response.has_focus() {
            for key in [Key::ArrowLeft, Key::ArrowRight, Key::Home, Key::End] {
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
                    let next = match key {
                        Key::ArrowLeft => (*selected + labels.len() - 1) % labels.len(),
                        Key::ArrowRight => (*selected + 1) % labels.len(),
                        Key::Home => 0,
                        _ => labels.len() - 1,
                    };
                    changed |= *selected != next;
                    *selected = next;
                    ui.memory_mut(|m| m.request_focus(ui.make_persistent_id((id, next))));
                }
            }
        }
        rt.record(&format!("segment:{label}"), &response);
        responses.push(response);
    }
    let x = rt.animate(
        (id, "thumb"),
        each * *selected as f32,
        0.47,
        Curve::SpringSnappy,
    );
    let thumb = Rect::from_min_size(rect.min + vec2(3. + x, 3.), vec2(each, 30.));
    ui.painter().rect(
        thumb,
        rounded(rt, 5.),
        if gallery {
            rt.tokens.surface2
        } else {
            rt.tokens.surface
        },
        Stroke::new(1., rt.tokens.border_soft),
        StrokeKind::Inside,
    );
    for (i, (label, response)) in labels.iter().zip(responses).enumerate() {
        if i == *selected {
            ui.painter().text(
                response.rect.center(),
                Align2::CENTER_CENTER,
                label,
                ui::strong_font(ui.ctx(), if gallery { 10. } else { 12. }),
                rt.tokens.text,
            );
        } else {
            centered(
                ui.painter(),
                response.rect.center(),
                label,
                if gallery { 10. } else { 12. },
                rt.tokens.text2,
            );
        }
    }
    changed
}
fn rounded(rt: &Runtime, r: f32) -> f32 {
    r * rt.prefs.radius
}
fn money(value: i32) -> String {
    let digits = value.to_string();
    let mut out = String::from("$");
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}
fn semantics(response: &Response, label: &str, selected: Option<bool>) {
    response.widget_info(|| {
        if let Some(selected) = selected {
            WidgetInfo::selected(
                WidgetType::SelectableLabel,
                response.enabled(),
                selected,
                label,
            )
        } else {
            WidgetInfo::labeled(WidgetType::Button, response.enabled(), label)
        }
    });
}
fn live_status(response: &Response) {
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_live(egui::accesskit::Live::Polite);
        node.set_live_atomic();
    });
}
fn focus_ring(ui: &Ui, rt: &Runtime, response: &Response, radius: f32) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(3.),
            radius,
            Stroke::new(2., rt.tokens.accent_ring),
            StrokeKind::Outside,
        );
    }
}
trait PainterOpacity {
    fn with_opacity(&self, opacity: f32) -> Self;
}
impl PainterOpacity for egui::Painter {
    fn with_opacity(&self, opacity: f32) -> Self {
        let mut painter = self.clone();
        painter.multiply_opacity(opacity);
        painter
    }
}
fn control(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    rect: Rect,
    label: &str,
    selected: Option<bool>,
) -> Response {
    let response = ui.interact(rect, ui.make_persistent_id(id), Sense::click());
    if response.clicked() {
        response.request_focus();
    }
    semantics(&response, label, selected);
    focus_ring(ui, rt, &response, 4.);
    response
}

impl State {
    fn header(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let width = ui.available_width();
        let viewport_width = ui.ctx().content_rect().width();
        let mut size = if viewport_width <= 360. {
            22.
        } else if viewport_width <= 600. {
            28.
        } else {
            (viewport_width * 0.03).clamp(28., 40.)
        };
        let natural = ui
            .painter()
            .layout_no_wrap(
                "Component library".into(),
                ui::strong_font(ui.ctx(), size),
                rt.tokens.text,
            )
            .size()
            .x;
        size *= ((width - 90.).max(1.) / natural.max(1.)).min(1.);
        let (rect, _) = ui.allocate_exact_size(vec2(width, 50.), Sense::hover());
        if rect.bottom() < ui.clip_rect().top() {
            self.entered = rt.now - 1000.;
        }
        let title_clip = Rect::from_min_max(rect.min, rect.max - vec2(90., 0.));
        let heading = ui.interact(
            title_clip,
            ui.make_persistent_id("ov-title"),
            Sense::hover(),
        );
        heading.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, "Component library"));
        ui.ctx().accesskit_node_builder(heading.id, |node| {
            node.set_role(egui::accesskit::Role::Heading);
            node.set_level(1);
        });
        rt.record("ov-title", &heading);
        let painter = ui
            .painter()
            .with_clip_rect(title_clip.intersect(ui.clip_rect()));
        let mut x = rect.left();
        for (i, word) in ["Component", "library"].iter().enumerate() {
            let p = self.finite(rt, Some(self.entered), 0.62, i as f32 * 0.065);
            if p < 1. && ui.is_rect_visible(rect) {
                rt.repaint();
            }
            let galley = painter.layout_no_wrap(
                (*word).into(),
                ui::strong_font(ui.ctx(), size),
                rt.tokens.text,
            );
            painter
                .with_clip_rect(
                    Rect::from_min_size(
                        pos2(x, rect.top()),
                        vec2(galley.size().x + 1., size * 1.23),
                    )
                    .intersect(title_clip),
                )
                .with_opacity(motion::ease(Curve::OutExpo, p))
                .galley(
                    pos2(
                        x,
                        rect.top() + size * 1.05 * (1. - motion::ease(Curve::OutExpo, p)),
                    ),
                    galley.clone(),
                    rt.tokens.text,
                );
            x += galley.size().x + size * 0.23;
        }
        let source = Rect::from_min_size(pos2(rect.right() - 82., rect.top() + 3.), vec2(82., 32.));
        let response = control(ui, rt, "ov-source", source, "Source", None);
        rt.record("ov-source", &response);
        ui.painter().rect_filled(
            source,
            rounded(rt, 6.),
            if response.hovered() {
                rt.tokens.surface2
            } else {
                rt.tokens.surface
            },
        );
        ui.painter().rect_stroke(
            source,
            rounded(rt, 6.),
            Stroke::new(1., rt.tokens.border),
            StrokeKind::Inside,
        );
        text(
            ui.painter(),
            source.min + vec2(12., 9.),
            "Source",
            12.,
            rt.tokens.text,
        );
        let progress =
            rt.icon_progress("ov-source-icon", response.hovered() || response.has_focus());
        icons::paint(
            ui.painter(),
            "arrow-up-right",
            Rect::from_center_size(source.right_center() - vec2(15., 0.), Vec2::splat(14.)),
            rt.tokens.text2,
            1.5,
            progress,
        );
        if response.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(
                "https://github.com/Dodothereal/schematic",
            ));
        }
        ui.add_space(32.);
    }
    fn search(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let entrance = self.finite(rt, Some(self.entered), 0.46, 0.125);
        let entrance = motion::ease(Curve::OutExpo, entrance);
        let (allocation, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), 50.), Sense::hover());
        let rect = allocation.translate(vec2(0., 8. * (1. - entrance)));
        let background = ui.interact(
            rect,
            ui.make_persistent_id("ov-search-background"),
            Sense::click(),
        );
        let mut field_ui = ui.new_child(UiBuilder::new().id_salt("ov-search-ui").max_rect(rect));
        field_ui.set_opacity(entrance);
        field_ui
            .painter()
            .rect_filled(rect, rounded(rt, 8.), rt.tokens.surface);
        let input_rect = Rect::from_min_max(rect.min + vec2(42., 11.), rect.max - vec2(40., 9.));
        let response = field_ui.put(
            input_rect,
            egui::TextEdit::singleline(&mut self.query)
                .id(Id::new("ov-search-input"))
                .hint_text("Search accordion, spring, button…")
                .font(FontId::proportional(13.))
                .frame(false)
                .desired_width(input_rect.width()),
        );
        rt.record("ov-search", &response);
        let focused = response.has_focus();
        // The border does not obscure native text selection or the caret.
        let fill = field_ui.painter().add(Shape::Noop);
        field_ui.painter().set(
            fill,
            Shape::rect_stroke(
                rect,
                rounded(rt, 8.),
                Stroke::new(
                    1.,
                    if focused {
                        rt.tokens.accent_ring
                    } else {
                        rt.tokens.border
                    },
                ),
                StrokeKind::Inside,
            ),
        );
        let focus = rt.animate(
            "ov-search-focus",
            if focused { 1. } else { 0. },
            0.32,
            Curve::Out,
        );
        let angle = -10_f32.to_radians() * focus;
        let center = rect.left_center() + vec2(21., 0.);
        let rotate = |v: Vec2| {
            vec2(
                v.x * angle.cos() - v.y * angle.sin(),
                v.x * angle.sin() + v.y * angle.cos(),
            )
        };
        field_ui.painter().circle_stroke(
            center + rotate(vec2(-1.5, -1.5)),
            5.3,
            Stroke::new(1.5, rt.tokens.text3),
        );
        field_ui.painter().line_segment(
            [
                center + rotate(vec2(2.4, 2.4)),
                center + rotate(vec2(6.6, 6.6)),
            ],
            Stroke::new(1.5, rt.tokens.text3),
        );
        let empty = self.query.is_empty();
        let reset = Rect::from_center_size(rect.right_center() - vec2(20., 0.), Vec2::splat(28.));
        if empty {
            centered(
                field_ui.painter(),
                reset.center(),
                "/",
                12.,
                rt.tokens.text3,
            );
        } else {
            let r = control(
                &mut field_ui,
                rt,
                "ov-search-reset",
                reset,
                "Clear search",
                None,
            );
            rt.record("ov-search-reset", &r);
            if r.hovered() {
                field_ui
                    .painter()
                    .rect_filled(reset, rounded(rt, 4.), rt.tokens.surface2);
            }
            icons::paint(
                field_ui.painter(),
                "x",
                reset.shrink(7.),
                rt.tokens.text3,
                1.5,
                rt.icon_progress("ov-search-reset-icon", r.hovered() || r.has_focus()),
            );
            if r.clicked() {
                self.clear_query(rt.now);
            }
        }
        if response.changed() {
            self.limit = 12;
            self.search_started = Some(rt.now);
            self.cancel_filtered_action();
        }
        if self.focus_search {
            response.request_focus();
            self.focus_search = false;
        }
        if self.scroll_to_search {
            ui.scroll_to_rect(allocation, Some(Align::Min));
            self.scroll_to_search = false;
        }
        let p = self.finite(rt, self.search_started, 0.52, 0.);
        if p < 1. && !rt.reduced() {
            let progress = motion::ease(Curve::InOut, p);
            let opacity = if progress < 0.7 {
                0.75
            } else {
                0.75 * (1. - progress) / 0.3
            };
            field_ui.painter().rect_filled(
                Rect::from_min_size(
                    pos2(rect.left() + 12., rect.bottom() - 1.),
                    vec2((rect.width() - 24.) * (progress / 0.7).min(1.), 1.5),
                ),
                0.,
                rt.tokens.accent.gamma_multiply(opacity),
            );
            rt.repaint();
        }
        if background.clicked() {
            response.request_focus();
        }
        if entrance < 1. {
            rt.repaint();
        }
        ui.add_space(8.);
    }
    fn filters(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let width = ui.available_width();
        let entrance = motion::ease(
            Curve::OutExpo,
            self.finite(rt, Some(self.entered), 0.46, 0.17),
        );
        let origin = ui.cursor().min + vec2(0., 8. * (1. - entrance));
        let mut x = 0.;
        let mut y = 0.;
        let height = 40.;
        let mut rects = Vec::new();
        for &(_, label) in &CATEGORIES {
            let w = ui
                .painter()
                .layout_no_wrap(label.into(), FontId::proportional(12.), rt.tokens.text)
                .size()
                .x
                + 32.;
            if x > 0. && x + w > width {
                x = 0.;
                y += height + 4.;
            }
            rects.push(Rect::from_min_size(origin + vec2(x, y), vec2(w, height)));
            x += w + 4.;
        }
        ui.allocate_exact_size(vec2(width, y + height), Sense::hover());
        let mut responses = Vec::new();
        for ((id, label), rect) in CATEGORIES.iter().zip(&rects) {
            let response = control(
                ui,
                rt,
                ("ov-filter", id),
                *rect,
                label,
                Some(self.category == *id),
            );
            rt.record(&format!("ov-filter-{id}"), &response);
            if response.clicked() {
                self.set_category(id);
            }
            responses.push(response);
        }
        let selected = CATEGORIES
            .iter()
            .position(|&(id, _)| id == self.category)
            .unwrap_or(0);
        let local = rects[selected].translate(-origin.to_vec2());
        let target = Visual {
            rect: local,
            opacity: 1.,
        };
        let travel = self.indicator.get_or_insert(Travel {
            from: target,
            to: target,
            start: rt.now,
            seconds: 0.,
            delay: 0.,
            exiting: false,
        });
        travel.retarget(target, rt.now, 0.44 * rt.motion_scale(), 0., false);
        if rt.reduced() {
            travel.from = travel.to;
            travel.seconds = 0.;
        }
        let displayed = travel.sample(rt.now).rect.translate(origin.to_vec2());
        let painter = ui.painter().with_opacity(entrance);
        painter.rect(
            displayed,
            rounded(rt, 6.),
            rt.tokens.surface,
            Stroke::new(1., rt.tokens.border_strong),
            StrokeKind::Inside,
        );
        if travel.moving(rt.now) {
            rt.repaint();
        }
        for ((rect, response), &(_, label)) in rects.iter().zip(&responses).zip(&CATEGORIES) {
            if response.hovered() && *rect != rects[selected] {
                painter.rect_filled(*rect, rounded(rt, 6.), rt.tokens.surface2);
            }
            centered(
                &painter,
                rect.center(),
                label,
                12.,
                if *rect == rects[selected] {
                    rt.tokens.text
                } else {
                    rt.tokens.text2
                },
            );
        }
        if entrance < 1. && ui.is_rect_visible(Rect::from_min_size(origin, vec2(width, y + height)))
        {
            rt.repaint();
        }
        ui.add_space(10.);
    }
    fn count(&mut self, ui: &mut Ui, rt: &mut Runtime, count: usize) {
        if self.last_count != count {
            self.last_count = count;
            self.count_started = Some(rt.now);
        }
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.), Sense::hover());
        let label = format!(
            "{} {}",
            count,
            if count == 1 { "example" } else { "examples" }
        );
        let status = ui.interact(
            Rect::from_min_size(rect.min, vec2(120., rect.height())),
            ui.make_persistent_id("ov-count"),
            Sense::hover(),
        );
        status.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &label));
        live_status(&status);
        rt.record("ov-count", &status);
        let p = self.finite(rt, self.count_started, 0.28, 0.);
        let eased = motion::ease(Curve::OutExpo, p);
        text(
            &ui.painter().with_opacity(eased),
            rect.min + vec2(0., 9. + 5. * (1. - eased)),
            format!(
                "{} {}",
                count,
                if count == 1 { "example" } else { "examples" }
            ),
            11.,
            rt.tokens.text3,
        );
        if p < 1. {
            rt.repaint();
        }
        if !self.query.trim().is_empty() || self.category != "all" {
            let r = Rect::from_min_size(pos2(rect.right() - 111., rect.top()), vec2(111., 30.));
            let response = control(ui, rt, "ov-clear-filters", r, "Clear filters", None);
            rt.record("ov-clear-filters", &response);
            if response.hovered() {
                ui.painter()
                    .rect_filled(r, rounded(rt, 5.), rt.tokens.surface2);
            }
            centered(
                ui.painter(),
                r.center() - vec2(7., 0.),
                "Clear filters",
                11.,
                rt.tokens.text2,
            );
            icons::paint(
                ui.painter(),
                "x",
                Rect::from_center_size(r.right_center() - vec2(13., 0.), Vec2::splat(12.)),
                rt.tokens.text3,
                1.5,
                0.,
            );
            if response.clicked() {
                self.clear_filters();
            }
        }
        ui.add_space(6.);
    }

    fn gallery(&mut self, ui: &mut Ui, rt: &mut Runtime, visible: &[usize]) {
        let width = ui.available_width();
        let viewport_width = ui.ctx().input(|i| i.content_rect().width());
        let cols = Self::columns(viewport_width);
        let gap = 12.;
        let card_width = ((width - gap * (cols - 1) as f32) / cols as f32).max(1.);
        let preview_height = if viewport_width <= 600. { 210. } else { 198. };
        let mut y = 0.;
        let mut targets = HashMap::new();
        for row in visible.chunks(cols) {
            let heights: Vec<f32> = row
                .iter()
                .map(|&i| {
                    if PRIORITY.contains(&ENTRIES[i].id) {
                        preview_height + 79.
                    } else {
                        126.
                    }
                })
                .collect();
            let height = heights.iter().copied().fold(0., f32::max);
            for (col, (&i, _)) in row.iter().zip(heights).enumerate() {
                targets.insert(
                    i,
                    Rect::from_min_size(
                        pos2(col as f32 * (card_width + gap), y),
                        vec2(card_width, height),
                    ),
                );
            }
            y += height + gap;
        }
        let target_height = if visible.is_empty() {
            176.
        } else {
            (y - gap).max(0.)
        };
        let initial = self.layout_width == 0.;
        let resized = !initial && (self.layout_width - width).abs() > 1.;
        self.layout_width = width;
        let now = rt.now;
        let scale = rt.motion_scale();
        let visible_set: HashSet<_> = visible.iter().copied().collect();
        let old_keys: Vec<_> = self.travels.keys().copied().collect();
        for i in old_keys {
            if !visible_set.contains(&i) {
                let travel = self.travels.get_mut(&i).unwrap();
                if !travel.exiting {
                    let before = travel.sample(now);
                    travel.retarget(
                        Visual {
                            rect: before.rect.translate(vec2(0., -6.)),
                            opacity: 0.,
                        },
                        now,
                        0.11 * scale,
                        0.,
                        true,
                    );
                }
            }
        }
        let mut entering = 0;
        for &i in visible {
            let rect = targets[&i];
            let target = Visual { rect, opacity: 1. };
            if let Some(travel) = self.travels.get_mut(&i) {
                travel.retarget(target, now, 0.42 * scale, 0.065 * scale, false);
                if resized || rt.reduced() {
                    travel.from = travel.to;
                    travel.seconds = 0.;
                    travel.delay = 0.;
                }
            } else {
                let from = if initial || rt.reduced() {
                    target
                } else {
                    Visual {
                        rect: rect.translate(vec2(0., 10.)),
                        opacity: 0.,
                    }
                };
                self.travels.insert(
                    i,
                    Travel {
                        from,
                        to: target,
                        start: now,
                        seconds: if initial { 0. } else { 0.38 * scale },
                        delay: if initial {
                            0.
                        } else {
                            (0.085 + entering.min(5) as f32 * 0.022) * scale
                        },
                        exiting: false,
                    },
                );
                entering += 1;
            }
        }
        self.travels.retain(|_, t| !t.exiting || t.moving(now));
        if initial || resized || rt.reduced() {
            self.results_height = Scalar::new(target_height);
        } else {
            self.results_height
                .target(target_height, now, 0.44 * scale, 0.);
        }
        let height = self.results_height.sample(now).max(0.);
        let (results, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        rt.anchor("ov-library", "Components", results);
        let clip = results.expand(4.).intersect(ui.clip_rect());
        if self.results_height.moving(now) && ui.is_rect_visible(results) {
            rt.repaint();
        }
        // Outgoing cards are cached vector shapes only. They never register an
        // interaction, accessibility node, control ID or focus target.
        let ghosts: Vec<_> = self
            .travels
            .iter()
            .filter(|(_, t)| t.exiting)
            .map(|(&i, t)| (i, t.sample(now), t.moving(now)))
            .collect();
        for (i, visual, moving) in ghosts {
            if let Some(cache) = self.snapshots.get(&i) {
                let offset = results.min.to_vec2() + visual.rect.min.to_vec2();
                let alpha = (visual.opacity / cache.opacity.max(0.001)).clamp(0., 1.);
                for cached in &cache.shapes {
                    let mut shape = cached.shape.clone();
                    shape.transform(egui::emath::TSTransform::from_translation(offset));
                    egui::epaint::shape_transform::adjust_colors(&mut shape, move |c| {
                        *c = c.gamma_multiply(alpha)
                    });
                    ui.painter()
                        .with_clip_rect(cached.clip_rect.translate(offset).intersect(clip))
                        .add(shape);
                }
                if moving && clip.intersects(visual.rect.translate(results.min.to_vec2())) {
                    rt.repaint();
                }
            }
        }
        let mut action_visible = false;
        let mut chart_visible = false;
        let mut sort_visible = false;
        for &i in visible {
            let visual = self.travels[&i].sample(now);
            let rect = visual.rect.translate(results.min.to_vec2());
            let intersection = rect.intersect(clip);
            let in_view =
                intersection.is_positive() && intersection.area() / rect.area().max(1.) >= 0.08;
            match ENTRIES[i].id {
                "components-2" => {
                    action_visible = in_view;
                    self.tick_action(now, in_view && self.active);
                }
                "data-0" => chart_visible = in_view,
                "primitives-3" => sort_visible = in_view,
                _ => {}
            }
            if !in_view {
                continue;
            }
            if self.travels[&i].moving(now) {
                rt.repaint();
            }
            let layer = ui.layer_id();
            let start = ui
                .ctx()
                .graphics(|g| g.get(layer).map(|p| p.all_entries().len()).unwrap_or(0));
            let mut child = ui.new_child(
                UiBuilder::new()
                    .id(("ov-card", ENTRIES[i].id))
                    .max_rect(rect)
                    .layout(Layout::top_down(Align::Min)),
            );
            child.set_clip_rect(clip.intersect(rect));
            child.set_opacity(visual.opacity);
            self.card(&mut child, rt, &ENTRIES[i], rect, preview_height);
            let shapes = ui.ctx().graphics(|g| {
                g.get(layer)
                    .map(|p| {
                        p.all_entries()
                            .skip(start)
                            .cloned()
                            .map(|mut s| {
                                let transform =
                                    egui::emath::TSTransform::from_translation(-rect.min.to_vec2());
                                s.shape.transform(transform);
                                s.clip_rect = s.clip_rect.translate(-rect.min.to_vec2());
                                s
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            });
            self.snapshots.insert(
                i,
                PaintSnapshot {
                    shapes,
                    opacity: visual.opacity,
                },
            );
        }
        self.snapshots.retain(|i, _| self.travels.contains_key(i));
        self.tick_action(rt.now, action_visible && self.active);
        if matches!(self.action, Action::Running { .. }) && action_visible {
            rt.repaint();
        }
        if !chart_visible {
            self.chart_visible = false;
            self.chart_started = None;
        }
        self.sort.set_enabled(sort_visible && self.active);
        if !sort_visible {
            self.task_order = self.sort.groups.first().cloned().unwrap_or_default();
        }
        if visible.is_empty() {
            let painter = ui.painter().with_clip_rect(clip);
            let center = results.center();
            icons::paint(
                &painter,
                "search",
                Rect::from_center_size(center - vec2(0., 43.), Vec2::splat(24.)),
                rt.tokens.text3,
                1.5,
                0.,
            );
            centered(
                &painter,
                center - vec2(0., 5.),
                "No matches.",
                18.,
                rt.tokens.text,
            );
            centered(
                &painter,
                center + vec2(0., 24.),
                "Try another search or clear the filters.",
                12.,
                rt.tokens.text2,
            );
            painter.rect_stroke(
                results,
                rounded(rt, 12.),
                Stroke::new(1., rt.tokens.border_strong),
                StrokeKind::Inside,
            );
        }
    }
    fn card(
        &mut self,
        ui: &mut Ui,
        rt: &mut Runtime,
        entry: &Entry,
        rect: Rect,
        preview_height: f32,
    ) {
        let preview = PRIORITY.contains(&entry.id);
        // Plain cards keep their natural link height even when a preview makes
        // the containing grid row taller.
        let rect = if preview {
            rect
        } else {
            Rect::from_min_size(rect.min, vec2(rect.width(), 126.))
        };
        let radius = rounded(rt, if preview { 12. } else { 10. });
        ui.painter().rect(
            rect,
            radius,
            rt.tokens.surface,
            Stroke::new(1., rt.tokens.border),
            StrokeKind::Inside,
        );
        if preview {
            let area = Rect::from_min_size(rect.min, vec2(rect.width(), preview_height));
            ui.painter().rect_filled(
                area,
                radius,
                if entry.id == "components-2" {
                    rt.tokens.text
                } else {
                    rt.tokens.surface2
                },
            );
            let mut preview_ui = ui.new_child(
                UiBuilder::new()
                    .id_salt("preview")
                    .max_rect(area.shrink(if entry.id == "primitives-3" { 16. } else { 20. }))
                    .layout(Layout::top_down(Align::Min)),
            );
            preview_ui.set_clip_rect(area.intersect(ui.clip_rect()));
            preview_ui.spacing_mut().item_spacing = vec2(6., 4.);
            preview_ui.spacing_mut().interact_size = vec2(20., 20.);
            match entry.id {
                "components-2" => self.action_preview(&mut preview_ui, rt),
                "data-0" => self.chart_preview(&mut preview_ui, rt),
                "primitives-3" => self.sort_preview(&mut preview_ui, rt),
                "components-10" => self.tabs_preview(&mut preview_ui, rt),
                "icons-0" => self.icons_preview(&mut preview_ui, rt),
                "primitives-5" => self.counter_preview(&mut preview_ui, rt),
                _ => {}
            }
        }
        let link = if preview {
            Rect::from_min_max(pos2(rect.left(), rect.top() + preview_height), rect.max)
        } else {
            rect
        };
        let response = control(
            ui,
            rt,
            ("ov-specimen", entry.id),
            link,
            &format!("{}: {}. {}", entry.collection, entry.title, entry.desc),
            None,
        );
        rt.record(&format!("ov-specimen-{}", entry.id), &response);
        if response.hovered() {
            ui.painter()
                .rect_filled(link, if preview { 0. } else { radius }, rt.tokens.surface2);
        }
        if preview {
            ui.painter().line_segment(
                [link.left_top(), link.right_top()],
                Stroke::new(1., rt.tokens.border_soft),
            );
        }
        let padding = if preview { 16. } else { 20. };
        let side = if preview { 30. } else { 32. };
        let icon_rect = Rect::from_min_size(link.min + vec2(padding, padding), Vec2::splat(side));
        ui.painter()
            .rect_filled(icon_rect, rounded(rt, 6.), rt.tokens.surface2);
        let progress = rt.icon_progress(
            ("ov-specimen-icon", entry.id),
            response.hovered() || response.has_focus(),
        );
        icons::paint(
            ui.painter(),
            entry.icon,
            icon_rect.shrink(if preview { 7. } else { 6. }),
            rt.tokens.accent_text,
            1.5,
            progress,
        );
        let x = icon_rect.right() + 12.;
        let top = link.top() + padding - 1.;
        let wrap = (link.right() - x - padding).max(1.);
        text(
            ui.painter(),
            pos2(x, top),
            entry.collection.to_uppercase(),
            if preview { 8. } else { 9. },
            rt.tokens.text3,
        );
        let mut title_job = egui::text::LayoutJob::simple(
            entry.title.into(),
            ui::strong_font(ui.ctx(), 13.),
            rt.tokens.text,
            wrap,
        );
        title_job.wrap.max_rows = 2;
        let title = ui.painter().layout_job(title_job);
        ui.painter()
            .galley(pos2(x, top + 14.), title.clone(), rt.tokens.text);
        if !preview {
            let desc = wrapped(ui, entry.desc, 11., rt.tokens.text3, wrap, 2);
            ui.painter()
                .galley(pos2(x, top + 18. + title.size().y), desc, rt.tokens.text3);
        }
        let reveal = rt.animate(
            ("ov-specimen-go", entry.id),
            if preview || response.hovered() || response.has_focus() {
                1.
            } else {
                0.
            },
            0.22,
            Curve::Out,
        );
        icons::paint(
            &ui.painter().with_opacity(reveal),
            "arrow-up-right",
            Rect::from_center_size(
                link.right_top() + vec2(-20. - 3. * (1. - reveal), 22. + 3. * (1. - reveal)),
                Vec2::splat(14.),
            ),
            if preview {
                rt.tokens.text4
            } else {
                rt.tokens.accent_text
            },
            1.5,
            progress,
        );
        if response.clicked() {
            rt.jump(Route::from_id(entry.page), entry.id);
        }
    }
    fn action_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let rect = ui.max_rect();
        let center = rect.center();
        let progress = self.action_progress(rt.now);
        let running = matches!(self.action, Action::Running { .. });
        let status = match self.action {
            Action::Ready => "Ready",
            Action::Running { .. } => "Running",
            Action::Complete => "Done",
        };
        let status_response = ui.interact(
            Rect::from_center_size(center - vec2(0., 54.), vec2(76., 14.)),
            ui.make_persistent_id("ov-action-status"),
            Sense::hover(),
        );
        status_response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, status));
        live_status(&status_response);
        let foreground = rt.tokens.bg;
        ui.painter().circle_filled(
            center - vec2(26., 54.),
            2.5,
            foreground.gamma_multiply(0.65),
        );
        centered(
            ui.painter(),
            center + vec2(4., -54.),
            status,
            9.,
            foreground.gamma_multiply(0.65),
        );
        let width = rt
            .animate(
                "ov-action-width",
                if running { 194. } else { 174. },
                0.46,
                Curve::SpringSmooth,
            )
            .min(rect.width());
        let button = Rect::from_center_size(center - vec2(0., 2.), vec2(width, 46.));
        let response = ui
            .add_enabled_ui(!running, |ui| {
                control(ui, rt, "ov-action", button, "Run action", None)
            })
            .inner;
        rt.record("ov-action", &response);
        let fill = match self.action {
            Action::Complete => rt.tokens.accent,
            Action::Running { .. } => ui::mix(rt.tokens.text, rt.tokens.bg, 0.09),
            Action::Ready => rt.tokens.bg,
        };
        let color = if matches!(self.action, Action::Complete) {
            rt.tokens.accent_contrast
        } else if running {
            foreground
        } else {
            rt.tokens.text
        };
        ui.painter().rect(
            button,
            rounded(rt, 8.),
            fill,
            Stroke::new(1., foreground.gamma_multiply(0.16)),
            StrokeKind::Inside,
        );
        if running {
            ui.painter().with_clip_rect(button).rect_filled(
                Rect::from_min_size(button.min, vec2(button.width() * progress, button.height())),
                0.,
                foreground.gamma_multiply(0.13),
            );
        }
        let label = match self.action {
            Action::Ready => "Run action".into(),
            Action::Complete => "Complete".into(),
            Action::Running { .. } => format!("Working {}%", (progress * 100.).floor() as i32),
        };
        let icon = match self.action {
            Action::Ready => "play",
            Action::Complete => "check",
            Action::Running { .. } => "loader",
        };
        let verb = if running {
            if rt.reduced() {
                0.
            } else {
                ((rt.now % 0.8) / 0.8) as f32
            }
        } else {
            rt.icon_progress(
                "ov-action-verb",
                self.action == Action::Complete || response.hovered() || response.has_focus(),
            )
        };
        icons::paint(
            ui.painter(),
            icon,
            Rect::from_center_size(button.center() - vec2(51., 0.), Vec2::splat(17.)),
            color,
            1.6,
            verb,
        );
        ui.painter().text(
            button.center() + vec2(10., 0.),
            Align2::CENTER_CENTER,
            label,
            ui::strong_font(ui.ctx(), 12.),
            color,
        );
        centered(
            ui.painter(),
            center + vec2(0., 43.),
            &self.action_caption,
            9.,
            foreground.gamma_multiply(0.65),
        );
        if response.clicked() {
            self.start_action(rt.now);
            rt.repaint();
        }
    }
    fn chart_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        if !self.chart_visible {
            self.chart_visible = true;
            self.chart_started = Some(rt.now);
            self.revenue.selected = None;
        }
        let rect = ui.max_rect();
        let ranges = Rect::from_min_size(pos2(rect.right() - 71., rect.top()), vec2(71., 28.));
        ui.painter().rect(
            ranges,
            rounded(rt, 6.),
            rt.tokens.surface,
            Stroke::new(1., rt.tokens.border_soft),
            StrokeKind::Inside,
        );
        for (i, range) in [7, 30].iter().enumerate() {
            let r = Rect::from_min_size(ranges.min + vec2(3. + i as f32 * 33., 3.), vec2(32., 22.));
            let response = control(
                ui,
                rt,
                ("ov-range", range),
                r,
                &format!("{range} day revenue preview"),
                Some(self.revenue.range == *range),
            );
            rt.record(&format!("ov-range-{range}"), &response);
            if self.revenue.range == *range || response.hovered() {
                ui.painter()
                    .rect_filled(r, rounded(rt, 4.), rt.tokens.surface2);
            }
            centered(
                ui.painter(),
                r.center(),
                format!("{range}D"),
                9.,
                if self.revenue.range == *range {
                    rt.tokens.text
                } else {
                    rt.tokens.text3
                },
            );
            if response.clicked() {
                self.set_range(*range, rt.now);
            }
        }
        let chart = Rect::from_min_max(rect.min + vec2(0., 58.), rect.max);
        let response = control(
            ui,
            rt,
            "ov-revenue-chart",
            chart,
            "Revenue preview. Left and Right inspect a day; Home and End reach the endpoints.",
            None,
        );
        rt.record("ov-revenue-chart", &response);
        if response.hovered() {
            if let Some(pointer) = response.hover_pos() {
                self.revenue
                    .inspect_fraction((pointer.x - chart.left()) / chart.width());
            }
        } else if self.chart_hovered || !response.has_focus() {
            self.revenue.selected = None;
        }
        self.chart_hovered = response.hovered();
        if response.clicked() {
            response.request_focus();
        }
        if response.has_focus() {
            for key in [Key::ArrowLeft, Key::ArrowRight, Key::Home, Key::End] {
                if ui.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    self.revenue.key(key);
                }
            }
        }
        response.widget_info(||WidgetInfo::labeled(WidgetType::Image,true,format!("Revenue preview: {}. {}. Left and Right inspect a day; Home and End reach the endpoints.",money(self.revenue.value()),self.revenue.caption())));
        strong_text(
            ui,
            rect.min,
            money(self.revenue.value()),
            28.,
            rt.tokens.text,
        );
        text(
            ui.painter(),
            rect.min + vec2(0., 36.),
            self.revenue.caption(),
            9.,
            rt.tokens.text3,
        );
        let map =
            |p: Pos2| chart.min + vec2(p.x / 300. * chart.width(), p.y / 116. * chart.height());
        for segment in &self.revenue.grid {
            ui.painter().line_segment(
                [map(segment[0]), map(segment[1])],
                Stroke::new(0.65, rt.tokens.border),
            );
        }
        let p = self.finite(rt, self.chart_started, 0.95, 0.);
        let progress = motion::ease(Curve::Out, p);
        if p < 1. {
            rt.repaint();
        }
        let color = rt.tokens.charts[0];
        let area_clip = Rect::from_min_max(
            chart.min,
            pos2(chart.left() + chart.width() * progress, chart.bottom()),
        )
        .intersect(ui.clip_rect());
        let area = ui.painter().with_clip_rect(area_clip);
        let mut mesh = self.revenue.area.clone();
        for vertex in &mut mesh.vertices {
            vertex.pos = map(vertex.pos);
            vertex.color = color.gamma_multiply(0.065);
        }
        area.add(Shape::mesh(mesh));
        let line: Vec<_> = self
            .revenue
            .traced_samples(progress)
            .into_iter()
            .map(map)
            .collect();
        if line.len() > 1 {
            ui.painter().add(Shape::line(line, Stroke::new(2., color)));
        }
        if let Some(i) = self.revenue.selected {
            let point = self.revenue.points[i];
            let pos = map(pos2(point.x, point.y));
            for y in (6..112).step_by(6) {
                ui.painter().line_segment(
                    [
                        map(pos2(point.x, y as f32)),
                        map(pos2(point.x, (y + 3).min(112) as f32)),
                    ],
                    Stroke::new(0.7, rt.tokens.text4),
                );
            }
            ui.painter()
                .circle(pos, 4., rt.tokens.surface, Stroke::new(2., color));
        }
    }
    fn sort_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let width = ui.available_width();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Release checklist")
                    .size(10.)
                    .color(rt.tokens.text3),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} / 3",
                        self.task_checks.iter().filter(|&&v| v).count()
                    ))
                    .monospace()
                    .size(9.)
                    .color(rt.tokens.text3),
                );
            });
        });
        ui.add_space(5.);
        ui.set_max_width(width);
        let checks = &mut self.task_checks;
        self.sort.set_enabled(self.active);
        self.sort
            .show(ui, rt, "ov-task-sort", &[], 39., |ui, rt, item| {
                ui.spacing_mut().item_spacing.x = 5.;
                // Like the reference's flexing native label, the whole region
                // after the grip toggles the task, including trailing space.
                let (rect, mut response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 23.), Sense::click());
                if response.clicked() {
                    checks[item] = !checks[item];
                    response.mark_changed();
                    response.request_focus();
                }
                response.widget_info(|| {
                    WidgetInfo::selected(
                        WidgetType::Checkbox,
                        ui.is_enabled(),
                        checks[item],
                        TASKS[item],
                    )
                });
                let fraction = rt.animate(
                    ("ov-task-check-state", item),
                    if checks[item] { 1. } else { 0. },
                    0.22,
                    Curve::Out,
                );
                let check =
                    Rect::from_center_size(rect.left_center() + vec2(6.5, 0.), Vec2::splat(13.));
                ui.painter().rect(
                    check,
                    rounded(rt, 3.),
                    ui::mix(rt.tokens.surface, rt.tokens.accent, fraction),
                    Stroke::new(
                        1.,
                        ui::mix(rt.tokens.border_strong, rt.tokens.accent, fraction),
                    ),
                    StrokeKind::Inside,
                );
                if fraction > 0.001 {
                    let a = check.min + vec2(3., 6.);
                    let b = check.min + vec2(5.5, 9.);
                    let c = check.min + vec2(10., 3.5);
                    let points = if fraction < 0.4 {
                        vec![a, a.lerp(b, fraction / 0.4)]
                    } else {
                        vec![a, b, b.lerp(c, (fraction - 0.4) / 0.6)]
                    };
                    ui.painter().add(Shape::line(
                        points,
                        Stroke::new(1.5, rt.tokens.accent_contrast),
                    ));
                }
                let color = if checks[item] {
                    rt.tokens.text3
                } else {
                    rt.tokens.text
                };
                let galley = wrapped(ui, TASKS[item], 10., color, (rect.width() - 38.).max(1.), 1);
                let text_pos = rect.left_center() + vec2(21., -galley.size().y / 2.);
                ui.painter().galley(text_pos, galley.clone(), color);
                if checks[item] {
                    ui.painter().line_segment(
                        [
                            text_pos + vec2(0., galley.size().y * 0.52),
                            text_pos + vec2(galley.size().x, galley.size().y * 0.52),
                        ],
                        Stroke::new(0.65, rt.tokens.text4),
                    );
                }
                icons::paint(
                    ui.painter(),
                    "chevron-right",
                    Rect::from_center_size(rect.right_center() - vec2(6., 0.), Vec2::splat(12.)),
                    rt.tokens.text4,
                    1.2,
                    0.,
                );
                focus_ring(ui, rt, &response, 4.);
                rt.record(&format!("ov-task-check-{item}"), &response);
                None::<()>
            });
        self.task_order = self.sort.groups.first().cloned().unwrap_or_default();
    }
    fn tabs_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        if block_tabs(
            ui,
            rt,
            "ov-gallery-tabs",
            &["Overview", "Activity", "Settings"],
            &mut self.gallery_tab,
        ) {
            self.tab_started = Some(rt.now);
        }
        ui.add_space(14.);
        let base = ui.available_rect_before_wrap();
        let p = self.finite(rt, self.tab_started, 0.32, 0.);
        let p = motion::ease(Curve::Out, p);
        let mut panel = ui.new_child(
            UiBuilder::new()
                .id_salt(("panel", self.gallery_tab))
                .max_rect(base.translate(vec2(12. * (1. - p), 0.)))
                .layout(Layout::top_down(Align::Min)),
        );
        panel.set_opacity(p);
        if p < 1. {
            rt.repaint();
        }
        match self.gallery_tab {
            0 => {
                panel.label(
                    RichText::new("Weekly activity")
                        .size(9.)
                        .color(rt.tokens.text3),
                );
                panel.add_space(8.);
                let (bars, _) =
                    panel.allocate_exact_size(vec2(panel.available_width(), 65.), Sense::hover());
                let heights = [32., 48., 37., 63., 52., 78., 91., 72., 84., 61., 94., 100.];
                let w = (bars.width() - 77.) / 12.;
                for (i, h) in heights.iter().enumerate() {
                    let r = Rect::from_min_max(
                        pos2(bars.left() + i as f32 * (w + 7.), bars.bottom() - h * 0.65),
                        pos2(bars.left() + i as f32 * (w + 7.) + w, bars.bottom()),
                    );
                    panel.painter().rect_filled(
                        r,
                        2.,
                        rt.tokens
                            .accent
                            .gamma_multiply(if i % 3 == 2 { 0.28 } else { 0.65 }),
                    );
                }
            }
            1 => {
                for (label, icon, time) in [
                    ("Build completed", "check-circle", "2m"),
                    ("Review requested", "git-pull-request", "8m"),
                ] {
                    let (r, _) = panel
                        .allocate_exact_size(vec2(panel.available_width(), 39.), Sense::hover());
                    icons::paint(
                        panel.painter(),
                        icon,
                        Rect::from_center_size(r.left_center() + vec2(8., 0.), Vec2::splat(16.)),
                        rt.tokens.accent_text,
                        1.5,
                        0.,
                    );
                    text(
                        panel.painter(),
                        r.left_center() + vec2(24., -6.),
                        label,
                        10.,
                        rt.tokens.text,
                    );
                    text(
                        panel.painter(),
                        r.right_center() - vec2(18., 6.),
                        time,
                        8.,
                        rt.tokens.text3,
                    );
                    panel.painter().line_segment(
                        [r.left_bottom(), r.right_bottom()],
                        Stroke::new(1., rt.tokens.border_soft),
                    );
                }
            }
            _ => {
                for (i, label) in ["Email updates", "Weekly digest"].iter().enumerate() {
                    let r = ui::toggle(
                        &mut panel,
                        rt,
                        ("ov-setting", i),
                        label,
                        &mut self.settings[i],
                    );
                    rt.record(&format!("ov-setting-{i}"), &r);
                    panel.add_space(8.);
                }
            }
        }
    }
    fn icons_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let rect = ui.max_rect();
        let step = 56_f32.min((rect.width() - 4.) / 4.);
        let side = 44_f32.min(step - 4.);
        let origin = rect.center() - vec2(step * 1.5 + side / 2., 58.);
        for (i, name) in ICONS.iter().enumerate() {
            let r = Rect::from_min_size(
                origin + vec2((i % 4) as f32 * step, (i / 4) as f32 * 56.),
                Vec2::splat(side),
            );
            let response = control(
                ui,
                rt,
                ("ov-icon", name),
                r,
                &format!("Play {name} icon"),
                None,
            );
            rt.record(&format!("ov-icon-{name}"), &response);
            let engaged = response.hovered() || response.has_focus();
            if engaged {
                self.icon_name = (*name).into();
            }
            let highlight = rt.animate(
                ("ov-icon-highlight", name),
                if engaged { 1. } else { 0. },
                0.22,
                Curve::Out,
            );
            ui.painter().rect(
                r,
                rounded(rt, 8.),
                egui::lerp(
                    egui::Rgba::from(rt.tokens.surface)..=egui::Rgba::from(rt.tokens.accent_soft),
                    highlight,
                ),
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
            if !rt.reduced()
                && self.icon_started[i].is_none()
                && ((engaged && !self.icon_engaged[i]) || response.clicked())
            {
                self.icon_started[i] = Some(rt.now);
            }
            self.icon_engaged[i] = engaged;
            let progress = if let Some(start) = self.icon_started[i] {
                let progress =
                    ((rt.now - start) / (1.2 * rt.motion_scale().max(0.001)) as f64) as f32;
                if progress >= 1. || rt.reduced() {
                    self.icon_started[i] = None;
                    0.
                } else {
                    rt.repaint();
                    progress.max(0.00001)
                }
            } else {
                0.
            };
            icons::paint(
                ui.painter(),
                name,
                Rect::from_center_size(r.center(), Vec2::splat(25.)),
                if engaged {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text2
                },
                1.5,
                progress,
            );
            response.on_hover_text(*name);
        }
        centered(
            ui.painter(),
            rect.center() + vec2(0., 70.),
            &self.icon_name,
            9.,
            rt.tokens.text3,
        );
    }
    fn counter_preview(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let rect = ui.max_rect();
        let center = rect.center() - vec2(0., 12.);
        let digit_width = ((rect.width() - 86.) / 3.).clamp(24., 38.);
        let digits = Rect::from_center_size(center, vec2(digit_width * 3., 72.));
        for (step, label, icon) in [
            (-1, "Decrease preview counter", "minus"),
            (1, "Increase preview counter", "plus"),
        ] {
            let x = if step < 0 {
                digits.left() - 28.
            } else {
                digits.right() + 28.
            };
            let r = Rect::from_center_size(pos2(x, center.y), Vec2::splat(30.));
            let response = control(ui, rt, ("ov-counter-step", step), r, label, None);
            rt.record(&format!("ov-counter-step-{step}"), &response);
            ui.painter().circle(
                r.center(),
                15.,
                if response.hovered() {
                    rt.tokens.accent_soft
                } else {
                    rt.tokens.surface
                },
                Stroke::new(
                    1.,
                    if response.hovered() {
                        rt.tokens.accent_ring
                    } else {
                        rt.tokens.border
                    },
                ),
            );
            icons::paint(
                ui.painter(),
                icon,
                r.shrink(6.),
                rt.tokens.text2,
                1.5,
                rt.icon_progress(
                    ("ov-counter-icon", step),
                    response.hovered() || response.has_focus(),
                ),
            );
            if response.clicked() {
                self.step_counter(step, rt.now, rt.motion_scale());
            }
        }
        let painter = ui
            .painter()
            .with_clip_rect(digits.intersect(ui.clip_rect()));
        for (i, digit) in self.counter_digits.iter().enumerate() {
            let v = digit.sample(rt.now);
            for n in 0..10 {
                let y = digits.top() + (n as f32 - v) * 72.;
                if y + 72. >= digits.top() && y < digits.bottom() {
                    painter.text(
                        pos2(digits.left() + (i as f32 + 0.5) * digit_width, y + 36.),
                        Align2::CENTER_CENTER,
                        n,
                        ui::strong_font(ui.ctx(), 65. * digit_width / 38.),
                        rt.tokens.text,
                    );
                }
            }
            if digit.moving(rt.now) {
                rt.repaint();
            }
        }
        let output = ui.interact(
            digits,
            ui.make_persistent_id("ov-counter-output"),
            Sense::hover(),
        );
        output.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Label, true, format!("{:03}", self.counter))
        });
        live_status(&output);
        rt.record("ov-counter", &output);
        let reset = Rect::from_center_size(pos2(center.x, rect.bottom() - 13.), vec2(72., 24.));
        let response = control(ui, rt, "ov-counter-reset", reset, "Reset counter", None);
        rt.record("ov-counter-reset", &response);
        centered(
            ui.painter(),
            reset.center() - vec2(8., 0.),
            "Reset",
            9.,
            rt.tokens.text3,
        );
        icons::paint(
            ui.painter(),
            "refresh",
            Rect::from_center_size(reset.center() + vec2(18., 0.), Vec2::splat(12.)),
            rt.tokens.text3,
            1.5,
            rt.icon_progress(
                "ov-counter-reset-icon",
                response.hovered() || response.has_focus(),
            ),
        );
        if response.clicked() {
            self.set_counter(24, rt.now, rt.motion_scale());
        }
    }
}

fn wrapped(
    ui: &Ui,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::simple(text.into(), FontId::proportional(size), color, width);
    job.wrap.max_rows = rows;
    ui.painter().layout_job(job)
}

impl State {
    fn studies(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.add_space(36.);
        ui.label(
            RichText::new("Preview")
                .font(ui::strong_font(ui.ctx(), 22.))
                .color(rt.tokens.text),
        );
        ui.add_space(12.);
        let width = ui.available_width().min(600.);
        let outer = egui::Frame::new()
            .fill(rt.tokens.surface)
            .stroke(Stroke::new(1., rt.tokens.border))
            .corner_radius(rounded(rt, 16.))
            .inner_margin(if width < 400. { 16. } else { 20. });
        let result = outer.show(ui, |ui| {
            ui.set_width((width - 40.).max(1.));
            let old = self.study;
            if block_tabs(
                ui,
                rt,
                "ov-study-tabs",
                &["Checklist", "Pinned list", "Accordion"],
                &mut self.study,
            ) {
                self.study_started = Some(rt.now);
                self.status = [
                    "Checklist selected.",
                    "Pinned list selected.",
                    "Accordion selected.",
                ][self.study]
                    .into();
            }
            if old != self.study {
                rt.repaint();
            }
            ui.add_space(16.);
            let mut stage_height=220_f32;
            if self.study==2 {
                let descriptions=["Yes. Open the native application locally. No network connection is required.","Open Tune tokens and select Reduced under Motion. Your system preference also applies."];
                let mut full_height=28.;
                for (i,description) in descriptions.iter().enumerate() {
                    let fraction=rt.animate(("ov-study-reveal-height",i),if self.disclosures[i]{1.}else{0.},0.32,Curve::Out);
                    full_height+=47.+fraction*(wrapped(ui,description,12.,rt.tokens.text2,ui.available_width()-20.,6).size().y+16.);
                }
                stage_height=stage_height.max(full_height);
            }
            let (stage, _) = ui.allocate_exact_size(vec2(ui.available_width(), stage_height), Sense::hover());
            let p = self.finite(rt, self.study_started, 0.32, 0.);
            let eased = motion::ease(Curve::Out, p);
            let mut panel = ui.new_child(
                UiBuilder::new()
                    .id_salt(("ov-study", self.study))
                    .max_rect(stage.translate(vec2(0., 8. * (1. - eased))))
                    .layout(Layout::top_down(Align::Min)),
            );
            panel.set_opacity(eased);
            panel.set_clip_rect(stage.intersect(ui.clip_rect()));
            panel.spacing_mut().item_spacing=vec2(8.,4.);
            panel.spacing_mut().interact_size=vec2(20.,18.);
            if p < 1. {
                rt.repaint();
            }
            match self.study {
                0 => self.checklist_study(&mut panel, rt),
                1 => self.pinned_study(&mut panel, rt),
                _ => self.accordion_study(&mut panel, rt),
            }
            ui.horizontal(|ui| {
                // The status is also exposed to native assistive technology.
                let status = ui.add(
                    egui::Label::new(RichText::new(&self.status).size(10.).color(rt.tokens.text3))
                        .truncate(),
                );
                rt.record("ov-study-status", &status);
                live_status(&status);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let r = ui::icon_button(ui, rt, "ov-study-reset", "refresh", "Reset previews");
                    rt.record("ov-study-reset", &r);
                    if r.clicked() {
                        self.reset_studies();
                    }
                });
            });
        });
        rt.anchor("ov-preview", "Preview", result.response.rect);
    }
    fn checklist_study(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Release checklist")
                    .size(11.)
                    .color(rt.tokens.text3),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} / 3",
                        self.study_tasks.iter().filter(|&&v| v).count()
                    ))
                    .monospace()
                    .size(11.)
                    .color(rt.tokens.text3),
                );
            });
        });
        ui.add_space(12.);
        for (i, label) in STUDY_TASKS.iter().enumerate() {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 44.), Sense::hover());
            let response = control(
                ui,
                rt,
                ("ov-study-task", i),
                rect,
                label,
                Some(self.study_tasks[i]),
            );
            rt.record(&format!("ov-study-task-{i}"), &response);
            if response.clicked() {
                self.toggle_study_task(i);
            }
            let selected = rt.animate(
                ("ov-study-check", i),
                if self.study_tasks[i] { 1. } else { 0. },
                0.26,
                Curve::Out,
            );
            ui.painter().rect(
                rect,
                rounded(rt, 8.),
                if self.study_tasks[i] {
                    rt.tokens.accent_soft
                } else {
                    rt.tokens.surface2
                },
                Stroke::new(
                    1.,
                    if self.study_tasks[i] {
                        rt.tokens.accent_ring
                    } else if response.hovered() {
                        rt.tokens.border_strong
                    } else {
                        rt.tokens.border_soft
                    },
                ),
                StrokeKind::Inside,
            );
            let check =
                Rect::from_center_size(rect.left_center() + vec2(14., 0.), Vec2::splat(20.));
            ui.painter().rect(
                check,
                rounded(rt, 5.),
                if self.study_tasks[i] {
                    rt.tokens.accent
                } else {
                    rt.tokens.surface
                },
                Stroke::new(
                    1.,
                    if self.study_tasks[i] {
                        rt.tokens.accent
                    } else {
                        rt.tokens.border_strong
                    },
                ),
                StrokeKind::Inside,
            );
            if selected > 0.001 {
                let a = check.min + vec2(5., 10.);
                let b = check.min + vec2(9., 14.);
                let c = check.min + vec2(15., 6.);
                let points = if selected < 0.4 {
                    vec![a, a.lerp(b, selected / 0.4)]
                } else {
                    vec![a, b, b.lerp(c, (selected - 0.4) / 0.6)]
                };
                ui.painter().add(Shape::line(
                    points,
                    Stroke::new(1.7, rt.tokens.accent_contrast),
                ));
            }
            text(
                ui.painter(),
                rect.left_center() + vec2(36., -7.),
                label,
                13.,
                if self.study_tasks[i] {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text
                },
            );
            ui.painter().text(
                rect.right_center() - vec2(4., 0.),
                Align2::RIGHT_CENTER,
                format!("0{}", i + 1),
                FontId::monospace(11.),
                rt.tokens.text4,
            );
            ui.add_space(4.);
        }
        ui.add_space(8.);
        let target = self.study_tasks.iter().filter(|&&v| v).count() as f32 / 3.;
        let fraction = rt.animate("ov-study-meter", target, 0.32, Curve::Out);
        ui::progress(ui, rt, fraction, 3.);
    }
    fn pinned_study(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.label(
            RichText::new("Project files")
                .size(11.)
                .color(rt.tokens.text3),
        );
        ui.add_space(12.);
        let (stage, _) = ui.allocate_exact_size(vec2(ui.available_width(), 156.), Sense::hover());
        for (position, i) in self.pinned_order().into_iter().enumerate() {
            let y = rt.animate(
                ("ov-pin-position", i),
                position as f32 * 50.,
                0.47,
                Curve::SpringSnappy,
            );
            let rect = Rect::from_min_size(stage.min + vec2(0., y), vec2(stage.width(), 44.));
            let response = control(
                ui,
                rt,
                ("ov-study-pin", i),
                rect,
                &format!(
                    "{} {}",
                    if self.pins[i] { "Unpin" } else { "Pin" },
                    FILES[i]
                ),
                Some(self.pins[i]),
            );
            rt.record(&format!("ov-study-pin-{i}"), &response);
            let highlight = response.hovered() || response.has_focus();
            ui.painter().rect(
                rect,
                rounded(rt, 7.),
                if self.pins[i] {
                    rt.tokens.accent_soft
                } else {
                    rt.tokens.surface2
                },
                Stroke::new(
                    1.,
                    if self.pins[i] || highlight {
                        rt.tokens.accent_ring
                    } else {
                        rt.tokens.border
                    },
                ),
                StrokeKind::Inside,
            );
            icons::paint(
                ui.painter(),
                if i == 0 { "layers" } else { "document" },
                Rect::from_center_size(rect.left_center() + vec2(18., 0.), Vec2::splat(16.)),
                rt.tokens.accent_text,
                1.5,
                rt.icon_progress(("ov-pin-file-icon", i), highlight),
            );
            text(
                ui.painter(),
                rect.left_center() + vec2(36., -7.),
                FILES[i],
                13.,
                if self.pins[i] {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text
                },
            );
            ui.painter().text(
                rect.right_center() - vec2(36., 0.),
                Align2::RIGHT_CENTER,
                if self.pins[i] { "Pinned" } else { "Pin" },
                FontId::proportional(11.),
                if self.pins[i] {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text3
                },
            );
            icons::paint(
                ui.painter(),
                "plus",
                Rect::from_center_size(rect.right_center() - vec2(17., 0.), Vec2::splat(14.)),
                rt.tokens.text3,
                1.5,
                rt.icon_progress(("ov-pin-action-icon", i), highlight),
            );
            if response.clicked() {
                self.toggle_pin(i);
            }
        }
    }
    fn accordion_study(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.label(RichText::new("FAQ").size(11.).color(rt.tokens.text3));
        ui.add_space(8.);
        for (i,(label,desc)) in [
            ("Does it work offline?","Yes. Open the native application locally. No network connection is required."),
            ("How do I reduce animations?","Open Tune tokens and select Reduced under Motion. Your system preference also applies.")
        ].iter().enumerate() {
            let (rect,_)=ui.allocate_exact_size(vec2(ui.available_width(),43.),Sense::hover());
            let response=control(ui,rt,("ov-study-reveal",i),rect,label,Some(self.disclosures[i]));rt.record(&format!("ov-study-reveal-{i}"),&response);
            if response.clicked(){self.disclosures[i] = !self.disclosures[i];self.status=if self.disclosures[i]{"Expanded."}else{"Collapsed."}.into();}
            ui.ctx().accesskit_node_builder(response.id,|node|{node.set_role(egui::accesskit::Role::Button);node.set_expanded(self.disclosures[i]);});
            text(ui.painter(),rect.left_center()+vec2(0.,-7.),label,13.,rt.tokens.text);
            let fraction=rt.animate(("ov-study-reveal-height",i),if self.disclosures[i]{1.}else{0.},0.32,Curve::Out);
            let icon=rect.right_center()-vec2(9.,0.);let angle=std::f32::consts::FRAC_PI_4*fraction;let x=vec2(angle.cos(),angle.sin())*5.;let y=vec2(-angle.sin(),angle.cos())*5.;
            ui.painter().line_segment([icon-x,icon+x],Stroke::new(1.5,rt.tokens.text3));ui.painter().line_segment([icon-y,icon+y],Stroke::new(1.5,rt.tokens.text3));
            let galley=wrapped(ui,desc,12.,rt.tokens.text2,ui.available_width()-20.,6);let target=galley.size().y+16.;
            if fraction>0.001 {
                let (content,_)=ui.allocate_exact_size(vec2(ui.available_width(),target*fraction),Sense::hover());
                let painter=ui.painter().with_clip_rect(content.intersect(ui.clip_rect())).with_opacity(fraction);
                painter.galley(content.min+vec2(0.,4.-4.*fraction),galley,rt.tokens.text2);
                if self.disclosures[i] {let answer=ui.interact(content,ui.make_persistent_id(("ov-study-answer",i)),Sense::hover());answer.widget_info(||WidgetInfo::labeled(WidgetType::Label,true,*desc));}
            }
            ui.painter().line_segment([pos2(rect.left(),ui.cursor().min.y),pos2(rect.right(),ui.cursor().min.y)],Stroke::new(1.,rt.tokens.border_soft));
        }
    }
    fn collections(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.add_space(36.);
        ui.label(
            RichText::new("Collections")
                .font(ui::strong_font(ui.ctx(), 22.))
                .color(rt.tokens.text),
        );
        ui.add_space(12.);
        let routes = [
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
        let cols = if ui.ctx().input(|i| i.content_rect().width()) > 600. {
            3
        } else {
            1
        };
        let width = (ui.available_width() - 8. * (cols - 1) as f32) / cols as f32;
        let start = ui.cursor().min;
        for row in routes.chunks(cols) {
            let (allocation, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 44.), Sense::hover());
            for (i, route) in row.iter().enumerate() {
                let rect = Rect::from_min_size(
                    allocation.min + vec2(i as f32 * (width + 8.), 0.),
                    vec2(width, 44.),
                );
                let response = control(
                    ui,
                    rt,
                    ("ov-collection", route.id()),
                    rect,
                    route.title(),
                    None,
                );
                rt.record(&format!("ov-collection-{}", route.id()), &response);
                let engaged = response.hovered() || response.has_focus();
                ui.painter().rect(
                    rect,
                    rounded(rt, 7.),
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
                let progress = rt.icon_progress(("ov-collection-icon", route.id()), engaged);
                icons::paint(
                    ui.painter(),
                    route.icon(),
                    Rect::from_center_size(rect.left_center() + vec2(21., 0.), Vec2::splat(18.)),
                    rt.tokens.text2,
                    1.5,
                    progress,
                );
                let galley = wrapped(
                    ui,
                    route.title(),
                    12.,
                    rt.tokens.text,
                    rect.width() - 76.,
                    1,
                );
                ui.painter().galley(
                    rect.left_center() + vec2(44., -galley.size().y / 2.),
                    galley,
                    rt.tokens.text,
                );
                icons::paint(
                    ui.painter(),
                    "arrow-right",
                    Rect::from_center_size(rect.right_center() - vec2(19., 0.), Vec2::splat(14.)),
                    rt.tokens.text3,
                    1.5,
                    progress,
                );
                if response.clicked() {
                    rt.navigate(*route);
                }
            }
            ui.add_space(8.);
        }
        rt.anchor(
            "ov-map",
            "Collections",
            Rect::from_min_max(start, ui.cursor().min),
        );
        ui.add_space(28.);
        ui.separator();
        ui.add_space(8.);
        let width = ui.available_width();
        let wrap = width < 272.;
        let (footer, _) =
            ui.allocate_exact_size(vec2(width, if wrap { 72. } else { 34. }), Sense::hover());
        for (id, label, icon, rect) in [
            (
                "ov-commands",
                "Commands",
                "command",
                Rect::from_min_size(footer.min, vec2(98., 32.)),
            ),
            (
                "ov-back-to-components",
                "Back to components",
                "chevron-down",
                Rect::from_min_size(
                    pos2(
                        footer.right() - 158.,
                        footer.top() + if wrap { 38. } else { 0. },
                    ),
                    vec2(158., 32.),
                ),
            ),
        ] {
            let response = control(ui, rt, id, rect, label, None);
            rt.record(id, &response);
            let engaged = response.hovered() || response.has_focus();
            let hover = rt.animate(
                (id, "hover"),
                if engaged { 1. } else { 0. },
                0.22,
                Curve::Out,
            );
            ui.painter().rect_filled(
                rect,
                rounded(rt, 5.),
                rt.tokens.surface2.gamma_multiply(hover),
            );
            icons::paint(
                ui.painter(),
                icon,
                Rect::from_center_size(rect.left_center() + vec2(16., 0.), Vec2::splat(14.)),
                rt.tokens.text2,
                1.5,
                rt.icon_progress((id, "icon"), engaged),
            );
            strong_text(
                ui,
                rect.left_center() + vec2(30., -6.),
                label,
                11.,
                rt.tokens.text2,
            );
            if response.clicked() {
                if id == "ov-commands" {
                    rt.request_commands();
                } else {
                    self.focus_search = true;
                    self.scroll_to_search = true;
                    rt.repaint();
                }
            }
        }
    }
}

impl Page for State {
    fn show(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        if !self.active {
            self.enter(rt);
        }
        // A max-width supplied by an outer scroll frame can exceed its actual
        // visible canvas on a narrow native window. Keep this page inside it.
        let clip = ui.clip_rect();
        let origin = ui.cursor().min;
        let remaining = (clip.right() - origin.x).max(1.);
        if ui.available_width() > remaining {
            let inset = (origin.x - clip.left()).clamp(0., 56.);
            ui.set_max_width((remaining - inset).max(1.));
        }
        // The shell's larger control spacing must not change the measured
        // gallery/study composition or clip the third task inside a preview.
        ui.spacing_mut().item_spacing = vec2(8., 4.);
        // A native window may remain visible while another window has focus.
        // Match document visibility, rather than cancelling work on mere blur.
        if ui.input(|i| i.viewport().minimized.unwrap_or(false)) {
            self.cancel_action();
            self.sort.set_enabled(false);
            self.settle();
            return;
        }
        if rt.reduced() {
            self.settle();
        }
        if !ui.ctx().wants_keyboard_input()
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Slash))
        {
            self.focus_search = true;
            self.scroll_to_search = true;
            ui.input_mut(|i| {
                i.events
                    .retain(|e| !matches!(e,egui::Event::Text(t) if t=="/"))
            });
        }
        self.header(ui, rt);
        self.search(ui, rt);
        self.filters(ui, rt);
        if self.last_query != self.query
            || self.last_category != self.category
            || self.last_limit != self.limit
        {
            self.last_query = self.query.clone();
            self.last_category = self.category.clone();
            self.last_limit = self.limit;
            self.cancel_filtered_action();
        }
        let matches = self.matches();
        self.count(ui, rt, matches.len());
        let visible = self.visible();
        self.gallery(ui, rt, &visible);
        if matches.len() > self.limit {
            ui.add_space(16.);
            ui.vertical_centered(|ui| {
                let response = ui::button(
                    ui,
                    rt,
                    "ov-show-more",
                    "Show more",
                    Some("plus"),
                    ButtonKind::Secondary,
                );
                rt.record("ov-show-more", &response);
                if response.clicked() {
                    self.show_more();
                }
            });
        }
        self.studies(ui, rt);
        self.collections(ui, rt);
        ui.add_space(24.);
    }
    fn enter(&mut self, rt: &mut Runtime) {
        self.active = true;
        self.entered = rt.now;
        self.chart_visible = false;
        self.chart_started = None;
        self.chart_hovered = false;
        self.revenue.selected = None;
        self.settle();
    }
    fn leave(&mut self, _rt: &mut Runtime) {
        self.active = false;
        self.cancel_action();
        self.sort.set_enabled(false);
        self.task_order = self.sort.groups.first().cloned().unwrap_or_default();
        self.chart_visible = false;
        self.settle();
        self.snapshots.clear();
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    fn draw(
        s: &mut State,
        rt: &mut Runtime,
        ctx: &egui::Context,
        time: f64,
        size: Vec2,
        events: Vec<egui::Event>,
        scroll: f32,
    ) -> egui::FullOutput {
        // Each helper key event is a tap, not a key held across later frames.
        // Supply real releases so egui does not reclassify the next tap as repeat.
        let mut released = ctx.input(|input| {
            input
                .keys_down
                .iter()
                .map(|&key| egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                })
                .collect::<Vec<_>>()
        });
        released.extend(events);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(time),
            events: released,
            focused: true,
            ..Default::default()
        };
        ctx.run(input, |ctx| {
            rt.begin_frame();
            egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(16).fill(rt.tokens.bg))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("overview-test-scroll")
                        .vertical_scroll_offset(scroll)
                        .show(ui, |ui| {
                            ui.set_max_width(1080.);
                            s.show(ui, rt)
                        });
                });
        })
    }
    fn record(rt: &Runtime, id: &str) -> Rect {
        let r = rt.records.iter().find(|r| r.id == id).unwrap_or_else(|| {
            panic!(
                "Missing record {id}; found {:?}",
                rt.records.iter().map(|r| &r.id).collect::<Vec<_>>()
            )
        });
        Rect::from_min_max(pos2(r.rect[0], r.rect[1]), pos2(r.rect[2], r.rect[3]))
    }
    fn pointer(pos: Pos2, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ]
    }
    fn key(key: Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }
    fn click(
        s: &mut State,
        rt: &mut Runtime,
        ctx: &egui::Context,
        id: &str,
        time: f64,
        size: Vec2,
        scroll: f32,
    ) {
        let pos = record(rt, id).center();
        let _ = draw(s, rt, ctx, time, size, pointer(pos, true), scroll);
        let _ = draw(s, rt, ctx, time + 0.016, size, pointer(pos, false), scroll);
    }

    #[test]
    fn native_input_runs_actions_search_navigation_and_all_three_column_layouts() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1000.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let output = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        assert!(!output.shapes.is_empty());
        let a = record(&rt, "ov-specimen-components-2");
        let b = record(&rt, "ov-specimen-data-0");
        let c = record(&rt, "ov-specimen-primitives-3");
        assert!((a.top() - b.top()).abs() < 1.);
        assert!((a.top() - c.top()).abs() < 1.);
        assert!(a.right() < b.left() && b.right() < c.left());
        click(&mut s, &mut rt, &ctx, "ov-action", 1.1, size, 0.);
        assert!(matches!(s.action, Action::Running { .. }));
        let _ = draw(&mut s, &mut rt, &ctx, 1.35, size, vec![], 0.);
        assert!(s.action_progress(rt.now) > 0. && s.action_progress(rt.now) < 1.);
        assert!(
            !rt.records
                .iter()
                .find(|r| r.id == "ov-action")
                .unwrap()
                .enabled
        );
        click(&mut s, &mut rt, &ctx, "ov-filter-data", 1.4, size, 0.);
        assert_eq!(s.action, Action::Ready);
        assert_eq!(s.action_caption, "Cancelled");
        assert_eq!(s.category, "data");
        assert_eq!(s.matches().len(), 5);
        assert!(s.travels.values().any(|t| t.exiting));
        let _ = draw(&mut s, &mut rt, &ctx, 2.2, size, vec![], 0.);
        assert!(s.travels.values().all(|t| !t.exiting));
        s.clear_filters();
        let _ = draw(&mut s, &mut rt, &ctx, 3., size, vec![], 0.);
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            4.,
            size,
            vec![egui::Event::Text("accordion".into())],
            0.,
        );
        assert_eq!(s.query, "accordion");
        assert!(!s.matches().is_empty());
        let _ = draw(&mut s, &mut rt, &ctx, 5., size, vec![], 0.);
        let id = ENTRIES[s.visible()[0]].id;
        click(
            &mut s,
            &mut rt,
            &ctx,
            &format!("ov-specimen-{id}"),
            5.1,
            size,
            0.,
        );
        assert_eq!(rt.jump_to.as_deref(), Some(id));
        assert_eq!(
            rt.requested,
            Some(Route::from_id(ENTRIES[s.visible()[0]].page))
        );
        s.clear_filters();
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        rt.set_palette(true);
        rt.set_theme(true);
        for (n, (width, cols)) in [(1000., 2), (390., 1), (320., 1)].into_iter().enumerate() {
            let size = vec2(width, 1000.);
            let _ = draw(&mut s, &mut rt, &ctx, 7. + n as f64, size, vec![], 0.);
            let a = record(&rt, "ov-specimen-components-2");
            let b = record(&rt, "ov-specimen-data-0");
            if cols == 1 {
                assert!(b.top() > a.bottom());
            } else {
                assert!((a.top() - b.top()).abs() < 1.);
            }
            for r in rt
                .records
                .iter()
                .filter(|r| r.id.starts_with("ov-specimen-") || r.id.starts_with("ov-filter-"))
            {
                assert!(
                    r.rect[0] >= -0.5 && r.rect[2] <= width + 0.5,
                    "overflow at {width}: {} {:?}",
                    r.id,
                    r.rect
                );
            }
        }
    }

    #[test]
    fn native_keyboard_sort_counter_tabs_and_cleanup_retain_state() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1200.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        let grip = rt
            .records
            .iter()
            .find(|r| r.id.starts_with("sort-grip-") && r.id.ends_with("-0"))
            .unwrap()
            .id
            .clone();
        click(&mut s, &mut rt, &ctx, &grip, 1.1, size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.2, size, vec![key(Key::Space)], 0.);
        assert!(s.sort.active());
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            1.3,
            size,
            vec![key(Key::ArrowDown)],
            0.,
        );
        let _ = draw(&mut s, &mut rt, &ctx, 1.4, size, vec![key(Key::Space)], 0.);
        assert_eq!(s.task_order, vec![1, 0, 2]);
        assert!(!s.sort.active());
        click(&mut s, &mut rt, &ctx, "ov-task-check-1", 1.5, size, 0.);
        assert_eq!(s.task_checks, [true, true, false]);
        click(&mut s, &mut rt, &ctx, "segment:Settings", 1.6, size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 2., size, vec![], 0.);
        assert_eq!(s.gallery_tab, 2);
        click(&mut s, &mut rt, &ctx, "ov-setting-0", 2.1, size, 0.);
        assert!(!s.settings[0]);
        click(&mut s, &mut rt, &ctx, "ov-counter-step-1", 2.2, size, 0.);
        assert_eq!(s.counter, 25);
        assert!(s.counter_digits.iter().any(|d| d.moving(rt.now)));
        s.start_action(2.4);
        s.leave(&mut rt);
        assert_eq!(s.action, Action::Ready);
        assert_eq!(s.action_caption, "Cancelled");
        assert!(!s.sort.active());
        assert!(s.snapshots.is_empty());
        assert!(s.travels.values().all(|t| !t.moving(2.4)));
        rt.now = 3.;
        s.enter(&mut rt);
        assert_eq!(s.task_order, vec![1, 0, 2]);
        assert_eq!(s.task_checks, [true, true, false]);
        assert!(!s.settings[0]);
        assert_eq!(s.counter, 25);
    }

    #[test]
    fn native_chart_keyboard_pointer_and_icon_replay_are_live() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1100.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        click(&mut s, &mut rt, &ctx, "ov-range-30", 1.1, size, 0.);
        assert_eq!(s.revenue.range, 30);
        assert!(s.finite(&rt, s.chart_started, 0.95, 0.) < 1.);
        let chart = record(&rt, "ov-revenue-chart");
        let pos = chart.min + vec2(chart.width() * 8. / 300., chart.height() * 0.8);
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            1.3,
            size,
            vec![egui::Event::PointerMoved(pos)],
            0.,
        );
        assert_eq!(s.revenue.selected, Some(0));
        click(&mut s, &mut rt, &ctx, "ov-revenue-chart", 1.4, size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.5, size, vec![key(Key::End)], 0.);
        assert_eq!(s.revenue.caption(), "Day 30 of 30");
        let _ = draw(&mut s, &mut rt, &ctx, 1.6, size, vec![key(Key::Home)], 0.);
        assert_eq!(s.revenue.caption(), "Day 1 of 30");
        click(&mut s, &mut rt, &ctx, "ov-range-7", 1.7, size, 0.);
        assert_eq!(s.revenue.value(), 4190);
        assert!(s.revenue.selected.is_none());
        click(&mut s, &mut rt, &ctx, "ov-icon-send", 2., size, 0.);
        assert_eq!(s.icon_name, "send");
        assert!(s.icon_started[0].is_some());
        let _ = draw(&mut s, &mut rt, &ctx, 3.5, size, vec![], 0.);
        assert!(s.icon_started[0].is_none());
        click(&mut s, &mut rt, &ctx, "ov-icon-send", 3.6, size, 0.);
        assert!(s.icon_started[0].is_some());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let _ = draw(&mut s, &mut rt, &ctx, 3.8, size, vec![], 0.);
        assert!(s.icon_started.iter().all(Option::is_none));
        assert!(s.chart_started.is_none());
        click(&mut s, &mut rt, &ctx, "ov-action", 4., size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 4.23, size, vec![], 0.);
        assert!(matches!(s.action, Action::Running { .. }));
        let _ = draw(&mut s, &mut rt, &ctx, 5.4, size, vec![], 0.);
        assert_eq!(s.action, Action::Complete);
    }

    #[test]
    fn native_studies_collection_commands_and_narrow_footer_are_operable() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        s.set_query("no-result-921", 0.);
        let size = vec2(1440., 1800.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        click(&mut s, &mut rt, &ctx, "ov-study-task-1", 1.1, size, 0.);
        assert_eq!(s.study_tasks, [true, true, false]);
        click(&mut s, &mut rt, &ctx, "segment:Pinned list", 1.2, size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.8, size, vec![], 0.);
        click(&mut s, &mut rt, &ctx, "ov-study-pin-2", 1.9, size, 0.);
        assert_eq!(s.pinned_order(), vec![2, 0, 1]);
        assert_eq!(s.status, "File pinned.");
        click(&mut s, &mut rt, &ctx, "segment:Accordion", 2., size, 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 2.6, size, vec![], 0.);
        click(&mut s, &mut rt, &ctx, "ov-study-reveal-0", 2.7, size, 0.);
        assert!(s.disclosures[0]);
        click(&mut s, &mut rt, &ctx, "ov-study-reveal-1", 2.8, size, 0.);
        assert_eq!(s.disclosures, [true, true]);
        let _ = draw(&mut s, &mut rt, &ctx, 3.3, size, vec![], 0.);
        click(&mut s, &mut rt, &ctx, "ov-collection-motion", 3.4, size, 0.);
        assert_eq!(rt.requested, Some(Route::Motion));
        click(&mut s, &mut rt, &ctx, "ov-commands", 3.5, size, 0.);
        assert!(rt.commands_requested);
        click(&mut s, &mut rt, &ctx, "ov-study-reset", 3.6, size, 0.);
        assert_eq!(s.study_tasks, [true, false, false]);
        assert_eq!(s.pins, [false; 3]);
        assert_eq!(s.disclosures, [false; 2]);
        assert_eq!(s.study, 2);
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        rt.set_theme(true);
        rt.set_palette(true);
        for (i, width) in [390., 320.].iter().enumerate() {
            let _ = draw(
                &mut s,
                &mut rt,
                &ctx,
                4. + i as f64,
                vec2(*width, 1800.),
                vec![],
                0.,
            );
            for r in rt.records.iter().filter(|r| {
                r.id.starts_with("ov-collection-")
                    || r.id == "ov-commands"
                    || r.id == "ov-back-to-components"
                    || r.id.starts_with("ov-study-reveal-")
            }) {
                assert!(
                    r.rect[0] >= -0.5 && r.rect[2] <= width + 0.5,
                    "overflow {} {:?}",
                    r.id,
                    r.rect
                );
            }
        }
        s.query.clear();
        s.category = "all".into();
        let _ = draw(&mut s, &mut rt, &ctx, 6., size, vec![], 0.);
        ctx.memory_mut(|m| m.surrender_focus(Id::new("ov-search-input")));
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            6.1,
            size,
            vec![key(Key::Slash), egui::Event::Text("/".into())],
            0.,
        );
        assert!(ctx.memory(|m| m.has_focus(Id::new("ov-search-input"))));
        assert!(s.query.is_empty());
    }

    #[test]
    fn native_pointer_sort_drop_escape_and_filter_cancellation_are_transactions() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1200.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        let grip_id = rt
            .records
            .iter()
            .find(|r| r.id.starts_with("sort-grip-") && r.id.ends_with("-0"))
            .unwrap()
            .id
            .clone();
        let start = record(&rt, &grip_id).center();
        let last = record(&rt, "ov-task-check-2").center();
        let end = pos2(start.x, last.y + 18.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.1, size, pointer(start, true), 0.);
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            1.2,
            size,
            vec![egui::Event::PointerMoved(end)],
            0.,
        );
        assert!(s.sort.active());
        let _ = draw(&mut s, &mut rt, &ctx, 1.3, size, pointer(end, false), 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.8, size, vec![], 0.);
        assert_eq!(s.task_order, vec![1, 2, 0]);
        assert!(!s.sort.active());
        let start = record(&rt, &grip_id).center();
        let end = pos2(start.x, record(&rt, "ov-task-check-1").center().y - 12.);
        let _ = draw(&mut s, &mut rt, &ctx, 2., size, pointer(start, true), 0.);
        let _ = draw(
            &mut s,
            &mut rt,
            &ctx,
            2.1,
            size,
            vec![egui::Event::PointerMoved(end)],
            0.,
        );
        assert!(s.sort.active());
        let _ = draw(&mut s, &mut rt, &ctx, 2.2, size, vec![key(Key::Escape)], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 2.3, size, pointer(end, false), 0.);
        assert_eq!(s.task_order, vec![1, 2, 0]);
        assert!(!s.sort.active());
        assert!(s.sort.begin_keyboard(0));
        s.sort.key(Key::Home);
        assert_ne!(s.sort.groups[0], vec![1, 2, 0]);
        s.set_category("data");
        let _ = draw(&mut s, &mut rt, &ctx, 3., size, vec![], 0.);
        assert!(!s.sort.active());
        assert_eq!(s.task_order, vec![1, 2, 0]);
        assert_eq!(s.task_checks, [true, false, false]);
    }

    #[test]
    fn native_entrance_has_intermediate_word_frames_and_reduced_settles_exits() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1000.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let output = draw(&mut s, &mut rt, &ctx, 0.1, size, vec![], 0.);
        let mut words: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let Shape::Text(t) = &s.shape {
                    if ["Component", "library"].contains(&t.galley.job.text.as_str()) {
                        Some((t.galley.job.text.clone(), t.pos.y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        words.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(words.len(), 2);
        assert!(words[1].1 > words[0].1, "staggered words {words:?}");
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        s.set_category("data");
        let _ = draw(&mut s, &mut rt, &ctx, 1.1, size, vec![], 0.);
        assert!(s.travels.values().any(|t| t.exiting));
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let _ = draw(&mut s, &mut rt, &ctx, 1.13, size, vec![], 0.);
        assert!(s.travels.values().all(|t| !t.exiting && !t.moving(1.13)));
        assert!(s.indicator.as_ref().is_some_and(|t| !t.moving(1.13)));
    }

    #[test]
    fn large_shell_spacing_does_not_clip_preview_or_study_controls() {
        let ctx = egui::Context::default();
        ctx.style_mut(|style| {
            style.spacing.item_spacing = vec2(12., 12.);
            style.spacing.interact_size = vec2(34., 34.);
        });
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        let size = vec2(1440., 1800.);
        let _ = draw(&mut s, &mut rt, &ctx, 0., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1., size, vec![], 0.);
        let footer = record(&rt, "ov-specimen-primitives-3");
        for i in 0..3 {
            let check = record(&rt, &format!("ov-task-check-{i}"));
            assert!(
                check.bottom() <= footer.top() + 0.5,
                "task {i} clips {check:?} against {footer:?}"
            );
        }
        let checkbox = record(&rt, "ov-task-check-1");
        let far = checkbox.right_center() - vec2(20., 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.1, size, pointer(far, true), 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 1.116, size, pointer(far, false), 0.);
        assert!(s.task_checks[1], "trailing label space toggles the task");
        let _ = draw(&mut s, &mut rt, &ctx, 1.2, size, vec![key(Key::Space)], 0.);
        assert!(
            !s.task_checks[1],
            "Space operates the focused whole-row checkbox"
        );
        s.set_query("no-result-921", 1.);
        let _ = draw(&mut s, &mut rt, &ctx, 2., size, vec![], 0.);
        let _ = draw(&mut s, &mut rt, &ctx, 3., size, vec![], 0.);
        assert!(record(&rt, "ov-study-task-2").bottom() < record(&rt, "ov-study-reset").top());
    }
    #[test]
    fn catalog_all_69_search_categories_and_priority_are_real() {
        let mut s = State::default();
        assert_eq!(ENTRIES.len(), 69);
        assert_eq!(s.matches().len(), 69);
        assert_eq!(s.visible().len(), 12);
        assert_eq!(
            s.visible()
                .iter()
                .take(6)
                .map(|&i| ENTRIES[i].id)
                .collect::<Vec<_>>(),
            PRIORITY
        );
        let mut all = HashSet::new();
        for (category, _) in CATEGORIES.iter().skip(1) {
            s.set_category(category);
            let matches = s.matches();
            assert!(!matches.is_empty());
            for i in matches {
                assert_eq!(ENTRIES[i].family, *category);
                assert!(all.insert(i));
            }
        }
        assert_eq!(all.len(), 69);
        s.clear_filters();
        s.show_more();
        assert_eq!(s.visible().len(), 36);
        s.show_more();
        s.show_more();
        assert_eq!(s.visible().len(), 69);
        s.set_query("  AcCoRdIoN  ", 1.);
        assert!(!s.matches().is_empty());
        assert!(s.matches().iter().all(|&i| {
            format!(
                "{} {} {}",
                ENTRIES[i].title, ENTRIES[i].desc, ENTRIES[i].collection
            )
            .to_lowercase()
            .contains("accordion")
        }));
        s.set_query("no-result-921", 2.);
        assert!(s.matches().is_empty());
        s.set_category("data");
        s.clear_query(3.);
        assert_eq!(s.category, "data");
        assert_eq!(s.matches().len(), 5);
        s.set_query("  ", 4.);
        assert_eq!(s.matches().len(), 5);
        assert_eq!(s.limit, 12);
    }
    #[test]
    fn async_action_uses_real_time_and_never_completes_after_cancellation() {
        let mut s = State::default();
        assert!(s.start_action(10.));
        assert!(!s.start_action(10.1));
        s.tick_action(10.22, true);
        assert!(matches!(s.action, Action::Running { .. }));
        assert!((s.action_progress(10.65) - 0.5).abs() < 0.0001);
        s.tick_action(11.301, true);
        assert_eq!(s.action, Action::Complete);
        assert!(s.start_action(12.));
        s.tick_action(12.2, false);
        s.tick_action(20., true);
        assert_eq!(s.action, Action::Ready);
        assert_eq!(s.action_caption, "Cancelled");
        s.start_action(21.);
        s.set_category("data");
        assert_eq!(s.action, Action::Ready);
        s.tick_action(100., true);
        assert_eq!(s.action_caption, "Cancelled");
    }
    #[test]
    fn revenue_geometry_totals_and_keyboard_match_reference() {
        let mut chart = Revenue::new(7);
        assert_eq!(chart.total(), 4190);
        assert_eq!(chart.points[0].x, 8.);
        assert_eq!(chart.points.last().unwrap().x, 292.);
        chart.inspect_fraction(8. / 300.);
        assert_eq!(chart.selected, Some(0));
        chart.key(Key::End);
        assert_eq!(chart.caption(), "Day 7 of 7");
        chart.key(Key::Home);
        chart.key(Key::ArrowLeft);
        assert_eq!(chart.selected, Some(0));
        let mut chart = Revenue::new(30);
        assert_eq!(chart.points.len(), 30);
        assert!(chart.length > 284.);
        assert!(
            chart
                .samples
                .iter()
                .all(|p| p.x.is_finite() && p.y.is_finite())
        );
        chart.key(Key::ArrowRight);
        assert_eq!(chart.selected, Some(1));
        chart.key(Key::End);
        chart.key(Key::ArrowRight);
        assert_eq!(chart.selected, Some(29));
        let trace = chart.traced_samples(0.5);
        assert!(trace.len() > 1 && trace.len() < chart.samples.len());
        assert_eq!(chart.traced_samples(1.), chart.samples);
        let mut s = State::default();
        s.set_range(30, 1.);
        s.set_range(7, 1.05);
        assert_eq!(s.revenue.total(), 4190);
        assert_eq!(s.chart_started, Some(1.05));
    }
    #[test]
    fn counter_and_studies_retain_independent_state() {
        let mut s = State::default();
        s.step_counter(1, 0., 1.);
        assert_eq!(s.counter, 25);
        assert!(s.counter_digits[2].sample(0.1) > 4. && s.counter_digits[2].sample(0.1) < 5.);
        s.step_counter(1000, 0.2, 0.);
        assert_eq!(s.counter, 999);
        s.step_counter(-2000, 0.3, 0.);
        assert_eq!(s.counter, 0);
        s.set_counter(24, 0.4, 0.);
        assert_eq!(s.counter, 24);
        s.toggle_study_task(1);
        s.toggle_pin(2);
        s.toggle_pin(0);
        assert_eq!(s.pinned_order(), vec![0, 2, 1]);
        s.toggle_pin(0);
        assert_eq!(s.pinned_order(), vec![2, 0, 1]);
        s.disclosures[1] = true;
        s.settings = [false, true];
        s.study = 2;
        s.reset_studies();
        assert_eq!(s.study_tasks, [true, false, false]);
        assert_eq!(s.pins, [false; 3]);
        assert_eq!(s.disclosures, [false; 2]);
        assert_eq!(s.settings, [false, true]);
        assert_eq!(s.study, 2);
        assert_eq!(s.counter, 24);
    }
    #[test]
    fn retargeting_and_exit_reversal_preserve_displayed_geometry() {
        let a = Visual {
            rect: Rect::from_min_size(Pos2::ZERO, vec2(100., 200.)),
            opacity: 1.,
        };
        let b = Visual {
            rect: a.rect.translate(vec2(200., 80.)),
            opacity: 0.,
        };
        let mut t = Travel {
            from: a,
            to: b,
            start: 1.,
            seconds: 0.42,
            delay: 0.,
            exiting: true,
        };
        let before = t.sample(1.17);
        t.retarget(a, 1.17, 0.42, 0.065, false);
        let after = t.sample(1.17);
        assert_eq!(before.rect, after.rect);
        assert_eq!(before.opacity, after.opacity);
        assert_eq!(t.sample(2.).rect, a.rect);
        assert_eq!(t.sample(2.).opacity, 1.);
        let mut h = Scalar::new(1000.);
        h.target(200., 1., 0.44, 0.);
        let before = h.sample(1.1);
        h.target(900., 1.1, 0.44, 0.);
        assert_eq!(before, h.sample(1.1));
        assert_eq!(h.sample(2.), 900.);
    }
    #[test]
    fn responsive_column_boundaries_match_requested_native_layout() {
        assert_eq!(State::columns(320.), 1);
        assert_eq!(State::columns(600.), 1);
        assert_eq!(State::columns(601.), 2);
        assert_eq!(State::columns(1099.), 2);
        assert_eq!(State::columns(1100.), 3);
        assert_eq!(State::columns(1440.), 3);
    }
}
