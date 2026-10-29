//! Native forms, local validation, credentials, file metadata and retained profile state.
use crate::{
    icons,
    motion::{self, Curve},
    runtime::{Page, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{
    self, Align2, FontId, Id, Key, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Vec2,
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

const ZONES: [(&str, &str); 6] = [
    ("Berlin", "Europe/Berlin"),
    ("London", "Europe/London"),
    ("New York", "America/New_York"),
    ("San Francisco", "America/Los_Angeles"),
    ("Singapore", "Asia/Singapore"),
    ("Tokyo", "Asia/Tokyo"),
];
const ROLES: [&str; 6] = [
    "",
    "Designer",
    "Engineer",
    "Product manager",
    "Researcher",
    "Other",
];
#[derive(Clone, Debug, PartialEq)]
struct Profile {
    name: String,
    email: String,
    handle: String,
    role: String,
    site: String,
    bio: String,
    public: bool,
    updates: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Alex Morgan".into(),
            email: "alex@example.com".into(),
            handle: "alexmorgan".into(),
            role: "Designer".into(),
            site: String::new(),
            bio: "Making thoughtful tools for people who care about the details.".into(),
            public: false,
            updates: true,
        }
    }
}
impl Profile {
    fn error(&self, index: usize) -> Option<&'static str> {
        match index {
            0 => (!(2..=64).contains(&utf16_len(self.name.trim())))
                .then_some("Enter a display name with 2–64 characters."),
            1 => email_error(&self.email),
            2 => (!(3..=24).contains(&self.handle.len())
                || !self
                    .handle
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_'))
            .then_some("Use 3–24 letters, numbers, or underscores; no spaces."),
            3 => self
                .role
                .trim()
                .is_empty()
                .then_some("Choose a role for the demo profile."),
            4 => (!self.site.trim().is_empty() && !http_url_valid(&self.site))
                .then_some("Use a complete http:// or https:// address."),
            5 => {
                (utf16_len(&self.bio) > 160).then_some("Keep your bio to 160 characters or fewer.")
            }
            _ => None,
        }
    }
    fn errors(&self) -> Vec<(usize, &'static str)> {
        (0..6)
            .filter_map(|i| self.error(i).map(|e| (i, e)))
            .collect()
    }
    fn initials(&self) -> String {
        let result = self
            .name
            .split_whitespace()
            .take(2)
            .filter_map(|s| s.chars().next())
            .collect::<String>()
            .to_uppercase();
        if result.is_empty() {
            "?".into()
        } else {
            result
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct LocalFile {
    name: String,
    size: u64,
}
#[derive(Clone, Copy, Debug)]
struct Simulation {
    start: f64,
    from: f32,
    duration: f64,
}
#[derive(Clone, Debug)]
struct Notice {
    text: String,
    tone: &'static str,
}
impl Notice {
    fn new(text: impl Into<String>, tone: &'static str) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
}
#[derive(Clone, Debug)]
struct Otp {
    digits: [String; 6],
    result: Option<bool>,
    notice: Notice,
    focus: Option<usize>,
}
impl Default for Otp {
    fn default() -> Self {
        Self {
            digits: Default::default(),
            result: None,
            notice: Notice::new("Ready for a six-digit sample.", ""),
            focus: None,
        }
    }
}
impl Otp {
    fn editing(&mut self) {
        self.result = None;
        self.notice = Notice::new(
            if self.digits.iter().all(|d| !d.is_empty()) {
                "Six digits entered. Choose Verify demo code."
            } else {
                "Enter the six-digit demo code. Use 123456."
            },
            "",
        );
    }
    fn distribute(&mut self, text: &str, start: usize) -> bool {
        let clean: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect();
        if clean.is_empty() || clean.len() > 6 || !clean.bytes().all(|b| b.is_ascii_digit()) {
            self.notice = Notice::new("Paste up to six digits. The demo code is 123456.", "error");
            return false;
        }
        let offset = if clean.len() == 6 { 0 } else { start.min(5) };
        for (i, c) in clean.chars().take(6 - offset).enumerate() {
            self.digits[offset + i] = c.to_string();
        }
        self.editing();
        self.focus = Some((offset + clean.len()).min(5));
        true
    }
    fn input(&mut self, index: usize) {
        let value = self.digits[index].clone();
        if value.chars().count() > 1 {
            if !self.distribute(&value, index) {
                self.digits[index] = value
                    .chars()
                    .find(char::is_ascii_digit)
                    .map(|c| c.to_string())
                    .unwrap_or_default();
            }
            return;
        }
        self.digits[index].retain(|c| c.is_ascii_digit());
        self.editing();
        if !self.digits[index].is_empty() && index < 5 {
            self.focus = Some(index + 1);
        }
    }
    fn backspace(&mut self, index: usize) {
        if index > 0 && self.digits[index].is_empty() {
            self.digits[index - 1].clear();
            self.focus = Some(index - 1);
            self.editing();
        }
    }
    fn verify(&mut self) -> bool {
        let code = self.digits.concat();
        let success = code == "123456";
        self.result = Some(success);
        self.notice = Notice::new(
            if success {
                "Demo code matches 123456. No account was verified."
            } else if code.len() < 6 {
                "Enter all six digits before checking the demo code."
            } else {
                "That code does not match. Try 123456 for this local demo."
            },
            if success { "success" } else { "error" },
        );
        if !success {
            self.focus = Some(self.digits.iter().position(String::is_empty).unwrap_or(0));
        }
        success
    }
}

pub struct State {
    compact: String,
    standard: String,
    large: String,
    floating: String,
    ready: String,
    valid: String,
    invalid: String,
    email: String,
    email_touched: bool,
    email_status: Notice,
    note: String,
    note_threshold: u8,
    note_announcement: String,
    password: String,
    reveal: bool,
    otp: Otp,
    zone: usize,
    zone_note: String,
    typeahead: String,
    typeahead_at: f64,
    seats: i32,
    seats_text: String,
    seats_error: String,
    seats_note: String,
    volume: i32,
    low: i32,
    high: i32,
    drag_bound: Option<bool>,
    files: Vec<LocalFile>,
    selection_count: usize,
    progress: f32,
    simulation: Option<Simulation>,
    upload_status: Notice,
    file_picker: Option<Receiver<Option<Vec<PathBuf>>>>,
    profile: Profile,
    saved: Profile,
    saved_once: bool,
    touched: [bool; 6],
    submitted: bool,
    profile_status: Notice,
    profile_focus: Option<usize>,
    profile_ids: [Option<Id>; 6],
    flash: Option<(f64, bool, &'static str)>,
    active: bool,
}
impl Default for State {
    fn default() -> Self {
        let profile = Profile::default();
        Self {
            compact: String::new(),
            standard: "Alex Morgan".into(),
            large: String::new(),
            floating: String::new(),
            ready: "Untitled project".into(),
            valid: "alex@example.com".into(),
            invalid: "alex@".into(),
            email: String::new(),
            email_touched: false,
            email_status: Notice::new("", ""),
            note: "Review the updated navigation before release.".into(),
            note_threshold: 0,
            note_announcement: String::new(),
            password: String::new(),
            reveal: false,
            otp: Otp::default(),
            zone: 0,
            zone_note: String::new(),
            typeahead: String::new(),
            typeahead_at: 0.0,
            seats: 4,
            seats_text: "4".into(),
            seats_error: String::new(),
            seats_note: String::new(),
            volume: 65,
            low: 20,
            high: 80,
            drag_bound: None,
            files: Vec::new(),
            selection_count: 0,
            progress: 0.0,
            simulation: None,
            upload_status: Notice::new("Choose files to try the local progress simulation.", ""),
            file_picker: None,
            profile: profile.clone(),
            saved: profile,
            saved_once: false,
            touched: [false; 6],
            submitted: false,
            profile_status: Notice::new("Edit a field to try validation and the live preview.", ""),
            profile_focus: None,
            profile_ids: [None; 6],
            flash: None,
            active: false,
        }
    }
}
impl State {
    fn dirty(&self) -> bool {
        self.profile != self.saved
    }
    fn save_profile(&mut self) -> bool {
        self.submitted = true;
        let errors = self.profile.errors();
        if let Some((index, _)) = errors.first() {
            self.profile_focus = Some(*index);
            self.profile_status =
                Notice::new("Demo save stopped. Correct the marked fields.", "error");
            false
        } else {
            self.saved = self.profile.clone();
            self.saved_once = true;
            self.submitted = false;
            self.profile_status = Notice::new(
                "Demo profile saved in memory for this visit. Nothing sent, published, or written to disk.",
                "success",
            );
            true
        }
    }
    fn reset_profile(&mut self) {
        self.profile = self.saved.clone();
        self.touched = [false; 6];
        self.submitted = false;
        self.profile_focus = None;
        self.profile_status = Notice::new(
            if self.saved_once {
                "Restored your last saved demo profile for this visit."
            } else {
                "Restored the example profile. Nothing sent or saved to disk."
            },
            "",
        );
    }
    fn profile_edited(&mut self) {
        self.profile_status = Notice::new(
            if self.dirty() {
                "Unsaved local changes. Save the demo profile when ready."
            } else {
                "Matches the current demo profile. Nothing sent."
            },
            "",
        );
    }
    fn update_note(&mut self) {
        limit_utf16(&mut self.note, 240);
        let remaining = 240 - utf16_len(&self.note);
        let threshold = if remaining == 0 {
            2
        } else if remaining <= 24 {
            1
        } else {
            0
        };
        if threshold != self.note_threshold {
            self.note_announcement = match threshold {
                2 => "Character limit reached.".into(),
                1 => format!("{remaining} characters remaining."),
                _ => "More than 24 characters available.".into(),
            };
            self.note_threshold = threshold;
        }
    }
    fn set_seats(&mut self, n: i32, announce: bool) {
        self.seats = n.clamp(1, 25);
        self.seats_text = self.seats.to_string();
        self.seats_error.clear();
        if announce {
            self.seats_note = format!(
                "{} demo team {}. No purchase made.",
                self.seats,
                if self.seats == 1 { "seat" } else { "seats" }
            );
        }
    }
    fn edit_seats(&mut self) {
        if let Ok(n) = self.seats_text.parse::<f64>()
            && n.is_finite()
            && n.fract() == 0.0
            && (1.0..=25.0).contains(&n)
        {
            self.set_seats(n as i32, true);
            return;
        }
        self.seats_error = "Enter a whole number from 1 to 25.".into();
    }
    fn bounds(&mut self, high_changed: bool) {
        self.low = self.low.clamp(0, 90);
        self.high = self.high.clamp(10, 100);
        if self.high - self.low < 10 {
            if high_changed {
                self.high = self.low + 10;
            } else {
                self.low = self.high - 10;
            }
        }
    }
    fn select_files(&mut self, incoming: Vec<LocalFile>) {
        self.simulation = None;
        self.selection_count = incoming.len();
        self.files = incoming
            .into_iter()
            .filter(|f| f.size <= 10 * 1_048_576)
            .take(4)
            .collect();
        self.progress = 0.0;
        let count = self.files.len();
        let skipped = self.selection_count - count;
        let message = if count == 0 {
            "No eligible files selected.".into()
        } else {
            format!(
                "{count} local {} selected. Ready to simulate progress.",
                if count == 1 { "file" } else { "files" }
            )
        };
        self.upload_status = Notice::new(
            format!(
                "{message}{} Nothing uploaded.",
                if skipped > 0 {
                    format!(" {skipped} skipped: maximum 4 files, 10 MiB per file.")
                } else {
                    String::new()
                }
            ),
            if skipped > 0 { "error" } else { "" },
        );
    }
    fn select_paths(&mut self, paths: Vec<PathBuf>) {
        let mut metadata_errors = 0;
        let incoming = paths
            .into_iter()
            .filter_map(|path| {
                let meta = path.metadata().ok();
                if let Some(meta) = meta
                    && meta.is_file()
                {
                    return Some(LocalFile {
                        name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                        size: meta.len(),
                    });
                }
                metadata_errors += 1;
                None
            })
            .collect();
        self.select_files(incoming);
        if metadata_errors > 0 {
            self.selection_count += metadata_errors;
            self.upload_status
                .text
                .push_str(&format!(" {metadata_errors} unavailable files skipped."));
            self.upload_status.tone = "error";
        }
    }
    fn pause(&mut self, now: f64, message: &str) {
        self.tick_simulation(now);
        if self.simulation.take().is_some() {
            self.upload_status = Notice::new(message, "");
        }
    }
    fn start_simulation(&mut self, now: f64) {
        if self.files.is_empty() || !self.active {
            return;
        }
        if self.progress >= 1.0 {
            self.progress = 0.0;
        }
        self.simulation = Some(Simulation {
            start: now,
            from: self.progress,
            duration: 2.6 * (1.0 - self.progress as f64),
        });
        self.upload_status = Notice::new(
            "Simulating progress locally. No upload is taking place.",
            "",
        );
    }
    fn tick_simulation(&mut self, now: f64) {
        if let Some(sim) = self.simulation {
            let p = ((now - sim.start) / sim.duration.max(0.001)).clamp(0.0, 1.0) as f32;
            self.progress = sim.from + (1.0 - sim.from) * motion::ease(Curve::InOut, p);
            if p >= 1.0 {
                self.progress = 1.0;
                self.simulation = None;
                self.upload_status = Notice::new(
                    "Simulation complete. Files remain on your device; nothing was uploaded.",
                    "success",
                );
            }
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
    fn sizes(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-0",
            "Input sizes",
            "Three sizes, a leading icon and a floating label that makes room for your answer.",
            280.0,
            |ui, rt| {
                for (key, label, icon, height, value) in [
                    ("compact", "Compact   28", "search", 28.0, &mut self.compact),
                    (
                        "standard",
                        "Comfortable   40",
                        "user",
                        40.0,
                        &mut self.standard,
                    ),
                    ("large", "Spacious   48", "mail", 48.0, &mut self.large),
                ] {
                    ui.label(label);
                    let r = ui
                        .horizontal(|ui| {
                            paint_icon(ui, rt, icon, 16.0);
                            ui.add_sized(
                                Vec2::new(ui.available_width(), height),
                                egui::TextEdit::singleline(value)
                                    .id(Id::new(("forms", key)))
                                    .hint_text(match key {
                                        "compact" => "Find a component",
                                        "large" => "you@example.com",
                                        _ => "",
                                    }),
                            )
                        })
                        .inner;
                    rt.record(&format!("forms-{key}"), &r);
                }
                let focused = ui.memory(|m| m.has_focus(Id::new("forms-floating")));
                let raised = !self.floating.is_empty() || focused;
                let p = rt.animate(
                    "forms-floating-label",
                    if raised { 1.0 } else { 0.0 },
                    0.2,
                    Curve::Out,
                );
                let output = egui::TextEdit::singleline(&mut self.floating)
                    .id(Id::new("forms-floating"))
                    .desired_width(f32::INFINITY)
                    .min_size(Vec2::new(0.0, 40.0))
                    .show(ui);
                let rect = output.response.rect;
                ui.painter().text(
                    rect.left_top() + Vec2::new(8.0, 14.0 - 18.0 * p),
                    Align2::LEFT_TOP,
                    "Project name",
                    FontId::proportional(13.0 - 2.0 * p),
                    rt.tokens.text2,
                );
                rt.record("forms-floating", &output.response);
            },
        );
    }
    fn states(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-1",
            "States with something to say",
            "Success and errors use words as well as colour. Read-only content remains selectable.",
            280.0,
            |ui, rt| {
                edit(
                    ui,
                    rt,
                    "forms-ready",
                    "Ready to edit",
                    &mut self.ready,
                    None,
                    "Untitled project",
                );
                let valid_error = email_error(&self.valid);
                edit(
                    ui,
                    rt,
                    "forms-valid",
                    if valid_error.is_some() {
                        "Format checked · Error example"
                    } else {
                        "Format checked · Valid example"
                    },
                    &mut self.valid,
                    valid_error,
                    "",
                );
                if valid_error.is_none() {
                    status(
                        ui,
                        rt,
                        &Notice::new("Format looks valid. This is a local example.", "success"),
                    );
                }
                let invalid_error = email_error(&self.invalid);
                edit(
                    ui,
                    rt,
                    "forms-invalid",
                    if invalid_error.is_some() {
                        "Needs attention · Error example"
                    } else {
                        "Format checked · Valid example"
                    },
                    &mut self.invalid,
                    invalid_error,
                    "",
                );
                ui.label("Read only");
                let mut read_only = "SCHEMATIC-001";
                let r = ui.add(
                    egui::TextEdit::singleline(&mut read_only)
                        .id(Id::new("forms-readonly"))
                        .desired_width(f32::INFINITY),
                );
                rt.record("forms-readonly", &r);
                ui.label("Unavailable");
                ui.add_enabled(false, egui::TextEdit::singleline(&mut "Managed by team"));
            },
        );
    }
    fn validation(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-2",
            "Validation, in context",
            "Try an empty submission or an incomplete address. Passing checks format; it does not verify a mailbox.",
            190.0,
            |ui, rt| {
                let error = if self.email_touched || !self.email.is_empty() {
                    email_error(&self.email)
                } else {
                    None
                };
                let r = edit(
                    ui,
                    rt,
                    "forms-email",
                    "Email address · Required",
                    &mut self.email,
                    error,
                    "you@example.com",
                );
                if r.changed() {
                    self.email_status = Notice::new("", "");
                }
                if r.lost_focus() {
                    self.email_touched = true;
                }
                if error.is_none() {
                    ui::muted(
                        ui,
                        rt,
                        if self.email.is_empty() {
                            "Use a sample address. Checked locally as you type."
                        } else {
                            "Format looks valid. Mailbox delivery has not been checked."
                        },
                    );
                }
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                if ui::button(
                    ui,
                    rt,
                    "forms-email-submit",
                    "Try demo submit",
                    Some("arrow-right"),
                    ButtonKind::Primary,
                )
                .clicked()
                    || enter
                {
                    self.email_touched = true;
                    let error = email_error(&self.email);
                    self.email_status = Notice::new(
                        if error.is_some() {
                            "Demo submit stopped. Correct the email address above."
                        } else {
                            "Demo submit accepted: the format is valid. No email was sent."
                        },
                        if error.is_some() { "error" } else { "success" },
                    );
                    if error.is_some() {
                        r.request_focus();
                    }
                    self.flash = Some((rt.now, error.is_some(), "email"));
                }
                status(ui, rt, &self.email_status);
                self.flash_overlay(ui, rt, "email");
            },
        );
    }
    fn note(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-3",
            "Auto-growing textarea",
            "The textarea grows with your words. A live counter keeps the limit in view.",
            190.0,
            |ui, rt| {
                ui.label("A note for your team · Optional");
                let r = growing_text(ui, rt, "forms-note", &mut self.note, 240);
                rt.record("forms-note", &r);
                if r.changed() {
                    self.update_note();
                }
                ui.horizontal(|ui| {
                    ui::muted(ui, rt, "Grows as you type · 240 character limit");
                    ui.colored_label(
                        if utf16_len(&self.note) >= 216 {
                            rt.tokens.warning_text
                        } else {
                            rt.tokens.text2
                        },
                        format!("{} / 240", utf16_len(&self.note)),
                    );
                });
                if !self.note_announcement.is_empty() {
                    ui::muted(ui, rt, &self.note_announcement);
                }
                ui::muted(ui, rt, "240 character limit.");
            },
        );
    }
    fn password(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-4",
            "Show the requirements",
            "A local rules meter, not a guarantee of password security. Use a sample password.",
            275.0,
            |ui, rt| {
                ui.label("Sample password");
                ui.horizontal(|ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .id(Id::new("forms-password"))
                            .password(!self.reveal)
                            .margin(Vec2::new(10.0, 10.0))
                            .min_size(Vec2::new(0.0, 40.0))
                            .hint_text("Build a sample password")
                            .desired_width((ui.available_width() - 100.0).max(40.0)),
                    );
                    rt.record("forms-password", &r);
                    if ui::button(
                        ui,
                        rt,
                        "forms-reveal",
                        if self.reveal { "Hide" } else { "Show" },
                        Some("eye"),
                        ButtonKind::Ghost,
                    )
                    .clicked()
                    {
                        self.reveal = !self.reveal;
                    }
                });
                ui::muted(
                    ui,
                    rt,
                    "Checked in memory only. Never included in a demo save.",
                );
                let checks = password_rules(&self.password);
                let score = checks.iter().filter(|c| **c).count();
                let (r, response) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 7.0), Sense::hover());
                response.widget_info(|| {
                    egui::WidgetInfo::slider(
                        true,
                        score as f64,
                        "Password demo rules met, out of four",
                    )
                });
                for i in 0..4 {
                    let p = rt.animate(
                        ("forms-strength", i),
                        if i < score { 1.0 } else { 0.0 },
                        0.22,
                        Curve::Out,
                    );
                    let segment = Rect::from_min_size(
                        r.min + Vec2::new(i as f32 * (r.width() + 5.0) / 4.0, 0.0),
                        Vec2::new((r.width() - 15.0) / 4.0, 7.0),
                    );
                    ui.painter().rect_filled(segment, 3.0, rt.tokens.surface3);
                    ui.painter().rect_filled(
                        segment,
                        3.0,
                        [
                            rt.tokens.danger,
                            rt.tokens.warning,
                            rt.tokens.accent,
                            rt.tokens.success,
                        ][score.saturating_sub(1)]
                        .linear_multiply(p),
                    );
                }
                ui.label(format!(
                    "{} · {score} of 4 demo rules met",
                    if self.password.is_empty() {
                        "Not started"
                    } else {
                        [
                            "Needs work",
                            "A start",
                            "Getting there",
                            "Almost there",
                            "Meets the demo rules",
                        ][score]
                    }
                ));
                for (i, label) in [
                    "At least 12 characters",
                    "Uppercase and lowercase letters",
                    "At least one number",
                    "At least one symbol",
                ]
                .iter()
                .enumerate()
                {
                    ui.horizontal(|ui| {
                        paint_icon(ui, rt, if checks[i] { "check" } else { "minus" }, 12.0);
                        ui.colored_label(
                            if checks[i] {
                                rt.tokens.success_text
                            } else {
                                rt.tokens.text2
                            },
                            *label,
                        );
                        ui.small(if checks[i] { "Met" } else { "Needed" });
                    });
                }
            },
        );
    }
    fn otp(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-5",
            "One code, six small steps",
            "Paste a whole code, review with arrows or Backspace through empty slots.",
            275.0,
            |ui, rt| {
                ui.strong("Six-digit demo code");
                ui::muted(
                    ui,
                    rt,
                    "Use 123456. This is a local comparison, not account verification.",
                );
                let focused =
                    (0..6).find(|i| ui.memory(|m| m.has_focus(Id::new(("forms-otp", *i)))));
                if let Some(index) = focused {
                    let pasted = ui.input_mut(|i| {
                        let texts: Vec<String> = i
                            .events
                            .iter()
                            .filter_map(|e| {
                                if let egui::Event::Paste(s) = e {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        i.events.retain(|e| !matches!(e, egui::Event::Paste(_)));
                        texts
                    });
                    for text in pasted {
                        self.otp.distribute(&text, index);
                    }
                    let back = ui.input_mut(|i| {
                        if self.otp.digits[index].is_empty() {
                            i.consume_key(egui::Modifiers::NONE, Key::Backspace)
                        } else {
                            false
                        }
                    });
                    if back {
                        self.otp.backspace(index);
                    }
                    let arrow = ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft) {
                            Some(index.saturating_sub(1))
                        } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight) {
                            Some((index + 1).min(5))
                        } else {
                            None
                        }
                    });
                    if arrow.is_some() {
                        self.otp.focus = arrow;
                    }
                }
                ui.horizontal(|ui| {
                    let width = ((ui.available_width() - 5.0 * ui.spacing().item_spacing.x) / 6.0
                        - 8.0)
                        .clamp(16.0, 44.0);
                    for index in 0..6 {
                        let id = Id::new(("forms-otp", index));
                        let output = egui::TextEdit::singleline(&mut self.otp.digits[index])
                            .id(id)
                            .desired_width(width)
                            .font(FontId::monospace(22.0))
                            .horizontal_align(egui::Align::Center)
                            .show(ui);
                        let r = output.response;
                        rt.record(&format!("forms-otp-{}", index + 1), &r);
                        r.widget_info(|| {
                            egui::WidgetInfo::text_edit(
                                true,
                                "",
                                &self.otp.digits[index],
                                format!("Digit {} of 6", index + 1),
                            )
                        });
                        if self.otp.focus == Some(index) {
                            r.request_focus();
                            self.otp.focus = None;
                            select_all(ui.ctx(), id, &self.otp.digits[index]);
                        }
                        if r.gained_focus() {
                            select_all(ui.ctx(), id, &self.otp.digits[index]);
                        }
                        if r.changed() {
                            self.otp.input(index);
                        }
                    }
                });
                if let Some(index) = self.otp.focus {
                    ui.memory_mut(|m| m.request_focus(Id::new(("forms-otp", index))));
                    rt.repaint();
                }
                let enter = focused.is_some() && ui.input(|i| i.key_pressed(Key::Enter));
                let verify = ui::button(
                    ui,
                    rt,
                    "forms-otp-verify",
                    "Verify demo code",
                    Some("check"),
                    ButtonKind::Primary,
                );
                rt.record("forms-otp-verify", &verify);
                if verify.clicked() || enter {
                    let success = self.otp.verify();
                    self.flash = Some((rt.now, !success, "otp"));
                    if let Some(index) = self.otp.focus {
                        ui.memory_mut(|m| m.request_focus(Id::new(("forms-otp", index))));
                    }
                }
                status(ui, rt, &self.otp.notice);
                ui::muted(
                    ui,
                    rt,
                    "Left / Right: review digits · Paste all six at once",
                );
                self.flash_overlay(ui, rt, "otp");
            },
        );
    }
    fn timezone(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-6",
            "Let the platform help",
            "Choose with arrows, type an option’s first letters and confirm with Enter.",
            185.0,
            |ui, rt| {
                ui.label("Workspace time zone");
                let before = self.zone;
                let result = egui::ComboBox::from_id_salt("forms-timezone")
                    .selected_text(format!("{} · {}", ZONES[self.zone].0, ZONES[self.zone].1))
                    .show_ui(ui, |ui| {
                        for (i, (name, zone)) in ZONES.iter().enumerate() {
                            ui.selectable_value(&mut self.zone, i, format!("{name} · {zone}"));
                        }
                    });
                rt.record("forms-timezone", &result.response);
                lock_keys(ui, &result.response, false, true);
                let engaged = result.response.has_focus() || result.inner.is_some();
                if engaged {
                    let typed = ui.input(|i| {
                        i.events
                            .iter()
                            .filter_map(|e| {
                                if let egui::Event::Text(s) = e {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<String>()
                    });
                    if !typed.is_empty() {
                        if rt.now - self.typeahead_at > 0.8 {
                            self.typeahead.clear();
                        }
                        self.typeahead.push_str(&typed.to_lowercase());
                        self.typeahead_at = rt.now;
                        if let Some(index) = ZONES
                            .iter()
                            .position(|(name, _)| name.to_lowercase().starts_with(&self.typeahead))
                        {
                            self.zone = index;
                        }
                    }
                    if result.response.has_focus()
                        && let Some(index) = zone_key(ui, self.zone)
                    {
                        self.zone = index;
                    }
                }
                if before != self.zone {
                    self.zone_note = format!(
                        "Local selection: {} · {}. No system setting changed.",
                        ZONES[self.zone].0, ZONES[self.zone].1
                    );
                    self.flash = Some((rt.now, false, "timezone"));
                }
                ui::muted(ui, rt, "Typeahead: try ‘Tok’ to jump to Tokyo.");
                ui.horizontal(|ui| {
                    paint_icon(ui, rt, "globe", 20.0);
                    ui.vertical(|ui| {
                        ui.small("Selected locally");
                        ui.strong(ZONES[self.zone].0);
                    });
                    ui::badge(ui, rt, "Time zone", "neutral");
                });
                ui::muted(ui, rt, &self.zone_note);
                self.flash_overlay(ui, rt, "timezone");
            },
        );
    }
    fn seats(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-7",
            "Every step accounted for",
            "Enter a whole number or use arrow keys. The readout rolls to the new quantity.",
            185.0,
            |ui, rt| {
                ui.label("Team seats · 1–25");
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled_ui(self.seats > 1, |ui| {
                            ui::icon_button(
                                ui,
                                rt,
                                "forms-seat-minus",
                                "minus",
                                "Remove one team seat",
                            )
                        })
                        .inner
                        .clicked()
                    {
                        self.set_seats(self.seats - 1, true);
                    }
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.seats_text)
                            .id(Id::new("forms-seats"))
                            .margin(Vec2::new(10.0, 10.0))
                            .min_size(Vec2::new(0.0, 40.0))
                            .desired_width(60.0),
                    );
                    rt.record("forms-seats", &r);
                    lock_keys(ui, &r, true, true);
                    if r.changed() {
                        self.edit_seats();
                    }
                    if r.has_focus() {
                        let delta = ui.input_mut(|i| {
                            if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                                1
                            } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                                -1
                            } else {
                                0
                            }
                        });
                        if delta != 0 {
                            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                            self.set_seats(self.seats + delta, true);
                        }
                    }
                    if r.lost_focus() && !self.seats_error.is_empty() {
                        self.set_seats(self.seats, false);
                        self.seats_error =
                            format!("Restored the last valid quantity: {}.", self.seats);
                    }
                    if ui
                        .add_enabled_ui(self.seats < 25, |ui| {
                            ui::icon_button(ui, rt, "forms-seat-plus", "plus", "Add one team seat")
                        })
                        .inner
                        .clicked()
                    {
                        self.set_seats(self.seats + 1, true);
                    }
                });
                ui::muted(
                    ui,
                    rt,
                    "Whole numbers from 1 to 25. Demo only; no purchase.",
                );
                if !self.seats_error.is_empty() {
                    ui.colored_label(rt.tokens.danger_text, &self.seats_error);
                }
                let count = rt.animate("forms-seat-count", self.seats as f32, 0.32, Curve::Out);
                ui.horizontal(|ui| {
                    paint_icon(ui, rt, "users", 22.0);
                    ui.label(egui::RichText::new(format!("{}", count.round() as i32)).size(30.0));
                    ui.vertical(|ui| {
                        ui.strong("seats in your demo team");
                        ui::muted(ui, rt, "A small team. Room to make things.");
                    });
                });
                ui::muted(ui, rt, &self.seats_note);
            },
        );
    }
    fn volume(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-8",
            "A single point of control",
            "A value bubble follows the thumb. Arrows adjust by one; Home and End reach the limits.",
            170.0,
            |ui, rt| {
                ui.label("Preview volume");
                let (bubble, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
                let p = rt.animate(
                    "forms-volume-bubble",
                    self.volume as f32 / 100.0,
                    0.12,
                    Curve::Out,
                );
                let x = bubble.left() + 12.0 + (bubble.width() - 24.0) * p;
                ui.painter().text(
                    Pos2::new(x, bubble.center().y),
                    Align2::CENTER_CENTER,
                    format!("{}%", self.volume),
                    FontId::monospace(12.0),
                    rt.tokens.accent_text,
                );
                let (track_rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
                let r = ui.interact(track_rect, Id::new("forms-volume"), Sense::click_and_drag());
                let track = Rect::from_center_size(
                    track_rect.center(),
                    Vec2::new((track_rect.width() - 24.0).max(1.0), 4.0),
                );
                if r.is_pointer_button_down_on() || r.clicked() || r.dragged() {
                    r.request_focus();
                    if let Some(pos) = r.interact_pointer_pos() {
                        self.volume = ((pos.x - track.left()) / track.width() * 100.0)
                            .round()
                            .clamp(0.0, 100.0) as i32;
                    }
                }
                lock_keys(ui, &r, true, true);
                if r.has_focus()
                    && let Some(value) = slider_key(ui, self.volume, 0, 100)
                {
                    self.volume = value;
                }
                r.widget_info(|| {
                    egui::WidgetInfo::slider(true, self.volume as f64, "Preview volume, percent")
                });
                rt.record("forms-volume", &r);
                ui.painter().rect_filled(track, 2.0, rt.tokens.surface3);
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        track.min,
                        Vec2::new(track.width() * self.volume as f32 / 100.0, 4.0),
                    ),
                    2.0,
                    rt.tokens.accent,
                );
                let thumb = Pos2::new(
                    track.left() + track.width() * self.volume as f32 / 100.0,
                    track.center().y,
                );
                ui.painter()
                    .circle_filled(thumb, 8.0, rt.tokens.surface_raised);
                ui.painter().circle_stroke(
                    thumb,
                    if r.has_focus() { 10.0 } else { 8.0 },
                    Stroke::new(2.0, rt.tokens.accent),
                );
                ui.horizontal(|ui| {
                    ui.small("Silent");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small("Full");
                    });
                });
                let (bars, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), Sense::hover());
                for i in 0..20 {
                    let h = bars.height() * (0.25 + i as f32 * 0.035);
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            bars.left() + i as f32 * bars.width() / 20.0,
                            bars.bottom() - h,
                        ),
                        Vec2::new(bars.width() / 20.0 - 3.0, h),
                    );
                    let active = (i as f32) < (self.volume as f32 / 5.0);
                    let p = rt.animate(
                        ("forms-volume-bar", i),
                        if active { 1.0 } else { 0.0 },
                        0.18,
                        Curve::Out,
                    );
                    ui.painter().rect_filled(
                        rect,
                        2.0,
                        rt.tokens.surface3.lerp_to_gamma(rt.tokens.accent, p),
                    );
                }
                ui::muted(ui, rt, "Changes the demo value only. No audio is played.");
            },
        );
    }
    fn interval(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-9",
            "Two bounds, one interval",
            "The handles cannot cross. Tab to each handle and use arrows for fine adjustments.",
            170.0,
            |ui, rt| {
                ui.strong("Demo intensity window");
                ui::muted(
                    ui,
                    rt,
                    "0–100 units, with at least 10 units between bounds.",
                );
                let (area, track_response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 72.0),
                    Sense::click_and_drag(),
                );
                let track = Rect::from_min_max(
                    area.left_top() + Vec2::new(12.0, 42.0),
                    area.right_top() + Vec2::new(-12.0, 46.0),
                );
                ui.painter().rect_filled(track, 2.0, rt.tokens.surface3);
                let x = |v: i32| track.left() + track.width() * v as f32 / 100.0;
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x(self.low), track.top()),
                        Pos2::new(x(self.high), track.bottom()),
                    ),
                    2.0,
                    rt.tokens.accent,
                );
                if (track_response.drag_started() || track_response.clicked())
                    && let Some(pos) = track_response.interact_pointer_pos()
                {
                    self.drag_bound =
                        Some((pos.x - x(self.high)).abs() < (pos.x - x(self.low)).abs());
                }
                for high in [false, true] {
                    let v = if high { self.high } else { self.low };
                    let center = Pos2::new(x(v), track.center().y);
                    let id = Id::new(if high { "forms-high" } else { "forms-low" });
                    let response = ui.interact(
                        Rect::from_center_size(center, Vec2::splat(26.0)),
                        id,
                        Sense::click_and_drag(),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::slider(
                            true,
                            v as f64,
                            if high {
                                "Upper intensity bound"
                            } else {
                                "Lower intensity bound"
                            },
                        )
                    });
                    rt.record(if high { "forms-high" } else { "forms-low" }, &response);
                    if response.drag_started() || response.is_pointer_button_down_on() {
                        self.drag_bound = Some(high);
                        response.request_focus();
                    }
                    lock_keys(ui, &response, true, true);
                    if response.has_focus() {
                        let next = slider_key(
                            ui,
                            v,
                            if high { self.low + 10 } else { 0 },
                            if high { 100 } else { self.high - 10 },
                        );
                        if let Some(next) = next {
                            if high {
                                self.high = next;
                            } else {
                                self.low = next;
                            }
                            self.bounds(high);
                        }
                    }
                    ui.painter()
                        .circle_filled(center, 8.0, rt.tokens.surface_raised);
                    ui.painter().circle_stroke(
                        center,
                        if response.has_focus() { 10.0 } else { 8.0 },
                        Stroke::new(2.0, rt.tokens.accent),
                    );
                    ui.painter().text(
                        Pos2::new(center.x, area.top() + 15.0),
                        Align2::CENTER_CENTER,
                        v,
                        FontId::monospace(12.0),
                        rt.tokens.accent_text,
                    );
                }
                if let Some(high) = self.drag_bound {
                    if ui.input(|i| i.pointer.primary_down()) {
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let n = ((pos.x - track.left()) / track.width() * 100.0)
                                .round()
                                .clamp(0.0, 100.0) as i32;
                            if high {
                                self.high = n;
                            } else {
                                self.low = n;
                            }
                            self.bounds(high);
                        }
                    } else {
                        self.drag_bound = None;
                    }
                }
                ui.horizontal(|ui| {
                    ui.small("0 units");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small("100 units");
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Selected interval");
                    ui.strong(format!("{}–{} units", self.low, self.high));
                });
            },
        );
    }
    fn files(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::demo(
            ui,
            rt,
            "forms-10",
            "Local files / simulated progress",
            "Only filenames and sizes are used. File contents are never read; no network request is made.",
            290.0,
            |ui, rt| {
                if ui.available_width() > 680.0 {
                    ui.columns(2, |cols| {
                        self.file_dropzone(&mut cols[0], rt);
                        self.file_details(&mut cols[1], rt);
                    });
                } else {
                    self.file_dropzone(ui, rt);
                    ui.add_space(16.0);
                    self.file_details(ui, rt);
                }
            },
        );
    }
    fn file_dropzone(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let hovered = ui.input(|i| !i.raw.hovered_files.is_empty());
        let fill = if hovered {
            rt.tokens.accent_soft
        } else {
            rt.tokens.surface2
        };
        let frame = egui::Frame::new()
            .fill(fill)
            .stroke(Stroke::new(
                1.0,
                if hovered {
                    rt.tokens.accent
                } else {
                    rt.tokens.border_strong
                },
            ))
            .corner_radius(8.0 * rt.tokens.radius)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(192.0);
                ui.horizontal(|ui| {
                    paint_icon(ui, rt, "upload", 26.0);
                    ui.vertical(|ui| {
                        ui.strong("Drop a few files here");
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("Up to 4 files · 10 MiB each · any file type")
                                    .size(12.0)
                                    .color(rt.tokens.text3),
                            )
                            .wrap(),
                        );
                    });
                });
                let r = ui
                    .add_enabled_ui(self.file_picker.is_none(), |ui| {
                        ui::button(
                            ui,
                            rt,
                            "forms-choose-files",
                            if self.file_picker.is_some() {
                                "Choosing local files…"
                            } else {
                                "Or choose local files"
                            },
                            Some("folder"),
                            ButtonKind::Secondary,
                        )
                    })
                    .inner;
                rt.record("forms-choose-files", &r);
                if r.clicked() {
                    let (send, recv) = mpsc::channel();
                    self.file_picker = Some(recv);
                    thread::spawn(move || {
                        let _ = send.send(
                            rfd::FileDialog::new()
                                .set_title("Choose local demo files")
                                .pick_files(),
                        );
                    });
                    rt.repaint_after(0.1);
                }
                ui::badge(ui, rt, "Simulation · no network upload", "warning");
            });
        rt.record("forms-dropzone", &frame.response);
        let dropped = ui.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty()
            && ui
                .input(|i| i.pointer.hover_pos())
                .is_none_or(|pos| frame.response.rect.contains(pos))
        {
            let paths: Vec<_> = dropped.iter().filter_map(|f| f.path.clone()).collect();
            if !paths.is_empty() {
                self.select_paths(paths);
            } else {
                self.select_files(
                    dropped
                        .into_iter()
                        .filter_map(|f| {
                            f.bytes.map(|bytes| LocalFile {
                                name: f.name,
                                size: bytes.len() as u64,
                            })
                        })
                        .collect(),
                );
            }
        }
    }
    fn file_details(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.strong("Local selection");
            ui.label(format!(
                "{} {}",
                self.files.len(),
                if self.files.len() == 1 {
                    "file"
                } else {
                    "files"
                }
            ));
        });
        if self.files.is_empty() {
            ui::muted(
                ui,
                rt,
                "Your selection will appear here. No files are selected.",
            );
        }
        for file in &self.files {
            ui.horizontal(|ui| {
                paint_icon(ui, rt, "document", 20.0);
                ui.vertical(|ui| {
                    ui.label(&file.name);
                    ui::muted(ui, rt, &format!("Local file · {}", size(file.size)));
                });
                ui::badge(ui, rt, "Selected", "neutral");
            });
        }
        ui.horizontal(|ui| {
            ui.label("Simulated progress");
            ui.label(format!("{}%", (self.progress * 100.0).round() as u32));
        });
        ui::progress(ui, rt, self.progress, 6.0);
        ui.horizontal_wrapped(|ui| {
            let label = if self.simulation.is_some() {
                "Pause simulation"
            } else if self.progress >= 1.0 {
                "Replay simulation"
            } else if self.progress > 0.0 {
                "Resume simulation"
            } else {
                "Simulate progress"
            };
            let r = ui
                .add_enabled_ui(!self.files.is_empty(), |ui| {
                    ui::button(
                        ui,
                        rt,
                        "forms-simulate",
                        label,
                        Some(if self.simulation.is_some() {
                            "pause"
                        } else {
                            "play"
                        }),
                        ButtonKind::Primary,
                    )
                })
                .inner;
            rt.record("forms-simulate", &r);
            if r.clicked() {
                if self.simulation.is_some() {
                    self.pause(
                        rt.now,
                        "Simulation paused. Resume when ready; nothing was uploaded.",
                    );
                } else {
                    self.start_simulation(rt.now);
                }
            }
            if ui
                .add_enabled_ui(self.selection_count > 0, |ui| {
                    ui::button(
                        ui,
                        rt,
                        "forms-clear-files",
                        "Clear files",
                        None,
                        ButtonKind::Ghost,
                    )
                })
                .inner
                .clicked()
            {
                self.select_files(Vec::new());
                self.upload_status =
                    Notice::new("Local selection cleared. Nothing was uploaded.", "");
            }
        });
        status(ui, rt, &self.upload_status);
    }
    fn profile_field(&mut self, ui: &mut egui::Ui, rt: &mut Runtime, index: usize) {
        let error = if self.submitted || self.touched[index] {
            self.profile.error(index)
        } else {
            None
        };
        let response = match index {
            0 => edit(
                ui,
                rt,
                "forms-profile-name",
                "Display name · Required",
                &mut self.profile.name,
                error,
                "",
            ),
            1 => edit(
                ui,
                rt,
                "forms-profile-email",
                "Email address · Required",
                &mut self.profile.email,
                error,
                "",
            ),
            2 => {
                let r = edit(
                    ui,
                    rt,
                    "forms-profile-handle",
                    "Username · Required",
                    &mut self.profile.handle,
                    error,
                    "3–24 letters, numbers, or underscores",
                );
                ui::muted(ui, rt, "3–24 letters, numbers, or underscores.");
                r
            }
            3 => {
                ui.label("Role · Required");
                let before = self.profile.role.clone();
                let result = egui::ComboBox::from_id_salt("forms-profile-role")
                    .selected_text(if self.profile.role.is_empty() {
                        "Choose a role"
                    } else {
                        &self.profile.role
                    })
                    .show_ui(ui, |ui| {
                        for role in ROLES {
                            ui.selectable_value(
                                &mut self.profile.role,
                                role.into(),
                                if role.is_empty() {
                                    "Choose a role"
                                } else {
                                    role
                                },
                            );
                        }
                    });
                if before != self.profile.role {
                    self.profile_edited();
                }
                if let Some(error) = error {
                    ui.colored_label(rt.tokens.danger_text, error);
                }
                rt.record("forms-profile-role", &result.response);
                result.response
            }
            4 => edit(
                ui,
                rt,
                "forms-profile-site",
                "Website · Optional",
                &mut self.profile.site,
                error,
                "https://example.com",
            ),
            _ => {
                ui.label("About you · Optional");
                let r = growing_text(ui, rt, "forms-profile-bio", &mut self.profile.bio, 160);
                ui::muted(ui, rt, &format!("{} / 160", utf16_len(&self.profile.bio)));
                if let Some(error) = error {
                    ui.colored_label(rt.tokens.danger_text, error);
                }
                rt.record("forms-profile-bio", &r);
                r
            }
        };
        self.profile_ids[index] = Some(response.id);
        if self.profile_focus == Some(index) {
            response.request_focus();
            response.scroll_to_me(Some(egui::Align::Center));
            self.profile_focus = None;
        }
        if response.lost_focus() {
            self.touched[index] = true;
        }
        if response.changed() {
            if index == 0 {
                limit_utf16(&mut self.profile.name, 64);
            }
            if index == 2 {
                limit_utf16(&mut self.profile.handle, 24);
            }
            self.profile_edited();
        }
    }
    fn profile(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::card(ui, rt, "forms-profile-form", |ui, rt| {
            ui.small("Composed example");
            ui.heading("Your workspace profile");
            ui.add(
                egui::Label::new(
                    egui::RichText::new(
                        "Required fields are marked. Save is in memory for this visit only.",
                    )
                    .size(12.0)
                    .color(rt.tokens.text3),
                )
                .wrap(),
            );
            ui.horizontal(|ui| {
                ui::badge(
                    ui,
                    rt,
                    if self.dirty() {
                        "Unsaved changes"
                    } else if self.saved_once {
                        "Saved for this visit"
                    } else {
                        "Example profile"
                    },
                    if self.dirty() {
                        "warning"
                    } else if self.saved_once {
                        "success"
                    } else {
                        "neutral"
                    },
                );
            });
            if self.submitted {
                let errors = self.profile.errors();
                if !errors.is_empty() {
                    egui::Frame::new()
                        .fill(rt.tokens.danger_soft)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.strong(if errors.len() == 1 {
                                "One field needs attention".into()
                            } else {
                                format!("{} fields need attention", errors.len())
                            });
                            for (index, error) in errors {
                                if ui.link(error).clicked() {
                                    self.profile_focus = Some(index);
                                }
                            }
                        });
                }
            }
            if ui.available_width() > 680.0 {
                ui.columns(2, |cols| {
                    self.profile_edit(&mut cols[0], rt);
                    self.preview(&mut cols[1], rt);
                });
            } else {
                self.profile_edit(ui, rt);
                ui.separator();
                self.preview(ui, rt);
            }
        });
    }
    fn profile_edit(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        for index in 0..6 {
            self.profile_field(ui, rt, index);
            ui.add_space(5.0);
        }
        ui.strong("Profile visibility");
        let before = self.profile.clone();
        radio_pair(
            ui,
            rt,
            "forms-visibility",
            ["Team only", "Public"],
            &mut self.profile.public,
        );
        ui::muted(
            ui,
            rt,
            "Changes the local preview only. Nothing is published.",
        );
        ui::checkbox(
            ui,
            rt,
            "forms-profile-updates",
            "Weekly product updates",
            &mut self.profile.updates,
        );
        ui::muted(ui, rt, "Demo preference only. This does not subscribe you.");
        if before != self.profile {
            self.profile_edited();
        }
        ui.horizontal_wrapped(|ui| {
            let save = ui::button(
                ui,
                rt,
                "forms-profile-save",
                "Save demo profile",
                Some("check"),
                ButtonKind::Primary,
            );
            rt.record("forms-profile-save", &save);
            if save.clicked() {
                let success = self.save_profile();
                self.flash = Some((rt.now, !success, "profile"));
                if let Some(index) = self.profile_focus {
                    crate::runtime::restore_focus(ui.ctx(), self.profile_ids[index]);
                }
                rt.repaint();
            }
            let reset = ui::button(
                ui,
                rt,
                "forms-profile-reset",
                "Reset changes",
                None,
                ButtonKind::Secondary,
            );
            rt.record("forms-profile-reset", &reset);
            if reset.clicked() {
                self.reset_profile();
            }
        });
        status(ui, rt, &self.profile_status);
        self.flash_overlay(ui, rt, "profile");
    }
    fn preview(&self, ui: &mut egui::Ui, rt: &mut Runtime) {
        egui::Frame::new()
            .fill(rt.tokens.surface2)
            .corner_radius(12.0 * rt.tokens.radius)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.small("Live / local preview");
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(64.0), Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 32.0, rt.tokens.accent_soft);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    self.profile.initials(),
                    FontId::proportional(22.0),
                    rt.tokens.accent_text,
                );
                ui.heading(if self.profile.name.trim().is_empty() {
                    "Your display name"
                } else {
                    self.profile.name.trim()
                });
                ui.label(if self.profile.handle.trim().is_empty() {
                    "Your username".into()
                } else {
                    format!("@{}", self.profile.handle.trim())
                });
                ui.horizontal(|ui| {
                    ui::badge(
                        ui,
                        rt,
                        if self.profile.role.is_empty() {
                            "Choose a role"
                        } else {
                            &self.profile.role
                        },
                        "accent",
                    );
                });
                ui.label(if self.profile.bio.trim().is_empty() {
                    "Add a short introduction to your local preview."
                } else {
                    self.profile.bio.trim()
                });
                ui.separator();
                ui.label(format!(
                    "{} · local preview",
                    if self.profile.public {
                        "Public"
                    } else {
                        "Team only"
                    }
                ));
                ui::muted(
                    ui,
                    rt,
                    "A preview, not a published profile. Closing the app clears demo saves.",
                );
            });
    }
    fn flash_overlay(&mut self, ui: &mut egui::Ui, rt: &mut Runtime, key: &str) {
        if let Some((start, error, which)) = self.flash {
            if which != key {
                return;
            }
            let p = rt.progress(start, if error { 0.32 } else { 0.22 });
            if p >= 1.0 {
                self.flash = None;
                return;
            }
            if !rt.reduced() {
                let rect = ui.min_rect();
                let alpha = (1.0 - p) * 0.16;
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(
                        1.0,
                        if error {
                            rt.tokens.danger
                        } else {
                            rt.tokens.success
                        }
                        .linear_multiply(alpha),
                    ),
                    StrokeKind::Inside,
                );
                rt.repaint();
            }
        }
    }
}
impl Page for State {
    fn enter(&mut self, _rt: &mut Runtime) {
        self.active = true;
    }
    fn leave(&mut self, rt: &mut Runtime) {
        self.pause(
            rt.now,
            "Simulation paused while the page is hidden. Resume when ready; nothing uploaded.",
        );
        self.active = false;
        self.drag_bound = None;
        self.file_picker = None;
        self.flash = None;
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        form_style(ui, rt);
        if !ui.input(|i| i.focused) {
            self.pause(rt.now,"Simulation paused while the window is inactive. Resume when ready; nothing uploaded.");
            self.drag_bound = None;
        } else {
            self.tick_simulation(rt.now);
            if self.simulation.is_some() {
                rt.repaint();
            }
        }
        if let Some(receiver) = &self.file_picker {
            match receiver.try_recv() {
                Ok(paths) => {
                    self.file_picker = None;
                    if let Some(paths) = paths {
                        self.select_paths(paths);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => rt.repaint_after(0.1),
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.file_picker = None;
                    self.upload_status = Notice::new(
                        "File picker closed without a selection. Nothing uploaded.",
                        "",
                    );
                }
            }
        }
        ui::page_head(
            ui,
            rt,
            "input",
            "Build / Interaction primitives",
            "Forms & Inputs",
            "Inputs, validation, uploads and account settings.",
        );
        egui::Frame::new().fill(rt.tokens.accent_soft).inner_margin(16.0).corner_radius(8.0*rt.tokens.radius).show(ui,|ui|{ui.strong("Try every state. Nothing leaves this page.");ui.label("Validation runs locally. Profile saves last for this visit; file progress is a simulation. Use sample details.");ui.monospace("input → check → feedback");});
        ui.add_space(24.0);
        ui::section(ui, rt, "forms-text", "01", "Text inputs", "");
        self.row(ui, rt, Self::sizes, Self::states);
        self.row(ui, rt, Self::validation, Self::note);
        ui::section(
            ui,
            rt,
            "forms-security",
            "02",
            "Password and verification",
            "",
        );
        self.row(ui, rt, Self::password, Self::otp);
        ui::section(ui, rt, "forms-choice", "03", "Selection controls", "");
        self.row(ui, rt, Self::timezone, Self::seats);
        ui::section(ui, rt, "forms-ranges", "04", "Sliders", "");
        self.row(ui, rt, Self::volume, Self::interval);
        ui::section(ui, rt, "forms-files", "05", "File upload", "");
        self.files(ui, rt);
        ui.add_space(24.0);
        ui::section(ui, rt, "forms-profile", "06", "Profile form", "");
        self.profile(ui, rt);
    }
}
fn edit(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    label: &str,
    value: &mut String,
    error: Option<&str>,
    hint: &str,
) -> Response {
    let label = ui.label(label);
    let response = ui
        .scope(|ui| {
            if error.is_some() {
                ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, rt.tokens.danger);
                ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, rt.tokens.danger);
            }
            ui.add(
                egui::TextEdit::singleline(value)
                    .id(Id::new(id))
                    .hint_text(hint)
                    .margin(Vec2::new(12.0, 10.0))
                    .min_size(Vec2::new(0.0, 40.0))
                    .desired_width(f32::INFINITY),
            )
        })
        .inner
        .labelled_by(label.id);
    rt.record(id, &response);
    if let Some(error) = error {
        ui.colored_label(rt.tokens.danger_text, error);
    }
    response
}
fn growing_text(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    value: &mut String,
    limit: usize,
) -> Response {
    let width = ui.available_width();
    let galley = ui.painter().layout(
        value.clone(),
        FontId::proportional(14.0),
        rt.tokens.text,
        (width - 16.0).max(40.0),
    );
    let target = galley.size().y.max(54.0) + 14.0;
    let height = rt.animate((id, "grow"), target, 0.22, Curve::Out);
    let response = ui.add(
        egui::TextEdit::multiline(value)
            .id(Id::new(id))
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .min_size(Vec2::new(0.0, height)),
    );
    if response.changed() {
        limit_utf16(value, limit);
    }
    response
}
fn select_all(ctx: &egui::Context, id: Id, text: &str) {
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(text.chars().count()),
            )));
        egui::TextEdit::store_state(ctx, id, state);
    }
}
fn status(ui: &mut egui::Ui, rt: &Runtime, notice: &Notice) {
    if !notice.text.is_empty() {
        ui.colored_label(
            match notice.tone {
                "error" => rt.tokens.danger_text,
                "success" => rt.tokens.success_text,
                _ => rt.tokens.text2,
            },
            &notice.text,
        );
    }
}
fn paint_icon(ui: &mut egui::Ui, rt: &Runtime, name: &str, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    icons::paint(ui.painter(), name, rect, rt.tokens.text2, 1.6, 0.0);
}
fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}
fn limit_utf16(value: &mut String, limit: usize) {
    let mut n = 0;
    let end = value.char_indices().find_map(|(i, c)| {
        n += c.len_utf16();
        (n > limit).then_some(i)
    });
    if let Some(end) = end {
        value.truncate(end);
    }
}
fn email_error(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.is_empty() {
        return Some("Enter an email address.");
    }
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let host = parts.next().unwrap_or_default();
    let valid = !local.is_empty()
        && local
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&c))
        && parts.next().is_none()
        && host.contains('.')
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.as_bytes()[0] != b'-'
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        });
    (!valid).then_some("Enter a complete address, such as alex@example.com.")
}
fn http_url_valid(value: &str) -> bool {
    let value = value.trim();
    let Some((scheme, tail)) = value.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return false;
    }
    let authority = tail.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit('@').next().unwrap_or_default();
    if host.is_empty()
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
        || host.contains('\\')
    {
        return false;
    }
    if let Some(tail) = host.strip_prefix('[') {
        let Some((address, port)) = tail.split_once(']') else {
            return false;
        };
        return address.parse::<std::net::Ipv6Addr>().is_ok()
            && (port.is_empty() || port.strip_prefix(':').is_some_and(valid_port));
    }
    let (hostname, port) = host
        .split_once(':')
        .map_or((host, None), |(h, p)| (h, Some(p)));
    !hostname.is_empty()
        && !hostname
            .chars()
            .any(|c| matches!(c, ':' | '%' | '[' | ']' | '<' | '>' | '"' | '{' | '}' | '|'))
        && port.is_none_or(valid_port)
}
fn valid_port(value: &str) -> bool {
    value.is_empty() || value.parse::<u16>().is_ok()
}
fn password_rules(value: &str) -> [bool; 4] {
    [
        utf16_len(value) >= 12,
        value.chars().any(|c| c.is_ascii_uppercase())
            && value.chars().any(|c| c.is_ascii_lowercase()),
        value.chars().any(|c| c.is_ascii_digit()),
        value
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !c.is_whitespace()),
    ]
}
fn size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
fn slider_key(ui: &mut egui::Ui, value: i32, min: i32, max: i32) -> Option<i32> {
    let next = ui.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight)
            || i.consume_key(egui::Modifiers::NONE, Key::ArrowUp)
        {
            Some((value + 1).min(max))
        } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft)
            || i.consume_key(egui::Modifiers::NONE, Key::ArrowDown)
        {
            Some((value - 1).max(min))
        } else if i.consume_key(egui::Modifiers::NONE, Key::Home) {
            Some(min)
        } else if i.consume_key(egui::Modifiers::NONE, Key::End) {
            Some(max)
        } else {
            None
        }
    });
    if next.is_some() {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    next
}

