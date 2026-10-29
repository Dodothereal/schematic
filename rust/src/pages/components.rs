//! Native components: retained Rust state, real-time operations, and egui vector drawing.
use crate::{
    icons,
    motion::Curve,
    runtime::{Page, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{
    self, Align2, FontId, Id, Key, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2,
};
use std::sync::mpsc::{self, Receiver};

const COPY_TEXT: &str = "Schematic / Components / Built to respond.";
const TAGS: [&str; 5] = ["Interface", "Motion", "Accessible", "Tokens", "Prototyping"];
const PEOPLE: [(&str, &str, &str); 4] = [
    ("AL", "Alex Lin", "Design systems"),
    ("MR", "Morgan Reed", "Product design"),
    ("SJ", "Sam Jordan", "Engineering"),
    ("PK", "Priya Kapoor", "Accessibility"),
];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Action {
    #[default]
    Idle,
    Pending {
        start: f64,
        fail: bool,
    },
    Error,
    Success,
}
impl Action {
    fn start(&mut self, now: f64, recovery: bool) {
        if !matches!(self, Self::Pending { .. }) {
            *self = Self::Pending {
                start: now,
                fail: recovery && *self != Self::Error,
            };
        }
    }
    fn tick(&mut self, now: f64) -> bool {
        if let Self::Pending { start, fail } = *self
            && now - start >= 1.05
        {
            *self = if fail { Self::Error } else { Self::Success };
            return true;
        }
        false
    }
    fn cancel(&mut self) -> bool {
        if matches!(self, Self::Pending { .. }) {
            *self = Self::Idle;
            true
        } else {
            false
        }
    }
    fn busy(self) -> bool {
        matches!(self, Self::Pending { .. })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum HoldSource {
    Pointer,
    Keyboard(Key),
}
#[derive(Clone, Copy, Debug)]
struct Hold {
    start: f64,
    source: HoldSource,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Presence {
    #[default]
    Available,
    Focusing,
    Away,
}
impl Presence {
    fn label(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Focusing => "Focusing",
            Self::Away => "Away",
        }
    }
    fn tone(self) -> &'static str {
        match self {
            Self::Available => "success",
            Self::Focusing => "warning",
            Self::Away => "neutral",
        }
    }
}

pub struct State {
    removed: bool,
    variant_note: String,
    blocks: Vec<usize>,
    block_enters: [Option<f64>; 8],
    size_note: String,
    actions: [Action; 2],
    async_note: String,
    copy: Option<Receiver<Result<(), String>>>,
    copy_result: Option<bool>,
    copy_note: String,
    archived: bool,
    hold: Option<Hold>,
    hold_progress: f32,
    hold_note: String,
    favourite: bool,
    particle_start: Option<f64>,
    favourite_note: String,
    insert: usize,
    inserted: Vec<usize>,
    inserted_enters: [Option<f64>; 4],
    menu_open: bool,
    menu_focus: Option<usize>,
    menu_ids: [Option<Id>; 3],
    split_note: String,
    tool: usize,
    tool_focus: usize,
    tool_note: String,
    guides: bool,
    layers: [bool; 3],
    tinted: bool,
    density: usize,
    tab: usize,
    tab_from: usize,
    tab_changed: Option<f64>,
    chips: [Option<bool>; 5],
    chip_note: String,
    person: usize,
    availability: Presence,
    people_note: String,
    person_changed: Option<f64>,
    active: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            removed: false,
            variant_note: "Choose a button to inspect its intent.".into(),
            blocks: vec![1, 2, 0],
            block_enters: [None; 8],
            size_note: "Add up to eight blocks. Reset starts a fresh composition.".into(),
            actions: [Action::Idle; 2],
            async_note: "Ready. Each simulated request takes about one second.".into(),
            copy: None,
            copy_result: None,
            copy_note: "Copy the sample, or select the text above manually.".into(),
            archived: false,
            hold: None,
            hold_progress: 0.0,
            hold_note: "No action until the full hold completes.".into(),
            favourite: false,
            particle_start: None,
            favourite_note: "128 favourites. Yours is not added yet.".into(),
            insert: 0,
            inserted: Vec::new(),
            inserted_enters: [None; 4],
            menu_open: false,
            menu_focus: None,
            menu_ids: [None; 3],
            split_note: "Note selected. Add a note or open the menu for another block.".into(),
            tool: 0,
            tool_focus: 0,
            tool_note: "Select tool. The shape’s selection boundary is visible.".into(),
            guides: true,
            layers: [true, true, false],
            tinted: false,
            density: 1,
            tab: 0,
            tab_from: 0,
            tab_changed: None,
            chips: [
                Some(true),
                Some(true),
                Some(false),
                Some(false),
                Some(false),
            ],
            chip_note: "Interface and Motion are selected.".into(),
            person: 0,
            availability: Presence::Available,
            people_note: "Alex Lin · Design systems · Available.".into(),
            person_changed: None,
            active: false,
        }
    }
}
impl State {
    fn cancel_hold(&mut self, message: &str) {
        if self.hold.take().is_some() {
            self.hold_progress = 0.0;
            self.hold_note = message.into();
        }
    }
    fn start_hold(&mut self, now: f64, source: HoldSource) {
        if self.active && !self.archived && self.hold.is_none() {
            self.hold = Some(Hold { start: now, source });
            self.hold_note = "Keep holding. Release to cancel.".into();
        }
    }
    fn tick_hold(&mut self, now: f64) {
        if let Some(hold) = self.hold {
            self.hold_progress = ((now - hold.start) / 1.5).clamp(0.0, 1.0) as f32;
            if self.hold_progress >= 1.0 {
                self.hold = None;
                self.archived = true;
                self.hold_note =
                    "Explorations archived in this demo. Restore folder reverses it.".into();
            }
        }
    }
    fn cancel_pending(&mut self) {
        self.cancel_hold("Hold cancelled because this page is no longer active.");
        let cancelled = self
            .actions
            .iter_mut()
            .map(Action::cancel)
            .fold(false, |a, b| a | b);
        if cancelled {
            self.async_note =
                "Pending simulations cancelled when the page became inactive. Ready to run again."
                    .into();
        }
        if self.copy.take().is_some() {
            self.copy_result = None;
            self.copy_note =
                "Clipboard confirmation interrupted. Copy again to verify the result.".into();
        }
        self.menu_open = false;
        self.menu_focus = None;
        self.particle_start = None;
        self.tab_changed = None;
        self.person_changed = None;
        self.block_enters = [None; 8];
        self.inserted_enters = [None; 4];
    }
    fn restore_folder(&mut self) {
        self.archived = false;
        self.hold = None;
        self.hold_progress = 0.0;
        self.hold_note =
            "Folder restored. A new confirmation still requires the full 1.5-second hold.".into();
    }
    fn add_block(&mut self, size: usize) {
        if self.blocks.len() < 8 {
            self.blocks.push(size);

            self.size_note = format!(
                "{} block added. {} of 8 blocks.",
                ["S", "M", "L", "XL"][size],
                self.blocks.len()
            );
        }
    }
    fn restore_chips(&mut self) {
        self.chips = [
            Some(true),
            Some(true),
            Some(false),
            Some(false),
            Some(false),
        ];
        self.chip_note = "All five labels restored. Interface and Motion selected.".into();
    }
    fn chip_selection(&self) -> Vec<&'static str> {
        TAGS.iter()
            .enumerate()
            .filter_map(|(i, t)| (self.chips[i] == Some(true)).then_some(*t))
            .collect()
    }
    fn show_person(&mut self, index: usize) {
        self.person = index;
        let status = self.presence(index);
        self.people_note = format!(
            "{} · {} · {}.",
            PEOPLE[index].1,
            PEOPLE[index].2,
            status.label()
        );
    }
    fn presence(&self, index: usize) -> Presence {
        match index {
            0 => self.availability,
            1 => Presence::Focusing,
            2 => Presence::Away,
            _ => Presence::Available,
        }
    }
    fn row(
        &mut self,
        ui: &mut egui::Ui,
        rt: &mut Runtime,
        left: fn(&mut Self, &mut egui::Ui, &mut Runtime),
        right: fn(&mut Self, &mut egui::Ui, &mut Runtime),
    ) {
        if ui.available_width() > 740.0 {
            ui.columns(2, |cols| {
                left(self, &mut cols[0], rt);
                right(self, &mut cols[1], rt);
            });
        } else {
            left(self, ui, rt);
            ui.add_space(16.0);
            right(self, ui, rt);
        }
        ui.add_space(16.0);
    }
    fn hierarchy(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-0",
            "Button hierarchy",
            "Choose an emphasis to see its role. The button stays put on hover.",
            250.0,
            |ui, rt| {
                let width = ui.available_width();
                let columns = if width > 620.0 {
                    5
                } else if width > 400.0 {
                    3
                } else if width > 270.0 {
                    2
                } else {
                    1
                };
                egui::Grid::new("cp-button-hierarchy")
                    .num_columns(columns)
                    .spacing(Vec2::new(8.0, 12.0))
                    .show(ui, |ui| {
                        for (i, (label, icon, kind, explanation)) in [
                            (
                                "Continue",
                                "arrow-right",
                                ButtonKind::Primary,
                                "Primary · the next step in a flow.",
                            ),
                            (
                                "Save draft",
                                "document",
                                ButtonKind::Secondary,
                                "Secondary · a supporting action.",
                            ),
                            (
                                "Quick action",
                                "bolt",
                                ButtonKind::Soft,
                                "Soft · useful without competing for attention.",
                            ),
                            (
                                "View details",
                                "eye",
                                ButtonKind::Ghost,
                                "Ghost · a quiet, optional action.",
                            ),
                            (
                                "Remove sample",
                                "trash",
                                ButtonKind::Danger,
                                "Destructive · removes a sample, with an undo.",
                            ),
                        ]
                        .iter()
                        .enumerate()
                        {
                            ui.vertical(|ui| {
                                ui.small(["primary", "secondary", "soft", "ghost", "danger"][i]);
                                let r = ui
                                    .add_enabled_ui(!(i == 4 && self.removed), |ui| {
                                        ui::button(
                                            ui,
                                            rt,
                                            ("cp-variant", i),
                                            if i == 4 && self.removed {
                                                "Sample removed"
                                            } else {
                                                label
                                            },
                                            Some(icon),
                                            *kind,
                                        )
                                    })
                                    .inner;
                                if r.clicked() {
                                    self.variant_note = (*explanation).into();
                                    if i == 4 {
                                        self.removed = true;
                                        self.variant_note =
                                            "Sample removed from this demo. Undo restores it."
                                                .into();
                                    }
                                }
                            });
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
                ui::muted(ui, rt, &self.variant_note);
                if self.removed
                    && ui::button(
                        ui,
                        rt,
                        "cp-undo",
                        "Undo sample removal",
                        None,
                        ButtonKind::Secondary,
                    )
                    .clicked()
                {
                    self.removed = false;
                    self.variant_note = "Sample restored.".into();
                }
            },
        );
    }
    fn sizes(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-1",
            "A size for the moment",
            "Each size adds a block to the local composition below.",
            250.0,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    for (i, size) in ["S", "M", "L", "XL"].iter().enumerate() {
                        let r = ui
                            .add_enabled_ui(self.blocks.len() < 8, |ui| {
                                sized_button(
                                    ui,
                                    rt,
                                    ("cp-size", i),
                                    size,
                                    [28.0, 34.0, 40.0, 48.0][i],
                                )
                            })
                            .inner;
                        rt.record(&format!("cp-add-block-{i}"), &r);
                        if r.clicked() {
                            self.add_block(i);
                            self.block_enters[self.blocks.len() - 1] = Some(rt.now);
                        }
                    }
                });
                let widths: Vec<f32> = self
                    .blocks
                    .iter()
                    .map(|i| [28.0, 34.0, 40.0, 48.0][*i])
                    .collect();
                let available = (ui.available_width() - 24.0).max(48.0);
                let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
                let mut used = 0.0;
                for (i, width) in widths.iter().enumerate() {
                    if used > 0.0 && used + width > available {
                        rows.push(Vec::new());
                        used = 0.0;
                    }
                    rows.last_mut().unwrap().push(i);
                    used += width + 8.0;
                }
                let height = (rows.len() as f32 * 56.0 + 24.0).max(80.0);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
                ui.painter().rect(
                    rect,
                    8.0 * rt.tokens.radius,
                    rt.tokens.surface2,
                    Stroke::new(1.0_f32, rt.tokens.border_strong),
                    StrokeKind::Inside,
                );
                for (row_index, row) in rows.iter().enumerate() {
                    let total = row.iter().map(|i| widths[*i]).sum::<f32>()
                        + row.len().saturating_sub(1) as f32 * 8.0;
                    let mut x = (rect.width() - total) / 2.0;
                    for i in row {
                        let ax = rt.animate(
                            ("cp-block-x", i),
                            x + widths[*i] / 2.0,
                            0.35,
                            Curve::SpringSnappy,
                        );
                        let ay = rt.animate(
                            ("cp-block-y", i),
                            12.0 + 24.0 + row_index as f32 * 56.0,
                            0.35,
                            Curve::SpringSnappy,
                        );
                        let progress =
                            self.block_enters[*i].map_or(1.0, |start| rt.progress(start, 0.32));
                        if progress >= 1.0 {
                            self.block_enters[*i] = None;
                        }
                        let r = Rect::from_center_size(
                            rect.min + Vec2::new(ax, ay),
                            Vec2::splat(widths[*i] * (0.85 + 0.15 * progress)),
                        );
                        ui.painter().rect(
                            r,
                            4.0 * rt.tokens.radius,
                            rt.tokens.accent_soft.linear_multiply(progress),
                            Stroke::new(1.0_f32, rt.tokens.accent_ring.linear_multiply(progress)),
                            StrokeKind::Inside,
                        );
                        ui.painter().text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            ["S", "M", "L", "XL"][self.blocks[*i]],
                            FontId::monospace(12.0),
                            rt.tokens.accent_text.linear_multiply(progress),
                        );
                        x += widths[*i] + 8.0;
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{} of 8 blocks", self.blocks.len()));
                    if ui::button(
                        ui,
                        rt,
                        "cp-reset-blocks",
                        "Reset blocks",
                        Some("refresh"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.blocks = vec![1, 2, 0];
                        self.block_enters = [None; 8];
                        self.size_note = "Composition reset to three blocks.".into();
                    }
                });
                ui::muted(ui, rt, &self.size_note);
            },
        );
    }
    fn actions(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-2",
            "Action button",
            "Two deterministic simulations. The second fails once, then succeeds on retry. Nothing is sent.",
            150.0,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..2 {
                        ui.vertical(|ui| {
                            ui.small(if i == 0 {
                                "Success path"
                            } else {
                                "Recovery path"
                            });
                            let (label, icon) = match self.actions[i] {
                                Action::Idle => (
                                    if i == 0 {
                                        "Save sample"
                                    } else {
                                        "Test recovery"
                                    },
                                    if i == 0 { "document" } else { "refresh" },
                                ),
                                Action::Pending { fail, .. } => (
                                    if i == 1 && !fail {
                                        "Retrying…"
                                    } else {
                                        "Working…"
                                    },
                                    "loader",
                                ),
                                Action::Error => ("Retry sample", "alert"),
                                Action::Success => (
                                    if i == 0 {
                                        "Saved · run again"
                                    } else {
                                        "Recovered · test again"
                                    },
                                    "check",
                                ),
                            };
                            let busy = self.actions[i].busy();
                            let display_label = if busy {
                                format!("      {label}")
                            } else {
                                label.into()
                            };
                            let r = ui
                                .add_enabled_ui(!busy, |ui| {
                                    ui::button(
                                        ui,
                                        rt,
                                        ("cp-async", i),
                                        &display_label,
                                        if busy { None } else { Some(icon) },
                                        if i == 0 {
                                            ButtonKind::Primary
                                        } else {
                                            ButtonKind::Secondary
                                        },
                                    )
                                })
                                .inner;
                            if let Action::Pending { start, .. } = self.actions[i] {
                                let angle = ((rt.now - start) as f32 * 7.0) % std::f32::consts::TAU;
                                let points = (0..20)
                                    .map(|j| {
                                        r.rect.left_center()
                                            + Vec2::new(20.0, 0.0)
                                            + Vec2::angled(angle + j as f32 / 20.0 * 4.8) * 7.0
                                    })
                                    .collect();
                                ui.painter().add(egui::Shape::line(
                                    points,
                                    Stroke::new(1.8_f32, rt.tokens.text2),
                                ));
                            }
                            rt.record(if i == 0 { "cp-save" } else { "cp-retry" }, &r);
                            if r.clicked() {
                                self.actions[i].start(rt.now, i == 1);
                                self.async_note = format!(
                                    "{} simulation in progress.",
                                    if i == 0 { "Save" } else { "Recovery" }
                                );
                            }
                        });
                    }
                });
                ui::muted(ui, rt, &self.async_note);
            },
        );
    }
    fn copy(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-3",
            "Copy with certainty",
            "Confirmation appears only if the native clipboard write reports success.",
            150.0,
            |ui, rt| {
                ui.small("Sample text");
                ui.add(
                    egui::Label::new(egui::RichText::new(COPY_TEXT).monospace()).selectable(true),
                );
                let label = if self.copy.is_some() {
                    "Copying…"
                } else {
                    match self.copy_result {
                        Some(true) => "Copied · copy again",
                        Some(false) => "Try copy again",
                        None => "Copy text",
                    }
                };
                let r = ui
                    .add_enabled_ui(self.copy.is_none(), |ui| {
                        ui::button(
                            ui,
                            rt,
                            "cp-copy",
                            label,
                            Some(if self.copy_result == Some(true) {
                                "check"
                            } else {
                                "copy"
                            }),
                            ButtonKind::Secondary,
                        )
                    })
                    .inner;
                if r.clicked() {
                    let (send, recv) = mpsc::channel();
                    self.copy = Some(recv);
                    self.copy_result = None;
                    let copied = rt.copy(ui.ctx(), COPY_TEXT);
                    let _ = send.send(if copied {
                        Ok(())
                    } else {
                        Err("Native clipboard write failed".into())
                    });
                    rt.repaint_after(0.02);
                }
                ui::muted(ui, rt, &self.copy_note);
            },
        );
    }
    fn hold(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-4",
            "A deliberate confirmation",
            "Hold with a pointer, Space or Enter. Release early, leave the button or press Escape to cancel.",
            230.0,
            |ui, rt| {
                ui.horizontal(|ui| {
                    paint_icon(ui, rt, "folder", 24.0);
                    ui.vertical(|ui| {
                        ui.strong("Explorations");
                        ui.small("A local sample folder");
                    });
                    ui::badge(
                        ui,
                        rt,
                        if self.archived { "Archived" } else { "Active" },
                        if self.archived { "success" } else { "neutral" },
                    );
                });
                let r = ui::button(
                    ui,
                    rt,
                    "cp-hold",
                    if self.archived {
                        "Folder archived"
                    } else {
                        "Hold to archive"
                    },
                    Some("lock"),
                    if self.archived {
                        ButtonKind::Soft
                    } else {
                        ButtonKind::Danger
                    },
                );
                rt.record("cp-hold", &r);
                let input = ui.input(|i| {
                    (
                        i.pointer.primary_pressed(),
                        i.pointer.primary_down(),
                        i.pointer.interact_pos(),
                        i.focused,
                        i.key_pressed(Key::Escape),
                        i.key_pressed(Key::Space),
                        i.key_pressed(Key::Enter),
                        i.key_down(Key::Space),
                        i.key_down(Key::Enter),
                    )
                });
                if !input.3 || input.4 {
                    self.cancel_hold(if input.4 {
                        "Hold cancelled. The folder is still active."
                    } else {
                        "Hold cancelled because the window lost focus."
                    });
                }
                if input.3 && !input.4 && !self.archived {
                    if input.0 && r.hovered() {
                        r.request_focus();
                        self.start_hold(rt.now, HoldSource::Pointer);
                    }
                    if r.has_focus() {
                        if input.5 {
                            self.start_hold(rt.now, HoldSource::Keyboard(Key::Space));
                        } else if input.6 {
                            self.start_hold(rt.now, HoldSource::Keyboard(Key::Enter));
                        }
                    }
                }
                if let Some(hold) = self.hold {
                    let held = match hold.source {
                        HoldSource::Pointer => {
                            input.1 && input.2.is_some_and(|p| r.rect.contains(p))
                        }
                        HoldSource::Keyboard(key) => {
                            r.has_focus() && if key == Key::Space { input.7 } else { input.8 }
                        }
                    };
                    if !held {
                        self.cancel_hold("Hold cancelled. The folder is still active.");
                    } else {
                        self.tick_hold(rt.now);
                        if self.hold.is_some() {
                            rt.repaint();
                        }
                    }
                }
                if self.hold_progress > 0.0 {
                    let fill = Rect::from_min_size(
                        r.rect.min,
                        Vec2::new(r.rect.width() * self.hold_progress, r.rect.height()),
                    );
                    ui.painter()
                        .rect_filled(fill, 4.0, rt.tokens.text.linear_multiply(0.12));
                }
                ui.small("Hold for 1.5 seconds. Releasing early cancels.");
                ui::progress(ui, rt, self.hold_progress, 4.0);
                if self.archived
                    && ui::button(
                        ui,
                        rt,
                        "cp-restore-folder",
                        "Restore folder",
                        Some("refresh"),
                        ButtonKind::Secondary,
                    )
                    .clicked()
                {
                    self.restore_folder();
                    r.request_focus();
                }
                ui::muted(ui, rt, &self.hold_note);
            },
        );
    }
    fn favourite(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-5",
            "A little appreciation",
            "One favourite per visitor. The count and the button describe the same state.",
            230.0,
            |ui, rt| {
                let (art, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 86.0), Sense::hover());
                for i in 0..4 {
                    let r = Rect::from_min_size(
                        art.center()
                            + Vec2::new((i % 2) as f32 * 42.0 - 42.0, (i / 2) as f32 * 42.0 - 42.0),
                        Vec2::splat(38.0),
                    );
                    ui.painter().rect(
                        r,
                        if i == 2 { 19.0 } else { 4.0 },
                        [
                            rt.tokens.accent_soft2,
                            rt.tokens.accent,
                            rt.tokens.surface3,
                            rt.tokens.accent_soft,
                        ][i],
                        Stroke::new(1.0_f32, rt.tokens.accent_ring),
                        StrokeKind::Inside,
                    );
                }
                ui.small("Collection 004");
                ui.strong("Useful little details");
                ui.small("Motion, made purposeful.");
                let count = rt.animate(
                    "cp-favourite-count",
                    if self.favourite { 129.0 } else { 128.0 },
                    0.3,
                    Curve::Out,
                );
                let r = ui::button(
                    ui,
                    rt,
                    "cp-favourite",
                    &format!(
                        "{}  {}",
                        if self.favourite {
                            "Favourited"
                        } else {
                            "Favourite"
                        },
                        count.round() as u32
                    ),
                    Some("heart"),
                    if self.favourite {
                        ButtonKind::Soft
                    } else {
                        ButtonKind::Secondary
                    },
                );
                rt.record("cp-favourite", &r);
                if r.clicked() {
                    self.favourite = !self.favourite;
                    self.particle_start = if self.favourite { Some(rt.now) } else { None };
                    self.favourite_note = if self.favourite {
                        "129 favourites. Yours is included."
                    } else {
                        "128 favourites. Your favourite was removed."
                    }
                    .into();
                }
                if let Some(start) = self.particle_start {
                    let p = rt.progress(start, 0.65);
                    if p >= 1.0 || rt.reduced() {
                        self.particle_start = None;
                    } else {
                        for i in 0..9 {
                            let a = std::f32::consts::TAU * i as f32 / 9.0;
                            let pos = r.rect.center() + Vec2::angled(a) * 48.0 * p;
                            ui.painter().circle_filled(
                                pos,
                                3.0 * (1.0 - 0.8 * p),
                                if i % 3 == 0 {
                                    rt.tokens.warning
                                } else {
                                    rt.tokens.accent
                                }
                                .linear_multiply(1.0 - p),
                            );
                        }
                        rt.repaint();
                    }
                }
                ui::muted(ui, rt, &self.favourite_note);
            },
        );
    }
    fn split(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-6",
            "One action, a few alternatives",
            "The main button adds a chosen block. The menu changes what it adds.",
            205.0,
            |ui, rt| {
                let choices = ["note", "task", "link"];
                ui.horizontal(|ui| {
                    let was_open = self.menu_open;
                    if ui::button(
                        ui,
                        rt,
                        "cp-insert",
                        &format!("Add {}", choices[self.insert]),
                        Some("plus"),
                        ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        if self.inserted.len() < 4 {
                            self.inserted.push(self.insert);
                            self.inserted_enters[self.inserted.len() - 1] = Some(rt.now);
                            self.split_note = format!(
                                "{} added. {} of 4 blocks.",
                                ["Note", "Task", "Link"][self.insert],
                                self.inserted.len()
                            );
                        } else {
                            self.split_note =
                                "Four sample blocks added. Clear blocks to start again.".into();
                        }
                    }
                    let trigger = ui::icon_button(
                        ui,
                        rt,
                        "cp-menu-trigger",
                        "chevron-down",
                        "Choose a block type",
                    );
                    rt.record("cp-menu-trigger", &trigger);
                    if trigger.clicked() {
                        self.menu_open = !self.menu_open;
                        self.menu_focus = self.menu_open.then_some(self.insert);
                    }
                    lock_keys(ui, &trigger, false, true);
                    if trigger.has_focus() {
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowDown)) {
                            self.menu_open = true;
                            self.menu_focus = Some(0);
                        }
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowUp)) {
                            self.menu_open = true;
                            self.menu_focus = Some(2);
                        }
                    }
                    let mut chosen = None;
                    let mut focus = self.menu_focus.take();
                    let mut menu_responses = Vec::new();
                    let mut next_focus = None;
                    egui::Popup::from_response(&trigger)
                        .id(Id::new("cp-insert-menu"))
                        .open_bool(&mut self.menu_open)
                        .show(|ui| {
                            for (i, label) in ["Note", "Task", "Link"].iter().enumerate() {
                                let r = ui.selectable_label(
                                    self.insert == i,
                                    format!(
                                        "{label}   {}",
                                        [
                                            "A thought, captured",
                                            "A next step",
                                            "A place to return to"
                                        ][i]
                                    ),
                                );
                                rt.record(&format!("cp-choice-{i}"), &r);
                                self.menu_ids[i] = Some(r.id);
                                menu_responses.push(r.clone());
                                if focus == Some(i) {
                                    r.request_focus();
                                    focus = None;
                                }
                                lock_keys(ui, &r, false, true);
                                if r.has_focus() {
                                    let next = ui.input_mut(|input| {
                                        if input.consume_key(egui::Modifiers::NONE, Key::ArrowDown)
                                        {
                                            Some((i + 1) % 3)
                                        } else if input
                                            .consume_key(egui::Modifiers::NONE, Key::ArrowUp)
                                        {
                                            Some((i + 2) % 3)
                                        } else if input
                                            .consume_key(egui::Modifiers::NONE, Key::Home)
                                        {
                                            Some(0)
                                        } else if input.consume_key(egui::Modifiers::NONE, Key::End)
                                        {
                                            Some(2)
                                        } else {
                                            None
                                        }
                                    });
                                    if next.is_some() {
                                        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                                        next_focus = next;
                                    }
                                    if ui.input(|input| input.key_pressed(Key::Tab)) {
                                        ui.close();
                                        trigger.request_focus();
                                    }
                                }
                                if r.clicked() {
                                    chosen = Some(i);
                                    ui.close();
                                }
                            }
                        });
                    if let Some(next) = next_focus
                        && let Some(response) = menu_responses.get(next)
                    {
                        response.request_focus();
                    }
                    if let Some(i) = chosen {
                        self.insert = i;
                        self.menu_open = false;
                        trigger.request_focus();
                        self.split_note = format!(
                            "{} selected. The main button now adds this block.",
                            ["Note", "Task", "Link"][i]
                        );
                    }
                    if was_open && !self.menu_open {
                        let focused = ui.memory(|m| m.focused());
                        if ui.input(|i| i.key_pressed(Key::Escape))
                            || focused.is_some_and(|id| self.menu_ids.contains(&Some(id)))
                        {
                            crate::runtime::restore_focus(ui.ctx(), Some(trigger.id));
                        }
                    }
                });
                egui::Frame::new()
                    .fill(rt.tokens.surface2)
                    .stroke(Stroke::new(1.0_f32, rt.tokens.border_strong))
                    .corner_radius(8.0 * rt.tokens.radius)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_height(48.0);
                        ui.set_min_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            if self.inserted.is_empty() {
                                ui::muted(ui, rt, "Your next idea goes here.");
                            }
                            for (i, kind) in self.inserted.iter().enumerate() {
                                let progress = self.inserted_enters[i]
                                    .map_or(1.0, |start| rt.progress(start, 0.32));
                                if progress >= 1.0 {
                                    self.inserted_enters[i] = None;
                                }
                                ui.scope(|ui| {
                                    ui.set_opacity(progress);
                                    ui::badge(ui, rt, choices[*kind], "accent");
                                });
                            }
                        });
                    });
                if ui::button(
                    ui,
                    rt,
                    "cp-clear-tray",
                    "Clear blocks",
                    None,
                    ButtonKind::Ghost,
                )
                .clicked()
                {
                    self.inserted.clear();
                    self.inserted_enters = [None; 4];
                    self.split_note = "Sample blocks cleared.".into();
                }
                ui::muted(ui, rt, &self.split_note);
            },
        );
    }
    fn toolbar(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-7",
            "A toolbar that travels with you",
            "Choose a tool to change the local preview. Hover or focus to reveal its name.",
            205.0,
            |ui, rt| {
                let labels = ["Select", "Draw", "Comment", "Measure"];
                let names = ["cursor", "pencil", "chat", "ruler"];
                let mut focus_next = None;
                let mut rects = Vec::new();
                let mut responses = Vec::new();
                let mut highlighted = self.tool;
                let mut engaged = false;
                let pill = ui.painter().add(egui::Shape::Noop);
                ui.horizontal(|ui| {
                    for i in 0..4 {
                        let response = toolbar_button(
                            ui,
                            rt,
                            ("cp-tool", i),
                            names[i],
                            labels[i],
                            self.tool_focus == i,
                            self.tool == i,
                        );
                        rects.push(response.rect);
                        responses.push(response.clone());
                        rt.record(&format!("cp-tool-{i}"), &response);
                        if response.hovered() || response.has_focus() {
                            highlighted = i;
                            engaged = true;
                        }
                        if response.is_pointer_button_down_on() && !response.has_focus() {
                            response.request_focus();
                        }
                        if response.clicked() {
                            response.request_focus();
                            self.tool = i;
                            self.tool_focus = i;
                            self.tool_note = format!(
                                "{} tool. {}",
                                labels[i],
                                [
                                    "The shape’s selection boundary is visible.",
                                    "A drawn path crosses the shape.",
                                    "An annotation appears on the shape.",
                                    "The shape’s dimensions are shown."
                                ][i]
                            );
                        }
                        lock_keys(ui, &response, true, false);
                        if response.has_focus()
                            && let Some(next) = roving_key(ui, i, 4)
                        {
                            self.tool_focus = next;
                            focus_next = Some(next);
                        }
                    }
                });
                if let Some(next) = focus_next {
                    responses[next].request_focus();
                }
                if let Some(rect) = rects.get(highlighted) {
                    let origin = rects[0].min;
                    let x = rt.animate(
                        "cp-toolbar-glide",
                        rect.left() - origin.x,
                        0.22,
                        Curve::InOut,
                    );
                    let opacity = rt.animate(
                        "cp-toolbar-glide-opacity",
                        if engaged { 1.0 } else { 0.0 },
                        0.14,
                        Curve::Out,
                    );
                    let highlight = Rect::from_min_size(origin + Vec2::new(x, 0.0), rect.size());
                    ui.painter().set(
                        pill,
                        egui::Shape::rect_filled(
                            highlight,
                            6.0 * rt.tokens.radius,
                            rt.tokens.accent_soft.linear_multiply(opacity),
                        ),
                    );
                    let selected = rects[self.tool];
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            selected.left_bottom() + Vec2::new(9.0, -3.0),
                            Vec2::new(selected.width() - 18.0, 2.0),
                        ),
                        1.0,
                        rt.tokens.accent,
                    );
                }
                let (area, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 116.0), Sense::hover());
                let shape = Rect::from_center_size(
                    area.center() - Vec2::new(0.0, 8.0),
                    Vec2::new(128.0, 80.0),
                );
                ui.painter().rect(
                    shape,
                    4.0 * rt.tokens.radius,
                    rt.tokens.accent_soft,
                    Stroke::new(1.0_f32, rt.tokens.accent_ring),
                    StrokeKind::Inside,
                );
                for mode in 0..4 {
                    let p = rt.animate(
                        ("cp-tool-visual", mode),
                        if self.tool == mode { 1.0 } else { 0.0 },
                        0.32,
                        Curve::Out,
                    );
                    if p <= 0.001 {
                        continue;
                    }
                    let color = rt.tokens.accent.linear_multiply(p);
                    match mode {
                        0 => {
                            ui.painter().rect_stroke(
                                shape.expand(4.0),
                                0.0,
                                Stroke::new(1.0_f32, color),
                                StrokeKind::Outside,
                            );
                        }
                        1 => {
                            let points = (0..25)
                                .map(|i| {
                                    let t = i as f32 / 24.0;
                                    Pos2::new(
                                        shape.left() + 16.0 + t * 96.0,
                                        shape.center().y + 10.0 * (t * std::f32::consts::TAU).sin(),
                                    )
                                })
                                .collect();
                            ui.painter()
                                .add(egui::Shape::line(points, Stroke::new(2.0_f32, color)));
                        }
                        2 => {
                            let r = Rect::from_min_size(
                                shape.right_top() + Vec2::new(-12.0, -8.0),
                                Vec2::new(30.0, 26.0),
                            );
                            ui.painter().rect_filled(r, 5.0, color);
                            ui.painter().text(
                                r.center(),
                                Align2::CENTER_CENTER,
                                "Aa",
                                FontId::proportional(12.0),
                                rt.tokens.accent_contrast.linear_multiply(p),
                            );
                        }
                        _ => {
                            ui.painter().text(
                                shape.center_bottom() + Vec2::new(0.0, 18.0),
                                Align2::CENTER_CENTER,
                                "128 × 80",
                                FontId::monospace(12.0),
                                color,
                            );
                        }
                    }
                }
                icons::paint(
                    ui.painter(),
                    names[self.tool],
                    Rect::from_center_size(shape.right_bottom(), Vec2::splat(20.0)),
                    rt.tokens.accent_text,
                    1.6,
                    rt.icon_progress("cp-tool-icon", true),
                );
                ui::muted(ui, rt, &self.tool_note);
            },
        );
    }
    fn settings(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-8",
            "Workspace preferences",
            "Switch, checkbox, mixed selection and radio controls retain their keyboard behavior.",
            330.0,
            |ui, rt| {
                ui::toggle(ui, rt, "cp-guides", "Layout guides", &mut self.guides);
                ui.small("Show the construction grid.");
                ui.separator();
                ui.strong("Visible layers");
                let count = self.layers.iter().filter(|v| **v).count();
                let mut all = count == 3;
                let parent = ui.add(
                    egui::Checkbox::new(&mut all, format!("All layers   {count} of 3 visible"))
                        .indeterminate(count > 0 && count < 3),
                );
                rt.record("cp-all-layers", &parent);
                if parent.changed() {
                    self.layers.fill(all);
                }
                for (i, label) in ["Typography", "Shapes", "Annotations"].iter().enumerate() {
                    ui::checkbox(ui, rt, ("cp-layer", i), label, &mut self.layers[i]);
                }
                ui.label("Preview canvas");
                radio_pair(ui, rt, "cp-canvas", ["Paper", "Tinted"], &mut self.tinted);
                let (r, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
                ui.painter().rect(
                    r,
                    8.0 * rt.tokens.radius,
                    if self.tinted {
                        rt.tokens.accent_soft
                    } else {
                        rt.tokens.surface
                    },
                    Stroke::new(1.0_f32, rt.tokens.border),
                    StrokeKind::Inside,
                );
                let guides = rt.animate(
                    "cp-guides-opacity",
                    if self.guides { 1.0 } else { 0.0 },
                    0.22,
                    Curve::Out,
                );
                for i in 1..8 {
                    let x = r.left() + r.width() * i as f32 / 8.0;
                    ui.painter().line_segment(
                        [Pos2::new(x, r.top()), Pos2::new(x, r.bottom())],
                        Stroke::new(1.0_f32, rt.tokens.accent_ring.linear_multiply(guides)),
                    );
                }
                if self.layers[0] {
                    ui.painter().text(
                        r.left_top() + Vec2::new(20.0, 22.0),
                        Align2::LEFT_TOP,
                        "FIELD NOTES / 001",
                        FontId::monospace(10.0),
                        rt.tokens.text2,
                    );
                    ui.painter().text(
                        r.left_top() + Vec2::new(20.0, 50.0),
                        Align2::LEFT_TOP,
                        "Make space\nfor the idea.",
                        FontId::proportional(25.0),
                        rt.tokens.text,
                    );
                }
                if self.layers[1] {
                    for i in 0..3 {
                        let center =
                            Pos2::new(r.right() - 34.0 - i as f32 * 21.0, r.center().y + 20.0);
                        ui.painter().circle_filled(
                            center,
                            18.0 - i as f32 * 3.0,
                            [rt.tokens.accent, rt.tokens.accent_soft2, rt.tokens.surface3][i],
                        );
                    }
                }
                if self.layers[2] {
                    ui.painter().text(
                        r.right_top() + Vec2::new(-12.0, 12.0),
                        Align2::RIGHT_TOP,
                        "Leave room to breathe.",
                        FontId::proportional(11.0),
                        rt.tokens.accent_text,
                    );
                }
                ui::muted(
                    ui,
                    rt,
                    &format!(
                        "Guides {}. {} of 3 layers visible. {} canvas.",
                        if self.guides { "on" } else { "off" },
                        self.layers.iter().filter(|v| **v).count(),
                        if self.tinted { "Tinted" } else { "Paper" }
                    ),
                );
            },
        );
    }
    fn density(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-9",
            "Segmented, connected",
            "Choose a density. The sample list reorganizes around the selection.",
            330.0,
            |ui, rt| {
                let labels = ["Compact", "Comfort", "Roomy"];
                ui::segmented(ui, rt, "cp-density", &labels, &mut self.density);
                let h = rt.animate(
                    "cp-density-height",
                    [42.0, 62.0, 84.0][self.density],
                    0.38,
                    Curve::SpringSnappy,
                );
                for (i, (icon, title, detail)) in [
                    ("document", "Getting started", "8 min read"),
                    ("cube", "Component anatomy", "12 min read"),
                    ("wave", "Motion principles", "6 min read"),
                ]
                .iter()
                .enumerate()
                {
                    let (r, _) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), h), Sense::hover());
                    ui.painter().line_segment(
                        [r.left_bottom(), r.right_bottom()],
                        Stroke::new(1.0_f32, rt.tokens.border_soft),
                    );
                    ui.painter().text(
                        r.left_center() + Vec2::new(4.0, 0.0),
                        Align2::LEFT_CENTER,
                        format!("0{}", i + 1),
                        FontId::monospace(12.0),
                        rt.tokens.text3,
                    );
                    icons::paint(
                        ui.painter(),
                        icon,
                        Rect::from_center_size(
                            r.left_center() + Vec2::new(40.0, 0.0),
                            Vec2::splat(18.0),
                        ),
                        rt.tokens.text2,
                        1.5,
                        0.0,
                    );
                    ui.painter().text(
                        r.left_center() + Vec2::new(60.0, -7.0),
                        Align2::LEFT_CENTER,
                        title,
                        FontId::proportional(13.0),
                        rt.tokens.text,
                    );
                    ui.painter().text(
                        r.left_center() + Vec2::new(60.0, 11.0),
                        Align2::LEFT_CENTER,
                        detail,
                        FontId::proportional(11.0),
                        rt.tokens.text2,
                    );
                }
                ui::muted(
                    ui,
                    rt,
                    &format!(
                        "{} density. {}",
                        labels[self.density],
                        [
                            "More information in less space.",
                            "Balanced spacing for everyday reading.",
                            "Extra space for focused reading."
                        ][self.density]
                    ),
                );
            },
        );
    }
    fn tabs(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-10",
            "Tabs",
            "Left and right arrows choose a tab. Home and End jump to the edges. Tab moves into the selected panel.",
            220.0,
            |ui, rt| {
                let old = self.tab;
                let mut responses = Vec::new();
                let mut next_focus = None;
                ui.horizontal(|ui| {
                    for (i, label) in ["Overview", "Anatomy", "Behavior"].iter().enumerate() {
                        let selected = self.tab == i;
                        let width = ui
                            .painter()
                            .layout_no_wrap(
                                (*label).into(),
                                FontId::proportional(13.0),
                                rt.tokens.text,
                            )
                            .size()
                            .x
                            + 24.0;
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(width, 34.0), Sense::hover());
                        let r = ui.interact(
                            rect,
                            ui.make_persistent_id(("cp-tab", i)),
                            if selected {
                                Sense::click()
                            } else {
                                Sense::CLICK
                            },
                        );
                        r.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                true,
                                selected,
                                *label,
                            )
                        });
                        if r.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                5.0 * rt.tokens.radius,
                                rt.tokens.surface2,
                            );
                        }
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            *label,
                            FontId::proportional(13.0),
                            if selected {
                                rt.tokens.text
                            } else {
                                rt.tokens.text2
                            },
                        );
                        if r.has_focus() {
                            ui.painter().rect_stroke(
                                rect.shrink(1.0),
                                5.0 * rt.tokens.radius,
                                Stroke::new(2.0_f32, rt.tokens.accent_ring),
                                StrokeKind::Inside,
                            );
                        }
                        rt.record(&format!("cp-tab-{i}"), &r);
                        if r.clicked() {
                            self.tab = i;
                            r.request_focus();
                        }
                        lock_keys(ui, &r, true, false);
                        if r.has_focus()
                            && let Some(next) = roving_key(ui, i, 3)
                        {
                            self.tab = next;
                            next_focus = Some(next);
                        }
                        responses.push(r);
                    }
                });
                if let Some(next) = next_focus {
                    responses[next].request_focus();
                }
                if old != self.tab {
                    self.tab_from = old;
                    self.tab_changed = Some(rt.now);
                }
                let selected = responses[self.tab].rect;
                let origin = responses[0].rect.left();
                let x =
                    origin + rt.animate("cp-tab-line-x", selected.left() - origin, 0.3, Curve::Out);
                let width = rt.animate("cp-tab-line-w", selected.width(), 0.3, Curve::Out);
                ui.painter().rect_filled(
                    Rect::from_min_size(Pos2::new(x, selected.bottom()), Vec2::new(width, 2.0)),
                    1.0,
                    rt.tokens.accent,
                );
                let progress = self
                    .tab_changed
                    .map_or(1.0, |start| rt.progress(start, 0.32));
                if progress < 1.0 {
                    rt.repaint();
                }
                let direction = if self.tab > self.tab_from { 1.0 } else { -1.0 };
                let offset = direction * 12.0 * (1.0 - progress);
                let panel = ui.scope(|ui|{ui.add_space(12.0);ui.set_opacity(0.3+0.7*progress);ui.horizontal(|ui|{ui.add_space(offset.max(0.0));ui.vertical(|ui|match self.tab{
                0=>{ui::badge(ui,rt,"Create something","accent");ui.monospace("intent → action → feedback");ui.add_space(10.0);ui.heading("One familiar gesture. A complete response.");ui.label("A control is more than its resting state. It has a focus, a press, a pending moment and a result.");ui::badge(ui,rt,"Made to be used","accent");},
                1=>{ui.horizontal_wrapped(|ui|{for (n,title,desc) in [("01","Container","A surface defines the target and its emphasis."),("02","Label","A verb explains what pressing will do."),("03","Icon","A visual cue reinforces the label."),("04","Feedback","A visible state confirms what happened.")]{ui.vertical(|ui|{ui.monospace(n);ui.strong(title);ui.add(egui::Label::new(desc).wrap());});}});},
                _=>{for (icon,title,desc) in [("cursor","Focus","The next action is clear."),("plus","Press","The target acknowledges input."),("check-circle","Resolve","The outcome stays visible.")]{ui.horizontal(|ui|{paint_icon(ui,rt,icon,20.0);ui.strong(title);ui.label(desc);});}ui::muted(ui,rt,"Reduced motion keeps each state and its meaning, with no directional travel.");}
            });});});
                let panel_response = ui.interact(
                    panel.response.rect,
                    ui.make_persistent_id("cp-tab-panel"),
                    Sense::focusable_noninteractive(),
                );
                panel_response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Label,
                        true,
                        ["Overview panel", "Anatomy panel", "Behavior panel"][self.tab],
                    )
                });
                rt.record("cp-tab-panel", &panel_response);
                if panel_response.has_focus() {
                    ui.painter().rect_stroke(
                        panel.response.rect.expand(3.0),
                        5.0,
                        Stroke::new(2.0_f32, rt.tokens.accent_ring),
                        StrokeKind::Outside,
                    );
                }
            },
        );
    }
    fn chips(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-11",
            "Labels that make room",
            "Select a chip to include it. Remove one and its neighbors move into the space.",
            180.0,
            |ui, rt| {
                let width = ui.available_width();
                let mut targets = Vec::new();
                let (mut x, mut y) = (0.0_f32, 0.0_f32);
                for (i, tag) in TAGS.iter().enumerate() {
                    if self.chips[i].is_none() {
                        continue;
                    }
                    let w = ui
                        .painter()
                        .layout_no_wrap((*tag).into(), FontId::proportional(12.0), rt.tokens.text)
                        .size()
                        .x
                        + 62.0;
                    if x > 0.0 && x + w > width {
                        x = 0.0;
                        y += 42.0;
                    }
                    targets.push((i, x, y, w));
                    x += w + 8.0;
                }
                let (area, _) =
                    ui.allocate_exact_size(Vec2::new(width, (y + 34.0).max(76.0)), Sense::hover());
                let mut removed = None;
                let mut responses: [Option<Response>; 5] = [None, None, None, None, None];
                for (i, x, y, w) in targets {
                    let ax = rt.animate(("cp-chip-x", i), x, 0.35, Curve::SpringSnappy);
                    let ay = rt.animate(("cp-chip-y", i), y, 0.35, Curve::SpringSnappy);
                    let rect =
                        Rect::from_min_size(area.min + Vec2::new(ax, ay), Vec2::new(w, 34.0));
                    let selected = self.chips[i] == Some(true);
                    let select_rect =
                        Rect::from_min_max(rect.min, rect.right_bottom() - Vec2::new(28.0, 0.0));
                    let select = ui.interact(
                        select_rect,
                        ui.make_persistent_id(("cp-chip", i)),
                        Sense::click(),
                    );
                    let remove_rect =
                        Rect::from_min_max(rect.right_top() - Vec2::new(28.0, 0.0), rect.max);
                    let remove = ui
                        .interact(
                            remove_rect,
                            ui.make_persistent_id(("cp-chip-remove", i)),
                            Sense::click(),
                        )
                        .on_hover_text(format!("Remove {}", TAGS[i]));
                    select.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::Button,
                            true,
                            selected,
                            TAGS[i],
                        )
                    });
                    remove.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            format!("Remove {}", TAGS[i]),
                        )
                    });
                    let amount = rt.animate(
                        ("cp-chip-selected", i),
                        if selected { 1.0 } else { 0.0 },
                        0.14,
                        Curve::Out,
                    );
                    ui.painter().rect(
                        rect,
                        17.0 * rt.tokens.radius,
                        rt.tokens
                            .surface
                            .lerp_to_gamma(rt.tokens.accent_soft, amount),
                        Stroke::new(
                            1.0_f32,
                            if selected {
                                rt.tokens.accent_ring
                            } else {
                                rt.tokens.border_strong
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    ui.painter().circle_filled(
                        select_rect.left_center() + Vec2::new(12.0, 0.0),
                        3.0,
                        if selected {
                            rt.tokens.accent
                        } else {
                            rt.tokens.surface3
                        },
                    );
                    ui.painter().text(
                        select_rect.left_center() + Vec2::new(22.0, 0.0),
                        Align2::LEFT_CENTER,
                        TAGS[i],
                        FontId::proportional(12.0),
                        if selected {
                            rt.tokens.accent_text
                        } else {
                            rt.tokens.text
                        },
                    );
                    if remove.hovered() {
                        ui.painter().rect_filled(
                            remove_rect.shrink(3.0),
                            12.0,
                            rt.tokens.danger_soft,
                        );
                    }
                    icons::paint(
                        ui.painter(),
                        "x",
                        Rect::from_center_size(remove_rect.center(), Vec2::splat(12.0)),
                        if remove.hovered() {
                            rt.tokens.danger_text
                        } else {
                            rt.tokens.text2
                        },
                        1.5,
                        0.0,
                    );
                    if select.has_focus() || remove.has_focus() {
                        ui.painter().rect_stroke(
                            rect.expand(2.0),
                            19.0 * rt.tokens.radius,
                            Stroke::new(2.0_f32, rt.tokens.accent_ring),
                            StrokeKind::Outside,
                        );
                    }
                    rt.record(&format!("cp-chip-{i}"), &select);
                    rt.record(&format!("cp-chip-remove-{i}"), &remove);
                    if select.clicked() {
                        self.chips[i] = Some(!selected);
                        let selected = self.chip_selection();
                        self.chip_note = if selected.is_empty() {
                            "No labels selected.".into()
                        } else {
                            format!("Selected: {}.", selected.join(", "))
                        };
                    }
                    if remove.clicked() {
                        removed = Some(i);
                    }
                    responses[i] = Some(select);
                }
                let mut focus = None;
                if let Some(i) = removed {
                    self.chips[i] = None;
                    focus = ((i + 1)..5)
                        .find(|j| self.chips[*j].is_some())
                        .or_else(|| (0..i).rev().find(|j| self.chips[*j].is_some()));
                    self.chip_note =
                        format!("{} removed. Restore labels brings all five back.", TAGS[i]);
                }
                let count = self.chip_selection().len();
                ui.label(format!(
                    "{count} label{} selected",
                    if count == 1 { "" } else { "s" }
                ));
                let reset = ui::button(
                    ui,
                    rt,
                    "cp-reset-chips",
                    "Restore labels",
                    Some("refresh"),
                    ButtonKind::Ghost,
                );
                if reset.clicked() {
                    self.restore_chips();
                }
                if removed.is_some() {
                    if let Some(i) = focus {
                        if let Some(r) = &responses[i] {
                            r.request_focus();
                        }
                    } else {
                        reset.request_focus();
                    }
                }
                ui::muted(ui, rt, &self.chip_note);
            },
        );
    }
    fn people(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "components-12",
            "People, with presence",
            "Choose a person to see their role and availability. Initials keep this demo local.",
            180.0,
            |ui, rt| {
                ui.horizontal(|ui| {
                    for (i, person) in PEOPLE.iter().enumerate() {
                        let presence = self.presence(i);
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(46.0), Sense::click());
                        let response =
                            response.on_hover_text(format!("{} · {}", person.1, presence.label()));
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::Button,
                                true,
                                self.person == i,
                                person.1,
                            )
                        });
                        let fill = match presence {
                            Presence::Available => rt.tokens.success_soft,
                            Presence::Focusing => rt.tokens.warning_soft,
                            Presence::Away => rt.tokens.surface3,
                        };
                        ui.painter().circle_filled(rect.center(), 21.0, fill);
                        ui.painter().circle_stroke(
                            rect.center(),
                            22.0,
                            Stroke::new(
                                if self.person == i || response.has_focus() {
                                    2.0
                                } else {
                                    1.0_f32
                                },
                                if self.person == i {
                                    rt.tokens.accent_ring
                                } else {
                                    rt.tokens.border
                                },
                            ),
                        );
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            person.0,
                            FontId::proportional(13.0),
                            rt.tokens.text,
                        );
                        ui.painter().circle_filled(
                            rect.right_bottom() - Vec2::splat(6.0),
                            4.0,
                            match presence {
                                Presence::Available => rt.tokens.success,
                                Presence::Focusing => rt.tokens.warning,
                                Presence::Away => rt.tokens.text3,
                            },
                        );
                        rt.record(&format!("cp-person-{i}"), &response);
                        if response.clicked() {
                            response.request_focus();
                            self.show_person(i);
                            self.person_changed = Some(rt.now);
                        }
                    }
                });
                let progress = self
                    .person_changed
                    .map_or(1.0, |start| rt.progress(start, 0.22));
                if progress >= 1.0 {
                    self.person_changed = None;
                }
                egui::Frame::new()
                    .fill(rt.tokens.surface2)
                    .stroke(Stroke::new(1.0_f32, rt.tokens.border))
                    .corner_radius(8.0 * rt.tokens.radius)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_opacity(0.45 + 0.55 * progress);
                        ui.horizontal(|ui| {
                            let (initials, name, role) = PEOPLE[self.person];
                            let status = self.presence(self.person);
                            let (avatar, _) =
                                ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                            ui.painter().circle_filled(
                                avatar.center(),
                                20.0,
                                match status {
                                    Presence::Available => rt.tokens.success_soft,
                                    Presence::Focusing => rt.tokens.warning_soft,
                                    Presence::Away => rt.tokens.surface3,
                                },
                            );
                            ui.painter().text(
                                avatar.center(),
                                Align2::CENTER_CENTER,
                                initials,
                                FontId::proportional(12.0),
                                rt.tokens.text,
                            );
                            ui.vertical(|ui| {
                                ui.strong(name);
                                ui::muted(ui, rt, role);
                            });
                            ui::badge(ui, rt, status.label(), status.tone());
                        });
                    });
                ui.label("Your availability");
                let before = self.availability;
                egui::ComboBox::from_id_salt("cp-availability")
                    .selected_text(self.availability.label())
                    .show_ui(ui, |ui| {
                        for status in [Presence::Available, Presence::Focusing, Presence::Away] {
                            ui.selectable_value(&mut self.availability, status, status.label());
                        }
                    });
                if before != self.availability {
                    self.show_person(0);
                    self.person_changed = Some(rt.now);
                    self.people_note = format!(
                        "Your availability is now {}.",
                        self.availability.label().to_lowercase()
                    );
                }
                ui::muted(ui, rt, &self.people_note);
            },
        );
    }
}
impl Page for State {
    fn enter(&mut self, _rt: &mut Runtime) {
        self.active = true;
    }
    fn leave(&mut self, _rt: &mut Runtime) {
        self.active = false;
        self.cancel_pending();
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        if !ui.input(|i| i.focused) {
            self.cancel_pending();
        } else {
            for (i, action) in self.actions.iter_mut().enumerate() {
                if action.tick(rt.now) {
                    self.async_note = match *action {
                        Action::Error => {
                            "Simulated failure. Your sample is safe. Retry sample will succeed."
                                .into()
                        }
                        _ => format!(
                            "{} You can run it again.",
                            if i == 0 {
                                "Sample saved in this demo."
                            } else {
                                "Retry succeeded in this demo."
                            }
                        ),
                    };
                }
                if action.busy() {
                    rt.repaint_after(0.016);
                }
            }
            if let Some(receiver) = &self.copy {
                match receiver.try_recv() {
                    Ok(result) => {
                        self.copy_result = Some(result.is_ok());
                        self.copy_note=if result.is_ok(){"Copied to your clipboard."}else{"Clipboard unavailable or permission denied. Select and copy the sample text manually."}.into();
                        self.copy = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => rt.repaint_after(0.03),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.copy = None;
                        self.copy_result = Some(false);
                        self.copy_note="Clipboard confirmation failed. Try copy again or copy the sample manually.".into();
                    }
                }
            }
        }
        ui::page_head(
            ui,
            rt,
            "cube",
            "Build / Interaction library",
            "Components",
            "Buttons, tabs, menus, chips and avatars.",
        );
        ui::section(ui, rt, "cp-buttons", "01", "Buttons", "");
        self.row(ui, rt, Self::hierarchy, Self::sizes);
        ui::section(ui, rt, "cp-states", "02", "Action states", "");
        self.row(ui, rt, Self::actions, Self::copy);
        self.row(ui, rt, Self::hold, Self::favourite);
        ui::section(ui, rt, "cp-menus", "03", "Menus and tooltips", "");
        self.row(ui, rt, Self::split, Self::toolbar);
        ui::section(ui, rt, "cp-settings", "04", "Settings", "");
        self.row(ui, rt, Self::settings, Self::density);
        ui::section(
            ui,
            rt,
            "cp-collections",
            "05",
            "Tabs, chips and avatars",
            "",
        );
        self.tabs(ui, rt);
        ui.add_space(16.0);
        self.row(ui, rt, Self::chips, Self::people);
    }
}
fn roving_key(ui: &mut egui::Ui, index: usize, count: usize) -> Option<usize> {
    let next = ui.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
            Some((index + 1) % count)
        } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
            Some((index + count - 1) % count)
        } else if i.consume_key(egui::Modifiers::NONE, Key::Home) {
            Some(0)
        } else if i.consume_key(egui::Modifiers::NONE, Key::End) {
            Some(count - 1)
        } else {
            None
        }
    });
    if next.is_some() {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    next
}

