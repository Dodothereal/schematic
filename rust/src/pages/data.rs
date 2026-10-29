//! Seeded analytics, native chart painters and retained customer-table state.
use crate::{
    icons,
    motion::{self, Curve},
    runtime::{Page, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{
    self, Align2, Color32, FontId, Key, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    pos2, vec2,
};
use std::collections::BTreeSet;

const CHANNELS: [&str; 6] = [
    "Organic search",
    "Direct",
    "Referral",
    "Social",
    "Email",
    "Paid search",
];
const SHARES: [f64; 6] = [0.32, 0.24, 0.17, 0.12, 0.09, 0.06];
const GROWTH: [f64; 6] = [14.2, 8.7, 23.4, -3.1, 18.9, 5.6];
const DEVICES: [(&str, f64); 3] = [("Desktop", 12400.0), ("Mobile", 5400.0), ("Tablet", 2200.0)];
const GOALS: [(&str, f32, &str, usize); 3] = [
    ("Revenue", 76.0, "$76k / $100k", 0),
    ("Activation", 88.0, "880 / 1,000", 1),
    ("Retention", 64.0, "640 / 1,000", 3),
];
fn random(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    *seed as f64 / 4294967296.0
}
fn number(value: f64) -> String {
    let digits = value.round().abs().to_string();
    let mut out = if value < 0.0 {
        "-".into()
    } else {
        String::new()
    };
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',')
        }
        out.push(c)
    }
    out
}
fn money(value: f64) -> String {
    if value < 0.0 {
        format!("-${}", number(-value))
    } else {
        format!("${}", number(value))
    }
}
fn signed(value: f64) -> String {
    format!("{}{value:.1}%", if value >= 0.0 { "+" } else { "" })
}
#[derive(Clone, Debug)]
pub struct Day {
    pub date: String,
    pub label: String,
    pub orders: f64,
    pub visitors: f64,
    pub revenue: f64,
}
#[derive(Clone, Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub status: String,
    pub orders: f64,
    pub revenue: f64,
    pub growth: f64,
}
#[derive(Clone, Debug)]
pub struct Bucket {
    pub label: String,
    pub new: f64,
    pub returning: f64,
}

fn dataset() -> (Vec<Day>, Vec<Account>) {
    let mut seed = 6028;
    let (mut month, mut day) = (5, 2);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = (0..180)
        .map(|i| {
            let orders =
                (27.0 + i as f64 * 0.16 + (i as f64 * 0.65).sin() * 9.0 + random(&mut seed) * 15.0)
                    .round();
            let visitors = (orders / (0.029 + random(&mut seed) * 0.011)).round();
            let revenue = (orders * (63.0 + random(&mut seed) * 24.0)).round();
            let row = Day {
                date: format!("2026-{month:02}-{day:02}"),
                label: format!("{} {day}", months[month - 1]),
                orders,
                visitors,
                revenue,
            };
            day += 1;
            if day
                > if [4, 6, 9, 11].contains(&month) {
                    30
                } else {
                    31
                }
            {
                day = 1;
                month += 1
            }
            row
        })
        .collect();
    let names = [
        "Forma Studio",
        "Linear House",
        "Orbit Supply",
        "Northstar Lab",
        "Mono Works",
        "Field Notes",
        "Common Ground",
        "Index Studio",
        "Parallel Co.",
        "Good Measure",
        "Open Edition",
        "Atlas Objects",
        "Studio Tuesday",
        "Quiet Practice",
        "Outer Office",
        "Paper Plane",
        "New Standard",
        "Basecamp Goods",
        "Another Country",
        "Small Hours",
        "Made of Matter",
        "Third Space",
        "Daybreak Co.",
        "Pattern House",
        "Local Assembly",
        "Frame & Form",
        "Workshop One",
        "Future Ordinary",
    ];
    let accounts = names
        .iter()
        .enumerate()
        .map(|(i, name)| Account {
            id: format!("SC-{}", 1024 + i),
            name: (*name).into(),
            channel: CHANNELS[i % 6].into(),
            status: if i % 7 == 3 {
                "Trial"
            } else if i % 9 == 4 {
                "Paused"
            } else {
                "Active"
            }
            .into(),
            orders: 12.0 + (random(&mut seed) * 140.0).floor(),
            revenue: 850.0 + (random(&mut seed) * 14500.0).round(),
            growth: ((random(&mut seed) * 38.0 - 9.0) * 10.0).round() / 10.0,
        })
        .collect();
    (days, accounts)
}
fn metrics(rows: &[Day]) -> [f64; 4] {
    let revenue = rows.iter().map(|d| d.revenue).sum::<f64>();
    let orders = rows.iter().map(|d| d.orders).sum::<f64>();
    let visitors = rows.iter().map(|d| d.visitors).sum::<f64>();
    [revenue, orders, revenue / orders, orders / visitors * 100.0]
}
fn sample(values: &[f64], n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let p = i as f64 / (n - 1) as f64 * (values.len() - 1) as f64;
            let j = p.floor() as usize;
            values[j] + (values[(j + 1).min(values.len() - 1)] - values[j]) * (p - j as f64)
        })
        .collect()
}

/// Same monotone cubic construction as the reference; computed only on data/size changes.
fn monotone(values: &[f64], rect: Rect, max: f64) -> Vec<Pos2> {
    let step = rect.width() as f64 / (values.len() - 1) as f64;
    let points: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            (
                rect.left() as f64 + i as f64 * step,
                rect.top() as f64 + (1.0 - v / max) * rect.height() as f64,
            )
        })
        .collect();
    let slopes: Vec<_> = points
        .windows(2)
        .map(|p| (p[1].1 - p[0].1) / step)
        .collect();
    let mut tangents: Vec<_> = (0..points.len())
        .map(|i| {
            if i == 0 {
                slopes[0]
            } else if i == points.len() - 1 {
                *slopes.last().unwrap()
            } else if slopes[i - 1] * slopes[i] <= 0.0 {
                0.0
            } else {
                2.0 * slopes[i - 1] * slopes[i] / (slopes[i - 1] + slopes[i])
            }
        })
        .collect();
    for (i, slope) in slopes.iter().enumerate() {
        if *slope == 0.0 {
            tangents[i] = 0.0;
            tangents[i + 1] = 0.0;
            continue;
        }
        let a = tangents[i] / slope;
        let b = tangents[i + 1] / slope;
        let norm = a.hypot(b);
        if norm > 3.0 {
            tangents[i] = 3.0 / norm * a * slope;
            tangents[i + 1] = 3.0 / norm * b * slope;
        }
    }
    let mut out = vec![pos2(points[0].0 as f32, points[0].1 as f32)];
    for i in 1..points.len() {
        let a = points[i - 1];
        let d = points[i];
        let b = (a.0 + step / 3.0, a.1 + tangents[i - 1] * step / 3.0);
        let c = (d.0 - step / 3.0, d.1 - tangents[i] * step / 3.0);
        for j in 1..=8 {
            let t = j as f64 / 8.0;
            let u = 1.0 - t;
            out.push(pos2(
                (u * u * u * a.0 + 3.0 * u * u * t * b.0 + 3.0 * u * t * t * c.0 + t * t * t * d.0)
                    as f32,
                (u * u * u * a.1 + 3.0 * u * u * t * b.1 + 3.0 * u * t * t * c.1 + t * t * t * d.1)
                    as f32,
            ));
        }
    }
    out
}

#[derive(Default, Clone)]
struct Trace {
    points: Vec<Pos2>,
    lengths: Vec<f32>,
    length: f32,
}
impl Trace {
    fn new(points: Vec<Pos2>) -> Self {
        let mut lengths = vec![0.0];
        for pair in points.windows(2) {
            lengths.push(lengths.last().copied().unwrap_or(0.0) + pair[1].distance(pair[0]));
        }
        let length = lengths.last().copied().unwrap_or(0.0);
        Self {
            points,
            lengths,
            length,
        }
    }
    fn prefix(&self, progress: f32) -> (Vec<Pos2>, Pos2) {
        let distance = self.length * progress.clamp(0.0, 1.0);
        let index = self
            .lengths
            .partition_point(|n| *n <= distance)
            .saturating_sub(1)
            .min(self.points.len() - 1);
        let mut points = self.points[..=index].to_vec();
        let point = if index + 1 < self.points.len() {
            let span = self.lengths[index + 1] - self.lengths[index];
            self.points[index].lerp(
                self.points[index + 1],
                if span > 0.0 {
                    (distance - self.lengths[index]) / span
                } else {
                    0.0
                },
            )
        } else {
            self.points[index]
        };
        points.push(point);
        (points, point)
    }
}
#[derive(Default)]
struct Geometry {
    generation: u64,
    size: egui::Vec2,
    current: Trace,
    previous: Trace,
    values: Vec<f64>,
    maximum: f64,
    grid: Vec<(f32, String)>,
    dates: Vec<(f32, String)>,
}
#[derive(Default, Clone)]
struct Reveal {
    start: Option<f64>,
    finished: bool,
}
impl Reveal {
    fn reset(&mut self) {
        self.start = None;
        self.finished = false;
    }
    fn value(
        &mut self,
        rect: Rect,
        ui: &egui::Ui,
        rt: &mut Runtime,
        duration: f32,
        delay: f32,
        curve: Curve,
    ) -> f32 {
        if rt.reduced() {
            self.finished = true;
            return 1.0;
        }
        if self.finished {
            return 1.0;
        }
        let intersection = rect.intersect(ui.clip_rect());
        let visible = intersection.height() >= ((rect.height() * 0.12).min(80.0))
            && intersection.width() > 0.0;
        if self.start.is_none() {
            if !visible {
                return 0.0;
            }
            self.start = Some(rt.now + delay as f64 * rt.motion_scale() as f64);
        }
        let p = rt.progress(self.start.unwrap(), duration);
        if p >= 1.0 {
            self.finished = true
        } else {
            rt.repaint()
        }
        motion::ease(curve, p)
    }
}

