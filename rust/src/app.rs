use crate::{
    commands, feedback, icons,
    motion::Curve,
    onboarding::Guide,
    pages,
    runtime::{MotionPreference, Page, Preferences, Route, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{self, *};
use std::collections::HashMap;

pub struct SchematicApp {
    pub rt: Runtime,
    pub pages: HashMap<Route, Box<dyn Page>>,
    pub guide: Guide,
    feedback: feedback::State,
    nav_open: bool,
    tune_open: bool,
    commands: Commands,
    history: Vec<Route>,
    history_index: usize,
    theme_seen: bool,
    theme_texture: Option<TextureHandle>,
    theme_request: Option<(u64, bool, f64)>,
    theme_generation: u64,
    theme_start: f64,
    capture: Option<Capture>,
    command_trigger: Option<Id>,
    guide_trigger: Option<Id>,
    nav_trigger: Option<Id>,
    tune_trigger: Option<Id>,
    route_started: f64,
    history_target: Option<usize>,
    window_hidden: bool,
    window_focused: bool,
}
struct Capture {
    path: std::path::PathBuf,
    started: Option<f64>,
    requested: bool,
    quit: bool,
}
#[derive(Clone, Debug)]
enum CaptureTag {
    Theme(u64),
    Artifact(std::path::PathBuf, bool),
}
struct Commands {
    open: bool,
    query: String,
    selected: usize,
    first: bool,
    origin: Option<Id>,
    surface: feedback::Surface,
    result_started: f64,
}
impl Default for Commands {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            first: false,
            origin: None,
            surface: feedback::Surface::new("commands-modal", feedback::SurfaceKind::Dialog),
            result_started: 0.0,
        }
    }
}
impl SchematicApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let saved: Option<Preferences> = cc
            .storage
            .and_then(|s| eframe::get_value(s, "schematic-native-preferences"));
        let mut prefs = saved.clone().unwrap_or_default();
        if saved.is_none() {
            prefs.dark = cc.egui_ctx.style().visuals.dark_mode;
        }
        if std::env::var_os("SCHEMATIC_NATIVE_CAPTURE").is_some() {
            prefs = Preferences::default();
            prefs.dark = std::env::var("SCHEMATIC_NATIVE_DARK").as_deref() == Ok("1");
            prefs.mono = std::env::var("SCHEMATIC_NATIVE_MONO").as_deref() == Ok("1");
        }
        let mut app = Self::with_context(cc.egui_ctx.clone(), prefs);
        app.rt.system_reduced = system_reduced();
        if let Ok(route) = std::env::var("SCHEMATIC_NATIVE_ROUTE") {
            app.rt.navigate(Route::from_id(&route));
        }
        app
    }
    pub fn with_context(ctx: Context, prefs: Preferences) -> Self {
        let theme = prefs.dark;
        let mut rt = Runtime::new(ctx, prefs);
        let mut pages = pages::create();
        pages
            .get_mut(&Route::Overview)
            .expect("Overview")
            .enter(&mut rt);
        rt.mounted.insert(Route::Overview);
        Self {
            rt,
            pages,
            guide: Guide::default(),
            feedback: feedback::State::default(),
            nav_open: false,
            tune_open: false,
            commands: Commands::default(),
            history: vec![Route::Overview],
            history_index: 0,
            theme_seen: theme,
            theme_texture: None,
            theme_request: None,
            theme_generation: 0,
            theme_start: 0.0,
            capture: std::env::var_os("SCHEMATIC_NATIVE_CAPTURE").map(|p| Capture {
                path: p.into(),
                started: None,
                requested: false,
                quit: std::env::var("SCHEMATIC_NATIVE_CAPTURE_QUIT").as_deref() == Ok("1"),
            }),
            command_trigger: None,
            guide_trigger: None,
            nav_trigger: None,
            tune_trigger: None,
            route_started: -10.0,
            history_target: None,
            window_hidden: false,
            window_focused: true,
        }
    }
    fn apply_style(&self, ctx: &Context) {
        let t = &self.rt.tokens;
        let mut style = (*ctx.style()).clone();
        style.visuals = if self.rt.prefs.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        style.visuals.override_text_color = Some(t.text);
        style.visuals.panel_fill = t.bg;
        style.visuals.window_fill = t.surface_raised;
        style.visuals.extreme_bg_color = t.bg_sunken;
        style.visuals.faint_bg_color = t.surface2;
        style.visuals.code_bg_color = t.surface2;
        style.visuals.window_stroke = Stroke::new(1.0, t.border);
        style.visuals.window_corner_radius = ui::rounded(16.0 * t.radius);
        style.visuals.selection.bg_fill = t.accent_soft2;
        style.visuals.selection.stroke = Stroke::new(1.0, t.accent_text);
        style.visuals.hyperlink_color = t.accent_text;
        style.visuals.widgets.noninteractive.bg_fill = t.surface;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border);
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text2);
        for widget in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = ui::rounded(6.0 * t.radius);
            widget.fg_stroke = Stroke::new(1.0, t.text);
            widget.bg_stroke = Stroke::new(1.0, t.border);
            widget.bg_fill = t.surface;
        }
        style.visuals.widgets.hovered.bg_fill = t.surface2;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, t.border_strong);
        style.visuals.widgets.active.bg_fill = t.accent_soft;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, t.accent_ring);
        style.spacing.item_spacing = vec2(12.0, 12.0);
        style.spacing.button_padding = vec2(12.0, 8.0);
        style.spacing.interact_size = vec2(34.0, 34.0);
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(11.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(22.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.0));
        ctx.set_style(style);
    }
    fn sidebar(&mut self, ui: &mut Ui) {
        let t = self.rt.tokens.clone();
        ui.set_width(216.0);
        ui.spacing_mut().item_spacing.y = 1.0;
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
            ui.painter().rect_filled(rect, ui::rounded(8.0), t.text);
            icons::paint(ui.painter(), "cube", rect.shrink(7.0), t.bg, 1.6, 0.0);
            ui.label(RichText::new("Schematic").size(14.0).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(ui, &self.rt, "Rust", "");
            });
        });
        ui.add_space(24.0);
        let search = ui::button(
            ui,
            &mut self.rt,
            "library-search",
            "Search…",
            Some("search"),
            ButtonKind::Secondary,
        );
        if search.clicked() {
            self.open_commands(Some(search.id));
        }
        ui.add_space(22.0);
        let mut group = "";
        for route in Route::ALL {
            if group != route.group() {
                group = route.group();
                ui.add_space(if group == "Start" { 4.0 } else { 16.0 });
                ui.label(
                    RichText::new(group.to_uppercase())
                        .size(10.0)
                        .color(t.text3),
                );
                ui.add_space(4.0);
            }
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
            response.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::SelectableLabel,
                    true,
                    self.rt.route == route,
                    route.title(),
                )
            });
            let hover = self.rt.animate(
                ("sidebar-hover", route.id()),
                if response.hovered() { 1.0 } else { 0.0 },
                0.14,
                Curve::Out,
            );
            if self.rt.route == route {
                let y = self
                    .rt
                    .animate("sidebar-active-y", rect.top(), 0.47, Curve::SpringSnappy);
                let active = Rect::from_min_size(pos2(rect.left(), y), rect.size());
                ui.painter().rect(
                    active,
                    ui::rounded(8.0 * t.radius),
                    t.surface,
                    Stroke::new(1.0, t.border),
                    StrokeKind::Inside,
                );
                ui.painter().line_segment(
                    [
                        pos2(active.left() - 3.0, active.top() + 10.0),
                        pos2(active.left() - 3.0, active.bottom() - 10.0),
                    ],
                    Stroke::new(2.0, t.accent),
                );
            } else if hover > 0.0 {
                ui.painter().rect_filled(
                    rect,
                    ui::rounded(8.0 * t.radius),
                    t.surface2.gamma_multiply(hover),
                );
            }
            let p = self.rt.icon_progress(
                ("nav-icon", route.id()),
                response.hovered() || response.has_focus(),
            );
            icons::paint(
                ui.painter(),
                route.icon(),
                Rect::from_center_size(pos2(rect.left() + 17.0, rect.center().y), vec2(16.0, 16.0)),
                if self.rt.route == route {
                    t.accent_text
                } else {
                    t.text2
                },
                1.6,
                p,
            );
            ui.painter().text(
                pos2(rect.left() + 38.0, rect.center().y),
                Align2::LEFT_CENTER,
                route.title(),
                FontId::proportional(13.0),
                if self.rt.route == route {
                    t.text
                } else {
                    t.text2
                },
            );
            if response.clicked() {
                self.rt.navigate(route);
                self.nav_open = false;
            }
            self.rt.record(&format!("route:{}", route.id()), &response);
        }
        let rest = ui.available_height() - 42.0;
        if rest > 0.0 {
            ui.add_space(rest);
        }
        Frame::new()
            .fill(t.surface)
            .stroke(Stroke::new(1.0, t.border))
            .corner_radius(ui::rounded(10.0 * t.radius))
            .inner_margin(8)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui::badge(ui, &self.rt, "Local", "success");
                    ui.label(RichText::new("Native Rust").size(11.0).color(t.text3));
                });
            });
    }
    fn topbar(&mut self, ui: &mut Ui, compact: bool) {
        ui.horizontal(|ui| {
            ui.set_min_height(40.0);
            if compact {
                ui.spacing_mut().item_spacing.x = 3.0;
            }
            if compact || self.rt.route == Route::Workspace {
                let menu = ui::icon_button(ui, &mut self.rt, "library-menu", "menu", "Menu");
                self.nav_trigger = Some(menu.id);
                if menu.clicked() {
                    self.nav_open = !self.nav_open;
                }
            }
            if !compact {
                ui.label(
                    RichText::new(self.rt.route.group())
                        .size(12.0)
                        .color(self.rt.tokens.text3),
                );
                ui.label(RichText::new("›").size(16.0).color(self.rt.tokens.text4));
            }
            if compact {
                ui.add_sized(
                    [80.0, 34.0],
                    Label::new(RichText::new(self.rt.route.title()).size(12.0).strong()).truncate(),
                );
            } else {
                ui.label(RichText::new(self.rt.route.title()).size(12.0).strong());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let commands = if compact {
                    ui::icon_button(ui, &mut self.rt, "commands", "command", "Commands")
                } else {
                    ui::button(
                        ui,
                        &mut self.rt,
                        "commands",
                        "Commands",
                        Some("command"),
                        ButtonKind::Secondary,
                    )
                };
                self.command_trigger = Some(commands.id);
                if commands.clicked() {
                    self.open_commands(Some(commands.id));
                }
                let dark = self.rt.desired_theme.unwrap_or(self.rt.prefs.dark);
                let response = ui::icon_button(
                    ui,
                    &mut self.rt,
                    "theme",
                    if dark { "moon" } else { "sun" },
                    "Toggle theme",
                );
                if response.clicked() {
                    self.rt.set_theme(!dark);
                }
                let tune =
                    ui::icon_button(ui, &mut self.rt, "tune", "sliders", "Tune design tokens");
                self.tune_trigger = Some(tune.id);
                if tune.clicked() {
                    self.tune_open = !self.tune_open;
                }
                let guide = ui::button(
                    ui,
                    &mut self.rt,
                    "guide-launch",
                    "Guide",
                    if compact { None } else { Some("book-open") },
                    ButtonKind::Ghost,
                );
                self.guide_trigger = Some(guide.id);
                if guide.clicked() && !self.commands.open {
                    self.guide
                        .open(&mut self.rt, Some(guide.rect), Some(guide.id));
                }
            });
        });
    }
    fn open_commands(&mut self, origin: Option<Id>) {
        if self.guide.visible() {
            return;
        }
        self.commands.open = true;
        self.commands.first = true;
        self.commands.query.clear();
        self.commands.selected = 0;
        self.commands.origin = origin;
        self.commands.surface.show_dialog(None, origin, self.rt.now);
        self.commands.result_started = self.rt.now;
        self.rt.repaint();
    }
    fn close_commands(&mut self, ctx: &Context) {
        self.commands.open = false;
        self.commands.surface.close();
        crate::runtime::restore_focus(
            ctx,
            if self.rt.requested.is_none() {
                self.commands.origin
            } else {
                None
            },
        );
    }
    fn show_commands(&mut self, ctx: &Context) {
        if !self.commands.surface.is_visible() {
            return;
        }
        let t = self.rt.tokens.clone();
        let mut chosen = None;
        let keys = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Enter),
            )
        });
        let mut focus_order = vec![];
        self.commands.surface.size = vec2(552.0, (ctx.content_rect().height() - 64.0).min(540.0));
        self.commands.surface.padding = 16.0;
        let response = self.commands.surface.show(
            ctx,
            self.rt.now,
            &t,
            self.rt.motion_scale(),
            self.rt.reduced(),
            |ui, _| {
                let search = ui.add(
                    TextEdit::singleline(&mut self.commands.query)
                        .hint_text("Search pages, actions, settings…")
                        .desired_width(f32::INFINITY)
                        .frame(false),
                );
                if self.commands.first {
                    search.request_focus();
                    self.commands.first = false;
                }
                focus_order.push(search.id);
                self.rt.record("command-search", &search);
                if search.changed() {
                    self.commands.selected = 0;
                    self.commands.result_started = self.rt.now;
                }
                let matches = commands::search(&self.commands.query);
                if !matches.is_empty() {
                    if keys.0 {
                        self.commands.selected = (self.commands.selected + 1) % matches.len();
                    }
                    if keys.1 {
                        self.commands.selected =
                            (self.commands.selected + matches.len() - 1) % matches.len();
                    }
                    self.commands.selected = self.commands.selected.min(matches.len() - 1);
                    if keys.2 && self.commands.open {
                        chosen = Some(matches[self.commands.selected].action);
                    }
                }
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("command-results")
                    .max_height((ctx.content_rect().height() - 200.0).clamp(160.0, 400.0))
                    .show(ui, |ui| {
                        let reveal = crate::motion::ease(
                            Curve::Out,
                            self.rt.progress(self.commands.result_started, 0.22),
                        );
                        ui.scope(|ui| {
                            ui.set_opacity(reveal);
                            if matches.is_empty() {
                                ui.label(format!("No results for “{}”", self.commands.query));
                            }
                            for (index, command) in matches.iter().enumerate() {
                                let (rect, _) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 44.0),
                                    Sense::hover(),
                                );
                                let response =
                                    ui.interact(rect, ui.id().with(&command.id), Sense::click());
                                response.widget_info(|| {
                                    WidgetInfo::selected(
                                        WidgetType::SelectableLabel,
                                        true,
                                        index == self.commands.selected,
                                        &command.label,
                                    )
                                });
                                focus_order.push(response.id);
                                if response.hovered()
                                    && ctx.input(|i| i.pointer.delta() != Vec2::ZERO)
                                {
                                    self.commands.selected = index;
                                }
                                if index == self.commands.selected {
                                    let y = self.rt.animate(
                                        "command-selection-y",
                                        rect.top(),
                                        0.35,
                                        Curve::SpringSnappy,
                                    );
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(pos2(rect.left(), y), rect.size()),
                                        ui::rounded(7.0 * t.radius),
                                        t.accent_soft,
                                    );
                                    if keys.0 || keys.1 {
                                        response.scroll_to_me(Some(Align::Center));
                                    }
                                }
                                let progress = self.rt.icon_progress(
                                    ("command-icon", &command.id),
                                    response.hovered() || response.has_focus(),
                                );
                                icons::paint(
                                    ui.painter(),
                                    command.icon,
                                    Rect::from_center_size(
                                        pos2(rect.left() + 19.0, rect.center().y),
                                        vec2(17.0, 17.0),
                                    ),
                                    t.text2,
                                    1.6,
                                    progress,
                                );
                                let mut job = egui::text::LayoutJob::default();
                                let mut query = self
                                    .commands
                                    .query
                                    .trim()
                                    .to_lowercase()
                                    .chars()
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .peekable();
                                for c in command.label.chars() {
                                    let matched = query
                                        .peek()
                                        .is_some_and(|next| c.to_lowercase().next() == Some(*next));
                                    if matched {
                                        query.next();
                                    }
                                    job.append(
                                        &c.to_string(),
                                        0.0,
                                        TextFormat {
                                            font_id: FontId::proportional(13.0),
                                            color: if matched { t.accent_text } else { t.text },
                                            ..Default::default()
                                        },
                                    );
                                }
                                let galley = ui.painter().layout_job(job);
                                ui.painter().galley(
                                    pos2(
                                        rect.left() + 39.0,
                                        rect.center().y - galley.size().y / 2.0,
                                    ),
                                    galley,
                                    t.text,
                                );
                                if rect.width() > 350.0 {
                                    ui.painter().text(
                                        pos2(rect.right() - 12.0, rect.center().y),
                                        Align2::RIGHT_CENTER,
                                        command.detail,
                                        FontId::proportional(11.0),
                                        t.text3,
                                    );
                                }
                                self.rt
                                    .record(&format!("command:{}", command.id), &response);
                                if response.clicked() && self.commands.open {
                                    chosen = Some(command.action);
                                }
                            }
                        });
                    });
                ui.separator();
                ui::muted(ui, &self.rt, "↑ ↓ Navigate     Enter Run     Esc Close");
            },
        );
        self.commands.surface.set_focus_order(&focus_order);
        if let Some(action) = chosen {
            commands::run(action, &mut self.rt);
            self.close_commands(ctx);
        } else if response.is_some_and(|r| r.should_close) {
            self.close_commands(ctx);
        }
    }
    fn show_tune(&mut self, ctx: &Context) {
        if !self.tune_open {
            return;
        }
        let t = self.rt.tokens.clone();
        let mut open = self.tune_open;
        Window::new("Tune design tokens")
            .id(Id::new("tune-window"))
            .open(&mut open)
            .default_width((ctx.content_rect().width() - 64.0).min(288.0))
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::RIGHT_TOP, vec2(-16.0, 62.0))
            .frame(
                Frame::new()
                    .fill(t.surface_raised)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(ui::rounded(12.0 * t.radius))
                    .inner_margin(16),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("Interface palette").size(12.0));
                let mut palette = usize::from(self.rt.prefs.mono);
                if ui::segmented(
                    ui,
                    &mut self.rt,
                    "palette-options",
                    &["Color", "Mono"],
                    &mut palette,
                ) {
                    self.rt.set_palette(palette == 1);
                }
                ui::muted(ui, &self.rt, "Mono keeps chart series in color.");
                ui.separator();
                ui.add_enabled_ui(!self.rt.prefs.mono, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for hue in [168.0, 195.0, 55.0, 18.0, 235.0, 290.0, 330.0, 0.0] {
                            let (rect, response) =
                                ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
                            ui.painter().circle_filled(
                                rect.center(),
                                9.0,
                                crate::tokens::oklch(0.6, 0.15, hue, 1.0),
                            );
                            if response.clicked() {
                                self.rt.set_hue(hue);
                            }
                            self.rt.record(&format!("hue:{hue}"), &response);
                        }
                    });
                    let mut hue = self.rt.prefs.hue;
                    if ui
                        .add(Slider::new(&mut hue, 0.0..=359.0).text("Accent"))
                        .changed()
                    {
                        self.rt.set_hue(hue);
                    }
                });
                ui.separator();
                let mut radius = if self.rt.prefs.radius == 0.0 {
                    0
                } else if self.rt.prefs.radius > 1.0 {
                    2
                } else {
                    1
                };
                if ui::segmented(
                    ui,
                    &mut self.rt,
                    "radius-options",
                    &["Sharp", "Default", "Round"],
                    &mut radius,
                ) {
                    self.rt.prefs.radius = [0.0, 1.0, 1.6][radius];
                }
                let mut motion = match self.rt.prefs.motion {
                    MotionPreference::Slow => 1,
                    MotionPreference::Reduced => 2,
                    MotionPreference::System => 3,
                    _ => 0,
                };
                if ui::segmented(
                    ui,
                    &mut self.rt,
                    "motion-options",
                    &["Full", "Slow-mo", "Reduced", "System"],
                    &mut motion,
                ) {
                    self.rt.set_motion(
                        [
                            MotionPreference::Full,
                            MotionPreference::Slow,
                            MotionPreference::Reduced,
                            MotionPreference::System,
                        ][motion],
                    );
                }
            });
        if !open {
            crate::runtime::restore_focus(ctx, self.tune_trigger);
        }
        self.tune_open = open;
    }
    pub fn render(&mut self, ctx: &Context) {
        self.process_screenshots(ctx);
        self.rt.begin_frame();
        let hidden = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        if hidden {
            if !self.window_hidden
                && let Some(page) = self.pages.get_mut(&self.rt.route)
            {
                page.leave(&mut self.rt);
            }
            self.window_hidden = true;
            return;
        }
        if self.window_hidden
            && let Some(page) = self.pages.get_mut(&self.rt.route)
        {
            page.enter(&mut self.rt);
        }
        self.window_hidden = false;
        let focused = ctx.input(|i| i.focused);
        if focused && !self.window_focused {
            self.rt.system_reduced = system_reduced();
        }
        self.window_focused = focused;
        self.apply_style(ctx);
        let modal = self.commands.surface.is_visible() || self.guide.visible();
        if !modal {
            let back = ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::OpenBracket))
                    || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::ALT, Key::ArrowLeft))
            });
            let forward = ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(
                    Modifiers::COMMAND,
                    Key::CloseBracket,
                )) || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::ALT, Key::ArrowRight))
            });
            if back {
                self.move_history(-1);
            } else if forward {
                self.move_history(1);
            }
        }
        let width = ctx.content_rect().width();
        let compact = width <= 900.0;
        let immersive = self.rt.route == Route::Workspace;
        let command = ctx
            .input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::K)));
        if command && !self.guide.visible() {
            if self.commands.open {
                self.close_commands(ctx);
            } else {
                self.open_commands(self.command_trigger);
            }
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::L,
            ))
        }) && !self.guide.visible()
        {
            self.rt
                .set_theme(!self.rt.desired_theme.unwrap_or(self.rt.prefs.dark));
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.nav_open || self.tune_open {
                crate::runtime::restore_focus(ctx, None);
            }
            self.nav_open = false;
            self.tune_open = false;
        }
        if !compact && !immersive {
            SidePanel::left("primary-sidebar")
                .exact_width(248.0)
                .show_separator_line(false)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(self.rt.tokens.bg_sunken)
                        .inner_margin(Margin::symmetric(16, 24))
                        .stroke(Stroke::new(1.0, self.rt.tokens.border_soft)),
                )
                .show(ctx, |ui| self.sidebar(ui));
        }
        TopBottomPanel::top("topbar")
            .exact_height(56.0)
            .frame(
                Frame::new()
                    .fill(self.rt.tokens.bg)
                    .inner_margin(Margin::symmetric(if compact { 16 } else { 24 }, 8)),
            )
            .show(ctx, |ui| self.topbar(ui, compact));
        let route_reveal =
            crate::motion::ease(Curve::Out, self.rt.progress(self.route_started, 0.28));
        let mut current_anchors = vec![];
        let mut content_rect = Rect::NOTHING;
        CentralPanel::default()
            .frame(Frame::new().fill(self.rt.tokens.bg))
            .show(ctx, |ui| {
                content_rect = ui.max_rect();
                ui.set_opacity(route_reveal);
                let route = self.rt.route;
                if route == Route::Workspace {
                    if let Some(page) = self.pages.get_mut(&route) {
                        page.show(ui, &mut self.rt);
                    }
                    if self.rt.jump_to.as_ref().is_some_and(|id| {
                        crate::anchors::resolve(route, id, &self.rt.anchors).is_some()
                    }) {
                        self.rt.jump_to = None;
                    }
                } else {
                    let padding = if compact { 16 } else { 56 };
                    ScrollArea::vertical()
                        .id_salt(("page-scroll", route.id()))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            Frame::new()
                                .inner_margin(Margin {
                                    left: padding,
                                    right: if width > 1480.0 { 116 } else { padding },
                                    top: 48 + (8.0 * (1.0 - route_reveal)) as i8,
                                    bottom: 48,
                                })
                                .show(ui, |ui| {
                                    ui.set_max_width(if width > 1480.0 {
                                        (ui.available_width() - 96.0).min(1080.0)
                                    } else {
                                        1080.0
                                    });
                                    if let Some(page) = self.pages.get_mut(&route) {
                                        page.show(ui, &mut self.rt);
                                    }
                                    current_anchors = self.rt.anchors.clone();
                                    if let Some(id) = self.rt.jump_to.as_ref()
                                        && let Some(anchor) =
                                            crate::anchors::resolve(route, id, &self.rt.anchors)
                                    {
                                        ui.scroll_to_rect(anchor.rect, Some(Align::Min));
                                        self.rt.jump_to = None;
                                    }
                                });
                        });
                }
            });
        if width > 1480.0 && !immersive {
            self.section_rail(ctx, &current_anchors, content_rect);
        }
        if self.nav_open && (compact || immersive) {
            let t = self.rt.tokens.clone();
            let result = Modal::new(Id::new("navigation-modal"))
                .area(
                    Modal::default_area(Id::new("navigation-modal"))
                        .anchor(Align2::LEFT_TOP, Vec2::ZERO),
                )
                .frame(
                    Frame::new()
                        .fill(t.bg_sunken)
                        .inner_margin(Margin::symmetric(16, 24)),
                )
                .show(ctx, |ui| {
                    ui.set_min_height(ctx.content_rect().height() - 48.0);
                    self.sidebar(ui);
                });
            if result.should_close() {
                crate::runtime::restore_focus(ctx, self.nav_trigger);
                self.nav_open = false;
            }
        }
        if self.rt.commands_requested {
            self.rt.commands_requested = false;
            self.open_commands(self.command_trigger);
        }
        if self.rt.guide_requested && !self.commands.surface.is_visible() {
            self.rt.guide_requested = false;
            if !self.commands.open {
                self.guide.open(&mut self.rt, None, self.guide_trigger);
            }
        }
        self.show_tune(ctx);
        self.show_commands(ctx);
        self.guide.show(ctx, &mut self.rt);
        for toast in self.rt.toasts.drain(..) {
            self.feedback
                .toast(&toast.title, &toast.desc, &toast.tone, toast.created);
        }
        self.feedback.show(
            ctx,
            self.rt.now,
            &self.rt.tokens,
            self.rt.motion_scale(),
            self.rt.reduced(),
        );
        self.finish_route();
        self.theme_overlay(ctx);
        if let Some(capture) = &mut self.capture {
            let started = *capture.started.get_or_insert(self.rt.now);
            if !capture.requested {
                if self.rt.now - started > 0.9 {
                    capture.requested = true;
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::new(
                        CaptureTag::Artifact(capture.path.clone(), capture.quit),
                    )));
                } else {
                    self.rt.repaint_after(0.1);
                }
            }
        }
    }
    fn finish_route(&mut self) {
        let Some(next) = self.rt.requested.take() else {
            return;
        };
        if next == self.rt.route {
            return;
        }
        crate::runtime::restore_focus(self.rt.context(), None);
        if let Some(page) = self.pages.get_mut(&self.rt.route) {
            page.leave(&mut self.rt);
        }
        self.rt.clear_page_motion();
        self.rt.route = next;
        self.route_started = self.rt.now;
        self.rt.mounted.insert(next);
        if let Some(page) = self.pages.get_mut(&next) {
            page.enter(&mut self.rt);
        }
        if let Some(index) = self.history_target.take() {
            self.history_index = index;
        } else if self.history.get(self.history_index) != Some(&next) {
            self.history.truncate(self.history_index + 1);
            self.history.push(next);
            self.history_index = self.history.len() - 1;
        }
        self.rt.repaint();
    }
    fn move_history(&mut self, direction: isize) {
        let index = self.history_index.saturating_add_signed(direction);
        if index < self.history.len() && index != self.history_index {
            self.history_target = Some(index);
            self.rt.navigate(self.history[index]);
        }
    }
    fn section_rail(&mut self, ctx: &Context, anchors: &[crate::runtime::Anchor], content: Rect) {
        let mut unique = std::collections::HashSet::new();
        let entries: Vec<_> = anchors
            .iter()
            .filter(|a| unique.insert(a.title.clone()))
            .take(16)
            .collect();
        if entries.is_empty() {
            return;
        }
        let x = (content.left() + 1192.0).min(content.right() - 164.0);
        Area::new(Id::new("section-rail"))
            .fixed_pos(pos2(x, 110.0))
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.set_width(144.0);
                ui.label(
                    RichText::new("ON THIS PAGE")
                        .size(10.0)
                        .color(self.rt.tokens.text3),
                );
                ui.add_space(12.0);
                for entry in entries {
                    let response = ui.add(
                        Label::new(RichText::new(&entry.title).size(11.0).color(
                            if entry.rect.top() < 160.0 && entry.rect.bottom() > 100.0 {
                                self.rt.tokens.accent_text
                            } else {
                                self.rt.tokens.text3
                            },
                        ))
                        .sense(Sense::click())
                        .wrap(),
                    );
                    if response.clicked() {
                        self.rt.jump_to = Some(entry.id.clone());
                    }
                    self.rt.record(&format!("anchor:{}", entry.id), &response);
                }
            });
    }
    fn process_screenshots(&mut self, ctx: &Context) {
        for event in ctx.input(|i| i.events.clone()) {
            if let Event::Screenshot {
                user_data, image, ..
            } = event
                && let Some(tag) = user_data
                    .data
                    .as_ref()
                    .and_then(|v| v.downcast_ref::<CaptureTag>())
            {
                match tag {
                    CaptureTag::Theme(generation) => {
                        if let Some((active, target, _)) = self.theme_request
                            && *generation == active
                        {
                            self.theme_texture = Some(ctx.load_texture(
                                "theme-previous-frame",
                                (*image).clone(),
                                TextureOptions::LINEAR,
                            ));
                            self.theme_start = self.rt.now;
                            self.rt.prefs.dark = target;
                            self.theme_seen = target;
                            self.rt.desired_theme = None;
                            self.rt.tokens = crate::tokens::Tokens::new(
                                target,
                                self.rt.prefs.mono,
                                self.rt.prefs.hue,
                                self.rt.prefs.radius,
                            );
                            self.theme_request = None;
                        }
                    }
                    CaptureTag::Artifact(path, quit) => {
                        let rgba: Vec<u8> = image
                            .pixels
                            .iter()
                            .flat_map(|pixel| pixel.to_array())
                            .collect();
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match image::save_buffer(
                            path,
                            &rgba,
                            image.width() as u32,
                            image.height() as u32,
                            image::ColorType::Rgba8,
                        ) {
                            Ok(()) => eprintln!("Captured native frame: {}", path.display()),
                            Err(error) => eprintln!("Native capture failed: {error}"),
                        };
                        if let Ok(report) = serde_json::to_string_pretty(&self.rt.records) {
                            let _ = std::fs::write(path.with_extension("json"), report);
                        }
                        if *quit {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    }
                }
            }
        }
    }
    fn theme_overlay(&mut self, ctx: &Context) {
        if let Some(target) = self.rt.desired_theme {
            if self.rt.reduced() {
                self.rt.prefs.dark = target;
                self.theme_seen = target;
                self.rt.desired_theme = None;
                self.theme_request = None;
                self.theme_texture = None;
                self.rt.repaint();
            } else if self.theme_request.is_none_or(|(_, old, _)| old != target) {
                self.theme_generation += 1;
                self.theme_request = Some((self.theme_generation, target, self.rt.now));
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::new(
                    CaptureTag::Theme(self.theme_generation),
                )));
                self.rt.repaint();
            }
        }
        if let Some((_, target, started)) = self.theme_request
            && self.rt.now - started > 0.25
        {
            self.rt.prefs.dark = target;
            self.rt.desired_theme = None;
            self.theme_request = None;
            self.theme_seen = target;
            self.rt.repaint();
        }
        if self.rt.reduced() {
            self.theme_texture = None;
            return;
        }
        let Some(texture) = self.theme_texture.as_ref() else {
            return;
        };
        let p = self.rt.progress(self.theme_start, 0.65);
        if p >= 1.0 {
            self.theme_texture = None;
            return;
        }
        let screen = ctx.content_rect();
        let origin = self
            .rt
            .theme_origin
            .unwrap_or(pos2(screen.right() - 120.0, screen.top() + 28.0))
            .clamp(screen.min + Vec2::splat(1.0), screen.max - Vec2::splat(1.0));
        let max_radius = [
            screen.left_top(),
            screen.right_top(),
            screen.right_bottom(),
            screen.left_bottom(),
        ]
        .iter()
        .map(|p| p.distance(origin))
        .fold(0.0_f32, f32::max);
        let radius = motion_ease(p) * max_radius;
        let mut mesh = Mesh::with_texture(texture.id());
        let uv = |p: Pos2| {
            pos2(
                (p.x - screen.left()) / screen.width(),
                (p.y - screen.top()) / screen.height(),
            )
        };
        let mut angles: Vec<_> = (0..192)
            .map(|i| i as f32 / 192.0 * std::f32::consts::TAU)
            .collect();
        // Exact corner rays prevent the mask from cutting diagonally across
        // window corners when its origin is near an edge.
        for corner in [
            screen.left_top(),
            screen.right_top(),
            screen.right_bottom(),
            screen.left_bottom(),
        ] {
            let delta = corner - origin;
            angles.push(delta.y.atan2(delta.x).rem_euclid(std::f32::consts::TAU));
        }
        angles.sort_by(f32::total_cmp);
        for i in 0..angles.len() {
            let a = angles[i];
            let b = angles[(i + 1) % angles.len()];
            let d1 = vec2(a.cos(), a.sin());
            let d2 = vec2(b.cos(), b.sin());
            let edge = |d: Vec2| {
                let mut distance = f32::INFINITY;
                for v in [
                    (screen.right() - origin.x) / d.x,
                    (screen.left() - origin.x) / d.x,
                    (screen.bottom() - origin.y) / d.y,
                    (screen.top() - origin.y) / d.y,
                ] {
                    if v >= 0.0 {
                        distance = distance.min(v);
                    }
                }
                distance
            };
            let e1 = edge(d1);
            let e2 = edge(d2);
            let vertices = [
                origin + d1 * radius.min(e1),
                origin + d1 * e1,
                origin + d2 * e2,
                origin + d2 * radius.min(e2),
            ];
            let offset = mesh.vertices.len() as u32;
            for point in vertices {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: point,
                    uv: uv(point),
                    color: Color32::WHITE,
                });
            }
            mesh.indices.extend_from_slice(&[
                offset,
                offset + 1,
                offset + 2,
                offset,
                offset + 2,
                offset + 3,
            ]);
        }
        ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("theme-reveal")))
            .add(Shape::mesh(mesh));
        self.rt.repaint();
    }
}
fn motion_ease(t: f32) -> f32 {
    crate::motion::ease(Curve::InOut, t)
}
impl eframe::App for SchematicApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "schematic-native-preferences", &self.rt.prefs);
    }
}
fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    for (name, path, family) in [
        (
            "native-sans",
            "/System/Library/Fonts/SFNS.ttf",
            FontFamily::Proportional,
        ),
        (
            "native-mono",
            "/System/Library/Fonts/SFNSMono.ttf",
            FontFamily::Monospace,
        ),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.into(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, name.into());
        }
    }
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/HelveticaNeue.ttc") {
        let mut data = FontData::from_owned(bytes);
        data.index = 1;
        fonts
            .font_data
            .insert("native-semibold".into(), data.into());
        fonts.families.insert(
            FontFamily::Name("Schematic Semibold".into()),
            vec!["native-semibold".into(), "native-sans".into()],
        );
        ctx.data_mut(|d| d.insert_temp(Id::new("schematic-native-strong-font"), true));
    }
    ctx.set_fonts(fonts);
}
fn system_reduced() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("defaults")
            .args(["read", "com.apple.universalaccess", "reduceMotion"])
            .output()
            .ok()
            .is_some_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "1")
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(app: &mut SchematicApp, ctx: &Context, time: f64, events: Vec<Event>) -> FullOutput {
        ctx.run(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1440.0, 1000.0))),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| app.render(ctx),
        )
    }
    #[test]
    fn keyboard_theme_reveals_cover_the_viewport_in_both_directions() {
        let ctx = Context::default();
        let mut app = SchematicApp::with_context(
            ctx.clone(),
            Preferences {
                motion: MotionPreference::Full,
                ..Default::default()
            },
        );
        frame(&mut app, &ctx, 0.0, vec![]);
        for (index, dark) in [true, false].into_iter().enumerate() {
            let t = index as f64 + 1.0;
            app.rt.set_theme(dark);
            let first = frame(&mut app, &ctx, t, vec![]);
            let tag = first.viewport_output[&ViewportId::ROOT]
                .commands
                .iter()
                .find_map(|command| {
                    if let ViewportCommand::Screenshot(data) = command {
                        Some(data.clone())
                    } else {
                        None
                    }
                })
                .expect("native old-frame capture request");
            frame(
                &mut app,
                &ctx,
                t + 0.02,
                vec![Event::Screenshot {
                    viewport_id: ViewportId::ROOT,
                    user_data: tag,
                    image: std::sync::Arc::new(ColorImage::filled([64, 64], Color32::RED)),
                }],
            );
            assert_eq!(app.rt.prefs.dark, dark);
            let output = frame(&mut app, &ctx, t + 0.15, vec![]);
            let id = app
                .theme_texture
                .as_ref()
                .expect("old frame during reveal")
                .id();
            let mesh = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    Shape::Mesh(mesh) if mesh.texture_id == id => Some(mesh),
                    _ => None,
                })
                .expect("native reveal mesh");
            let bounds = mesh.vertices.iter().fold(Rect::NOTHING, |rect, v| {
                rect.union(Rect::from_min_max(v.pos, v.pos))
            });
            assert!(
                bounds.width() > 1000.0 && bounds.height() > 900.0,
                "theme overlay collapsed at its origin: {bounds:?}"
            );
            for corner in [
                pos2(0.0, 0.0),
                pos2(1440.0, 0.0),
                pos2(1440.0, 1000.0),
                pos2(0.0, 1000.0),
            ] {
                assert!(
                    mesh.vertices.iter().any(|v| v.pos.distance(corner) < 0.1),
                    "theme overlay must cover corner {corner:?}"
                );
            }
            frame(&mut app, &ctx, t + 0.9, vec![]);
            assert!(app.theme_texture.is_none());
        }
    }
}