fn zone_key(ui: &mut egui::Ui, value: usize) -> Option<usize> {
    let next = ui.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
            Some((value + 1) % ZONES.len())
        } else if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
            Some((value + ZONES.len() - 1) % ZONES.len())
        } else if i.consume_key(egui::Modifiers::NONE, Key::Home) {
            Some(0)
        } else if i.consume_key(egui::Modifiers::NONE, Key::End) {
            Some(ZONES.len() - 1)
        } else {
            None
        }
    });
    if next.is_some() {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    next
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn email_format_including_required_and_native_restrictions() {
        for input in [
            "",
            "alex@",
            "x@host",
            "x@a..com",
            "x y@host.com",
            "x@-host.com",
            "x@@host.com",
        ] {
            assert!(email_error(input).is_some(), "{input}");
        }
        for input in [
            "sample@example.com",
            " a+b@local.example ",
            "a.b@example.co.uk",
        ] {
            assert!(email_error(input).is_none(), "{input}");
        }
    }
    #[test]
    fn profile_all_rules_are_checked() {
        let mut p = Profile::default();
        assert!(p.errors().is_empty());
        p.name = " ".into();
        p.email = "bad".into();
        p.handle = "has spaces".into();
        p.role.clear();
        p.site = "javascript:alert(1)".into();
        p.bio = "x".repeat(161);
        assert_eq!(p.errors().len(), 6);
        for url in [
            "https://example.com/path?x=1",
            "http://localhost:8780",
            "https://[::1]:443/a",
            "HTTPS://example.com",
        ] {
            assert!(http_url_valid(url), "{url}");
        }
        for url in [
            "https://",
            "ftp://example.com",
            "http://a:99999",
            "http://bad host",
            "http://[invalid]",
        ] {
            assert!(!http_url_valid(url), "{url}");
        }
    }
    #[test]
    fn profile_save_reset_restores_entire_snapshot() {
        let mut s = State::default();
        s.profile.name.clear();
        assert!(!s.save_profile());
        assert_eq!(s.profile_focus, Some(0));
        s.profile.name = "Saved Sample".into();
        s.profile.public = true;
        s.profile.updates = false;
        assert!(s.save_profile());
        s.profile.name = "Changed".into();
        s.profile.public = false;
        s.profile.updates = true;
        assert!(s.dirty());
        s.reset_profile();
        assert_eq!(s.profile.name, "Saved Sample");
        assert!(s.profile.public);
        assert!(!s.profile.updates);
        assert!(!s.dirty());
        assert_eq!(s.password, "");
    }
    #[test]
    fn otp_full_paste_resets_offset_and_partial_paste_preserves_rest() {
        let mut otp = Otp::default();
        assert!(otp.distribute("123-456", 4));
        assert_eq!(otp.digits.concat(), "123456");
        assert_eq!(otp.focus, Some(5));
        assert!(otp.verify());
        assert!(otp.distribute("9 8", 2));
        assert_eq!(otp.digits.concat(), "129856");
        assert_eq!(otp.result, None);
        assert!(!otp.distribute("1234567", 0));
        assert!(!otp.distribute("１２３", 0));
        assert_eq!(otp.digits.concat(), "129856");
    }
    #[test]
    fn otp_backspace_and_invalid_verification_focus() {
        let mut otp = Otp::default();
        otp.digits[0] = "1".into();
        otp.backspace(1);
        assert!(otp.digits[0].is_empty());
        assert_eq!(otp.focus, Some(0));
        assert!(!otp.verify());
        assert_eq!(otp.result, Some(false));
        otp.digits[0] = "a".into();
        otp.input(0);
        assert!(otp.digits[0].is_empty());
    }
    #[test]
    fn password_meter_uses_four_independent_rules() {
        assert_eq!(password_rules(""), [false; 4]);
        assert_eq!(password_rules("abcdefghijkl"), [true, false, false, false]);
        assert_eq!(password_rules("SamplePass12!"), [true; 4]);
        assert_eq!(password_rules("Sample 12"), [false, true, true, false]);
    }
    #[test]
    fn limits_count_utf16_without_splitting_unicode() {
        let mut text = "a𝄞b".into();
        limit_utf16(&mut text, 2);
        assert_eq!(text, "a");
        let mut s = State::default();
        s.note = "x".repeat(240);
        s.update_note();
        assert_eq!(s.note_threshold, 2);
        assert_eq!(s.note_announcement, "Character limit reached.");
    }
    #[test]
    fn seats_invalid_edit_and_recovery() {
        let mut s = State::default();
        for text in ["", "0", "26", "3.5", "nope"] {
            s.seats_text = text.into();
            s.edit_seats();
            assert_eq!(s.seats, 4);
            assert!(!s.seats_error.is_empty());
        }
        s.seats_text = "12".into();
        s.edit_seats();
        assert_eq!(s.seats, 12);
        assert!(s.seats_error.is_empty());
        s.set_seats(999, true);
        assert_eq!(s.seats, 25);
    }
    #[test]
    fn bounds_never_cross() {
        let mut s = State::default();
        s.low = 99;
        s.bounds(false);
        assert_eq!(s.low, 70);
        s.high = 1;
        s.bounds(true);
        assert_eq!(s.high, 80);
        for n in -50..150 {
            s.low = n;
            s.bounds(false);
            assert!(s.high - s.low >= 10);
            assert!((0..=90).contains(&s.low));
        }
    }
    #[test]
    fn file_limits_and_escaped_names_remain_plain_data() {
        let mut s = State::default();
        let mut files = vec![
            LocalFile {
                name: "<script>.txt".into(),
                size: 0,
            },
            LocalFile {
                name: "too big".into(),
                size: 10 * 1_048_576 + 1,
            },
        ];
        files.extend((0..6).map(|i| LocalFile {
            name: i.to_string(),
            size: 10 * 1_048_576,
        }));
        s.select_files(files);
        assert_eq!(s.files.len(), 4);
        assert_eq!(s.files[0].name, "<script>.txt");
        assert_eq!(s.selection_count, 8);
        assert!(s.upload_status.text.contains("4 skipped"));
    }
    #[test]
    fn simulation_pause_resume_and_replay_use_real_time() {
        let mut s = State::default();
        s.active = true;
        s.select_files(vec![LocalFile {
            name: "a".into(),
            size: 10,
        }]);
        s.start_simulation(10.0);
        s.tick_simulation(11.3);
        assert!((s.progress - 0.5).abs() < 0.001);
        s.pause(11.3, "paused");
        assert!(s.simulation.is_none());
        let p = s.progress;
        s.tick_simulation(500.0);
        assert_eq!(s.progress, p);
        s.start_simulation(1000.0);
        s.tick_simulation(1001.31);
        assert_eq!(s.progress, 1.0);
        assert!(s.simulation.is_none());
        s.start_simulation(2000.0);
        assert_eq!(s.progress, 0.0);
        s.select_files(Vec::new());
        assert!(s.simulation.is_none());
        assert_eq!(s.progress, 0.0);
    }
    #[test]
    fn profile_initials_use_unicode_scalars() {
        let mut p = Profile::default();
        p.name = " Élodie  Müller ".into();
        assert_eq!(p.initials(), "ÉM");
        p.name.clear();
        assert_eq!(p.initials(), "?");
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
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 14000.0))),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| s.show(ui, rt));
            },
        );
        assert!(output.platform_output.accesskit_update.is_some());
        for p in ctx.tessellate(output.shapes, output.pixels_per_point) {
            if let egui::epaint::Primitive::Mesh(mesh) = p.primitive {
                assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
            }
        }
    }
    fn click(s: &mut State, rt: &mut Runtime, ctx: &egui::Context, id: &str, t: f64) {
        let rect = rt
            .records
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("missing {id}"))
            .rect;
        let pos = Pos2::new((rect[0] + rect[2]) / 2.0, (rect[1] + rect[3]) / 2.0);
        frame(
            s,
            rt,
            ctx,
            t,
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
            s,
            rt,
            ctx,
            t + 0.01,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    #[test]
    fn actual_egui_otp_paste_and_profile_submit() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        frame(&mut s, &mut rt, &ctx, 0.02, vec![]);
        click(&mut s, &mut rt, &ctx, "forms-otp-1", 0.1);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            0.2,
            vec![egui::Event::Paste("123456".into())],
        );
        assert_eq!(s.otp.digits.concat(), "123456");
        click(&mut s, &mut rt, &ctx, "forms-otp-verify", 0.3);
        assert_eq!(s.otp.result, Some(true));
        s.profile.name.clear();
        click(&mut s, &mut rt, &ctx, "forms-profile-save", 0.4);
        assert!(s.submitted);
        assert!(s.profile_status.text.contains("stopped"));
        s.profile.name = "Saved Sample".into();
        click(&mut s, &mut rt, &ctx, "forms-profile-save", 0.5);
        assert!(s.saved_once);
        s.profile.name = "Changed".into();
        click(&mut s, &mut rt, &ctx, "forms-profile-reset", 0.6);
        assert_eq!(s.profile.name, "Saved Sample");
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
    fn actual_egui_keyboard_operates_seats_bounds_and_volume() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
        frame(&mut s, &mut rt, &ctx, 0.02, vec![]);
        click(&mut s, &mut rt, &ctx, "forms-seats", 0.1);
        frame(&mut s, &mut rt, &ctx, 0.2, vec![key(Key::ArrowUp, true)]);
        assert_eq!(s.seats, 5);
        frame(&mut s, &mut rt, &ctx, 0.21, vec![key(Key::ArrowUp, false)]);
        click(&mut s, &mut rt, &ctx, "forms-high", 0.3);
        frame(&mut s, &mut rt, &ctx, 0.4, vec![key(Key::Home, true)]);
        assert_eq!(s.high, 30);
        frame(&mut s, &mut rt, &ctx, 0.41, vec![key(Key::Home, false)]);
        frame(&mut s, &mut rt, &ctx, 0.5, vec![key(Key::End, true)]);
        assert_eq!(s.high, 100);
        frame(&mut s, &mut rt, &ctx, 0.51, vec![key(Key::End, false)]);
        click(&mut s, &mut rt, &ctx, "forms-low", 0.6);
        frame(&mut s, &mut rt, &ctx, 0.7, vec![key(Key::End, true)]);
        assert_eq!(s.low, 90);
        frame(&mut s, &mut rt, &ctx, 0.71, vec![key(Key::End, false)]);
        click(&mut s, &mut rt, &ctx, "forms-volume", 0.8);
        frame(&mut s, &mut rt, &ctx, 0.9, vec![key(Key::Home, true)]);
        assert_eq!(s.volume, 0);
        frame(&mut s, &mut rt, &ctx, 0.91, vec![key(Key::Home, false)]);
        click(&mut s, &mut rt, &ctx, "forms-timezone", 1.0);
        frame(
            &mut s,
            &mut rt,
            &ctx,
            1.1,
            vec![egui::Event::Text("Tok".into())],
        );
        assert_eq!(s.zone, 5);
        frame(&mut s, &mut rt, &ctx, 1.2, vec![key(Key::Enter, true)]);
        assert_eq!(s.zone, 5);
        frame(&mut s, &mut rt, &ctx, 1.21, vec![key(Key::Enter, false)]);
        click(&mut s, &mut rt, &ctx, "forms-visibility-0", 1.3);
        frame(&mut s, &mut rt, &ctx, 1.4, vec![key(Key::ArrowRight, true)]);
        assert!(s.profile.public);
        assert!(s.dirty());
    }
    #[test]
    fn simulation_duration_does_not_follow_motion_preferences() {
        for preference in [
            crate::runtime::MotionPreference::Reduced,
            crate::runtime::MotionPreference::Slow,
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut prefs = Preferences::default();
            prefs.motion = preference;
            let mut rt = Runtime::new(ctx.clone(), prefs);
            let mut s = State::default();
            s.enter(&mut rt);
            s.select_files(vec![LocalFile {
                name: "sample".into(),
                size: 10,
            }]);
            s.start_simulation(10.0);
            frame(&mut s, &mut rt, &ctx, 11.3, vec![]);
            assert!((s.progress - 0.5).abs() < 0.001);
            assert!(s.simulation.is_some());
            frame(&mut s, &mut rt, &ctx, 12.61, vec![]);
            assert_eq!(s.progress, 1.0);
            assert!(s.simulation.is_none());
        }
    }
    #[test]
    fn native_dropped_files_use_only_metadata() {
        let dir = std::env::temp_dir().join(format!(
            "schematic-forms-native-drop-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let small = dir.join("sample.txt");
        let large = dir.join("large.bin");
        std::fs::File::create(&small)
            .unwrap()
            .set_len(1024)
            .unwrap();
        std::fs::File::create(&large)
            .unwrap()
            .set_len(10 * 1_048_576 + 1)
            .unwrap();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut rt = Runtime::new(ctx.clone(), Preferences::default());
        let mut s = State::default();
        s.enter(&mut rt);
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 14000.0))),
                time: Some(0.0),
                dropped_files: vec![
                    egui::DroppedFile {
                        path: Some(small),
                        ..Default::default()
                    },
                    egui::DroppedFile {
                        path: Some(large),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| s.show(ui, &mut rt));
            },
        );
        assert_eq!(
            s.files,
            vec![LocalFile {
                name: "sample.txt".into(),
                size: 1024
            }]
        );
        assert!(s.upload_status.text.contains("1 skipped"));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn accesskit_profile_errors_focus_the_actual_rendered_field() {
        for index in 0..6 {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            let mut state = State::default();
            state.enter(&mut rt);
            match index {
                0 => state.profile.name.clear(),
                1 => state.profile.email = "invalid".into(),
                2 => state.profile.handle.clear(),
                3 => state.profile.role.clear(),
                4 => state.profile.site = "invalid".into(),
                _ => state.profile.bio = "x".repeat(161),
            }
            frame(&mut state, &mut rt, &ctx, 0.0, vec![]);
            frame(&mut state, &mut rt, &ctx, 0.02, vec![]);
            click(&mut state, &mut rt, &ctx, "forms-profile-save", 0.1);
            assert_eq!(
                ctx.memory(|m| m.focused()),
                state.profile_ids[index],
                "field {index}"
            );
            frame(&mut state, &mut rt, &ctx, 0.15, vec![]);
            assert_eq!(
                ctx.memory(|m| m.focused()),
                state.profile_ids[index],
                "field {index} lost focus"
            );
        }
    }
    #[test]
    fn narrow_form_controls_remain_inside_the_window() {
        for width in [320.0, 390.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            let mut s = State::default();
            s.enter(&mut rt);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 16000.0))),
                    time: Some(0.0),
                    ..Default::default()
                },
                |ctx| {
                    rt.begin_frame();
                    egui::CentralPanel::default().show(ctx, |ui| s.show(ui, &mut rt));
                },
            );
            for record in &rt.records {
                assert!(
                    record.rect[0] >= -1.0 && record.rect[2] <= width + 1.0,
                    "{} exceeds {width}px: {:?}",
                    record.id,
                    record.rect
                );
            }
        }
    }
    #[test]
    fn all_forms_render_in_four_appearances() {
        for (dark, mono) in [(false, false), (false, true), (true, false), (true, true)] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut prefs = Preferences::default();
            prefs.dark = dark;
            prefs.mono = mono;
            let mut rt = Runtime::new(ctx.clone(), prefs);
            let mut s = State::default();
            s.enter(&mut rt);
            frame(&mut s, &mut rt, &ctx, 0.0, vec![]);
            assert!(rt.records.iter().any(|r| r.id == "forms-profile-bio"));
            assert!(
                rt.records
                    .iter()
                    .all(|r| r.rect.iter().all(|n| n.is_finite()))
            );
        }
    }
}