pub struct State {
    pub days: Vec<Day>,
    pub accounts: Vec<Account>,
    pub period: usize,
    pub compare: bool,
    pub day_index: usize,
    pub stacked: bool,
    pub enabled: [bool; 3],
    pub query: String,
    pub status: String,
    pub sort: String,
    pub ascending: bool,
    pub table_page: usize,
    pub page_size: usize,
    pub selected: BTreeSet<String>,
    pub rank_sort: usize,
    pub bar_pinned: Option<usize>,
    pub rank_pinned: Option<usize>,
    pub goal_pinned: Option<usize>,
    selection_anchor: Option<String>,
    live: bool,
    next_tick: Option<f64>,
    pub ticks: u32,
    increment: f64,
    active: bool,
    generation: u64,
    geometry: Geometry,
    line_reveal: Reveal,
    spark_reveals: [Reveal; 4],
    spark_geometry: [Trace; 4],
    spark_sizes: [egui::Vec2; 4],
    spark_generations: [u64; 4],
    bar_reveals: [Reveal; 14],
    rank_reveals: [Reveal; 6],
    donut_reveal: Reveal,
    goal_reveals: [Reveal; 3],
    kpi_start: f64,
    kpi_from: [f64; 4],
    kpi_display: [f64; 4],
    kpi_to: [f64; 4],
    export_status: String,
    inspecting: bool,
    rank_order: [usize; 6],
    row_positions: std::collections::HashMap<String, f32>,
    visible_rows: BTreeSet<String>,
}
impl Default for State {
    fn default() -> Self {
        let (days, accounts) = dataset();
        let initial = metrics(&days[150..]);
        Self {
            days,
            accounts,
            period: 30,
            compare: true,
            day_index: 29,
            stacked: false,
            enabled: [true; 3],
            query: String::new(),
            status: "all".into(),
            sort: "revenue".into(),
            ascending: false,
            table_page: 0,
            page_size: 8,
            selected: BTreeSet::new(),
            rank_sort: 0,
            bar_pinned: None,
            rank_pinned: None,
            goal_pinned: None,
            selection_anchor: None,
            live: false,
            next_tick: None,
            ticks: 0,
            increment: 0.0,
            active: false,
            generation: 1,
            geometry: Geometry::default(),
            line_reveal: Reveal::default(),
            spark_reveals: Default::default(),
            spark_geometry: Default::default(),
            spark_sizes: [egui::Vec2::ZERO; 4],
            spark_generations: [0; 4],
            bar_reveals: Default::default(),
            rank_reveals: Default::default(),
            donut_reveal: Reveal::default(),
            goal_reveals: Default::default(),
            kpi_start: 0.0,
            kpi_from: initial,
            kpi_display: initial,
            kpi_to: initial,
            export_status: String::new(),
            inspecting: false,
            rank_order: [0, 1, 2, 3, 4, 5],
            row_positions: Default::default(),
            visible_rows: Default::default(),
        }
    }
}
impl State {
    pub fn current(&self) -> Vec<Day> {
        let mut rows = self.days[180 - self.period..].to_vec();
        rows.last_mut().unwrap().revenue += self.increment;
        rows
    }
    pub fn previous(&self) -> &[Day] {
        &self.days[180 - self.period * 2..180 - self.period]
    }
    pub fn totals(&self) -> [f64; 4] {
        metrics(&self.current())
    }
    pub fn set_period(&mut self, period: usize, now: f64) {
        if ![7, 30, 90].contains(&period) || period == self.period {
            return;
        }
        self.period = period;
        self.day_index = period - 1;
        self.changed(now);
    }
    fn changed(&mut self, now: f64) {
        self.kpi_from = self.kpi_display;
        self.generation += 1;
        self.line_reveal.reset();
        self.spark_reveals.iter_mut().for_each(Reveal::reset);
        self.bar_reveals.iter_mut().for_each(Reveal::reset);
        self.rank_reveals.iter_mut().for_each(Reveal::reset);
        self.kpi_start = now;
        self.kpi_to = self.totals();
    }
    pub fn buckets(&self) -> Vec<Bucket> {
        let rows = self.current();
        (0..7)
            .map(|i| {
                let chunk = &rows[i * rows.len() / 7..(i + 1) * rows.len() / 7];
                let total = chunk.iter().map(|d| d.orders).sum::<f64>();
                let new = (total * (0.57 + (i as f64 + self.period as f64).sin() * 0.09)).round();
                Bucket {
                    label: chunk[0].label.clone(),
                    new,
                    returning: total - new,
                }
            })
            .collect()
    }
    pub fn device_mix(&self) -> ([f64; 3], f64) {
        let total = DEVICES
            .iter()
            .enumerate()
            .map(|(i, (_, n))| if self.enabled[i] { *n } else { 0.0 })
            .sum::<f64>();
        (
            std::array::from_fn(|i| {
                if total > 0.0 && self.enabled[i] {
                    DEVICES[i].1 / total * 100.0
                } else {
                    0.0
                }
            }),
            total,
        )
    }
    pub fn filtered_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        let mut indices: Vec<_> = self
            .accounts
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                (self.status == "all" || self.status == a.status)
                    && format!("{} {} {}", a.name, a.id, a.channel)
                        .to_lowercase()
                        .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        indices.sort_by(|a, b| {
            let (a, b) = (&self.accounts[*a], &self.accounts[*b]);
            let order = match self.sort.as_str() {
                "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                "status" => a.status.cmp(&b.status),
                "orders" => a.orders.total_cmp(&b.orders),
                "growth" => a.growth.total_cmp(&b.growth),
                _ => a.revenue.total_cmp(&b.revenue),
            };
            let order = if self.ascending {
                order
            } else {
                order.reverse()
            };
            order.then(a.id.cmp(&b.id))
        });
        indices
    }
    pub fn visible_indices(&self) -> Vec<usize> {
        self.filtered_indices()
            .into_iter()
            .skip(self.table_page * self.page_size)
            .take(self.page_size)
            .collect()
    }
    pub fn toggle_row(&mut self, id: &str, extend: bool) {
        let rows = self.visible_indices();
        let anchor = self
            .selection_anchor
            .as_ref()
            .and_then(|id| rows.iter().position(|i| self.accounts[*i].id == *id));
        let end = rows.iter().position(|i| self.accounts[*i].id == id);
        if let (true, Some(anchor), Some(end)) = (extend, anchor, end) {
            for i in &rows[anchor.min(end)..=anchor.max(end)] {
                self.selected.insert(self.accounts[*i].id.clone());
            }
        } else {
            if !self.selected.remove(id) {
                self.selected.insert(id.into());
            }
            self.selection_anchor = Some(id.into());
        }
    }
    pub fn select_page(&mut self, select: bool) {
        for i in self.visible_indices() {
            if select {
                self.selected.insert(self.accounts[i].id.clone());
            } else {
                self.selected.remove(&self.accounts[i].id);
            }
        }
    }
    pub fn export_csv(&self) -> (String, usize) {
        let rows = if self.selected.is_empty() {
            self.filtered_indices()
        } else {
            self.accounts
                .iter()
                .enumerate()
                .filter(|(_, a)| self.selected.contains(&a.id))
                .map(|(i, _)| i)
                .collect()
        };
        let mut records = vec![vec![
            "Generated demo data".into(),
            "Account ID".into(),
            "Customer".into(),
            "Channel".into(),
            "Status".into(),
            "Orders".into(),
            "Revenue USD".into(),
            "Growth percent".into(),
        ]];
        for i in &rows {
            let a = &self.accounts[*i];
            records.push(vec![
                "Simulated".into(),
                a.id.clone(),
                a.name.clone(),
                a.channel.clone(),
                a.status.clone(),
                a.orders.to_string(),
                a.revenue.to_string(),
                a.growth.to_string(),
            ]);
        }
        let csv = records
            .iter()
            .map(|record| {
                record
                    .iter()
                    .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("\r\n");
        (format!("\u{feff}{csv}"), rows.len())
    }
    pub fn tick(&mut self, now: f64) {
        self.ticks += 1;
        let mut seed = 6028 + self.ticks;
        self.increment += 15.0 + (random(&mut seed) * 80.0).round();
        self.changed(now);
    }
    pub fn update_live(&mut self, now: f64, reduced: bool, visible: bool) {
        if !self.active || !self.live || reduced || !visible {
            self.next_tick = None;
            return;
        }
        match self.next_tick {
            Some(next) if now >= next => {
                self.tick(now);
                self.next_tick = Some(now + 4.0)
            }
            None => self.next_tick = Some(now + 4.0),
            _ => {}
        }
    }
    fn reset_filters(&mut self) {
        self.query.clear();
        self.status = "all".into();
        self.table_page = 0;
    }
    fn ranks(&self) -> [usize; 6] {
        let mut order = [0, 1, 2, 3, 4, 5];
        order.sort_by(|a, b| match self.rank_sort {
            1 => GROWTH[*b].total_cmp(&GROWTH[*a]),
            2 => CHANNELS[*a].cmp(CHANNELS[*b]),
            _ => SHARES[*b].total_cmp(&SHARES[*a]),
        });
        order
    }
    fn prepare_geometry(&mut self, size: egui::Vec2) {
        if self.geometry.generation == self.generation && self.geometry.size == size {
            return;
        }
        let rows = self.current();
        let previous = self.previous();
        let max = rows
            .iter()
            .chain(previous.iter().filter(|_| self.compare))
            .map(|d| d.revenue)
            .fold(0.0_f64, f64::max);
        let max = (max / 1000.0).ceil() * 1000.0;
        let plot = Rect::from_min_max(pos2(48.0, 22.0), pos2(size.x - 18.0, size.y - 36.0));
        let values = sample(&rows.iter().map(|d| d.revenue).collect::<Vec<_>>(), 90);
        let current = Trace::new(monotone(&values, plot, max));
        let previous = Trace::new(monotone(
            &sample(&previous.iter().map(|d| d.revenue).collect::<Vec<_>>(), 90),
            plot,
            max,
        ));
        self.geometry = Geometry {
            generation: self.generation,
            size,
            current,
            previous,
            values,
            maximum: max,
            grid: (0..5)
                .map(|i| {
                    (
                        22.0 + i as f32 / 4.0 * (size.y - 58.0),
                        if i == 4 {
                            "$0".into()
                        } else {
                            format!("${:.1}k", max * (1.0 - i as f64 / 4.0) / 1000.0)
                        },
                    )
                })
                .collect(),
            dates: (0..5)
                .map(|i| {
                    let t = i as f32 / 4.0;
                    (
                        48.0 + t * (size.x - 66.0),
                        rows[(t * (self.period - 1) as f32).round() as usize]
                            .label
                            .clone(),
                    )
                })
                .collect(),
        };
    }
}

fn line_shape(painter: &egui::Painter, points: Vec<Pos2>, offset: egui::Vec2, stroke: Stroke) {
    if points.len() > 1 {
        painter.add(Shape::line(
            points.into_iter().map(|p| p + offset).collect(),
            stroke,
        ));
    }
}
fn arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start: f32,
    fraction: f32,
    color: Color32,
    width: f32,
) {
    if fraction <= 0.0 {
        return;
    }
    let segments = (fraction * 160.0).ceil().max(2.0) as usize;
    let points = (0..=segments)
        .map(|i| {
            let a = (start + fraction * i as f32 / segments as f32) * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            center + vec2(a.cos(), a.sin()) * radius
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(width, color)));
}
fn legend(ui: &mut egui::Ui, rt: &Runtime, items: &[(&str, usize)]) {
    ui.horizontal_wrapped(|ui| {
        for (label, index) in items {
            let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter()
                .circle_filled(r.center(), 4.0, rt.tokens.charts[*index]);
            ui.label(
                egui::RichText::new(*label)
                    .size(11.0)
                    .color(rt.tokens.text2),
            );
            ui.add_space(8.0);
        }
    });
}
fn activated(ui: &egui::Ui, response: &Response) -> bool {
    response.clicked()
        || response.has_focus()
            && ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, Key::Enter)
                    || i.consume_key(egui::Modifiers::NONE, Key::Space)
            })
}
fn chart_focus(ui: &egui::Ui, rt: &Runtime, response: &Response) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            6.0,
            Stroke::new(2.0, rt.tokens.accent_ring),
            StrokeKind::Inside,
        );
    }
}

