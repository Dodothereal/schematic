use crate::{
    icons,
    motion::{self, Curve},
    runtime::{Page, Runtime},
    sortable::Sortable,
    ui::{self, ButtonKind},
};
use eframe::egui::{self, Align, Align2, FontId, Key, Layout, Rect, RichText, Sense, Stroke};
use std::collections::HashMap;

const DOCUMENTS: [(&str, &str, &str); 4] = [
    ("Release brief", "Project requirements", "document"),
    ("Token inventory", "CSS variables", "palette"),
    ("Motion studies", "Timing & transitions", "wave"),
    ("Review notes", "Review checklist", "pencil"),
];
const TASKS: [(&str, &str); 4] = [
    ("Interface audit", "Find the gaps"),
    ("Layout & rhythm", "Spacing and alignment"),
    ("Motion review", "Transitions and focus"),
    ("Final handoff", "Release documentation"),
];
const PHRASES: [&str; 3] = ["Queued", "Running", "Complete"];

#[derive(Clone, Copy, Debug)]
struct Word {
    from: [f32; 2],
    to: [f32; 2],
}
pub struct State {
    pub accordion: Option<usize>,
    pub collapsed: bool,
    pub notes: bool,
    pub pinned: Sortable,
    pub tasks: Sortable,
    pub word: usize,
    pub count: u16,
    pub flipped: bool,
    pub saved: bool,
    pub flip_reviewed: bool,
    pub expanded: bool,
    pub reviewed: bool,
    words: [Word; 3],
    word_started: f64,
    word_scale: f32,
    heights: [f32; 5],
    statuses: [String; 8],
    active: bool,
    ids: HashMap<String, egui::Id>,
    focus_request: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            accordion: Some(0),
            collapsed: true,
            notes: false,
            pinned: Sortable::new(vec![vec![0], vec![1, 2, 3]]),
            tasks: Sortable::new(vec![vec![0, 1, 2, 3]]),
            word: 0,
            count: 128,
            flipped: false,
            saved: false,
            flip_reviewed: false,
            expanded: false,
            reviewed: false,
            words: [
                Word {
                    from: [1., 0.],
                    to: [1., 0.],
                },
                Word {
                    from: [0., 90.],
                    to: [0., 90.],
                },
                Word {
                    from: [0., 90.],
                    to: [0., 90.],
                },
            ],
            word_started: 0.,
            word_scale: 1.,
            heights: [80., 80., 80., 170., 230.],
            statuses: [
                "Installation is open.".into(),
                "Checklist collapsed. Review notes are optional.".into(),
                "1 pinned document. Pin another to keep it at the top.".into(),
                "Interface audit is first. Use each row's up and down buttons to reorder.".into(),
                "Status: Queued. Queued selected.".into(),
                "128 items. This sample tally stays on this page.".into(),
                "Front of card. Not saved to the reading list.".into(),
                "Project context closed. Review pending.".into(),
            ],
            active: false,
            ids: HashMap::new(),
            focus_request: None,
        }
    }
}
impl State {
    pub fn toggle_accordion(&mut self, index: usize) {
        self.accordion = if self.accordion == Some(index) {
            None
        } else {
            Some(index)
        };
        self.statuses[0] = self
            .accordion
            .map(|i| {
                format!(
                    "{} is open.",
                    ["Installation", "Motion settings", "Source files"][i]
                )
            })
            .unwrap_or_else(|| "All topics are closed.".into());
    }
    pub fn pin_document(&mut self, item: usize) {
        self.pinned.cancel();
        let Some((g, _)) = self.pinned.position(item) else {
            return;
        };
        let dest = 1 - g;
        let index = if dest == 0 {
            self.pinned.groups[0].len()
        } else {
            self.pinned.groups[1]
                .iter()
                .position(|i| *i > item)
                .unwrap_or(self.pinned.groups[1].len())
        };
        self.pinned.move_item(item, dest, index);
        let count = self.pinned.groups[0].len();
        self.statuses[2] = format!(
            "{} {}. {count} pinned {}.",
            DOCUMENTS[item].0,
            if dest == 0 { "pinned" } else { "unpinned" },
            if count == 1 { "document" } else { "documents" }
        );
    }
    pub fn move_task(&mut self, item: usize, up: bool) {
        self.tasks.cancel();
        let Some((_, i)) = self.tasks.position(item) else {
            return;
        };
        let next = i as i32 + if up { -1 } else { 1 };
        if !(0..4).contains(&next) {
            return;
        }
        self.tasks.move_item(item, 0, next as usize);
        if next == 0 && up || next == 3 && !up {
            self.focus_request = Some(format!("pr-move-{item}-{}", if up { "down" } else { "up" }));
        }
        self.statuses[3] = format!("{} moved to position {} of 4.", TASKS[item].0, next + 1);
    }
    fn word_sample(&self, index: usize, now: f64, reduced: bool) -> [f32; 2] {
        let p = if reduced || self.word_scale <= 0. {
            1.
        } else {
            ((now - self.word_started) as f32 / (0.46 * self.word_scale)).clamp(0., 1.)
        };
        let p = motion::ease(Curve::InOut, p);
        let w = self.words[index];
        [
            w.from[0] + (w.to[0] - w.from[0]) * p,
            w.from[1] + (w.to[1] - w.from[1]) * p,
        ]
    }
    pub fn select_word(&mut self, index: usize, now: f64, motion_scale: f32) {
        if index >= 3 || index == self.word {
            return;
        }
        let direction = if index > self.word { 1. } else { -1. };
        let samples =
            std::array::from_fn::<_, 3, _>(|i| self.word_sample(i, now, motion_scale == 0.));
        for (i, sample) in samples.into_iter().enumerate() {
            let incoming = i == index;
            self.words[i] = Word {
                from: if incoming && sample[0] < 0.001 {
                    [0., direction * 90.]
                } else {
                    sample
                },
                to: if incoming {
                    [1., 0.]
                } else {
                    [0., -direction * 90.]
                },
            };
        }
        self.word = index;
        self.word_started = now;
        self.word_scale = motion_scale;
        self.statuses[4] = format!("Status: {}. {} selected.", PHRASES[index], PHRASES[index]);
    }
    pub fn change_count(&mut self, delta: i32) {
        self.count = (self.count as i32 + delta).clamp(0, 999) as u16;
        self.statuses[5] = format!(
            "{} {}.",
            self.count,
            if self.count == 1 { "item" } else { "items" }
        );
    }
    fn context_status(&mut self) {
        self.statuses[7] = format!(
            "Project context {}. Review {}.",
            if self.expanded { "open" } else { "closed" },
            if self.reviewed { "complete" } else { "pending" }
        );
    }
    fn status(&self, ui: &mut egui::Ui, rt: &Runtime, i: usize) {
        ui.add_space(14.);
        let response = ui::muted(ui, rt, &self.statuses[i]);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_live(egui::accesskit::Live::Polite)
        });
    }
    fn accordion_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let topics = [
            (
                "Installation",
                "Download the native app and open it locally. No server is required.",
                "Offline Rust",
            ),
            (
                "Motion settings",
                "Open Tune tokens to select Full, Slow-mo or Reduced. System reduced motion also applies.",
                "Full / Slow-mo / Reduced",
            ),
            (
                "Source files",
                "Tokens and examples are Rust modules in rust/src/.",
                "Native Rust",
            ),
        ];
        ui.horizontal(|ui| {
            ui.label(RichText::new("HELP").small().color(rt.tokens.text3));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("03 topics").small().color(rt.tokens.text3));
            });
        });
        for (i, (title, desc, tag)) in topics.iter().enumerate() {
            ui.separator();
            let id = ui.make_persistent_id(("pr-accordion-trigger", i));
            let response = ui
                .push_id(id, |ui| {
                    ui::button(
                        ui,
                        rt,
                        id,
                        &format!("{:02}   {title}", i + 1),
                        Some(if self.accordion == Some(i) {
                            "minus"
                        } else {
                            "plus"
                        }),
                        ButtonKind::Ghost,
                    )
                })
                .inner;
            rt.record(&format!("pr-accordion-trigger-{i}"), &response);
            self.ids
                .insert(format!("pr-accordion-trigger-{i}"), response.id);
            if response.clicked() {
                response.request_focus();
                self.toggle_accordion(i)
            }
            if response.has_focus()
                && let Some(next) = roving(ui, i, 3, false)
            {
                self.focus_request = Some(format!("pr-accordion-trigger-{next}"));
            }
            let open = self.accordion == Some(i);
            let height = reveal(
                ui,
                rt,
                ("pr-accordion-panel", i),
                open,
                self.heights[i],
                |ui, rt| {
                    ui.add_space(6.);
                    ui.label(RichText::new(*desc).color(rt.tokens.text2));
                    ui.add_space(8.);
                    ui::badge(ui, rt, tag, "");
                    ui.add_space(12.);
                },
            );
            self.heights[i] = height;
        }
        self.status(ui, rt, 0);
    }
    fn collapsible_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RELEASE PACKET")
                    .small()
                    .color(rt.tokens.text3),
            );
            ui::badge(ui, rt, "Ready", "accent");
        });
        ui.add_space(12.);
        ui.heading("Release checklist");
        fact(ui, rt, "Owner", "Interface team");
        fact(ui, rt, "Scope", "8 primitives");
        ui.add_space(12.);
        let b = ui::button(
            ui,
            rt,
            "pr-collapse",
            if self.collapsed {
                "Show checklist"
            } else {
                "Hide checklist"
            },
            Some("chevron-down"),
            ButtonKind::Ghost,
        );
        rt.record("pr-collapse", &b);
        if b.clicked() {
            b.request_focus();
            self.collapsed = !self.collapsed;
            self.statuses[1] = format!(
                "Checklist {}.{}",
                if self.collapsed {
                    "collapsed"
                } else {
                    "expanded"
                },
                if self.collapsed {
                    ""
                } else {
                    " Three handoff checks complete."
                }
            );
        }
        self.heights[3] = reveal(
            ui,
            rt,
            "pr-collapse-panel",
            !self.collapsed,
            self.heights[3],
            |ui, rt| {
                for label in [
                    "Four appearances checked",
                    "Keyboard paths reviewed",
                    "Motion can change its mind",
                ] {
                    ui.horizontal(|ui| {
                        icon(ui, "check", rt.tokens.success, 18.);
                        ui.label(label);
                    });
                }
                let b = ui::button(
                    ui,
                    rt,
                    "pr-detail-toggle",
                    if self.notes {
                        "Hide review notes"
                    } else {
                        "Include review notes"
                    },
                    Some(if self.notes { "minus" } else { "plus" }),
                    ButtonKind::Ghost,
                );
                rt.record("pr-detail-toggle", &b);
                if b.clicked() {
                    b.request_focus();
                    self.notes = !self.notes;
                    self.statuses[1] = format!(
                        "Checklist expanded. Review notes {}.",
                        if self.notes { "included" } else { "hidden" }
                    );
                }
                if self.notes {
                    ui.add_space(8.);
                    ui.label(RichText::new("REVIEW NOTES").small().color(rt.tokens.text3));
                    ui.label("Check keyboard focus, reduced motion and layout before publishing.");
                }
                ui.add_space(12.);
            },
        );
        self.status(ui, rt, 1);
    }
    fn pinned_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        self.pinned.set_labels(&DOCUMENTS.map(|d| d.0));
        let pinned = self.pinned.groups.first().cloned().unwrap_or_default();
        let actions = self.pinned.show(
            ui,
            rt,
            "pr-pinned-list",
            &["Pinned", "All other documents"],
            68.,
            |ui, rt, item| {
                icon(ui, DOCUMENTS[item].2, rt.tokens.text2, 20.);
                ui.vertical(|ui| {
                    ui.label(RichText::new(DOCUMENTS[item].0).strong());
                    ui.label(
                        RichText::new(DOCUMENTS[item].1)
                            .small()
                            .color(rt.tokens.text3),
                    );
                });
                let mut action = None;
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let b = ui::icon_button(
                        ui,
                        rt,
                        ("pr-pin", item),
                        "pin",
                        &format!(
                            "{} {}",
                            if pinned.contains(&item) {
                                "Unpin"
                            } else {
                                "Pin"
                            },
                            DOCUMENTS[item].0
                        ),
                    );
                    rt.record(&format!("pr-pin-{item}"), &b);
                    self.ids.insert(format!("pr-pin-{item}"), b.id);
                    if b.clicked() {
                        b.request_focus();
                        action = Some(())
                    }
                });
                action
            },
        );
        if !self.pinned.announcement.is_empty() {
            self.statuses[2] =
                named_announcement(&self.pinned.announcement, &DOCUMENTS.map(|d| d.0));
        }
        self.pinned.announcement.clear();
        for (item, ()) in actions {
            self.pin_document(item)
        }
        self.status(ui, rt, 2);
    }
    fn tasks_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        self.tasks.set_labels(&TASKS.map(|t| t.0));
        let order = self.tasks.groups[0].clone();
        let actions = self
            .tasks
            .show(ui, rt, "pr-sort-list", &["Tasks"], 68., |ui, rt, item| {
                let position = order.iter().position(|i| *i == item).unwrap_or(0);
                ui.label(
                    RichText::new(format!("{:02}", position + 1))
                        .monospace()
                        .color(rt.tokens.text3),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(TASKS[item].0).strong());
                    ui.label(RichText::new(TASKS[item].1).small().color(rt.tokens.text3));
                });
                let mut action = None;
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    for (up, disabled, icon_name) in [
                        (false, position == 3, "arrow-down"),
                        (true, position == 0, "arrow-up"),
                    ] {
                        let b = ui
                            .add_enabled_ui(!disabled, |ui| {
                                ui::icon_button(
                                    ui,
                                    rt,
                                    ("pr-move", item, up),
                                    icon_name,
                                    &format!(
                                        "Move {} {}",
                                        TASKS[item].0,
                                        if up { "up" } else { "down" }
                                    ),
                                )
                            })
                            .inner;
                        rt.record(
                            &format!("pr-move-{item}-{}", if up { "up" } else { "down" }),
                            &b,
                        );
                        self.ids.insert(
                            format!("pr-move-{item}-{}", if up { "up" } else { "down" }),
                            b.id,
                        );
                        if b.clicked() {
                            b.request_focus();
                            action = Some(up)
                        }
                    }
                });
                action
            });
        if !self.tasks.announcement.is_empty() {
            self.statuses[3] = named_announcement(&self.tasks.announcement, &TASKS.map(|t| t.0));
            self.tasks.announcement.clear();
        }
        for (item, up) in actions {
            self.move_task(item, up)
        }
        if ui::button(
            ui,
            rt,
            "pr-sort-reset",
            "Reset",
            Some("refresh"),
            ButtonKind::Ghost,
        )
        .clicked()
        {
            self.tasks.reset(vec![vec![0, 1, 2, 3]]);
            self.tasks.announcement.clear();
            self.statuses[3] = "Original order restored. Interface audit is first.".into();
        }
        self.status(ui, rt, 3);
    }
    fn text_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.label(RichText::new("BUILD STATUS").small().color(rt.tokens.text3));
        ui.add_space(24.);
        let (r, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 104.), Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Label,
                true,
                format!("Status: {}", PHRASES[self.word]),
            )
        });
        let painter = ui.painter().with_clip_rect(r.intersect(ui.clip_rect()));
        painter.text(
            r.left_top(),
            Align2::LEFT_TOP,
            "Status:",
            FontId::proportional(32.),
            rt.tokens.text2,
        );
        for (i, phrase) in PHRASES.iter().enumerate() {
            let [alpha, y] = self.word_sample(i, rt.now, rt.reduced());
            if alpha > 0.001 {
                painter.text(
                    egui::pos2(r.left(), r.top() + 64. + y * 0.55),
                    Align2::LEFT_CENTER,
                    phrase,
                    FontId::proportional(32.),
                    rt.tokens.accent_text.gamma_multiply(alpha),
                );
            }
            if alpha > 0.001 && alpha < 0.999 {
                rt.repaint()
            }
        }
        let (rule, _) = ui.allocate_exact_size(egui::vec2(80., 2.), Sense::hover());
        for i in 0..3 {
            let x = rule.left() + i as f32 * 28.;
            ui.painter().line_segment(
                [egui::pos2(x, rule.top()), egui::pos2(x + 22., rule.top())],
                Stroke::new(
                    2.0_f32,
                    if i == 0 {
                        rt.tokens.accent
                    } else {
                        rt.tokens.border_strong
                    },
                ),
            );
        }
        ui.add_space(18.);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for (i, label) in PHRASES.iter().enumerate() {
                let b = ui::button(
                    ui,
                    rt,
                    ("pr-swap", i),
                    label,
                    Some(if i == self.word {
                        "check-circle"
                    } else {
                        "circle"
                    }),
                    if i == self.word {
                        ButtonKind::Soft
                    } else {
                        ButtonKind::Ghost
                    },
                );
                rt.record(&format!("pr-swap-{i}"), &b);
                self.ids.insert(format!("pr-swap-{i}"), b.id);
                if b.clicked() {
                    b.request_focus();
                    self.select_word(i, rt.now, rt.motion_scale())
                }
                if b.has_focus()
                    && let Some(next) = roving(ui, i, 3, true)
                {
                    self.select_word(next, rt.now, rt.motion_scale());
                    self.focus_request = Some(format!("pr-swap-{next}"));
                }
            }
        });
        self.status(ui, rt, 4);
    }
    fn render_demo(&mut self, index: usize, ui: &mut egui::Ui, rt: &mut Runtime) {
        if rt.reduced() {
            for word in &mut self.words {
                word.from = word.to;
            }
            self.word_scale = 0.;
        }
        let (id, title, desc, height) = match index {
            0 => (
                "pr-accordion",
                "Accordion",
                "Open one panel at a time. Up/Down, Home and End move focus.",
                360.,
            ),
            1 => (
                "pr-collapsible",
                "Collapsible",
                "Expand the checklist and add review notes.",
                360.,
            ),
            2 => (
                "pr-pinned-list",
                "Pinned list",
                "Drag files between groups, or use the pin button.",
                420.,
            ),
            3 => (
                "pr-sortable-list",
                "Sortable list",
                "Drag a handle or use the arrow buttons.",
                420.,
            ),
            4 => (
                "pr-text-swap",
                "Text swap",
                "Switch between build states.",
                280.,
            ),
            5 => (
                "pr-counter",
                "Animated counter",
                "Increase, decrease or reset the count.",
                280.,
            ),
            6 => (
                "pr-flip-card",
                "Flip card",
                "Flip the card to view its details.",
                420.,
            ),
            _ => (
                "pr-expandable",
                "Expandable surface",
                "Expand project details and complete the review.",
                420.,
            ),
        };
        let response = ui::demo(ui, rt, id, title, desc, height, |ui, rt| match index {
            0 => self.accordion_ui(ui, rt),
            1 => self.collapsible_ui(ui, rt),
            2 => self.pinned_ui(ui, rt),
            3 => self.tasks_ui(ui, rt),
            4 => self.text_ui(ui, rt),
            5 => self.counter_ui(ui, rt),
            6 => self.flip_ui(ui, rt),
            _ => self.expandable_ui(ui, rt),
        });
        rt.anchor(id, title, response.response.rect);
    }
    fn counter_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.label(
            RichText::new("YOUR LOCAL TALLY                 000–999")
                .small()
                .color(rt.tokens.text3),
        );
        ui.add_space(18.);
        let (r, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 92.), Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Label,
                true,
                format!("{} items", self.count),
            )
        });
        let painter = ui.painter().with_clip_rect(r.intersect(ui.clip_rect()));
        let digits = format!("{:03}", self.count);
        for (i, n) in digits.chars().enumerate() {
            let target = n.to_digit(10).unwrap_or(0) as f32;
            let offset = rt.animate(("pr-digit", i), target, 0.46, Curve::OutExpo);
            for d in 0..10 {
                painter.text(
                    egui::pos2(
                        r.center().x - 108. + i as f32 * 44.,
                        r.center().y + (d as f32 - offset) * 66.,
                    ),
                    Align2::LEFT_CENTER,
                    d.to_string(),
                    FontId::monospace(60.),
                    rt.tokens.text,
                );
            }
        }
        painter.text(
            egui::pos2(r.center().x + 38., r.center().y + 10.),
            Align2::LEFT_CENTER,
            "items",
            FontId::proportional(14.),
            rt.tokens.text3,
        );
        ui.add_space(18.);
        ui.horizontal(|ui| {
            let b = ui
                .add_enabled_ui(self.count > 0, |ui| {
                    ui::icon_button(ui, rt, "pr-count-down", "minus", "Decrease count by one")
                })
                .inner;
            rt.record("pr-count-down", &b);
            if b.clicked() {
                b.request_focus();
                self.change_count(-1)
            }
            let b = ui
                .add_enabled_ui(self.count < 999, |ui| {
                    ui::button(
                        ui,
                        rt,
                        "pr-count-up",
                        "Add one",
                        Some("plus"),
                        ButtonKind::Primary,
                    )
                })
                .inner;
            rt.record("pr-count-up", &b);
            if b.clicked() {
                b.request_focus();
                self.change_count(1)
            }
            if ui::button(
                ui,
                rt,
                "pr-counter-reset",
                "Reset",
                Some("refresh"),
                ButtonKind::Ghost,
            )
            .clicked()
            {
                self.count = 128;
                self.statuses[5] = "128 items. Tally reset.".into();
            }
        });
        ui::muted(ui, rt, "Try the carry from 129 to 130.");
        self.status(ui, rt, 5);
    }
    fn flip_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let angle = rt.animate(
            "pr-flip-angle",
            if self.flipped { 1. } else { 0. },
            0.64,
            Curve::InOut,
        );
        let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 310.), Sense::hover());
        let layer = ui.layer_id();
        let first_shape = ui
            .ctx()
            .graphics(|g| g.get(layer).map(|p| p.next_idx().0).unwrap_or(0));
        let face = r;
        let back = angle > 0.5;
        ui.painter().rect(
            face,
            ui::rounded(12. * rt.tokens.radius),
            if back {
                rt.tokens.surface2
            } else {
                rt.tokens.surface
            },
            Stroke::new(1.0_f32, rt.tokens.border),
            egui::StrokeKind::Inside,
        );
        let mut content = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("pr-face")
                .max_rect(r.shrink(20.))
                .layout(Layout::top_down(Align::Min)),
        );
        content.set_clip_rect(face.shrink(1.).intersect(ui.clip_rect()));
        content.add_enabled_ui(back == self.flipped, |ui| {
            ui.label(
                RichText::new(if back {
                    "DETAILS                    02 / 02"
                } else {
                    "RELEASE / 0.3"
                })
                .small()
                .color(rt.tokens.text3),
            );
            ui.add_space(22.);
            if !back {
                let (art, _) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 48.), Sense::hover());
                let p = ui.painter();
                p.circle(
                    egui::pos2(art.left() + 20., art.center().y),
                    20.,
                    rt.tokens.accent_soft,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                );
                p.rect(
                    Rect::from_center_size(
                        egui::pos2(art.left() + 64., art.center().y),
                        egui::vec2(32., 48.),
                    ),
                    ui::rounded(4. * rt.tokens.radius),
                    rt.tokens.accent,
                    Stroke::NONE,
                    egui::StrokeKind::Inside,
                );
                p.rect(
                    Rect::from_center_size(
                        egui::pos2(art.left() + 112., art.center().y),
                        egui::vec2(48., 32.),
                    ),
                    egui::CornerRadius {
                        nw: 24,
                        ne: 24,
                        sw: 4,
                        se: 4,
                    },
                    rt.tokens.accent_soft,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                    egui::StrokeKind::Inside,
                );
                ui.add_space(12.);
            }
            ui.heading(if back {
                "Release details"
            } else {
                "Release notes"
            });
            ui.add_space(12.);
            if back {
                fact(ui, rt, "Status", "Draft");
                fact(ui, rt, "Owner", "Design team");
                fact(ui, rt, "Updated", "Today");
                ui.add_space(12.);
                let b = ui::button(
                    ui,
                    rt,
                    "pr-flip-review",
                    if self.flip_reviewed {
                        "Reviewed"
                    } else {
                        "Mark as reviewed"
                    },
                    Some("check"),
                    ButtonKind::Ghost,
                );
                rt.record("pr-flip-review", &b);
                if b.clicked() {
                    b.request_focus();
                    self.flip_reviewed = !self.flip_reviewed;
                    self.statuses[6] = if self.flip_reviewed {
                        "Idea marked as reviewed."
                    } else {
                        "Idea review cleared."
                    }
                    .into();
                }
            } else {
                ui.label("Updates for version 0.3.");
                ui.add_space(24.);
                let b = ui::button(
                    ui,
                    rt,
                    "pr-flip-save",
                    if self.saved {
                        "Saved to reading list"
                    } else {
                        "Save to reading list"
                    },
                    Some(if self.saved { "check" } else { "plus" }),
                    ButtonKind::Ghost,
                );
                rt.record("pr-flip-save", &b);
                if b.clicked() {
                    b.request_focus();
                    self.saved = !self.saved;
                    self.statuses[6] = if self.saved {
                        "Saved to the local reading list."
                    } else {
                        "Removed from the local reading list."
                    }
                    .into();
                }
            }
        });
        if angle > 0.0001 && angle < 0.9999 {
            project_card(ui, r, angle, first_shape);
        }
        ui.add_space(12.);
        let b = ui::button(
            ui,
            rt,
            "pr-flip",
            if self.flipped {
                "Show front"
            } else {
                "Show reverse"
            },
            Some("refresh"),
            ButtonKind::Secondary,
        );
        rt.record("pr-flip", &b);
        if b.clicked() {
            b.request_focus();
            self.flipped = !self.flipped;
            b.request_focus();
            self.statuses[6] = if self.flipped {
                format!(
                    "Reverse of card. {}",
                    if self.flip_reviewed {
                        "Reviewed."
                    } else {
                        "Not reviewed yet."
                    }
                )
            } else {
                format!(
                    "Front of card. {} the reading list.",
                    if self.saved {
                        "Saved to"
                    } else {
                        "Not saved to"
                    }
                )
            };
        }
        self.status(ui, rt, 6);
    }
    fn expandable_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let (trigger, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 88.), Sense::hover());
        let b = ui.interact(
            trigger,
            ui.make_persistent_id("pr-expand-trigger"),
            Sense::click(),
        );
        b.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                ui.is_enabled(),
                self.expanded,
                "Field notes. Expand project details",
            )
        });
        let mark = Rect::from_center_size(
            egui::pos2(trigger.left() + 24., trigger.center().y),
            egui::vec2(48., 48.),
        );
        ui.painter().rect(
            mark,
            ui::rounded(10. * rt.tokens.radius),
            rt.tokens.accent_soft,
            Stroke::new(1.0_f32, rt.tokens.accent_ring),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            mark.center(),
            Align2::CENTER_CENTER,
            "fn.",
            FontId::proportional(22.),
            rt.tokens.accent_text,
        );
        for (text, y, font, color) in [
            ("PROJECT / 008", trigger.top() + 17., 11., rt.tokens.text3),
            ("Field notes", trigger.top() + 39., 21., rt.tokens.text),
            ("Website release", trigger.top() + 64., 11., rt.tokens.text2),
        ] {
            ui.painter().text(
                egui::pos2(trigger.left() + 64., y),
                Align2::LEFT_CENTER,
                text,
                FontId::proportional(font),
                color,
            );
        }
        let turn = rt.animate(
            "pr-expand-chevron",
            if self.expanded { 1. } else { 0. },
            0.32,
            Curve::InOut,
        );
        let center = egui::pos2(trigger.right() - 14., trigger.center().y);
        let points = [
            egui::vec2(-5., -2.),
            egui::vec2(0., 3.),
            egui::vec2(5., -2.),
        ]
        .map(|v| {
            let a = turn * std::f32::consts::PI;
            center + egui::vec2(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos())
        });
        ui.painter().add(egui::Shape::line(
            points.to_vec(),
            Stroke::new(1.6_f32, rt.tokens.text3),
        ));
        if b.has_focus() {
            ui.painter().rect_stroke(
                trigger,
                ui::rounded(10. * rt.tokens.radius),
                Stroke::new(2.0_f32, rt.tokens.accent_ring),
                egui::StrokeKind::Inside,
            );
        }
        rt.record("pr-expand", &b);
        if b.clicked() {
            b.request_focus();
            self.expanded = !self.expanded;
            self.context_status()
        }
        ui.add_space(14.);
        ui.horizontal(|ui| {
            let p = rt.animate(
                "pr-review-confirm",
                if self.reviewed { 1. } else { 0. },
                0.36,
                Curve::Out,
            );
            let (r, _) = ui.allocate_exact_size(egui::vec2(20., 20.), Sense::hover());
            icons::paint(
                ui.painter(),
                "check",
                r,
                if self.reviewed {
                    rt.tokens.success
                } else {
                    rt.tokens.text3
                },
                1.8,
                p,
            );
            ui.label(if self.reviewed {
                "Reviewed"
            } else {
                "Ready for review"
            });
            ui.label(
                RichText::new("3 contributors")
                    .small()
                    .color(rt.tokens.text3),
            );
        });
        self.heights[4] = reveal(
            ui,
            rt,
            "pr-expand-panel",
            self.expanded,
            self.heights[4],
            |ui, rt| {
                ui.add_space(16.);
                ui.label("Review navigation changes before publishing.");
                ui.add_space(12.);
                fact(ui, rt, "Next milestone", "Editorial handoff");
                fact(ui, rt, "What changed", "Navigation and forms");
                fact(ui, rt, "Review window", "Today, 14:00–16:00");
                ui.add_space(12.);
                let b = ui::button(
                    ui,
                    rt,
                    "pr-review",
                    if self.reviewed {
                        "Review complete · Undo"
                    } else {
                        "Mark review complete"
                    },
                    Some("check"),
                    ButtonKind::Soft,
                );
                rt.record("pr-review", &b);
                if b.clicked() {
                    b.request_focus();
                    self.reviewed = !self.reviewed;
                    self.context_status()
                }
                ui.add_space(10.);
            },
        );
        ui.separator();
        ui::muted(
            ui,
            rt,
            if self.expanded {
                "All the context, in one place"
            } else {
                "Open to see the next step"
            },
        );
        self.status(ui, rt, 7);
    }
}
impl Page for State {
    fn enter(&mut self, _: &mut Runtime) {
        self.active = true;
        self.pinned.set_enabled(true);
        self.tasks.set_enabled(true);
    }
    fn leave(&mut self, _: &mut Runtime) {
        self.active = false;
        self.pinned.set_enabled(false);
        self.tasks.set_enabled(false);
        for word in &mut self.words {
            word.from = word.to;
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::page_head(
            ui,
            rt,
            "layers",
            "Build / The moving parts",
            "Primitives",
            "Disclosure, lists, text, counters and cards.",
        );
        for (i, (id, title)) in [
            ("pr-disclosure", "Disclosure"),
            ("pr-layout", "Lists"),
            ("pr-type", "Text and numbers"),
            ("pr-surfaces", "Cards"),
        ]
        .iter()
        .enumerate()
        {
            ui::section(ui, rt, id, &format!("{:02}", i + 1), title, "");
            if ui.available_width() >= 740. {
                ui.columns(2, |cols| {
                    self.render_demo(i * 2, &mut cols[0], rt);
                    self.render_demo(i * 2 + 1, &mut cols[1], rt);
                });
            } else {
                self.render_demo(i * 2, ui, rt);
                ui.add_space(16.);
                self.render_demo(i * 2 + 1, ui, rt);
            }
            ui.add_space(24.);
        }
        if let Some(name) = self.focus_request.take()
            && let Some(id) = self.ids.get(&name)
        {
            ui.memory_mut(|m| m.request_focus(*id));
            rt.repaint();
        }
    }
}
// Project the native face's tessellated vertices around its vertical axis.
// Text, controls and artwork rotate together, with the same perspective as the
// reference; interactions remain native and disabled on the departing face.
fn project_card(ui: &egui::Ui, rect: Rect, progress: f32, first: usize) {
    let layer = ui.layer_id();
    let shapes = ui.ctx().graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        let shapes = list.all_entries().skip(first).cloned().collect::<Vec<_>>();
        let end = list.next_idx().0;
        for i in first..end {
            list.reset_shape(egui::layers::ShapeIdx(i));
        }
        shapes
    });
    let theta = (if progress > 0.5 {
        progress - 1.
    } else {
        progress
    }) * std::f32::consts::PI;
    let center = rect.center();
    for primitive in ui.ctx().tessellate(shapes, ui.ctx().pixels_per_point()) {
        if let egui::epaint::Primitive::Mesh(mut mesh) = primitive.primitive {
            for v in &mut mesh.vertices {
                let delta = v.pos - center;
                let perspective = 1000. / (1000. - delta.x * theta.sin());
                v.pos =
                    center + egui::vec2(delta.x * theta.cos() * perspective, delta.y * perspective);
            }
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }
}
fn icon(ui: &mut egui::Ui, name: &str, color: egui::Color32, size: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    icons::paint(ui.painter(), name, r, color, 1.6, 0.);
}
fn fact(ui: &mut egui::Ui, rt: &Runtime, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).small().color(rt.tokens.text3));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).small().color(rt.tokens.text2));
        });
    });
}
fn reveal(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    open: bool,
    natural: f32,
    body: impl FnOnce(&mut egui::Ui, &mut Runtime),
) -> f32 {
    let child_id = ui.make_persistent_id(&id);
    let height = rt.animate(id, if open { natural } else { 0. }, 0.46, Curve::Out);
    let start = ui.cursor().min;
    let width = ui.available_width();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(child_id)
            .max_rect(Rect::from_min_size(start, egui::vec2(width, 4096.)))
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(
        Rect::from_min_size(start, egui::vec2(width, height.max(0.))).intersect(ui.clip_rect()),
    );
    child.set_opacity(if natural > 0. {
        (height / natural).clamp(0., 1.)
    } else {
        0.
    });
    if !open {
        child.disable()
    }
    body(&mut child, rt);
    let measured = child.min_rect().height().max(1.);
    ui.allocate_space(egui::vec2(width, height.max(0.)));
    if (measured - natural).abs() > 0.5 && open {
        rt.repaint()
    }
    measured
}
fn roving(ui: &mut egui::Ui, current: usize, count: usize, horizontal: bool) -> Option<usize> {
    let plus = if horizontal {
        Key::ArrowRight
    } else {
        Key::ArrowDown
    };
    let minus = if horizontal {
        Key::ArrowLeft
    } else {
        Key::ArrowUp
    };
    for key in [Key::Home, Key::End, plus, minus] {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            return Some(if key == Key::Home {
                0
            } else if key == Key::End {
                count - 1
            } else if key == plus {
                (current + 1) % count
            } else {
                (current + count - 1) % count
            });
        }
    }
    None
}
fn named_announcement(message: &str, names: &[&str]) -> String {
    let mut message = message.to_string();
    for (i, name) in names.iter().enumerate() {
        message = message
            .replace(&format!("Item {},", i + 1), &format!("{name},"))
            .replace(&format!("item {}.", i + 1), &format!("{name}."));
    }
    message
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut State,
        demo: usize,
        time: f64,
        events: Vec<egui::Event>,
    ) {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 900.),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    s.render_demo(demo, ui, rt);
                    if let Some(name) = s.focus_request.take()
                        && let Some(id) = s.ids.get(&name)
                    {
                        ui.memory_mut(|m| m.request_focus(*id));
                    }
                });
            },
        );
    }
    fn click(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut State,
        demo: usize,
        time: &mut f64,
        name: &str,
    ) {
        let r = rt
            .records
            .iter()
            .find(|r| r.id == name && r.enabled)
            .unwrap_or_else(|| panic!("missing enabled {name}"));
        let pos = egui::pos2((r.rect[0] + r.rect[2]) / 2., (r.rect[1] + r.rect[3]) / 2.);
        for pressed in [true, false] {
            *time += 0.02;
            frame(
                ctx,
                rt,
                s,
                demo,
                *time,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        *time += 0.02;
        frame(ctx, rt, s, demo, *time, vec![]);
    }
    #[test]
    fn native_clicks_operate_all_eight_specimens() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let mut s = State::default();
        s.enter(&mut rt);
        let mut t = 0.;
        for demo in 0..8 {
            t += 1.;
            frame(&ctx, &mut rt, &mut s, demo, t, vec![]);
            t += 0.02;
            frame(&ctx, &mut rt, &mut s, demo, t, vec![]);
            match demo {
                0 => {
                    click(
                        &ctx,
                        &mut rt,
                        &mut s,
                        demo,
                        &mut t,
                        "pr-accordion-trigger-1",
                    );
                    assert_eq!(s.accordion, Some(1));
                }
                1 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-collapse");
                    assert!(!s.collapsed);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-detail-toggle");
                    assert!(s.notes);
                }
                2 => {
                    ctx.memory_mut(|m| m.request_focus(s.ids["pr-pin-1"]));
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-pin-1");
                    assert_eq!(s.pinned.groups[0], vec![0, 1]);
                    assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["pr-pin-1"]));
                }
                3 => {
                    ctx.memory_mut(|m| m.request_focus(s.ids["pr-move-2-up"]));
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-move-2-up");
                    assert_eq!(s.tasks.groups[0], vec![0, 2, 1, 3]);
                    assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["pr-move-2-up"]));
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-move-2-up");
                    assert_eq!(s.tasks.groups[0][0], 2);
                    assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["pr-move-2-down"]));
                }
                4 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-swap-2");
                    assert_eq!(s.word, 2);
                }
                5 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-count-up");
                    assert_eq!(s.count, 129);
                }
                6 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-flip-save");
                    assert!(s.saved);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-flip");
                    assert!(s.flipped);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-flip-review");
                    assert!(s.flip_reviewed);
                }
                _ => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-expand");
                    assert!(s.expanded);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "pr-review");
                    assert!(s.reviewed);
                }
            }
        }
        let order = s.pinned.groups.clone();
        s.pinned.begin_keyboard(1);
        s.pinned.key(Key::ArrowRight);
        s.leave(&mut rt);
        assert_eq!(s.pinned.groups, order);
        assert!(s.saved && s.flip_reviewed && s.reviewed && s.notes);
        s.enter(&mut rt);
        assert_eq!(s.count, 129);
    }
    #[test]
    fn native_pages_render_all_appearances_and_narrow_width() {
        for width in [320., 390., 1440.] {
            for dark in [false, true] {
                for mono in [false, true] {
                    let ctx = egui::Context::default();
                    let mut prefs = crate::runtime::Preferences::default();
                    prefs.dark = dark;
                    prefs.mono = mono;
                    let mut rt = Runtime::new(ctx.clone(), prefs);
                    let mut s = State::default();
                    s.enter(&mut rt);
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 1000.),
                            )),
                            time: Some(1.),
                            ..Default::default()
                        },
                        |ctx| {
                            rt.begin_frame();
                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| s.show(ui, &mut rt));
                            });
                        },
                    );
                    assert!(!out.shapes.is_empty());
                    for r in rt.records.iter().filter(|r| {
                        r.enabled && (r.id.starts_with("pr-") || r.id.starts_with("sort-grip-"))
                    }) {
                        assert!(
                            r.rect[0] >= -1. && r.rect[2] <= width + 1.,
                            "{} escapes {width}: {:?}",
                            r.id,
                            r.rect
                        );
                    }
                    assert_eq!(
                        rt.anchors
                            .iter()
                            .filter(|a| a.id.starts_with("pr-"))
                            .count(),
                        12
                    );
                }
            }
        }
    }
    #[test]
    fn native_keyboard_roving_and_counter_use_actual_widget_focus() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let mut s = State::default();
        let mut t = 0.;
        frame(&ctx, &mut rt, &mut s, 0, t, vec![]);
        t += 0.02;
        frame(&ctx, &mut rt, &mut s, 0, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["pr-accordion-trigger-0"]));
        let press = |ctx: &egui::Context,
                     rt: &mut Runtime,
                     s: &mut State,
                     demo: usize,
                     t: &mut f64,
                     key: Key| {
            for pressed in [true, false] {
                *t += 0.02;
                frame(
                    ctx,
                    rt,
                    s,
                    demo,
                    *t,
                    vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
            }
        };
        press(&ctx, &mut rt, &mut s, 0, &mut t, Key::ArrowDown);
        assert_eq!(s.accordion, Some(0));
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(s.ids["pr-accordion-trigger-1"])
        );
        press(&ctx, &mut rt, &mut s, 0, &mut t, Key::Space);
        assert_eq!(s.accordion, Some(1));
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 4, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["pr-swap-0"]));
        press(&ctx, &mut rt, &mut s, 4, &mut t, Key::End);
        assert_eq!(s.word, 2);
        assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["pr-swap-2"]));
    }
    #[test]
    fn native_flip_projects_text_and_artwork_meshes_at_intermediate_frames() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = State::default();
        frame(&ctx, &mut rt, &mut s, 6, 0., vec![]);
        s.flipped = true;
        frame(&ctx, &mut rt, &mut s, 6, 0.01, vec![]);
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 900.),
                )),
                time: Some(0.27),
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| s.render_demo(6, ui, &mut rt));
            },
        );
        let meshes = out
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Mesh(mesh) = &s.shape {
                    Some(mesh)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(!meshes.is_empty());
        let points = meshes
            .iter()
            .flat_map(|m| m.vertices.iter().map(|v| v.pos))
            .collect::<Vec<_>>();
        assert!(points.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
        let min = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min > 10. && max - min < 500.,
            "projected width {}",
            max - min
        );
    }
    #[test]
    fn all_primitive_state_transitions() {
        let mut s = State::default();
        s.toggle_accordion(1);
        assert_eq!(s.accordion, Some(1));
        s.toggle_accordion(1);
        assert_eq!(s.accordion, None);
        s.pin_document(1);
        assert_eq!(s.pinned.groups[0], vec![0, 1]);
        s.pin_document(1);
        assert_eq!(s.pinned.groups[1], vec![1, 2, 3]);
        s.move_task(2, true);
        assert_eq!(s.tasks.groups[0], vec![0, 2, 1, 3]);
        s.change_count(2);
        assert_eq!(s.count, 130);
        s.change_count(1000);
        assert_eq!(s.count, 999);
        s.change_count(-1000);
        assert_eq!(s.count, 0);
    }
    #[test]
    fn text_reversal_starts_from_displayed_layers() {
        let mut s = State::default();
        s.select_word(1, 0., 1.);
        let before = std::array::from_fn::<_, 3, _>(|i| s.word_sample(i, 0.2, false));
        s.select_word(0, 0.2, 1.);
        for (i, value) in before.iter().enumerate() {
            assert_eq!(*value, s.word_sample(i, 0.2, false));
        }
        assert_eq!(s.word_sample(0, 1., false), [1., 0.]);
        assert_eq!(s.word_sample(1, 1., false)[0], 0.);
    }
}