fn form_style(ui: &mut egui::Ui, rt: &Runtime) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(rt.tokens.text);
    visuals.text_edit_bg_color = Some(rt.tokens.surface);
    visuals.selection.bg_fill = rt.tokens.accent_soft2;
    visuals.selection.stroke = Stroke::new(1.0, rt.tokens.accent);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.bg_stroke = Stroke::new(1.0, rt.tokens.border_strong);
        widget.corner_radius = egui::CornerRadius::same((6.0 * rt.tokens.radius) as u8);
        widget.fg_stroke.color = rt.tokens.text;
    }
    visuals.widgets.hovered.bg_stroke.color = rt.tokens.accent_ring;
    visuals.widgets.active.bg_stroke.color = rt.tokens.accent;
    ui.spacing_mut().interact_size.y = 32.0;
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
                if let Some(n) = radio_key(ui, i) {
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
                    1.0,
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
                ui.painter()
                    .circle_stroke(center, 10.0, Stroke::new(2.0, rt.tokens.accent_ring));
            }
            rt.record(&format!("{id}-{i}"), &r);
            responses.push(r);
        }
    });
    if let Some(next) = next {
        responses[next].request_focus();
    }
}

fn radio_key(ui: &mut egui::Ui, index: usize) -> Option<usize> {
    let next = ui.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, Key::ArrowRight)
            || i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft)
        {
            Some(1 - index)
        } else if i.consume_key(egui::Modifiers::NONE, Key::Home) {
            Some(0)
        } else if i.consume_key(egui::Modifiers::NONE, Key::End) {
            Some(1)
        } else {
            None
        }
    });
    if next.is_some() {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    next
}