fn paint_icon(ui: &mut egui::Ui, rt: &Runtime, name: &str, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    icons::paint(ui.painter(), name, rect, rt.tokens.text2, 1.6, 0.0);
    response
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn recovery_fails_then_succeeds_then_repeats_failure() {
        let mut a = Action::Idle;
        a.start(10.0, true);
        assert!(!a.tick(11.049));
        assert!(a.tick(11.05));
        assert_eq!(a, Action::Error);
        a.start(20.0, true);
        a.tick(21.1);
        assert_eq!(a, Action::Success);
        a.start(30.0, true);
        a.tick(31.1);
        assert_eq!(a, Action::Error);
    }
    #[test]
    fn duplicate_pending_press_cannot_restart_deadline() {
        let mut a = Action::Idle;
        a.start(0.0, false);
        a.start(0.9, false);
        a.tick(1.051);
        assert_eq!(a, Action::Success);
    }
    #[test]
    fn holds_require_fixed_real_time_and_cancel() {
        let mut s = State::default();
        s.active = true;
        s.start_hold(10.0, HoldSource::Keyboard(Key::Space));
        s.tick_hold(10.25);
        assert!(!s.archived);
        s.cancel_hold("cancel");
        assert_eq!(s.hold_progress, 0.0);
        s.start_hold(20.0, HoldSource::Pointer);
        s.tick_hold(21.499);
        assert!(!s.archived);
        s.tick_hold(21.5);
        assert!(s.archived);
        s.restore_folder();
        assert!(!s.archived);
        assert_eq!(s.hold_progress, 0.0);
    }
    #[test]
    fn leave_cancels_work_but_retains_user_choices() {
        let mut s = State::default();
        s.active = true;
        s.actions[0].start(1.0, false);
        s.start_hold(1.0, HoldSource::Pointer);
        s.favourite = true;
        s.tab = 2;
        s.layers = [false, true, true];
        s.cancel_pending();
        assert_eq!(s.actions[0], Action::Idle);
        assert!(s.hold.is_none());
        assert!(s.favourite);
        assert_eq!(s.tab, 2);
        assert_eq!(s.layers, [false, true, true]);
    }
    #[test]
    fn composition_is_bounded_and_chips_restore() {
        let mut s = State::default();
        for _ in 0..20 {
            s.add_block(3);
        }
        assert_eq!(s.blocks.len(), 8);
        s.chips[0] = None;
        s.chips[3] = Some(true);
        s.restore_chips();
        assert_eq!(s.chip_selection(), vec!["Interface", "Motion"]);
        assert!(s.chips.iter().all(Option::is_some));
    }
    #[test]
    fn availability_is_instance_local() {
        let mut a = State::default();
        a.availability = Presence::Away;
        a.show_person(0);
        assert!(a.people_note.ends_with("Away."));
        assert_eq!(State::default().availability, Presence::Available);
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod native_interaction_tests {
    use super::*;
    use crate::runtime::Preferences;
    fn frame(
        s: &mut State,
        rt: &mut Runtime,
        ctx: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 9000.0))),
            time: Some(time),
            events,
            focused: true,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| {
            rt.begin_frame();
            egui::CentralPanel::default().show(ctx, |ui| s.show(ui, rt));
        });
        assert!(output.platform_output.accesskit_update.is_some());
        assert!(!output.shapes.is_empty());
        for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
            if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
            }
        }
    }
    fn center(rt: &Runtime, id: &str) -> Pos2 {
        let r = rt
            .records
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        Pos2::new((r.rect[0] + r.rect[2]) / 2.0, (r.rect[1] + r.rect[3]) / 2.0)
    }
    fn click(s: &mut State, rt: &mut Runtime, ctx: &egui::Context, id: &str, t: f64) {
        let p = center(rt, id);
        frame(
            s,
            rt,
            ctx,
            t,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(
            s,
            rt,
            ctx,
            t + 0.01,
            vec![egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    #[test]
    fn actual_egui_pointer_actions_recover_and_toggle() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        frame(&mut s, &mut rt, &ctx, 0.02, vec![]);
        click(&mut s, &mut rt, &ctx, "cp-retry", 0.1);
        assert!(s.actions[1].busy());
        frame(&mut s, &mut rt, &ctx, 1.2, vec![]);
        assert_eq!(s.actions[1], Action::Error);
        click(&mut s, &mut rt, &ctx, "cp-retry", 1.3);
        frame(&mut s, &mut rt, &ctx, 2.4, vec![]);
        assert_eq!(s.actions[1], Action::Success);
        click(&mut s, &mut rt, &ctx, "cp-favourite", 2.5);
        assert!(s.favourite);
    }
    fn key(key: Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn keyboard_hold_tabs_toolbar_and_menu_use_real_egui_focus() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        frame(&mut s, &mut rt, &ctx, 0.02, vec![]);
        click(&mut s, &mut rt, &ctx, "cp-hold", 0.1);
        frame(&mut s, &mut rt, &ctx, 0.2, vec![key(Key::Space, true)]);
        frame(&mut s, &mut rt, &ctx, 0.5, vec![key(Key::Space, false)]);
        assert!(!s.archived);
        assert_eq!(s.hold_progress, 0.0);
        frame(&mut s, &mut rt, &ctx, 0.6, vec![key(Key::Enter, true)]);
        frame(&mut s, &mut rt, &ctx, 2.11, vec![]);
        assert!(s.archived);
        frame(&mut s, &mut rt, &ctx, 2.12, vec![key(Key::Enter, false)]);
        click(&mut s, &mut rt, &ctx, "cp-tab-0", 2.2);
        frame(&mut s, &mut rt, &ctx, 2.3, vec![key(Key::ArrowRight, true)]);
        assert_eq!(s.tab, 1);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            2.31,
            vec![key(Key::ArrowRight, false)],
        );
        frame(&mut s, &mut rt, &ctx, 2.32, vec![key(Key::Tab, true)]);
        frame(&mut s, &mut rt, &ctx, 2.33, vec![key(Key::Tab, false)]);
        assert!(
            rt.records
                .iter()
                .any(|r| r.id == "cp-tab-panel" && r.focused)
        );
        click(&mut s, &mut rt, &ctx, "cp-tool-0", 2.4);
        assert!(
            ctx.memory(|m| m.focused()).is_some(),
            "toolbar click did not acquire focus"
        );
        frame(&mut s, &mut rt, &ctx, 2.5, vec![key(Key::ArrowRight, true)]);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            2.51,
            vec![key(Key::ArrowRight, false)],
        );
        assert_eq!(s.tool, 0);
        assert!(
            rt.records.iter().any(|r| r.id == "cp-tool-1" && r.focused),
            "roving={}, focused={:?}",
            s.tool_focus,
            rt.records
                .iter()
                .filter(|r| r.focused)
                .map(|r| r.id.clone())
                .collect::<Vec<_>>()
        );
        click(&mut s, &mut rt, &ctx, "cp-menu-trigger", 2.6);
        assert!(s.menu_open);
        frame(&mut s, &mut rt, &ctx, 2.7, vec![]);
        click(&mut s, &mut rt, &ctx, "cp-choice-1", 2.8);
        assert_eq!(s.insert, 1);
        assert!(!s.menu_open);
        click(&mut s, &mut rt, &ctx, "cp-canvas-0", 3.0);
        frame(&mut s, &mut rt, &ctx, 3.1, vec![key(Key::ArrowRight, true)]);
        assert!(s.tinted);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            3.11,
            vec![key(Key::ArrowRight, false)],
        );
        click(&mut s, &mut rt, &ctx, "cp-all-layers", 3.2);
        assert_eq!(s.layers, [true; 3]);
        click(&mut s, &mut rt, &ctx, "cp-chip-remove-0", 3.4);
        assert_eq!(s.chips[0], None);
        frame(&mut s, &mut rt, &ctx, 3.5, vec![]);
        assert!(rt.records.iter().any(|r| r.id == "cp-chip-1" && r.focused));
        click(&mut s, &mut rt, &ctx, "button:Restore labels", 3.6);
        assert_eq!(s.chip_selection(), vec!["Interface", "Motion"]);
    }
    #[test]
    fn accesskit_menu_arrows_activate_immediately_and_dismiss_without_stale_focus() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        frame(&mut s, &mut rt, &ctx, 0.02, vec![]);
        click(&mut s, &mut rt, &ctx, "cp-menu-trigger", 0.1);
        frame(&mut s, &mut rt, &ctx, 0.2, vec![key(Key::ArrowDown, true)]);
        assert_eq!(ctx.memory(|m| m.focused()), s.menu_ids[1]);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            0.21,
            vec![key(Key::ArrowDown, false), key(Key::Enter, true)],
        );
        assert_eq!(s.insert, 1);
        assert!(!s.menu_open);
        frame(&mut s, &mut rt, &ctx, 0.22, vec![key(Key::Enter, false)]);
        assert!(
            rt.records
                .iter()
                .any(|r| r.id == "cp-menu-trigger" && r.focused)
        );
        click(&mut s, &mut rt, &ctx, "cp-menu-trigger", 0.3);
        frame(&mut s, &mut rt, &ctx, 0.4, vec![key(Key::Escape, true)]);
        assert!(!s.menu_open);
        frame(&mut s, &mut rt, &ctx, 0.41, vec![key(Key::Escape, false)]);
        assert!(
            rt.records
                .iter()
                .any(|r| r.id == "cp-menu-trigger" && r.focused)
        );
        click(&mut s, &mut rt, &ctx, "cp-menu-trigger", 0.5);
        let pos = Pos2::new(2.0, 2.0);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            0.6,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame(
            &mut s,
            &mut rt,
            &ctx,
            0.61,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(!s.menu_open);
        frame(&mut s, &mut rt, &ctx, 0.7, vec![]);
        let focus = ctx.memory(|m| m.focused());
        assert!(focus.is_none_or(|id| !s.menu_ids.contains(&Some(id))));
    }
    #[test]
    fn clipboard_failure_cannot_be_shown_as_success() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        let (send, receive) = mpsc::channel();
        s.copy = Some(receive);
        send.send(Err("permission denied".into())).unwrap();
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        assert_eq!(s.copy_result, Some(false));
        assert!(s.copy_note.contains("permission denied"));
    }
    #[test]
    fn native_page_tessellates_in_four_appearances_and_narrow_width() {
        for (dark, mono) in [(false, false), (false, true), (true, false), (true, true)] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut prefs = Preferences::default();
            prefs.dark = dark;
            prefs.mono = mono;
            let mut rt = Runtime::new(ctx.clone(), prefs);
            let mut s = State::default();
            s.enter(&mut rt);
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 14000.0))),
                    time: Some(0.0),
                    ..Default::default()
                },
                |ctx| {
                    rt.begin_frame();
                    egui::CentralPanel::default().show(ctx, |ui| s.show(ui, &mut rt));
                },
            );
            let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
            assert!(!primitives.is_empty());
            assert!(rt.records.iter().any(|r| r.id == "cp-person-3"));
            for record in &rt.records {
                assert!(
                    record.rect[0] >= -1.0 && record.rect[2] <= 391.0,
                    "{} exceeds narrow window: {:?}",
                    record.id,
                    record.rect
                );
            }

            assert!(
                rt.records
                    .iter()
                    .all(|r| r.rect.iter().all(|n| n.is_finite()))
            );
        }
    }
}