impl State {
    fn kpis(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let rows = self.current();
        let prev = metrics(self.previous());
        let cols = if ui.available_width() > 880.0 { 4 } else { 2 };
        let width = (ui.available_width() - (cols - 1) as f32 * 16.0) / cols as f32;
        egui::Grid::new("dt-kpis")
            .num_columns(cols)
            .spacing(vec2(16.0, 16.0))
            .show(ui, |ui| {
                for (i, (label, icon)) in [
                    ("Revenue", "trending"),
                    ("Orders", "layers"),
                    ("Avg. order value", "gauge"),
                    ("Conversion", "cursor"),
                ]
                .iter()
                .enumerate()
                {
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(width, 174.0), Sense::hover());
                    rt.record(&format!("dt-kpi-{i}"), &response);
                    let p = ui.painter().with_clip_rect(rect);
                    p.rect(
                        rect,
                        16.0 * rt.prefs.radius,
                        rt.tokens.surface,
                        Stroke::new(1.0, rt.tokens.border),
                        StrokeKind::Inside,
                    );
                    p.text(
                        rect.min + vec2(20.0, 24.0),
                        Align2::LEFT_CENTER,
                        *label,
                        FontId::proportional(12.0),
                        rt.tokens.text2,
                    );
                    icons::paint(
                        &p,
                        icon,
                        Rect::from_min_size(rect.right_top() + vec2(-36.0, 16.0), vec2(16.0, 16.0)),
                        rt.tokens.text3,
                        1.6,
                        0.0,
                    );
                    let progress = motion::ease(Curve::OutExpo, rt.progress(self.kpi_start, 0.65));
                    let value =
                        self.kpi_from[i] + (self.kpi_to[i] - self.kpi_from[i]) * progress as f64;
                    if progress < 1.0 {
                        rt.repaint()
                    }
                    let label = match i {
                        0 | 2 => money(value),
                        1 => number(value),
                        _ => format!("{value:.1}%"),
                    };
                    p.text(
                        rect.min + vec2(20.0, 66.0),
                        Align2::LEFT_CENTER,
                        label,
                        FontId::proportional(if width < 220.0 { 24.0 } else { 28.0 }),
                        rt.tokens.text,
                    );
                    let change = (self.kpi_to[i] / prev[i] - 1.0) * 100.0;
                    p.text(
                        rect.min + vec2(20.0, 112.0),
                        Align2::LEFT_CENTER,
                        signed(change),
                        FontId::proportional(12.0),
                        if change < 0.0 {
                            rt.tokens.danger_text
                        } else {
                            rt.tokens.success_text
                        },
                    );
                    let spark = Rect::from_min_max(
                        rect.min + vec2(width * 0.44, 94.0),
                        rect.right_top() + vec2(-16.0, 134.0),
                    );
                    if self.spark_generations[i] != self.generation
                        || self.spark_sizes[i] != spark.size()
                    {
                        let values = sample(
                            &rows
                                .iter()
                                .map(|row| match i {
                                    0 => row.revenue,
                                    1 => row.orders,
                                    2 => row.revenue / row.orders,
                                    _ => row.orders / row.visitors * 100.0,
                                })
                                .collect::<Vec<_>>(),
                            24,
                        );
                        let max = values.iter().copied().fold(0.0_f64, f64::max) * 1.12;
                        let points =
                            monotone(&values, Rect::from_min_size(Pos2::ZERO, spark.size()), max);
                        self.spark_geometry[i] = Trace::new(points);
                        self.spark_sizes[i] = spark.size();
                        self.spark_generations[i] = self.generation;
                    }
                    let progress = self.spark_reveals[i].value(
                        rect,
                        ui,
                        rt,
                        0.95,
                        i as f32 * 0.07,
                        Curve::InOut,
                    );
                    let (points, _) = self.spark_geometry[i].prefix(progress);
                    line_shape(
                        &p,
                        points,
                        spark.min.to_vec2(),
                        Stroke::new(2.0, rt.tokens.charts[i]),
                    );
                    p.text(
                        rect.min + vec2(20.0, 152.0),
                        Align2::LEFT_CENTER,
                        format!("vs. previous {} days", self.period),
                        FontId::proportional(11.0),
                        rt.tokens.text3,
                    );
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
        self.kpi_display = std::array::from_fn(|i| {
            let progress = motion::ease(Curve::OutExpo, rt.progress(self.kpi_start, 0.65));
            self.kpi_from[i] + (self.kpi_to[i] - self.kpi_from[i]) * progress as f64
        });
    }
    fn line_chart(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "dt-line",
            "Revenue line chart",
            "Change the period. Hover or use the day selector to inspect values.",
            0.0,
            |ui, rt| {
                let rows = self.current();
                let total = self.totals()[0];
                let previous = metrics(self.previous())[0];
                let layout = if ui.available_width() < 620.0 {
                    egui::Layout::top_down(egui::Align::Min)
                } else {
                    egui::Layout::left_to_right(egui::Align::Min)
                };
                ui.with_layout(layout, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Revenue over time")
                                .size(11.0)
                                .color(rt.tokens.text2),
                        );
                        ui.label(egui::RichText::new(money(total)).size(36.0));
                        ui::muted(
                            ui,
                            rt,
                            &format!("{} - {}, 2026", rows[0].label, rows.last().unwrap().label),
                        );
                    });
                    ui.vertical(|ui| {
                        if ui::checkbox(
                            ui,
                            rt,
                            "dt-compare",
                            "Compare previous period",
                            &mut self.compare,
                        )
                        .changed()
                        {
                            self.generation += 1;
                            self.line_reveal.reset();
                        }
                        legend(ui, rt, &[("Current", 0), ("Previous", 1)]);
                        if ui::button(
                            ui,
                            rt,
                            "dt-line-replay",
                            "Replay",
                            Some("refresh"),
                            ButtonKind::Ghost,
                        )
                        .clicked()
                        {
                            self.line_reveal.reset();
                            self.spark_reveals.iter_mut().for_each(Reveal::reset);
                            self.kpi_from = [0.0; 4];
                            self.kpi_start = rt.now;
                        }
                    });
                });
                ui.add_space(24.0);
                let width = ui.available_width().max(180.0);
                let height = if width < 500.0 { 280.0 } else { 300.0 };
                let (rect, response) =
                    ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
                rt.record("dt-main-chart", &response);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Other,
                        true,
                        "Daily revenue chart. Use the Inspect a day slider for keyboard values.",
                    )
                });
                self.prepare_geometry(rect.size());
                let progress = self.line_reveal.value(rect, ui, rt, 1.1, 0.0, Curve::InOut);
                let p = ui.painter().with_clip_rect(rect);
                let offset = rect.min.to_vec2();
                for (y, label) in &self.geometry.grid {
                    p.line_segment(
                        [rect.min + vec2(48.0, *y), rect.min + vec2(width - 18.0, *y)],
                        Stroke::new(1.0, rt.tokens.border_soft),
                    );
                    p.text(
                        rect.min + vec2(38.0, *y + 4.0),
                        Align2::RIGHT_CENTER,
                        label,
                        FontId::monospace(11.0),
                        rt.tokens.text2,
                    );
                }
                for (i, (x, label)) in self.geometry.dates.iter().enumerate() {
                    p.text(
                        rect.min + vec2(*x, height - 10.0),
                        if i == 0 {
                            Align2::LEFT_CENTER
                        } else if i == 4 {
                            Align2::RIGHT_CENTER
                        } else {
                            Align2::CENTER_CENTER
                        },
                        label,
                        FontId::monospace(11.0),
                        rt.tokens.text2,
                    );
                }
                let (points, head) = self.geometry.current.prefix(progress);
                let mut mesh = egui::Mesh::default();
                let bottom = rect.top() + height - 36.0;
                for point in &points {
                    let top = *point + offset;
                    mesh.colored_vertex(
                        top,
                        rt.tokens.charts[0].gamma_multiply(0.23 * (progress * 2.5).min(1.0)),
                    );
                    mesh.colored_vertex(
                        pos2(top.x, bottom),
                        rt.tokens.charts[0].gamma_multiply(0.015),
                    );
                }
                for i in 0..points.len().saturating_sub(1) {
                    let n = (i * 2) as u32;
                    mesh.add_triangle(n, n + 1, n + 2);
                    mesh.add_triangle(n + 1, n + 2, n + 3);
                }
                p.add(Shape::mesh(mesh));
                if self.compare {
                    let (prior, _) = self
                        .geometry
                        .previous
                        .prefix(((progress - 0.06) / 0.94).clamp(0.0, 1.0));
                    let prior: Vec<_> = prior.into_iter().map(|point| point + offset).collect();
                    if prior.len() > 1 {
                        p.add(Shape::dashed_line(
                            &prior,
                            Stroke::new(1.8, rt.tokens.charts[1].gamma_multiply(0.85)),
                            4.0,
                            5.0,
                        ));
                    }
                }
                line_shape(&p, points, offset, Stroke::new(2.5, rt.tokens.charts[0]));
                if progress > 0.0 && progress < 1.0 {
                    let alpha = (progress * 10.0).min(1.0) * ((1.0 - progress) * 10.0).min(1.0);
                    p.circle_filled(
                        head + offset,
                        8.0,
                        rt.tokens.charts[0].gamma_multiply(0.13 * alpha),
                    );
                    p.circle(
                        head + offset,
                        3.5,
                        rt.tokens.surface,
                        Stroke::new(2.0, rt.tokens.charts[0].gamma_multiply(alpha)),
                    );
                }
                if (response.hovered() || response.dragged())
                    && let Some(pointer) = response
                        .interact_pointer_pos()
                        .or_else(|| ui.ctx().pointer_hover_pos())
                {
                    self.day_index = (((pointer.x - rect.left() - 48.0) / (width - 66.0))
                        .clamp(0.0, 1.0)
                        * (self.period - 1) as f32)
                        .round() as usize;
                    self.inspecting = true;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label("Inspect a day");
                    ui::muted(ui, rt, "Arrow keys / Home / End");
                });
                let slider = ui
                    .scope(|ui| {
                        ui.spacing_mut().slider_width = (ui.available_width() - 16.0).max(100.0);
                        ui.visuals_mut().widgets.inactive.bg_fill = rt.tokens.surface3;
                        ui.visuals_mut().widgets.inactive.bg_stroke =
                            Stroke::new(1.0, rt.tokens.border_strong);
                        ui.visuals_mut().selection.bg_fill = rt.tokens.accent;
                        ui.add(
                            egui::Slider::new(&mut self.day_index, 0..=self.period - 1)
                                .show_value(false)
                                .trailing_fill(true),
                        )
                    })
                    .inner;
                rt.record("dt-day", &slider);
                if slider.has_focus() {
                    self.inspecting = true;
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Home)) {
                        self.day_index = 0
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::End)) {
                        self.day_index = self.period - 1
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                        self.inspecting = false
                    }
                }
                if !response.hovered() && !response.dragged() && !slider.has_focus() {
                    self.inspecting = false
                }
                let row = &rows[self.day_index];
                let prior = &self.previous()[self.day_index];
                let readout = format!(
                    "{}: {}{}",
                    row.label,
                    money(row.revenue),
                    if self.compare {
                        format!("; previous {}", money(prior.revenue))
                    } else {
                        String::new()
                    }
                );
                ui.label(egui::RichText::new(&readout).monospace().size(12.0));
                let inspector_opacity = rt.animate(
                    "dt-inspector-opacity",
                    if self.inspecting { 1.0 } else { 0.0 },
                    0.14,
                    Curve::Out,
                );
                if inspector_opacity > 0.0 {
                    let mut p = p.clone();
                    p.set_opacity(inspector_opacity);
                    let fraction = self.day_index as f64 / (self.period - 1) as f64;
                    let sample = fraction * 89.0;
                    let index = sample.floor() as usize;
                    let value = self.geometry.values[index]
                        + (self.geometry.values[(index + 1).min(89)] - self.geometry.values[index])
                            * (sample - index as f64);
                    let point = rect.min
                        + vec2(
                            48.0 + fraction as f32 * (width - 66.0),
                            22.0 + (1.0 - value / self.geometry.maximum) as f32 * (height - 58.0),
                        );
                    p.add(Shape::dashed_line(
                        &[pos2(point.x, rect.top() + 22.0), pos2(point.x, bottom)],
                        Stroke::new(1.0, rt.tokens.text3),
                        3.0,
                        5.0,
                    ));
                    p.add(Shape::dashed_line(
                        &[
                            pos2(rect.left() + 48.0, point.y),
                            pos2(rect.right() - 18.0, point.y),
                        ],
                        Stroke::new(1.0, rt.tokens.text3.gamma_multiply(0.35)),
                        3.0,
                        5.0,
                    ));
                    p.circle_filled(point, 8.0, rt.tokens.accent_soft2);
                    p.circle(
                        point,
                        4.0,
                        rt.tokens.charts[0],
                        Stroke::new(2.0, rt.tokens.surface),
                    );
                    let text = if self.compare {
                        format!(
                            "{}, 2026\n{}\n{} previous period",
                            row.label,
                            money(row.revenue),
                            money(prior.revenue)
                        )
                    } else {
                        format!("{}, 2026\n{}", row.label, money(row.revenue))
                    };
                    let galley = p.layout_no_wrap(text, FontId::proportional(12.0), rt.tokens.text);
                    let size = galley.size() + vec2(24.0, 16.0);
                    let tip = Rect::from_min_size(
                        pos2(
                            (point.x - size.x / 2.0).clamp(
                                rect.left() + 4.0,
                                (rect.right() - size.x - 4.0).max(rect.left() + 4.0),
                            ),
                            (point.y - size.y - 16.0).max(rect.top()),
                        ),
                        size,
                    );
                    let x = rt.animate("dt-tooltip-x", tip.left() - rect.left(), 0.32, Curve::Out);
                    let y = rt.animate("dt-tooltip-y", tip.top() - rect.top(), 0.32, Curve::Out);
                    let tip = Rect::from_min_size(rect.min + vec2(x, y), size);
                    p.rect(
                        tip,
                        8.0 * rt.prefs.radius,
                        rt.tokens.surface_raised,
                        Stroke::new(1.0, rt.tokens.border_strong),
                        StrokeKind::Inside,
                    );
                    p.galley(tip.min + vec2(12.0, 8.0), galley, rt.tokens.text);
                }
                let peak = rows
                    .iter()
                    .max_by(|a, b| a.revenue.total_cmp(&b.revenue))
                    .unwrap();
                let change = (total / previous - 1.0) * 100.0;
                ui::muted(
                    ui,
                    rt,
                    &format!(
                        "Generated {}-day revenue is {}, {:.1}% {} the preceding {} days. Peak: {} on {}.",
                        self.period,
                        money(total),
                        change.abs(),
                        if change >= 0.0 { "above" } else { "below" },
                        self.period,
                        money(peak.revenue),
                        peak.label
                    ),
                );
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui::checkbox(ui, rt, "dt-live", "Simulate live updates", &mut self.live)
                        .changed()
                    {
                        self.next_tick = None
                    }
                    if ui::button(
                        ui,
                        rt,
                        "dt-tick",
                        "Add sample",
                        Some("plus"),
                        ButtonKind::Secondary,
                    )
                    .clicked()
                    {
                        self.tick(rt.now);
                        self.next_tick = if self.live && !rt.reduced() {
                            Some(rt.now + 4.0)
                        } else {
                            None
                        };
                    }
                });
                ui::muted(
                    ui,
                    rt,
                    &if !self.live {
                        if self.ticks == 0 {
                            "Off. Seeded daily values; no external connection.".into()
                        } else {
                            format!("Off. {} simulated samples retained.", self.ticks)
                        }
                    } else if rt.reduced() {
                        "Reduced motion: automatic updates paused. Use Add sample.".into()
                    } else {
                        format!(
                            "Simulation enabled. One local sample every 4 seconds. {} samples added to Oct 28.",
                            self.ticks
                        )
                    },
                );
            },
        );
    }
    fn bars(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "dt-bars",
            "Grouped and stacked bars",
            "Group to compare series; stack to read the combined volume. Labels retain exact values.",
            0.0,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("Order mix");
                        ui::muted(ui, rt, "Seven buckets across the chosen period");
                    });
                    let mut mode = usize::from(self.stacked);
                    if ui::segmented(ui, rt, "dt-bar-mode", &["Grouped", "Stacked"], &mut mode) {
                        self.stacked = mode == 1;
                        self.bar_reveals.iter_mut().for_each(Reveal::reset);
                    }
                    if ui::button(
                        ui,
                        rt,
                        "dt-bars-replay",
                        "Replay",
                        Some("refresh"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.bar_reveals.iter_mut().for_each(Reveal::reset);
                    }
                });
                legend(ui, rt, &[("New customers", 0), ("Returning", 1)]);
                let buckets = self.buckets();
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 380.0), Sense::hover());
                let base = rect.bottom() - 40.0;
                let top = rect.top() + 28.0;
                let plot = (rect.width() - 64.0).max(120.0);
                let step = plot / 7.0;
                let max = (buckets
                    .iter()
                    .map(|b| {
                        if self.stacked {
                            b.new + b.returning
                        } else {
                            b.new.max(b.returning)
                        }
                    })
                    .fold(0.0_f64, f64::max)
                    / 100.0)
                    .ceil()
                    * 100.0;
                let p = ui.painter();
                for i in 0..4 {
                    let y = top + (base - top) * i as f32 / 3.0;
                    p.line_segment(
                        [pos2(rect.left() + 44.0, y), pos2(rect.right() - 10.0, y)],
                        Stroke::new(1.0, rt.tokens.border_soft),
                    );
                    p.text(
                        pos2(rect.left() + 35.0, y + 4.0),
                        Align2::RIGHT_CENTER,
                        number(max * (1.0 - i as f64 / 3.0)),
                        FontId::monospace(11.0),
                        rt.tokens.text2,
                    );
                }
                let mut hover = None;
                for (i, b) in buckets.iter().enumerate() {
                    let bucket_rect = Rect::from_min_size(
                        pos2(rect.left() + 44.0 + i as f32 * step, rect.top() + 16.0),
                        vec2(step, base - rect.top() - 8.0),
                    );
                    let r =
                        ui.interact(bucket_rect, ui.id().with(("dt-bucket", i)), Sense::click());
                    rt.record(&format!("dt-bar-group-{i}"), &r);
                    r.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            true,
                            self.bar_pinned == Some(i),
                            format!(
                                "{}: {} new, {} returning. Press to pin.",
                                b.label, b.new, b.returning
                            ),
                        )
                    });
                    if r.hovered() || r.has_focus() {
                        hover = Some(i)
                    }
                    if activated(ui, &r) {
                        r.request_focus();
                        self.bar_pinned = if self.bar_pinned == Some(i) {
                            None
                        } else {
                            Some(i)
                        }
                    }
                    if r.has_focus()
                        && ui.input_mut(|input| {
                            input.consume_key(egui::Modifiers::NONE, Key::Escape)
                        })
                    {
                        self.bar_pinned = None
                    }
                    chart_focus(ui, rt, &r);
                }
                let selected = hover.or(self.bar_pinned);
                for (i, b) in buckets.iter().enumerate() {
                    let selected_here = selected == Some(i);
                    let halo = rt.animate(
                        ("dt-bar-highlight", i),
                        if selected_here { 1.0 } else { 0.0 },
                        0.22,
                        Curve::Out,
                    );
                    if halo > 0.0 {
                        p.rect_filled(
                            Rect::from_min_size(
                                pos2(rect.left() + 44.0 + i as f32 * step, rect.top() + 16.0),
                                vec2(step, base - rect.top() - 8.0),
                            ),
                            8.0,
                            rt.tokens.accent_soft.gamma_multiply(halo),
                        );
                    }
                    let bw = step * if self.stacked { 0.56 } else { 0.27 };
                    let x = rect.left() + 48.0 + i as f32 * step + step * 0.18;
                    let ah = (b.new / max) as f32 * (base - top);
                    let bh = (b.returning / max) as f32 * (base - top);
                    let bars = [
                        (x, base - ah, ah, b.new),
                        (
                            x + if self.stacked { 0.0 } else { bw + step * 0.04 },
                            base - bh - if self.stacked { ah } else { 0.0 },
                            bh,
                            b.returning,
                        ),
                    ];
                    let mut label_y = [0.0; 2];
                    for (j, (_, y, h, _)) in bars.iter().enumerate() {
                        label_y[j] = y + if self.stacked {
                            18.0_f32.min(h / 2.0 + 4.0)
                        } else {
                            -7.0
                        };
                    }
                    if !self.stacked && (label_y[0] - label_y[1]).abs() < 14.0 {
                        let j = if label_y[0] <= label_y[1] { 0 } else { 1 };
                        label_y[j] =
                            label_y[0].min(label_y[1]) - (14.0 - (label_y[0] - label_y[1]).abs());
                    }
                    for (j, (x, y, h, value)) in bars.iter().enumerate() {
                        let progress = self.bar_reveals[i * 2 + j].value(
                            rect,
                            ui,
                            rt,
                            0.85,
                            i as f32 * 0.065 + j as f32 * 0.035,
                            Curve::OutExpo,
                        );
                        let color = rt.tokens.charts[j].gamma_multiply(
                            if selected.is_some() && !selected_here {
                                0.22
                            } else {
                                0.88
                            },
                        );
                        p.rect_filled(
                            Rect::from_min_max(
                                pos2(*x, base - (base - y) * progress),
                                pos2(x + bw, base - (base - y - h) * progress),
                            ),
                            3.0,
                            color,
                        );
                        p.text(
                            pos2(x + bw / 2.0, label_y[j]),
                            Align2::CENTER_CENTER,
                            number(*value),
                            FontId::monospace(10.0),
                            rt.tokens
                                .text2
                                .gamma_multiply(((progress - 0.25) / 0.75).clamp(0.0, 1.0)),
                        );
                    }
                    p.text(
                        pos2(
                            rect.left() + 48.0 + i as f32 * step + step * 0.46,
                            rect.bottom() - 12.0,
                        ),
                        Align2::CENTER_CENTER,
                        &b.label,
                        FontId::monospace(10.0),
                        rt.tokens.text2,
                    );
                }
                ui::muted(
                    ui,
                    rt,
                    &selected
                        .map(|i| {
                            let b = &buckets[i];
                            format!(
                                "{} · {} new + {} returning = {} orders{}",
                                b.label,
                                number(b.new),
                                number(b.returning),
                                number(b.new + b.returning),
                                if self.bar_pinned == Some(i) {
                                    " · pinned"
                                } else {
                                    ""
                                }
                            )
                        })
                        .unwrap_or_else(|| {
                            "Hover a bucket to compare exact values. Click to keep it in view."
                                .into()
                        }),
                );
                ui.collapsing("Read order values", |ui| {
                    for b in &buckets {
                        ui.label(format!(
                            "{}: {} new, {} returning, {} total.",
                            b.label,
                            b.new,
                            b.returning,
                            b.new + b.returning
                        ));
                    }
                });
            },
        );
    }
    fn ranking(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "dt-ranks",
            "Animated ranking",
            "Stable rows move to their new rank. Their meters keep exact attributed values.",
            0.0,
            |ui, rt| {
                ui.heading("Acquisition channels");
                ui::muted(ui, rt, "Generated attribution snapshot");
                ui.horizontal_wrapped(|ui| {
                    ui.label("Sort by");
                    let old = self.rank_sort;
                    egui::ComboBox::from_id_salt("dt-rank-sort")
                        .selected_text(["Revenue", "Growth", "Name"][self.rank_sort])
                        .show_ui(ui, |ui| {
                            for (i, label) in ["Revenue", "Growth", "Name"].iter().enumerate() {
                                ui.selectable_value(&mut self.rank_sort, i, *label);
                            }
                        });
                    if old != self.rank_sort {
                        self.rank_reveals.iter_mut().for_each(Reveal::reset);
                    }
                    if ui::button(
                        ui,
                        rt,
                        "dt-ranks-replay",
                        "Replay",
                        Some("refresh"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.rank_reveals.iter_mut().for_each(Reveal::reset);
                    }
                });
                self.rank_order = self.ranks();
                let total = self.totals()[0];
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 6.0 * 78.0), Sense::hover());
                let mut hover = None;
                for (rank, i) in self.rank_order.iter().copied().enumerate() {
                    let y = rt.animate(
                        ("dt-rank-y", i),
                        rank as f32 * 78.0,
                        0.47,
                        Curve::SpringSnappy,
                    );
                    let row =
                        Rect::from_min_size(rect.min + vec2(0.0, y), vec2(rect.width(), 72.0));
                    let response = ui.interact(row, ui.id().with(("dt-rank", i)), Sense::click());
                    rt.record(&format!("dt-rank-inspect-{i}"), &response);
                    if response.hovered() || response.has_focus() {
                        hover = Some(i)
                    }
                    if activated(ui, &response) {
                        response.request_focus();
                        self.rank_pinned = if self.rank_pinned == Some(i) {
                            None
                        } else {
                            Some(i)
                        }
                    }
                    chart_focus(ui, rt, &response);
                }
                let selected = hover.or(self.rank_pinned);
                for (rank, i) in self.rank_order.iter().copied().enumerate() {
                    let y = rt.animate(
                        ("dt-rank-y", i),
                        rank as f32 * 78.0,
                        0.47,
                        Curve::SpringSnappy,
                    );
                    let row =
                        Rect::from_min_size(rect.min + vec2(0.0, y), vec2(rect.width(), 72.0));
                    let engaged = selected == Some(i);
                    let amount = rt.animate(
                        ("dt-rank-highlight", i),
                        if engaged { 1.0 } else { 0.0 },
                        0.22,
                        Curve::Out,
                    );
                    let p = ui.painter().with_clip_rect(rect);
                    if amount > 0.0 {
                        p.rect(
                            row,
                            12.0 * rt.prefs.radius,
                            rt.tokens.surface2.gamma_multiply(amount),
                            Stroke::new(1.0, rt.tokens.border.gamma_multiply(amount)),
                            StrokeKind::Inside,
                        );
                    }
                    p.text(
                        row.min + vec2(12.0, 20.0),
                        Align2::LEFT_CENTER,
                        format!("{:02}", rank + 1),
                        FontId::monospace(11.0),
                        rt.tokens.text3,
                    );
                    p.text(
                        row.min + vec2(40.0, 20.0),
                        Align2::LEFT_CENTER,
                        CHANNELS[i],
                        FontId::proportional(12.0),
                        rt.tokens.text,
                    );
                    let revenue = (total * SHARES[i]).round();
                    p.text(
                        row.right_top() + vec2(-28.0, 20.0),
                        Align2::RIGHT_CENTER,
                        if self.rank_sort == 1 {
                            signed(GROWTH[i])
                        } else {
                            money(revenue)
                        },
                        FontId::proportional(12.0),
                        rt.tokens.text,
                    );
                    let track = Rect::from_min_max(
                        row.min + vec2(40.0, 36.0),
                        row.right_top() + vec2(-12.0, 42.0),
                    );
                    p.rect_filled(track, 3.0, rt.tokens.surface3);
                    let target = if self.rank_sort == 1 {
                        GROWTH[i].abs() / 25.0
                    } else {
                        revenue / (total * SHARES[0]).round()
                    };
                    let progress = self.rank_reveals[i].value(
                        rect,
                        ui,
                        rt,
                        0.95,
                        rank as f32 * 0.06,
                        Curve::OutExpo,
                    );
                    let bar = Rect::from_min_size(
                        track.min,
                        vec2(track.width() * target as f32 * progress, 6.0 + amount * 3.0),
                    );
                    p.rect_filled(
                        bar,
                        4.0,
                        rt.tokens.charts[i % 5].gamma_multiply(if selected.is_some() && !engaged {
                            0.35
                        } else {
                            1.0
                        }),
                    );
                    p.text(
                        row.min + vec2(40.0, 58.0),
                        Align2::LEFT_CENTER,
                        format!(
                            "{} growth · {} attributed revenue",
                            signed(GROWTH[i]),
                            money(revenue)
                        ),
                        FontId::proportional(10.0),
                        rt.tokens.text2,
                    );
                    if amount > 0.0 {
                        icons::paint(
                            &p,
                            "arrow-up-right",
                            Rect::from_min_size(
                                row.right_top() + vec2(-24.0, 14.0),
                                vec2(12.0, 12.0),
                            ),
                            rt.tokens.accent_text.gamma_multiply(amount),
                            1.5,
                            amount,
                        );
                    }
                }
                ui::muted(
                    ui,
                    rt,
                    &if let Some(i) = selected {
                        format!(
                            "{} · {} attributed revenue · {} growth{}",
                            CHANNELS[i],
                            money((total * SHARES[i]).round()),
                            signed(GROWTH[i]),
                            if self.rank_pinned == Some(i) {
                                " · pinned"
                            } else {
                                ""
                            }
                        )
                    } else {
                        let first = self.rank_order[0];
                        format!(
                            "Sorted by {}. First: {}, {}.",
                            ["revenue", "growth", "name"][self.rank_sort],
                            CHANNELS[first],
                            if self.rank_sort == 1 {
                                format!("{}% growth", GROWTH[first])
                            } else {
                                money((total * SHARES[first]).round())
                            }
                        )
                    },
                );
            },
        );
    }
    fn donut(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "dt-donut",
            "Donut chart",
            "Toggle a device to recalculate the visible mix. Segment length and total respond together.",
            0.0,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("Audience by device");
                        ui::muted(ui, rt, "Generated snapshot · 20,000 sessions");
                    });
                    if ui::button(
                        ui,
                        rt,
                        "dt-donut-replay",
                        "Replay",
                        Some("refresh"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.donut_reveal.reset();
                    }
                });
                let (shares, _) = self.device_mix();
                let mut highlight = None;
                let mut toggled = None;
                let width = ui.available_width();
                let diameter = 220.0_f32.min(width);
                let (rect, response) =
                    ui.allocate_exact_size(vec2(width, diameter + 20.0), Sense::hover());
                rt.record("dt-donut-chart", &response);
                let center = rect.center();
                let radius = diameter * 82.0 / 220.0;
                let pointer = ui.ctx().pointer_hover_pos();
                let mut offset = 0.0;
                for (i, share) in shares.iter().copied().enumerate() {
                    if share > 0.0 {
                        let mid = (offset + share / 2.0) as f32 / 100.0 * std::f32::consts::TAU
                            - std::f32::consts::FRAC_PI_2;
                        let target = Rect::from_center_size(
                            center + vec2(mid.cos(), mid.sin()) * radius,
                            vec2(30.0, 30.0),
                        );
                        let r = ui.interact(target, ui.id().with(("dt-arc", i)), Sense::click());
                        let on_arc = pointer.is_some_and(|point| {
                            let delta = point - center;
                            let distance = delta.length();
                            let fraction = (delta.y.atan2(delta.x) + std::f32::consts::FRAC_PI_2)
                                .rem_euclid(std::f32::consts::TAU)
                                / std::f32::consts::TAU
                                * 100.0;
                            distance >= radius - 18.0
                                && distance <= radius + 18.0
                                && fraction >= offset as f32
                                && fraction < (offset + share) as f32
                        });
                        if on_arc {
                            highlight = Some(i);
                            if ui.input(|input| input.pointer.primary_clicked()) {
                                toggled = Some(i);
                                r.request_focus();
                            }
                        }
                        if r.has_focus() {
                            highlight = Some(i);
                            if activated(ui, &r) {
                                toggled = Some(i)
                            }
                        }
                        r.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::Button,
                                true,
                                self.enabled[i],
                                format!("Toggle {} sessions", DEVICES[i].0),
                            )
                        });
                        rt.record(&format!("dt-donut-{i}"), &r);
                        chart_focus(ui, rt, &r);
                    }
                    offset += share;
                }
                let mut legend_rects = Vec::new();
                for (i, (name, _)) in DEVICES.iter().enumerate() {
                    let (legend_rect, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 60.0), Sense::click());
                    legend_rects.push(legend_rect);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            true,
                            self.enabled[i],
                            format!("Include {name} sessions"),
                        )
                    });
                    rt.record(&format!("dt-device-{i}"), &response);
                    if response.hovered() || response.has_focus() {
                        highlight = Some(i)
                    }
                    if response.clicked() {
                        response.request_focus();
                        toggled = Some(i)
                    }
                }
                if let Some(i) = toggled {
                    self.enabled[i] = !self.enabled[i];
                    self.donut_reveal.reset();
                }
                let (shares, total) = self.device_mix();
                for (i, legend_rect) in legend_rects.iter().enumerate() {
                    let emphasized = highlight == Some(i);
                    let p = ui.painter();
                    p.rect(
                        *legend_rect,
                        12.0 * rt.prefs.radius,
                        if emphasized {
                            rt.tokens.surface2
                        } else {
                            rt.tokens.surface
                        },
                        Stroke::new(
                            1.0,
                            if emphasized {
                                rt.tokens.border_strong
                            } else {
                                rt.tokens.border_soft
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    p.circle_filled(
                        legend_rect.left_center() + vec2(16.0, 0.0),
                        if emphasized { 6.0 } else { 4.0 },
                        if self.enabled[i] {
                            rt.tokens.charts[i]
                        } else {
                            rt.tokens.border_strong
                        },
                    );
                    p.text(
                        legend_rect.min + vec2(32.0, 21.0),
                        Align2::LEFT_CENTER,
                        DEVICES[i].0,
                        FontId::proportional(12.0),
                        rt.tokens.text,
                    );
                    p.text(
                        legend_rect.min + vec2(32.0, 41.0),
                        Align2::LEFT_CENTER,
                        format!("{} sessions", number(DEVICES[i].1)),
                        FontId::proportional(11.0),
                        rt.tokens.text2,
                    );
                    p.text(
                        legend_rect.right_center() - vec2(38.0, 0.0),
                        Align2::RIGHT_CENTER,
                        if self.enabled[i] {
                            format!("{}%", shares[i].round())
                        } else {
                            "Off".into()
                        },
                        FontId::monospace(12.0),
                        rt.tokens.text,
                    );
                    if self.enabled[i] {
                        icons::paint(
                            p,
                            "check",
                            Rect::from_center_size(
                                legend_rect.right_center() - vec2(18.0, 0.0),
                                vec2(14.0, 14.0),
                            ),
                            rt.tokens.accent_text,
                            1.6,
                            0.0,
                        );
                    }
                }
                let progress = if highlight.is_some() {
                    self.donut_reveal.finished = true;
                    1.0
                } else {
                    self.donut_reveal
                        .value(rect, ui, rt, 1.15, 0.0, Curve::InOut)
                };
                let p = ui.painter();
                p.circle_stroke(center, radius, Stroke::new(22.0, rt.tokens.surface3));
                let mut offset = 0.0;
                for (i, share) in shares.iter().copied().enumerate() {
                    let selected = highlight == Some(i);
                    let emphasis = rt.animate(
                        ("dt-device-highlight", i),
                        if selected { 1.0 } else { 0.0 },
                        0.47,
                        Curve::SpringSnappy,
                    );
                    let angle = (offset + share / 2.0) as f32 / 100.0 * std::f32::consts::TAU
                        - std::f32::consts::FRAC_PI_2;
                    let moved = center + vec2(angle.cos(), angle.sin()) * emphasis * 4.0;
                    let visible =
                        (share.min(progress as f64 * 100.0 - offset) - 1.3).max(0.0) as f32;
                    arc(
                        p,
                        moved,
                        radius,
                        offset as f32 / 100.0,
                        visible / 100.0,
                        rt.tokens.charts[i].gamma_multiply(if highlight.is_some() && !selected {
                            0.22
                        } else {
                            1.0
                        }),
                        22.0 + emphasis * 6.0,
                    );
                    offset += share;
                }
                let count = highlight
                    .map(|i| if self.enabled[i] { DEVICES[i].1 } else { 0.0 })
                    .unwrap_or(total);
                let shown = rt.animate("dt-donut-total", count as f32, 0.42, Curve::OutExpo);
                p.text(
                    center - vec2(0.0, 8.0),
                    Align2::CENTER_CENTER,
                    number(shown as f64),
                    FontId::proportional(24.0),
                    rt.tokens.text,
                );
                p.text(
                    center + vec2(0.0, 18.0),
                    Align2::CENTER_CENTER,
                    highlight
                        .map(|i| {
                            format!(
                                "{} {}",
                                DEVICES[i].0.to_lowercase(),
                                if self.enabled[i] {
                                    "sessions"
                                } else {
                                    "excluded"
                                }
                            )
                        })
                        .unwrap_or_else(|| "included sessions".into()),
                    FontId::proportional(11.0),
                    rt.tokens.text2,
                );
                let (shares, total) = self.device_mix();
                ui::muted(
                    ui,
                    rt,
                    &if total > 0.0 {
                        format!(
                            "{} included sessions. {}.",
                            number(total),
                            DEVICES
                                .iter()
                                .enumerate()
                                .map(|(i, (name, _))| format!(
                                    "{name}: {}",
                                    if self.enabled[i] {
                                        format!("{:.1}%", shares[i])
                                    } else {
                                        "excluded".into()
                                    }
                                ))
                                .collect::<Vec<_>>()
                                .join("; ")
                        )
                    } else {
                        "No devices included. Select a device to show its sessions.".into()
                    },
                );
            },
        );
    }
    fn goals(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "dt-goals",
            "Progress rings",
            "Each ring traces a finite, labeled target. Replay without changing the data.",
            0.0,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("Monthly targets");
                        ui::muted(ui, rt, "Independent illustrative goals · October");
                    });
                    ui::badge(ui, rt, "3 targets", "accent");
                    if ui::button(
                        ui,
                        rt,
                        "dt-goal-replay",
                        "Replay",
                        Some("refresh"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.goal_reveals.iter_mut().for_each(Reveal::reset);
                    }
                });
                let width = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(vec2(width, 240.0), Sense::hover());
                let step = width / 3.0;
                let mut hover = None;
                for (i, (name, value, _, _)) in GOALS.iter().enumerate() {
                    let row = Rect::from_min_size(
                        rect.min + vec2(step * i as f32, 0.0),
                        vec2(step, 220.0),
                    );
                    let r = ui.interact(row, ui.id().with(("dt-goal", i)), Sense::click());
                    r.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            true,
                            self.goal_pinned == Some(i),
                            format!("Inspect {name}, {value}% of target"),
                        )
                    });
                    rt.record(&format!("dt-goal-{i}"), &r);
                    if r.hovered() || r.has_focus() {
                        hover = Some(i)
                    }
                    if activated(ui, &r) {
                        r.request_focus();
                        self.goal_pinned = if self.goal_pinned == Some(i) {
                            None
                        } else {
                            Some(i)
                        }
                    }
                    chart_focus(ui, rt, &r);
                }
                let selected = hover.or(self.goal_pinned);
                for (i, (name, value, detail, color)) in GOALS.iter().enumerate() {
                    let row = Rect::from_min_size(
                        rect.min + vec2(step * i as f32, 0.0),
                        vec2(step, 220.0),
                    );
                    let highlight = rt.animate(
                        ("dt-goal-highlight", i),
                        if selected == Some(i) { 1.0 } else { 0.0 },
                        0.22,
                        Curve::Out,
                    );
                    let progress = if selected == Some(i) {
                        self.goal_reveals[i].finished = true;
                        1.0
                    } else {
                        self.goal_reveals[i].value(rect, ui, rt, 1.1, i as f32 * 0.09, Curve::InOut)
                    };
                    let center = pos2(row.center().x, row.top() + 90.0);
                    let radius = ((step - 24.0) / 2.0).clamp(20.0, 44.0);
                    let p = ui.painter();
                    if highlight > 0.0 {
                        p.rect(
                            row.shrink(3.0),
                            16.0 * rt.prefs.radius,
                            rt.tokens.surface2.gamma_multiply(highlight),
                            Stroke::new(1.0, rt.tokens.border.gamma_multiply(highlight)),
                            StrokeKind::Inside,
                        );
                    }
                    p.circle_stroke(center, radius, Stroke::new(7.0, rt.tokens.surface3));
                    arc(
                        p,
                        center,
                        radius,
                        0.0,
                        value / 100.0 * progress,
                        rt.tokens.charts[*color],
                        7.0 + highlight * 2.0,
                    );
                    let angle = value / 100.0 * progress * std::f32::consts::TAU
                        - std::f32::consts::FRAC_PI_2;
                    let end = center + vec2(angle.cos(), angle.sin()) * radius;
                    if progress > 0.7 {
                        p.circle(
                            end,
                            3.5 * (1.0 + highlight * 0.4),
                            rt.tokens.charts[*color]
                                .gamma_multiply(((progress - 0.7) / 0.3).min(1.0)),
                            Stroke::new(2.0, rt.tokens.surface),
                        );
                    }
                    p.text(
                        center - vec2(0.0, 7.0 * highlight),
                        Align2::CENTER_CENTER,
                        format!("{value}%"),
                        FontId::proportional(20.0),
                        rt.tokens.text,
                    );
                    if highlight > 0.0 {
                        p.text(
                            center + vec2(0.0, 16.0),
                            Align2::CENTER_CENTER,
                            format!("{}% to go", 100.0 - value),
                            FontId::proportional(10.0),
                            rt.tokens.text2.gamma_multiply(highlight),
                        );
                    }
                    p.text(
                        pos2(center.x, row.top() + 160.0),
                        Align2::CENTER_CENTER,
                        *name,
                        FontId::proportional(12.0),
                        rt.tokens.text,
                    );
                    p.text(
                        pos2(center.x, row.top() + 184.0),
                        Align2::CENTER_CENTER,
                        *detail,
                        FontId::proportional(11.0),
                        rt.tokens.text2,
                    );
                }
                ui.separator();
                ui::muted(
                    ui,
                    rt,
                    &selected
                        .map(|i| {
                            format!(
                                "{} · {} · {}% remaining{}",
                                GOALS[i].0,
                                GOALS[i].2,
                                100.0 - GOALS[i].1,
                                if self.goal_pinned == Some(i) {
                                    " · pinned"
                                } else {
                                    ""
                                }
                            )
                        })
                        .unwrap_or_else(|| {
                            "Hover a target to see what remains. Click to keep its detail visible."
                                .into()
                        }),
                );
            },
        );
    }
    fn table(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::card(ui, rt, "dt-customers", |ui, rt| {
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Customer activity");
                    ui::muted(ui, rt, "28 generated accounts · independent snapshot");
                });
                let label = if self.selected.is_empty() {
                    "Export filtered CSV".into()
                } else {
                    format!("Export {} selected", self.selected.len())
                };
                let can_export = !self.selected.is_empty() || !self.filtered_indices().is_empty();
                ui.add_enabled_ui(can_export, |ui| {
                    if ui::button(
                        ui,
                        rt,
                        "dt-export",
                        &label,
                        Some("download"),
                        ButtonKind::Secondary,
                    )
                    .clicked()
                    {
                        let (csv, count) = self.export_csv();
                        self.export_status = match rt.download(
                            "schematic-generated-customers.csv",
                            &csv,
                            "text/csv;charset=utf-8",
                        ) {
                            Ok(Some(path)) => format!(
                                "Generated CSV for {count} {} accounts. Saved to {}.",
                                if self.selected.is_empty() {
                                    "filtered"
                                } else {
                                    "selected"
                                },
                                path.display()
                            ),
                            Ok(None) => "CSV save cancelled.".into(),
                            Err(error) => format!("CSV export failed: {error}"),
                        };
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                let search = ui.add_sized(
                    [ui.available_width().min(320.0), 34.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Filter customers or channels…"),
                );
                rt.record("dt-search", &search);
                if search.changed() {
                    self.table_page = 0;
                }
                ui.label("Status");
                egui::ComboBox::from_id_salt("dt-status")
                    .selected_text(if self.status == "all" {
                        "All statuses"
                    } else {
                        &self.status
                    })
                    .show_ui(ui, |ui| {
                        for (value, label) in [
                            ("all", "All statuses"),
                            ("Active", "Active"),
                            ("Trial", "Trial"),
                            ("Paused", "Paused"),
                        ] {
                            if ui
                                .selectable_value(&mut self.status, value.into(), label)
                                .changed()
                            {
                                self.table_page = 0;
                            }
                        }
                    });
                if ui::button(ui, rt, "dt-reset", "Reset filters", None, ButtonKind::Ghost)
                    .clicked()
                {
                    self.reset_filters();
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} selected", self.selected.len()));
                if !self.selected.is_empty()
                    && ui::button(
                        ui,
                        rt,
                        "dt-clear-selection",
                        "Clear selection",
                        None,
                        ButtonKind::Ghost,
                    )
                    .clicked()
                {
                    self.selected.clear();
                }
                ui::muted(ui, rt, "Selection persists across pages.");
            });
            let filtered = self.filtered_indices();
            let pages = filtered.len().div_ceil(self.page_size).max(1);
            self.table_page = self.table_page.min(pages - 1);
            let visible = self.visible_indices();
            let previous_visible = self.visible_rows.clone();
            self.visible_rows = visible
                .iter()
                .map(|i| self.accounts[*i].id.clone())
                .collect();
            let selected_count = visible
                .iter()
                .filter(|i| self.selected.contains(&self.accounts[**i].id))
                .count();
            let all = !visible.is_empty() && selected_count == visible.len();
            let mixed = selected_count > 0 && !all;
            egui::ScrollArea::horizontal().id_salt("dt-table-scroll").show(ui,|ui|{
            let width=ui.available_width().max(790.0);let(header,_)=ui.allocate_exact_size(vec2(width,42.0),Sense::hover());ui.painter().rect_filled(header,0.0,rt.tokens.surface2);let select_rect=Rect::from_center_size(pos2(header.left()+24.0,header.center().y),vec2(24.0,24.0));let checkbox=table_check(ui,rt,"dt-select-page",select_rect,all,mixed,!visible.is_empty(),"Select all customers on this page",None);if checkbox.clicked(){self.select_page(!all)}
            let columns=[("name","Customer",48.0,330.0),("status","Status",330.0,438.0),("orders","Orders",438.0,535.0),("revenue","Revenue",535.0,665.0),("growth","Growth",665.0,width)];for(key,label,start,end)in columns{let rect=Rect::from_min_max(pos2(header.left()+start,header.top()),pos2(header.left()+end,header.bottom()));let response=ui.interact(rect,ui.id().with(("dt-sort",key)),Sense::click());rt.record(&format!("dt-table-sort-{key}"),&response);let active=self.sort==key;let next=if active&&self.ascending{"descending"}else{"ascending"};response.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,true,format!("{label}, sort {next}")));if activated(ui,&response){self.ascending=if active{!self.ascending}else{true};self.sort=key.into();}ui.painter().text(rect.left_center()+vec2(12.0,0.0),Align2::LEFT_CENTER,label,FontId::proportional(11.0),rt.tokens.text2);icons::paint(ui.painter(),if active{if self.ascending{"arrow-up"}else{"arrow-down"}}else{"chevrons-up-down"},Rect::from_center_size(rect.right_center()-vec2(16.0,0.0),vec2(12.0,12.0)),if active{rt.tokens.accent_text}else{rt.tokens.text3},1.5,0.0);chart_focus(ui,rt,&response);}
            let(table_rect,_)=ui.allocate_exact_size(vec2(width,visible.len() as f32*72.0),Sense::hover());let mut focus_next=None;
            for(position,index)in visible.iter().copied().enumerate(){let account=self.accounts[index].clone();let target=position as f32*72.0;let previous=self.row_positions.insert(account.id.clone(),target);let y=if previous.is_some(){rt.animate(("dt-row-y",&account.id),target,0.47,Curve::SpringSnappy)}else{target};let row=Rect::from_min_size(table_rect.min+vec2(0.0,y),vec2(width,72.0));let response=ui.interact(row,ui.id().with(("dt-row",&account.id)),Sense::click());rt.record(&format!("dt-account-{}",account.id),&response);let is_selected=self.selected.contains(&account.id);response.widget_info(||egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel,true,is_selected,format!("{}, press Space to select; {} {}, {} orders, {} revenue, {} growth",account.name,account.channel,account.status,account.orders,money(account.revenue),signed(account.growth))));let selection=rt.animate(("dt-row-selected",&account.id),if is_selected{1.0}else{0.0},0.32,Curve::Out);if !previous_visible.contains(&account.id){rt.animate(("dt-row-enter",&account.id),0.0,0.0,Curve::Out);}let arrival=rt.animate(("dt-row-enter",&account.id),1.0,0.32,Curve::Out);let row_clip=table_rect.intersect(row.with_max_y(row.top()+row.height()*arrival));let mut p=ui.painter().with_clip_rect(row_clip);p.set_opacity(arrival);if response.hovered(){p.rect_filled(row,0.0,rt.tokens.surface2);}
if selection>0.0{p.rect_filled(Rect::from_min_size(row.min,vec2(row.width()*selection,row.height())),0.0,rt.tokens.accent_soft);}p.line_segment([row.left_bottom(),row.right_bottom()],Stroke::new(1.0,rt.tokens.border_soft));
                let check_rect=Rect::from_center_size(pos2(row.left()+24.0,row.center().y),vec2(24.0,24.0));let check=table_check(ui,rt,format!("dt-select-{}",account.id),check_rect,is_selected,false,true,&format!("Select {}",account.name),Some((row_clip,arrival)));let extend=ui.input(|i|i.modifiers.shift);let pointer_in_check=ui.ctx().pointer_interact_pos().is_some_and(|p|check_rect.contains(p));
                if check.clicked(){self.toggle_row(&account.id,false)}else if response.clicked()&&(!response.clicked_by(egui::PointerButton::Primary)||!pointer_in_check){self.toggle_row(&account.id,extend);response.request_focus();}
                if response.has_focus(){let down=ui.input_mut(|i|i.consume_key(egui::Modifiers::NONE,Key::ArrowDown));let up=ui.input_mut(|i|i.consume_key(egui::Modifiers::NONE,Key::ArrowUp));if down||up{let next=if down{(position+1).min(visible.len()-1)}else{position.saturating_sub(1)};focus_next=Some(self.accounts[visible[next]].id.clone());}}
                chart_focus(ui,rt,&response);let avatar=Rect::from_center_size(pos2(row.left()+72.0,row.center().y),vec2(32.0,32.0));p.rect(avatar,8.0*rt.prefs.radius,if is_selected{rt.tokens.accent}else{rt.tokens.accent_soft},Stroke::new(1.0,rt.tokens.border_soft),StrokeKind::Inside);let initials=account.name.split_whitespace().take(2).filter_map(|s|s.chars().next()).collect::<String>();p.text(avatar.center(),Align2::CENTER_CENTER,initials,FontId::monospace(11.0),if is_selected{rt.tokens.accent_contrast}else{rt.tokens.accent_text});p.text(row.min+vec2(100.0,27.0),Align2::LEFT_CENTER,&account.name,FontId::proportional(12.0),rt.tokens.text);p.text(row.min+vec2(100.0,47.0),Align2::LEFT_CENTER,format!("{} · {}",account.channel,account.id),FontId::proportional(10.0),rt.tokens.text2);let(status_color,status_bg)=match account.status.as_str(){"Active"=>(rt.tokens.success_text,rt.tokens.success_soft),"Trial"=>(rt.tokens.info_text,rt.tokens.info_soft),_=>(rt.tokens.text2,rt.tokens.surface2)};let badge=Rect::from_min_size(pos2(row.left()+342.0,row.center().y-12.0),vec2(66.0,24.0));p.rect_filled(badge,6.0*rt.prefs.radius,status_bg);p.text(badge.center(),Align2::CENTER_CENTER,&account.status,FontId::proportional(11.0),status_color);p.text(row.min+vec2(510.0,36.0),Align2::RIGHT_CENTER,number(account.orders),FontId::monospace(12.0),rt.tokens.text);p.text(row.min+vec2(640.0,36.0),Align2::RIGHT_CENTER,money(account.revenue),FontId::monospace(12.0),rt.tokens.text);p.text(row.min+vec2(width-24.0,36.0),Align2::RIGHT_CENTER,signed(account.growth),FontId::monospace(12.0),if account.growth>=0.0{rt.tokens.success_text}else{rt.tokens.danger_text});
            }
            if let Some(id)=focus_next{ui.memory_mut(|memory|memory.request_focus(ui.id().with(("dt-row",id))));}
        });
            if filtered.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.strong("No matching customers");
                    ui::muted(ui, rt, "Try a different name, channel, or status.");
                    if ui::button(
                        ui,
                        rt,
                        "dt-empty-reset",
                        "Clear filters",
                        None,
                        ButtonKind::Secondary,
                    )
                    .clicked()
                    {
                        self.reset_filters()
                    }
                    ui.add_space(24.0);
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui::muted(
                    ui,
                    rt,
                    &if filtered.is_empty() {
                        "0 of 28 customers match".into()
                    } else {
                        format!(
                            "{}-{} of {} customers",
                            self.table_page * self.page_size + 1,
                            ((self.table_page + 1) * self.page_size).min(filtered.len()),
                            filtered.len()
                        )
                    },
                );
                ui.label("Rows");
                egui::ComboBox::from_id_salt("dt-page-size")
                    .selected_text(self.page_size.to_string())
                    .show_ui(ui, |ui| {
                        for size in [8, 16, 28] {
                            if ui
                                .selectable_value(&mut self.page_size, size, size.to_string())
                                .changed()
                            {
                                self.table_page = 0;
                            }
                        }
                    });
                ui.add_enabled_ui(self.table_page > 0, |ui| {
                    if ui::icon_button(ui, rt, "dt-prev-page", "arrow-left", "Previous page")
                        .clicked()
                    {
                        self.table_page -= 1;
                    }
                });
                ui.monospace(format!("{} / {pages}", self.table_page + 1));
                ui.add_enabled_ui(self.table_page + 1 < pages, |ui| {
                    if ui::icon_button(ui, rt, "dt-next-page", "arrow-right", "Next page").clicked()
                    {
                        self.table_page += 1;
                    }
                });
            });
            if !self.export_status.is_empty() {
                ui::muted(ui, rt, &self.export_status);
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn table_check(
    ui: &egui::Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    rect: Rect,
    checked: bool,
    mixed: bool,
    enabled: bool,
    label: &str,
    reveal: Option<(Rect, f32)>,
) -> Response {
    let response = ui.interact(
        rect,
        ui.id().with(&id),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, checked, label)
    });
    rt.record(label, &response);
    let mut p = ui.painter().clone();
    if let Some((clip, opacity)) = reveal {
        p = p.with_clip_rect(clip.intersect(ui.clip_rect()));
        p.set_opacity(opacity);
    }
    let face = Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
    let progress = rt.animate(
        ("dt-check", ui.id(), &id),
        if checked { 1.0 } else { 0.0 },
        0.32,
        Curve::Out,
    );
    if response.hovered() {
        p.rect_filled(face.expand(4.0), 6.0, rt.tokens.accent_soft);
    }
    p.rect(
        face,
        4.0 * rt.prefs.radius,
        if checked || mixed {
            rt.tokens.accent
        } else {
            rt.tokens.surface
        },
        Stroke::new(
            1.0,
            if checked || mixed {
                rt.tokens.accent
            } else {
                rt.tokens.border_strong
            },
        ),
        StrokeKind::Inside,
    );
    if mixed {
        p.line_segment(
            [
                face.center() - vec2(4.0, 0.0),
                face.center() + vec2(4.0, 0.0),
            ],
            Stroke::new(2.0, rt.tokens.accent_contrast),
        );
    } else if progress > 0.0 {
        let points = [
            face.left_center() + vec2(3.0, 0.0),
            face.center() + vec2(-1.0, 3.0),
            face.right_top() + vec2(-3.0, 4.0),
        ];
        let trace = Trace::new(points.to_vec());
        let (points, _) = trace.prefix(progress);
        p.add(Shape::line(
            points,
            Stroke::new(1.8, rt.tokens.accent_contrast),
        ));
    }
    chart_focus(ui, rt, &response);
    response
}

impl Page for State {
    fn enter(&mut self, rt: &mut Runtime) {
        self.active = true;
        self.next_tick = None;
        self.changed(rt.now);
        self.kpi_from = [0.0; 4];
        self.donut_reveal.reset();
        self.goal_reveals.iter_mut().for_each(Reveal::reset);
    }
    fn leave(&mut self, _rt: &mut Runtime) {
        self.active = false;
        self.next_tick = None;
        self.inspecting = false;
        self.line_reveal.start = None;
        self.spark_reveals.iter_mut().for_each(|r| r.start = None);
        self.bar_reveals.iter_mut().for_each(|r| r.start = None);
        self.rank_reveals.iter_mut().for_each(|r| r.start = None);
        self.donut_reveal.start = None;
        self.goal_reveals.iter_mut().for_each(|r| r.start = None);
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        // A retained scroll area can have a wider layout rectangle than its viewport.
        let viewport_width = (ui.clip_rect().right() - ui.cursor().min.x - 16.0).max(100.0);
        ui.set_max_width(ui.available_width().min(viewport_width));
        self.update_live(
            rt.now,
            rt.reduced(),
            ui.input(|i| !i.viewport().minimized.unwrap_or(false)),
        );
        if let Some(next) = self.next_tick {
            rt.repaint_after((next - rt.now).max(0.0));
        }
        ui::page_head(
            ui,
            rt,
            "chart-bar",
            "Build / 06",
            "Data & Charts",
            "Charts, comparisons, progress and customer records.",
        );
        ui::section(
            ui,
            rt,
            "dt-overview",
            "01",
            "Revenue",
            "Change the period to update cards and charts.",
        );
        ui::card(ui, rt, "dt-period-bar", |ui, rt| {
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.strong("Schematic Commerce");
                    ui::muted(ui, rt, "Analytics playground · USD");
                });
                let mut choice = match self.period {
                    7 => 0,
                    90 => 2,
                    _ => 1,
                };
                if ui::segmented(ui, rt, "dt-period", &["7D", "30D", "90D"], &mut choice) {
                    self.set_period([7, 30, 90][choice], rt.now);
                }
            });
        });
        ui.add_space(20.0);
        self.kpis(ui, rt);
        ui.add_space(20.0);
        self.line_chart(ui, rt);
        ui::section(
            ui,
            rt,
            "dt-comparisons",
            "02",
            "Comparisons",
            "Compare grouped or stacked values and reorder the ranking.",
        );
        if ui.available_width() > 900.0 {
            ui.columns(2, |columns| {
                self.bars(&mut columns[0], rt);
                self.ranking(&mut columns[1], rt);
            });
        } else {
            self.bars(ui, rt);
            ui.add_space(20.0);
            self.ranking(ui, rt);
        }
        ui::section(
            ui,
            rt,
            "dt-composition",
            "03",
            "Composition and progress",
            "Select a segment or inspect progress toward a target.",
        );
        if ui.available_width() > 760.0 {
            ui.columns(2, |columns| {
                self.donut(&mut columns[0], rt);
                self.goals(&mut columns[1], rt);
            });
        } else {
            self.donut(ui, rt);
            ui.add_space(20.0);
            self.goals(ui, rt);
        }
        ui::section(
            ui,
            rt,
            "dt-records",
            "04",
            "Customer records",
            "Click anywhere in a customer row to select it, or press Space when focused. Filter, sort and export selected records.",
        );
        self.table(ui, rt);
        ui.separator();
        ui.horizontal_wrapped(|ui|{ui.monospace("[ DATA / DEMO ]");ui::muted(ui,rt,"All figures, accounts, and live samples are generated for this prototype. No production analytics or network requests.");});
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::runtime::Preferences;

    fn frame(
        ctx: &egui::Context,
        rt: &mut Runtime,
        time: f64,
        events: Vec<egui::Event>,
        mut body: impl FnMut(&mut egui::Ui, &mut Runtime),
    ) {
        rt.now = time;
        rt.records.clear();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1100.0, 1000.0))),
            time: Some(time),
            events,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| body(ui, rt));
        });
        assert!(!output.shapes.is_empty());
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        assert!(primitives.iter().all(|p| p.clip_rect.is_finite()));
    }
    fn click(ctx: &egui::Context, rt: &mut Runtime, state: &mut State, point: Pos2, time: f64) {
        frame(
            ctx,
            rt,
            time,
            vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
            |ui, rt| state.table(ui, rt),
        );
        frame(
            ctx,
            rt,
            time + 0.01,
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
            |ui, rt| state.table(ui, rt),
        );
    }
    fn press_release(
        ctx: &egui::Context,
        rt: &mut Runtime,
        point: Pos2,
        time: f64,
        mut draw: impl FnMut(&mut egui::Ui, &mut Runtime),
    ) {
        frame(
            ctx,
            rt,
            time,
            vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
            |ui, rt| draw(ui, rt),
        );
        frame(
            ctx,
            rt,
            time + 0.01,
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
            |ui, rt| draw(ui, rt),
        );
    }
    fn center(rt: &Runtime, id: &str) -> Pos2 {
        let r = rt.records.iter().find(|r| r.id == id).unwrap().rect;
        pos2((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0)
    }
    #[test]
    fn native_linked_bars_devices_and_goals_support_pointer_and_keyboard() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut state = State::default();
        frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| state.bars(ui, rt));
        let point = center(&rt, "dt-bar-group-2");
        press_release(&ctx, &mut rt, point, 0.1, |ui, rt| state.bars(ui, rt));
        assert_eq!(state.bar_pinned, Some(2));
        frame(
            &ctx,
            &mut rt,
            0.2,
            vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            |ui, rt| state.bars(ui, rt),
        );
        assert_eq!(state.bar_pinned, None);
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| state.donut(ui, rt));
        let point = center(&rt, "dt-device-1");
        press_release(&ctx, &mut rt, point, 0.1, |ui, rt| state.donut(ui, rt));
        assert!(!state.enabled[1]);
        assert_eq!(state.device_mix().1, 14600.0);
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| state.goals(ui, rt));
        let point = center(&rt, "dt-goal-0");
        press_release(&ctx, &mut rt, point, 0.1, |ui, rt| state.goals(ui, rt));
        assert_eq!(state.goal_pinned, Some(0));
    }
    #[test]
    fn native_row_whitespace_checkbox_and_keyboard_toggle_once() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut state = State::default();
        state.query = "Orbit Supply".into();
        frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| state.table(ui, rt));
        let row = rt
            .records
            .iter()
            .find(|r| r.id == "dt-account-SC-1026")
            .unwrap()
            .rect;
        click(
            &ctx,
            &mut rt,
            &mut state,
            pos2(row[2] - 10.0, (row[1] + row[3]) / 2.0),
            0.1,
        );
        assert!(state.selected.contains("SC-1026"));
        let check = rt
            .records
            .iter()
            .find(|r| r.id == "Select Orbit Supply")
            .unwrap()
            .rect;
        click(
            &ctx,
            &mut rt,
            &mut state,
            pos2((check[0] + check[2]) / 2.0, (check[1] + check[3]) / 2.0),
            0.2,
        );
        assert!(!state.selected.contains("SC-1026"));
        click(
            &ctx,
            &mut rt,
            &mut state,
            pos2(row[2] - 10.0, (row[1] + row[3]) / 2.0),
            0.3,
        );
        let key = egui::Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        };
        frame(&ctx, &mut rt, 0.4, vec![key], |ui, rt| state.table(ui, rt));
        assert!(!state.selected.contains("SC-1026"));
    }
    #[test]
    fn native_line_waits_for_visibility_then_traces_without_rebuilding() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut state = State::default();
        frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| {
            ui.add_space(1400.0);
            state.line_chart(ui, rt)
        });
        assert!(state.line_reveal.start.is_none());
        let points = state.geometry.current.points.clone();
        let grid = state.geometry.grid.clone();
        frame(&ctx, &mut rt, 0.1, vec![], |ui, rt| {
            state.line_chart(ui, rt)
        });
        assert_eq!(state.line_reveal.start, Some(0.1));
        frame(&ctx, &mut rt, 0.6, vec![], |ui, rt| {
            state.line_chart(ui, rt)
        });
        assert!(!state.line_reveal.finished);
        assert_eq!(points, state.geometry.current.points);
        assert_eq!(grid, state.geometry.grid);
        frame(&ctx, &mut rt, 1.3, vec![], |ui, rt| {
            state.line_chart(ui, rt)
        });
        assert!(state.line_reveal.finished);
    }
    #[test]
    fn native_painters_render_all_appearances_without_nonfinite_geometry() {
        for dark in [false, true] {
            for mono in [false, true] {
                let ctx = egui::Context::default();
                let prefs = Preferences {
                    dark,
                    mono,
                    ..Default::default()
                };
                let mut rt = Runtime::new(ctx.clone(), prefs);
                let mut state = State::default();
                frame(&ctx, &mut rt, 0.0, vec![], |ui, rt| {
                    state.bars(ui, rt);
                    state.ranking(ui, rt);
                    state.donut(ui, rt);
                    state.goals(ui, rt);
                });
            }
        }
    }
    #[test]
    fn seeded_data_dates_and_metrics_are_stable() {
        let state = State::default();
        assert_eq!(state.days.len(), 180);
        assert_eq!(state.days[0].date, "2026-05-02");
        assert_eq!(state.days[179].date, "2026-10-28");
        assert_eq!(state.accounts[2].id, "SC-1026");
        assert_eq!(state.accounts[2].name, "Orbit Supply");
        // Values independently exported from the original reference templates.
        assert_eq!(state.totals()[0], 139_994.0);
        assert_eq!(state.totals()[1], 1_833.0);
        assert_eq!(money(state.totals()[2]), "$76");
        assert_eq!(format!("{:.1}%", state.totals()[3]), "3.5%");
        assert_eq!(state.accounts[2].revenue, 14_563.0);
    }
    #[test]
    fn rapid_ranges_replace_geometry_and_inspector_endpoint() {
        let mut state = State::default();
        for period in [7, 90, 30] {
            state.set_period(period, 0.0);
            state.prepare_geometry(vec2(800.0, 300.0));
        }
        assert_eq!(state.day_index, 29);
        assert_eq!(state.geometry.generation, state.generation);
        assert_eq!(state.geometry.current.points.len(), 713);
        let points = state.geometry.current.points.clone();
        state.prepare_geometry(vec2(800.0, 300.0));
        assert_eq!(state.geometry.current.points, points);
    }
    #[test]
    fn curve_geometry_is_monotone_between_adjacent_samples() {
        let values = [1.0, 3.0, 2.0, 4.0];
        let points = monotone(
            &values,
            Rect::from_min_size(Pos2::ZERO, vec2(300.0, 100.0)),
            5.0,
        );
        for (segment, values) in values.windows(2).enumerate() {
            let low = values[0].min(values[1]);
            let high = values[0].max(values[1]);
            for point in &points[segment * 8..=segment * 8 + 8] {
                let value = (1.0 - point.y / 100.0) * 5.0;
                assert!(value >= low as f32 - 1e-5 && value <= high as f32 + 1e-5);
            }
        }
    }
    #[test]
    fn trace_uses_arc_length_and_keeps_final_points_unchanged() {
        let trace = Trace::new(vec![pos2(0.0, 0.0), pos2(3.0, 0.0), pos2(3.0, 4.0)]);
        assert_eq!(trace.prefix(0.5).1, pos2(3.0, 0.5));
        assert_eq!(trace.prefix(1.0).1, pos2(3.0, 4.0));
        assert_eq!(trace.points.len(), 3);
    }
    #[test]
    fn table_filter_sort_selection_and_shift_range() {
        let mut state = State::default();
        state.query = " Orbit Supply ".into();
        assert_eq!(state.filtered_indices(), vec![2]);
        state.toggle_row("SC-1026", false);
        assert_eq!(state.selected.len(), 1);
        state.query = "no-such-account".into();
        assert!(state.filtered_indices().is_empty());
        assert_eq!(state.selected.len(), 1);
        state.reset_filters();
        state.sort = "name".into();
        state.ascending = true;
        let visible = state.visible_indices();
        let first = state.accounts[visible[0]].id.clone();
        let third = state.accounts[visible[2]].id.clone();
        state.toggle_row(&first, false);
        state.toggle_row(&third, true);
        assert!(
            visible[..3]
                .iter()
                .all(|i| state.selected.contains(&state.accounts[*i].id))
        );
    }
    #[test]
    fn export_selection_persists_outside_filter_and_has_bom() {
        let mut state = State::default();
        state.selected.insert("SC-1026".into());
        state.query = "no-such-account".into();
        let (csv, count) = state.export_csv();
        assert_eq!(count, 1);
        assert!(csv.starts_with('\u{feff}'));
        assert!(csv.contains("\"Orbit Supply\""));
        assert!(csv.contains("\r\n"));
    }
    #[test]
    fn select_page_is_independent_from_other_pages() {
        let mut state = State::default();
        state.select_page(true);
        assert_eq!(state.selected.len(), 8);
        state.table_page = 1;
        state.select_page(true);
        assert_eq!(state.selected.len(), 16);
        state.select_page(false);
        assert_eq!(state.selected.len(), 8);
    }
    #[test]
    fn device_mix_handles_everything_excluded() {
        let mut state = State::default();
        assert_eq!(state.device_mix(), ([62.0, 27.0, 11.0], 20000.0));
        state.enabled = [false; 3];
        assert_eq!(state.device_mix(), ([0.0; 3], 0.0));
        state.enabled[1] = true;
        assert_eq!(state.device_mix(), ([0.0, 100.0, 0.0], 5400.0));
    }
    #[test]
    fn live_timing_is_real_time_and_stops_when_hidden_or_reduced() {
        let mut state = State::default();
        state.active = true;
        state.live = true;
        state.update_live(10.0, false, true);
        assert_eq!(state.next_tick, Some(14.0));
        state.update_live(13.99, false, true);
        assert_eq!(state.ticks, 0);
        state.update_live(14.0, false, true);
        assert_eq!(state.ticks, 1);
        state.update_live(18.0, true, true);
        assert_eq!(state.ticks, 1);
        assert_eq!(state.next_tick, None);
        state.update_live(30.0, false, false);
        assert_eq!(state.next_tick, None);
        state.update_live(31.0, false, true);
        assert_eq!(state.next_tick, Some(35.0));
    }
    #[test]
    fn grouped_buckets_preserve_all_orders_and_rank_sorts() {
        let mut state = State::default();
        for period in [7, 30, 90] {
            state.set_period(period, 0.0);
            let sum = state
                .buckets()
                .iter()
                .map(|b| b.new + b.returning)
                .sum::<f64>();
            assert_eq!(sum, state.totals()[1]);
        }
        state.rank_sort = 1;
        assert_eq!(state.ranks()[0], 2);
        state.rank_sort = 2;
        assert_eq!(CHANNELS[state.ranks()[0]], "Direct");
    }
}