fn sized_button(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    label: &str,
    height: f32,
) -> Response {
    let id = ui.make_persistent_id(id);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(height + 24.0, height), Sense::hover());
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("Add {label} block"),
        )
    });
    let p = rt.animate(
        (id, "press"),
        if response.is_pointer_button_down_on() {
            1.0
        } else {
            0.0
        },
        0.08,
        Curve::Out,
    );
    let paint = rect.shrink(p * 0.8);
    ui.painter().rect(
        paint,
        6.0 * rt.tokens.radius,
        if response.hovered() {
            rt.tokens.surface2
        } else {
            rt.tokens.surface
        },
        Stroke::new(
            1.0_f32,
            if response.hovered() {
                rt.tokens.border_strong
            } else {
                rt.tokens.border
            },
        ),
        StrokeKind::Inside,
    );
    icons::paint(
        ui.painter(),
        "plus",
        Rect::from_center_size(
            paint.left_center() + Vec2::new(14.0, 0.0),
            Vec2::splat(13.0),
        ),
        rt.tokens.text2,
        1.5,
        rt.icon_progress((id, "icon"), response.hovered() || response.has_focus()),
    );
    ui.painter().text(
        paint.center() + Vec2::new(7.0, 0.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        rt.tokens.text,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            8.0 * rt.tokens.radius,
            Stroke::new(2.0_f32, rt.tokens.accent_ring),
            StrokeKind::Outside,
        );
    }
    response
}

fn lock_keys(ui: &mut egui::Ui, response: &Response, horizontal: bool, vertical: bool) {
    if response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: horizontal,
                    vertical_arrows: vertical,
                    ..Default::default()
                },
            )
        });
    }
}

fn toolbar_button(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    icon: &str,
    label: &str,
    focusable: bool,
    selected: bool,
) -> Response {
    let id = ui.make_persistent_id(id);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
    let response = ui.interact(
        rect,
        id,
        if focusable {
            Sense::click()
        } else {
            Sense::CLICK
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, label)
    });
    let progress = rt.icon_progress((id, "icon"), response.hovered() || response.has_focus());
    icons::paint(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
        if selected {
            rt.tokens.accent_text
        } else {
            rt.tokens.text2
        },
        1.6,
        progress,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            5.0 * rt.tokens.radius,
            Stroke::new(2.0_f32, rt.tokens.accent_ring),
            StrokeKind::Inside,
        );
    }
    response.on_hover_text(label)
}
fn radio_pair(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    labels: [&str; 2],
    selected: &mut bool,
) {
    let mut next = None;
    let mut responses = Vec::new();
    ui.horizontal(|ui| {
        for (i, label) in labels.iter().copied().enumerate() {
            let yes = *selected == (i == 1);
            let w = ui
                .painter()
                .layout_no_wrap(label.into(), FontId::proportional(13.0), rt.tokens.text)
                .size()
                .x
                + 30.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 28.0), Sense::hover());
            let r = ui.interact(
                rect,
                ui.make_persistent_id((id, i)),
                if yes { Sense::click() } else { Sense::CLICK },
            );
            r.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, yes, label)
            });
            if r.clicked() {
                *selected = i == 1;
                r.request_focus();
            }
            lock_keys(ui, &r, true, true);
            if r.has_focus() {
                if let Some(n) = roving_key(ui, i, 2) {
                    *selected = n == 1;
                    next = Some(n);
                } else if ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::NONE, Key::ArrowDown)
                        || input.consume_key(egui::Modifiers::NONE, Key::ArrowUp)
                }) {
                    *selected = !*selected;
                    next = Some(1 - i);
                    ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                }
            }
            let center = rect.left_center() + Vec2::new(9.0, 0.0);
            ui.painter().circle_stroke(
                center,
                7.0,
                Stroke::new(
                    1.0_f32,
                    if yes {
                        rt.tokens.accent
                    } else {
                        rt.tokens.border_strong
                    },
                ),
            );
            let p = rt.animate(
                (id, i, "selected"),
                if yes { 1.0 } else { 0.0 },
                0.32,
                Curve::SpringSnappy,
            );
            ui.painter()
                .circle_filled(center, 4.0 * p.max(0.0), rt.tokens.accent);
            ui.painter().text(
                rect.left_center() + Vec2::new(25.0, 0.0),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(13.0),
                rt.tokens.text,
            );
            if r.has_focus() {
                ui.painter().circle_stroke(
                    center,
                    10.0,
                    Stroke::new(2.0_f32, rt.tokens.accent_ring),
                );
            }
            rt.record(&format!("{id}-{i}"), &r);
            responses.push(r);
        }
    });
    if let Some(next) = next {
        responses[next].request_focus();
    }
}
