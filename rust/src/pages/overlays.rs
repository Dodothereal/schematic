//! Native implementation of every Overlays & Feedback specimen.
use crate::{
    feedback::{self, Surface, SurfaceKind, ToastId, ToastOptions, TooltipState},
    motion::Curve,
    runtime::{Page, Runtime},
    ui::{self as kit, ButtonKind},
};
use eframe::egui::{
    self, Align, Area, Context, Frame, Id, Key, Layout, Order, Pos2, Rect, Response, RichText,
    Sense, Stroke, Ui, Vec2, pos2, vec2,
};

const TONES: [(&str, &str, &str, &str); 4] = [
    (
        "success",
        "check-circle",
        "Changes saved",
        "Your local specimen is ready to review.",
    ),
    (
        "info",
        "info",
        "A little context",
        "This message adds information without blocking your work.",
    ),
    (
        "warning",
        "alert",
        "Review before sharing",
        "Two reviewers still need access to this draft.",
    ),
    (
        "danger",
        "x-circle",
        "Upload interrupted",
        "This is a demo error. No files have been sent.",
    ),
];
const SINGLE: [(&str, &str); 3] = [
    (
        "When should I use a dialog?",
        "Use a dialog when the current task needs a decision before work can continue. Keep its title specific and its primary action unambiguous.",
    ),
    (
        "Where does focus go?",
        "Move focus inside on open. For destructive actions, start with the safe choice. Return focus to the initiating control on close.",
    ),
    (
        "How much motion is enough?",
        "Enough to explain the change in context. Reduced motion preserves every state and skips the spatial journey.",
    ),
];
const MULTI: [(&str, &str); 3] = [
    (
        "Keyboard",
        "Tab reaches each heading. Enter or Space toggles an answer. Arrow keys, Home and End move between headings.",
    ),
    (
        "Motion",
        "The content track interpolates from zero to one fraction. Opacity follows the disclosure; reversing mid-transition stays smooth.",
    ),
    (
        "Semantics",
        "Each real button exposes its expanded state and controls a named region. Closed content is excluded from navigation.",
    ),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialog {
    Project,
    Rename,
    Delete,
    Drawer,
    Sheet,
}
#[derive(Clone, Debug)]
struct Menu {
    context: bool,
    anchor: Rect,
    point: Option<Pos2>,
    origin: Id,
    index: usize,
    submenu: bool,
    subindex: usize,
    rect: Rect,
    subrect: Rect,
    subanchor: Rect,
    opened: f64,
    focus: bool,
}
#[derive(Clone, Debug)]
struct FloatingExit {
    rect: Rect,
    items: Vec<(&'static str, &'static str)>,
    at: f64,
}
#[derive(Clone, Debug, Default)]
struct SheetDrag {
    start: f32,
    y: f32,
    samples: Vec<(f64, f32)>,
    moved: bool,
}
impl SheetDrag {
    fn move_to(&mut self, now: f64, y: f32) {
        self.y = y - self.start;
        self.moved |= self.y.abs() > 4.;
        self.samples.push((now, y));
        while self.samples.len() > 2 && now - self.samples[0].0 > 0.1 {
            self.samples.remove(0);
        }
    }
    fn velocity(&self, now: f64, y: f32) -> f32 {
        self.samples.first().map_or(0., |(at, y0)| {
            (y - *y0) / (((now - *at) * 1000.) as f32).max(1.)
        })
    }
}
/// Tactile negative drag uses a bounded exponential rubberband, independent of animation speed.
pub fn sheet_resistance(dy: f32) -> f32 {
    if dy < 0. {
        -48. * (1. - (dy / 96.).exp())
    } else {
        dy
    }
}
pub fn sheet_dismiss(dy: f32, height: f32, velocity: f32) -> bool {
    dy > height * 0.28 || (dy > 16. && velocity > 0.65)
}

pub struct State {
    pub workspace: String,
    rename: String,
    rename_error: bool,
    pub removed: bool,
    focus_delete: bool,
    banner_restore_id: Option<Id>,
    pub favorite: bool,
    pub copies: u32,
    pub density: usize,
    pub annotations: bool,
    draft_density: usize,
    draft_annotations: bool,
    pub result: String,
    rename_result: String,
    inspector_result: String,
    pub status: String,
    pub dialog: Option<Dialog>,
    surface: Surface,
    pending_action: Option<(&'static str, Rect, Id)>,
    menu: Option<Menu>,
    floating_exits: Vec<FloatingExit>,
    card_open: bool,
    card_motion: feedback::Tween,
    card_since: Option<f64>,
    card_exit: Option<f64>,
    card_anchor: Rect,
    card_origin: Option<Id>,
    card_rect: Rect,
    card_suppress: bool,
    card_focus: bool,
    toolbar: usize,
    toolbar_ids: [Option<Id>; 4],
    tooltip: TooltipState,
    pub single: [bool; 3],
    pub multi: [bool; 3],
    pub more: bool,
    pub banner: bool,
    pub loading_running: bool,
    pub loading_elapsed: f64,
    pub loading_speed: f64,
    last_tick: Option<f64>,
    pub loading_status: String,
    promise: Option<(f64, ToastId)>,
    pub promise_status: String,
    pub feedback: feedback::State,
    drag: Option<SheetDrag>,
    spring: feedback::Tween,
    suppress_sheet_click: bool,
    visible: bool,
    last_screen: Rect,
}
impl Default for State {
    fn default() -> Self {
        Self {
            workspace: "Atlas workspace".into(),
            rename: "Atlas workspace".into(),
            rename_error: false,
            removed: false,
            focus_delete: false,
            banner_restore_id: None,
            favorite: false,
            copies: 0,
            density: 0,
            annotations: true,
            draft_density: 0,
            draft_annotations: true,
            result: "Choose an action to try it.".into(),
            rename_result: "Edits stay in this page.".into(),
            inspector_result: "Current: Comfortable · annotations on".into(),
            status: String::new(),
            dialog: None,
            surface: Surface::new("ov-dialog", SurfaceKind::Dialog),
            pending_action: None,
            menu: None,
            floating_exits: Vec::new(),
            card_open: false,
            card_motion: feedback::Tween::new(0.),
            card_since: None,
            card_exit: None,
            card_anchor: Rect::NOTHING,
            card_origin: None,
            card_rect: Rect::NOTHING,
            card_suppress: false,
            card_focus: false,
            toolbar: 0,
            toolbar_ids: [None; 4],
            tooltip: TooltipState::default(),
            single: [true, false, false],
            multi: [true, false, false],
            more: false,
            banner: true,
            loading_running: false,
            loading_elapsed: 0.,
            loading_speed: 1.,
            last_tick: None,
            loading_status: "Idle · press Run simulation to begin.".into(),
            promise: None,
            promise_status: "Local simulation · no network".into(),
            feedback: feedback::State::default(),
            drag: None,
            spring: feedback::Tween::new(0.),
            suppress_sheet_click: false,
            visible: false,
            last_screen: Rect::NOTHING,
        }
    }
}
impl State {
    pub fn loading_value(&self) -> u32 {
        let points = [
            (0., 0.),
            (0.14, 16.),
            (0.3, 32.),
            (0.48, 51.),
            (0.63, 68.),
            (0.85, 88.),
            (1., 100.),
        ];
        let t = (self.loading_elapsed / 8.).clamp(0., 1.);
        let mut i = 1;
        while i < points.len() - 1 && t > points[i].0 {
            i += 1;
        }
        let (a, av) = points[i - 1];
        let (b, bv) = points[i];
        (av + (bv - av) * (t - a) / (b - a)).round() as u32
    }
    pub fn loading_phase(&self) -> usize {
        match self.loading_value() {
            100.. => 3,
            68.. => 2,
            32.. => 1,
            _ => 0,
        }
    }
    pub fn start_loading(&mut self, now: f64, reset: bool) {
        if reset || self.loading_elapsed >= 8. - 1e-9 {
            self.loading_elapsed = 0.;
        }
        self.loading_running = true;
        self.last_tick = Some(now);
        self.loading_status = "Step 1 of 3: reading specimen. Local simulation.".into();
    }
    pub fn pause_loading(&mut self, reason: &str) {
        if self.loading_running {
            self.loading_status = reason.into();
        }
        self.loading_running = false;
        self.last_tick = None;
    }
    /// Real time is never divided by visual motion scale.
    pub fn advance(&mut self, now: f64) {
        if self.loading_running {
            let previous = self.loading_phase();
            if let Some(last) = self.last_tick {
                self.loading_elapsed =
                    (self.loading_elapsed + (now - last).max(0.) * self.loading_speed).min(8.);
            }
            self.last_tick = Some(now);
            let phase = self.loading_phase();
            if phase != previous {
                self.loading_status = if phase == 3 {
                    "Complete · local preview is ready.".into()
                } else {
                    format!(
                        "Step {} of 3: {}. Local simulation.",
                        phase + 1,
                        ["reading specimen", "resolving tokens", "composing preview"][phase]
                    )
                };
            }
            if self.loading_elapsed >= 8. - 1e-9 {
                self.loading_running = false;
                self.last_tick = None;
            }
        }
        if let Some((start, pending)) = self.promise
            && now - start >= 1.8
        {
            self.feedback.dismiss(pending);
            self.promise = None;
            self.feedback.toast(
                "Example saved",
                "The local simulation completed successfully.",
                "success",
                now,
            );
            self.promise_status = "Local simulation complete. Run it again anytime.".into();
        }
    }
    pub fn start_promise(&mut self, now: f64) {
        if self.promise.is_some() {
            return;
        }
        let id = self.feedback.push(
            ToastOptions {
                title: "Saving example…".into(),
                desc: "Local simulation. No data is sent.".into(),
                icon: "loader".into(),
                duration: None,
                ..Default::default()
            },
            now,
        );
        self.promise = Some((now, id));
        self.promise_status = "Simulating a save…".into();
    }
    pub fn suspend(&mut self, ctx: &Context, now: f64, reason: &str) {
        self.pause_loading(reason);
        if let Some((_, id)) = self.promise.take() {
            self.feedback.dismiss(id);
            self.promise_status = "Simulation canceled while away. Replay anytime.".into();
        }
        self.feedback.clear_with_focus(ctx);
        self.menu = None;
        self.floating_exits.clear();
        self.card_open = false;
        self.card_motion.snap(0.);
        self.card_since = None;
        self.card_exit = None;
        self.drag = None;
        self.spring.snap(0.);
        self.surface.close_immediately(ctx, false);
        self.dialog = None;
        self.pending_action = None;
        self.tooltip.hide(now);
    }
    pub fn rename_workspace(&mut self) -> bool {
        let name = self.rename.trim().chars().take(40).collect::<String>();
        if name.is_empty() {
            self.rename_error = true;
            return false;
        }
        self.rename = name.clone();
        self.workspace = name;
        self.rename_error = false;
        self.rename_result = "Name saved in this specimen.".into();
        self.status = format!("Workspace renamed to {}.", self.workspace);
        true
    }
    pub fn open(&mut self, dialog: Dialog, rect: Rect, id: Id, now: f64) {
        if let Some(menu) = self.menu.take() {
            self.queue_menu_exit(&menu, now);
        }
        self.card_open = false;
        self.card_since = None;
        self.dialog = Some(dialog);
        self.surface.kind = match dialog {
            Dialog::Project => SurfaceKind::Morph,
            Dialog::Drawer => SurfaceKind::Drawer,
            Dialog::Sheet => SurfaceKind::Sheet,
            _ => SurfaceKind::Dialog,
        };
        self.surface.size = match dialog {
            Dialog::Project => vec2(600., 500.),
            Dialog::Drawer => vec2(420., 600.),
            Dialog::Sheet => vec2(560., 520.),
            _ => vec2(480., 360.),
        };
        self.surface.show_dialog(Some(rect), Some(id), now);
        self.drag = None;
        self.spring.snap(0.);
        if dialog == Dialog::Drawer {
            self.draft_density = self.density;
            self.draft_annotations = self.annotations;
        }
    }
    fn action(&mut self, action: &str, rect: Rect, id: Id, now: f64) {
        match action {
            "rename" => self.open(Dialog::Rename, rect, id, now),
            "delete" => {
                if self.removed {
                    self.status = "The example draft is already removed. Use Restore draft.".into();
                } else {
                    self.open(Dialog::Delete, rect, id, now);
                }
            }
            "inspect" => self.open(Dialog::Project, rect, id, now),
            "duplicate" => {
                self.copies += 1;
                self.result = format!(
                    "Local copy {} created for this interaction demo.",
                    self.copies
                );
                self.status = self.result.clone();
            }
            "favorite" => {
                self.favorite = !self.favorite;
                self.result = if self.favorite {
                    "Specimen marked as favorite."
                } else {
                    "Favorite removed."
                }
                .into();
                self.status = self.result.clone();
            }
            "link" => {
                self.result = "Example path: atlas / specimen.svg. No link was shared.".into();
                self.status = self.result.clone();
            }
            _ => {
                self.result = format!(
                    "{} selected. Export is simulated; no file was written.",
                    match action {
                        "svg" => "SVG markup",
                        "tokens" => "Design tokens",
                        _ => "Component spec",
                    }
                );
                self.status = self.result.clone();
            }
        }
    }
    fn open_menu(
        &mut self,
        context: bool,
        response: &Response,
        point: Option<Pos2>,
        last: bool,
        now: f64,
    ) {
        self.card_open = false;
        self.card_since = None;
        self.floating_exits.clear();
        self.menu = Some(Menu {
            context,
            anchor: response.rect,
            point,
            origin: response.id,
            index: if last { if context { 2 } else { 3 } } else { 0 },
            submenu: false,
            subindex: 0,
            rect: Rect::NOTHING,
            subrect: Rect::NOTHING,
            subanchor: Rect::NOTHING,
            opened: now,
            focus: true,
        });
    }
    fn queue_menu_exit(&mut self, menu: &Menu, now: f64) {
        if menu.rect.is_finite() {
            let items = if menu.context {
                vec![
                    ("eye", "Inspect specimen"),
                    ("copy", "Duplicate specimen"),
                    ("star", "Mark as favorite"),
                ]
            } else {
                vec![
                    ("pencil", "Rename workspace"),
                    ("copy", "Duplicate specimen"),
                    ("download", "Export as"),
                    ("trash", "Remove draft…"),
                ]
            };
            self.floating_exits.push(FloatingExit {
                rect: menu.rect,
                items,
                at: now,
            });
        }
        if menu.submenu && menu.subrect.is_finite() {
            self.floating_exits.push(FloatingExit {
                rect: menu.subrect,
                items: vec![
                    ("code", "SVG markup"),
                    ("document", "Design tokens"),
                    ("copy", "Component spec"),
                ],
                at: now,
            });
        }
    }
    fn paint_floating_exits(&mut self, ctx: &Context, rt: &mut Runtime) {
        self.floating_exits
            .retain(|e| !rt.reduced() && rt.now - e.at < 0.14 * rt.motion_scale() as f64);
        for (n, exit) in self.floating_exits.iter().enumerate() {
            let p = 1. - crate::motion::ease(Curve::In, rt.progress(exit.at, 0.14));
            let pos = exit.rect.min - vec2(0., 4. * (1. - p));
            Area::new(Id::new(("ov-menu-exit", n)))
                .order(Order::Foreground)
                .interactable(false)
                .fade_in(false)
                .fixed_pos(pos)
                .show(ctx, |ui| {
                    ui.set_width(exit.rect.width());
                    ui.set_opacity(p);
                    ui.disable();
                    frame(rt).inner_margin(6.).show(ui, |ui| {
                        for (icon, label) in &exit.items {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(16., 34.), Sense::hover());
                                crate::icons::paint(
                                    ui.painter(),
                                    icon,
                                    Rect::from_center_size(r.center(), vec2(16., 16.)),
                                    rt.tokens.text2,
                                    1.6,
                                    0.,
                                );
                                ui.label(RichText::new(*label).size(13.).color(rt.tokens.text2));
                            });
                        }
                    });
                });
        }
    }
    fn menu_trigger(&mut self, ui: &mut Ui, rt: &mut Runtime, context: bool) -> Response {
        let r = kit::button(
            ui,
            rt,
            if context {
                "ov-context-trigger"
            } else {
                "ov-actions-trigger"
            },
            if context {
                "Open context menu"
            } else {
                "Actions"
            },
            Some(if context { "more" } else { "chevron-down" }),
            ButtonKind::Secondary,
        );
        if r.clicked() {
            if self.menu.as_ref().is_some_and(|m| m.origin == r.id) {
                if let Some(menu) = self.menu.take() {
                    self.queue_menu_exit(&menu, rt.now);
                }
                r.request_focus();
            } else {
                self.open_menu(context, &r, None, false, rt.now);
            }
        }
        if r.has_focus() {
            let down = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowDown));
            let up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowUp));
            if down || up {
                self.open_menu(context, &r, None, up, rt.now);
            }
        }
        ui.ctx().accesskit_node_builder(r.id, |node| {
            node.set_has_popup(egui::accesskit::HasPopup::Menu);
            node.set_expanded(self.menu.as_ref().is_some_and(|m| m.origin == r.id));
        });
        r
    }
    fn origin_card(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("YOUR WORKSPACE")
                    .size(11.)
                    .color(rt.tokens.text3),
            );
            kit::muted(ui, rt, "Select a project to inspect");
        });
        let wide = ui.available_width() > 680.;
        let width = ui.available_width().min(380.);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(width);
                let width = ui.available_width().min(380.);
                let (rect, r) = ui.allocate_exact_size(vec2(width, 204.), Sense::click());
                let t = rt.tokens.clone();
                let hover = rt.animate(
                    "ov-project-hover",
                    if r.hovered() || r.has_focus() { 1. } else { 0. },
                    0.22,
                    Curve::Out,
                );
                ui.painter()
                    .rect_filled(rect, t.radius * 12., t.surface_raised);
                ui.painter().rect_stroke(
                    rect,
                    t.radius * 12.,
                    Stroke::new(1., if r.has_focus() { t.accent } else { t.border }),
                    egui::StrokeKind::Inside,
                );
                let art = Rect::from_min_size(rect.min, vec2(rect.width(), 140.));
                project_art(ui, rt, art, hover);
                let name = ui.painter().layout(
                    self.workspace.clone(),
                    egui::FontId::proportional(15.),
                    rt.tokens.text,
                    rect.width() - 32.,
                );
                ui.painter().galley(
                    pos2(rect.left() + 16., rect.bottom() - 58.),
                    name,
                    rt.tokens.text,
                );
                ui.painter().text(
                    pos2(rect.left() + 16., rect.bottom() - 22.),
                    egui::Align2::LEFT_CENTER,
                    "Design foundations · 12 components",
                    egui::FontId::proportional(12.),
                    rt.tokens.text3,
                );
                r.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        true,
                        format!("Open {} project details", self.workspace),
                    )
                });
                rt.record("ov-open-project", &r);
                if r.clicked() {
                    self.open(Dialog::Project, r.rect, r.id, rt.now);
                }
                kit::muted(ui, rt, "Enter to open / Esc to return");
            });
            if wide {
                ui.add_space(32.);
                ui.vertical(|ui| {
                    ui.set_max_width(260.);
                    ui.add_space(32.);
                    let (r, _) = ui.allocate_exact_size(vec2(28., 28.), Sense::hover());
                    crate::icons::paint(ui.painter(), "layers", r, rt.tokens.accent_text, 1.6, 0.);
                    ui.heading("Project details");
                    kit::muted(
                        ui,
                        rt,
                        "The surface starts where your attention already is.",
                    );
                    kit::muted(ui, rt, "Enter to open / Esc to return");
                });
            }
        });
    }

    fn toolbar(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let mut chosen = None;
        let tools = [
            ("copy", "Duplicate", "duplicate"),
            ("star", "Favorite", "favorite"),
            ("link", "Inspect link", "link"),
            ("info", "Show details", "inspect"),
        ];
        ui.horizontal(|ui| {
            for (n, (icon, label, action)) in tools.iter().enumerate() {
                let id = ui.make_persistent_id(("ov-tool", n));
                let (_, rect) = ui.allocate_space(vec2(34., 34.));
                let sense = if n == self.toolbar {
                    Sense::click()
                } else {
                    Sense::CLICK
                };
                let r = ui.interact(rect, id, sense);
                self.toolbar_ids[n] = Some(id);
                r.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::Button,
                        ui.is_enabled(),
                        *action == "favorite" && self.favorite,
                        *label,
                    )
                });
                let h = rt.animate(
                    (id, "hover"),
                    if r.hovered() || r.has_focus() { 1. } else { 0. },
                    0.14,
                    Curve::Out,
                );
                let press = rt.animate(
                    (id, "press"),
                    if r.is_pointer_button_down_on() {
                        1.
                    } else {
                        0.
                    },
                    0.08,
                    Curve::Out,
                );
                let drawn = rect.shrink(press * 0.6);
                ui.painter().rect_filled(
                    drawn,
                    rt.tokens.radius * 6.,
                    rt.tokens.surface2.gamma_multiply(h),
                );
                let p = rt.icon_progress((id, "icon"), r.hovered() || r.has_focus());
                crate::icons::paint(
                    ui.painter(),
                    icon,
                    Rect::from_center_size(drawn.center(), vec2(18., 18.)),
                    rt.tokens.text2,
                    1.6,
                    p,
                );
                if r.has_focus() {
                    ui.painter().rect_stroke(
                        drawn.expand(2.),
                        rt.tokens.radius * 8.,
                        Stroke::new(2., rt.tokens.accent_ring),
                        egui::StrokeKind::Outside,
                    );
                }
                rt.record(&format!("ov-tool-{action}"), &r);
                self.tooltip
                    .show(ui.ctx(), &r, label, &[], rt.now, &rt.tokens, rt.reduced());
                if r.has_focus() {
                    self.toolbar = n;
                    let key = take_navigation(
                        ui,
                        &[Key::ArrowRight, Key::ArrowLeft, Key::Home, Key::End],
                    );
                    if let Some(key) = key {
                        self.toolbar = navigate_index(key, n, 4, Key::ArrowRight);
                        if let Some(id) = self.toolbar_ids[self.toolbar] {
                            ui.memory_mut(|m| m.request_focus(id));
                        }
                    }
                }
                if r.clicked() {
                    self.toolbar = n;
                    r.request_focus();
                    chosen = Some((*action, r.rect, r.id));
                }
                if *action == "favorite" && self.favorite {
                    ui.painter().rect_stroke(
                        r.rect.shrink(1.),
                        rt.tokens.radius * 6.,
                        Stroke::new(1., rt.tokens.accent),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        });
        self.tooltip.finish(
            ui.ctx(),
            rt.now,
            &rt.tokens,
            rt.motion_scale(),
            rt.reduced(),
        );
        if let Some((action, rect, id)) = chosen {
            self.action(action, rect, id, rt.now);
        }
        kit::muted(ui, rt, &self.result);
    }
    fn show_card(&mut self, ctx: &Context, rt: &mut Runtime) {
        self.card_motion.to(
            if self.card_open { 1. } else { 0. },
            rt.now,
            if rt.reduced() {
                0.
            } else {
                (if self.card_open { 0.32 } else { 0.14 }) * rt.motion_scale()
            },
            if self.card_open {
                Curve::OutExpo
            } else {
                Curve::In
            },
        );
        let p = self.card_motion.sample(rt.now);
        if self.card_motion.moving(rt.now) {
            rt.repaint();
        }
        if p <= 0.001 && !self.card_open {
            return;
        }
        let screen = ctx.content_rect();
        let size = vec2(
            300_f32.min(screen.width() - 24.),
            if self.card_rect.is_finite() {
                self.card_rect.height()
            } else {
                236.
            },
        );
        let pos = place_popup(screen, self.card_anchor, size, None, false);
        let pos = pos + vec2(0., if rt.reduced() { 0. } else { 6. * (1. - p) });
        let mut close = false;
        let mut keep_focus = false;
        let result = Area::new(Id::new("ov-person-card"))
            .order(Order::Foreground)
            .fixed_pos(pos)
            .fade_in(false)
            .show(ctx, |ui| {
                ui.set_width(size.x);
                ui.set_opacity(p);
                if !self.card_open {
                    ui.disable();
                }
                frame(rt).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("MC").size(24.).color(rt.tokens.accent_text));
                        ui.vertical(|ui| {
                            ui.strong("Maya Chen");
                            kit::muted(ui, rt, "@maya · example profile");
                        });
                    });
                    ui.add_space(12.);
                    kit::muted(
                        ui,
                        rt,
                        "Designing calm interfaces, one deliberate transition at a time.",
                    );
                    ui.horizontal(|ui| {
                        ui.label("24 components");
                        ui.label("8 reviews");
                    });
                    let r = kit::button(
                        ui,
                        rt,
                        "ov-profile-action",
                        "Mark review as read",
                        Some("check"),
                        ButtonKind::Secondary,
                    );
                    keep_focus = r.has_focus();
                    if r.clicked() {
                        self.status = "Maya's example review marked as read.".into();
                        close = true;
                    }
                });
            });
        self.card_rect = result.response.rect;
        let hovered = result.response.contains_pointer()
            || ctx
                .pointer_hover_pos()
                .is_some_and(|p| self.card_anchor.contains(p));
        let focused = keep_focus || self.card_focus;
        let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        if hovered || focused {
            self.card_exit = None;
        } else if self.card_exit.is_none() {
            self.card_exit = Some(rt.now);
        }
        if let Some(at) = self.card_exit {
            if rt.now - at >= 0.18 {
                close = true;
            } else {
                rt.repaint_after(0.18 - (rt.now - at));
            }
        }
        if ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !self.card_anchor.contains(p) && !self.card_rect.contains(p))
        }) {
            close = true;
        }
        if close || escape {
            self.card_open = false;
            self.card_since = None;
            self.card_suppress = true;
            if let Some(id) = self.card_origin {
                crate::runtime::restore_focus(ctx, Some(id));
            }
        }
    }
    fn accordion(&mut self, ui: &mut Ui, rt: &mut Runtime, single: bool) {
        let items = if single { SINGLE } else { MULTI };
        let key = if single { "single" } else { "multi" };
        let mut focused = None;
        let mut ids = Vec::new();
        for (n, (title, desc)) in items.iter().enumerate() {
            let is_open = if single {
                self.single[n]
            } else {
                self.multi[n]
            };
            let r = kit::button(
                ui,
                rt,
                ("ov-accordion", key, n),
                title,
                Some(if is_open {
                    "chevron-down"
                } else {
                    "chevron-right"
                }),
                ButtonKind::Ghost,
            );
            ids.push(r.id);
            if r.clicked() {
                if single {
                    let next = !self.single[n];
                    self.single = [false; 3];
                    self.single[n] = next;
                } else {
                    self.multi[n] = !self.multi[n];
                }
            }
            if r.has_focus() {
                focused = Some(n);
            }
            let is_open = if single {
                self.single[n]
            } else {
                self.multi[n]
            };
            ui.ctx()
                .accesskit_node_builder(r.id, |node| node.set_expanded(is_open));
            let p = rt.animate(
                ("ov-collapse", key, n),
                if is_open { 1. } else { 0. },
                0.36,
                Curve::Out,
            );
            collapse(
                ui,
                rt,
                ("ov-collapse-body", key, n),
                p,
                is_open,
                |ui, rt| {
                    kit::muted(ui, rt, desc);
                },
            );
        }
        if let Some(index) = focused
            && let Some(key) =
                take_navigation(ui, &[Key::ArrowDown, Key::ArrowUp, Key::Home, Key::End])
        {
            let next = navigate_index(key, index, 3, Key::ArrowDown);
            ui.memory_mut(|m| m.request_focus(ids[next]));
        }
    }
    fn menus(&mut self, ctx: &Context, rt: &mut Runtime) {
        let Some(mut menu) = self.menu.take() else {
            return;
        };
        let screen = ctx.content_rect();
        let entries: &[(&str, &str, &str)] = if menu.context {
            &[
                ("eye", "Inspect specimen", "inspect"),
                ("copy", "Duplicate specimen", "duplicate"),
                ("star", "Mark as favorite", "favorite"),
            ]
        } else {
            &[
                ("pencil", "Rename workspace", "rename"),
                ("copy", "Duplicate specimen", "duplicate"),
                ("download", "Export as", "export"),
                ("trash", "Remove draft…", "delete"),
            ]
        };
        let mut close = false;
        let mut action = None;
        let mut restore = false;
        let mut request_focus = menu.focus;
        menu.focus = false;
        let keys = ctx.input(|i| i.events.clone());
        for event in keys {
            match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => match key {
                    Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End => {
                        let index = if menu.submenu {
                            &mut menu.subindex
                        } else {
                            &mut menu.index
                        };
                        *index = navigate_index(
                            key,
                            *index,
                            if menu.submenu { 3 } else { entries.len() },
                            Key::ArrowDown,
                        );
                        request_focus = true;
                        consume(ctx, key, modifiers);
                    }
                    Key::ArrowRight if !menu.context && menu.index == 2 => {
                        menu.submenu = true;
                        menu.subindex = 0;
                        request_focus = true;
                        consume(ctx, key, modifiers);
                    }
                    Key::ArrowLeft if menu.submenu => {
                        menu.submenu = false;
                        request_focus = true;
                        consume(ctx, key, modifiers);
                    }
                    Key::Escape => {
                        if menu.submenu {
                            menu.submenu = false;
                            request_focus = true;
                        } else {
                            close = true;
                            restore = true;
                        }
                        consume(ctx, key, modifiers);
                    }
                    Key::Tab => {
                        close = true;
                        restore = true;
                    }
                    Key::Enter | Key::Space => {
                        action = Some(if menu.submenu {
                            ["svg", "tokens", "spec"][menu.subindex]
                        } else {
                            entries[menu.index].2
                        });
                        consume(ctx, key, modifiers);
                    }
                    _ => {}
                },
                egui::Event::Text(text) if text.chars().count() == 1 && !text.trim().is_empty() => {
                    let labels: Vec<&str> = if menu.submenu {
                        vec!["SVG markup", "Design tokens", "Component spec"]
                    } else {
                        entries.iter().map(|e| e.1).collect()
                    };
                    let index = if menu.submenu {
                        &mut menu.subindex
                    } else {
                        &mut menu.index
                    };
                    for n in 1..=labels.len() {
                        let next = (*index + n) % labels.len();
                        if labels[next]
                            .to_lowercase()
                            .starts_with(&text.to_lowercase())
                        {
                            *index = next;
                            request_focus = true;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        let size = if menu.rect.is_finite() {
            menu.rect.size()
        } else {
            vec2(
                240_f32.min(screen.width() - 24.),
                entries.len() as f32 * 42. + 24.,
            )
        };
        let pos = place_popup(screen, menu.anchor, size, menu.point, false);
        let p = rt.progress(menu.opened, 0.32);
        let p = crate::motion::ease(Curve::OutExpo, p);
        let output = Area::new(Id::new("ov-actions-menu"))
            .order(Order::Foreground)
            .fixed_pos(pos + vec2(0., if rt.reduced() { 0. } else { -4. * (1. - p) }))
            .fade_in(false)
            .show(ctx, |ui| {
                ui.set_width(size.x);
                ui.set_opacity(p);
                frame(rt).inner_margin(6.).show(ui, |ui| {
                    for (n, (icon, label, value)) in entries.iter().enumerate() {
                        if !menu.context && n == 3 {
                            ui.separator();
                        }
                        let r = kit::button(
                            ui,
                            rt,
                            ("ov-menuitem", n),
                            label,
                            Some(icon),
                            if n == menu.index && !menu.submenu {
                                ButtonKind::Soft
                            } else {
                                ButtonKind::Ghost
                            },
                        );
                        rt.record(&format!("ov-menu-{value}"), &r);
                        if n == 2 && !menu.context {
                            menu.subanchor = r.rect;
                        }
                        if request_focus && !menu.submenu && n == menu.index {
                            r.request_focus();
                        }
                        if r.clicked() {
                            action = Some(*value);
                        }
                    }
                });
            });
        menu.rect = output.response.rect;
        if menu.submenu {
            let size = if menu.subrect.is_finite() {
                menu.subrect.size()
            } else {
                vec2(240., 150.)
            };
            let pos = place_popup(screen, menu.subanchor, size, None, true);
            let out = Area::new(Id::new("ov-export-menu"))
                .order(Order::Foreground)
                .fixed_pos(pos)
                .fade_in(false)
                .show(ctx, |ui| {
                    ui.set_width(size.x);
                    frame(rt).inner_margin(6.).show(ui, |ui| {
                        for (n, (label, value)) in [
                            ("SVG markup", "svg"),
                            ("Design tokens", "tokens"),
                            ("Component spec", "spec"),
                        ]
                        .iter()
                        .enumerate()
                        {
                            let r = kit::button(
                                ui,
                                rt,
                                ("ov-export", n),
                                label,
                                Some(if n == 0 {
                                    "code"
                                } else if n == 1 {
                                    "document"
                                } else {
                                    "copy"
                                }),
                                if n == menu.subindex {
                                    ButtonKind::Soft
                                } else {
                                    ButtonKind::Ghost
                                },
                            );
                            rt.record(&format!("ov-export-{value}"), &r);
                            if request_focus && n == menu.subindex {
                                r.request_focus();
                            }
                            if r.clicked() {
                                action = Some(*value);
                            }
                        }
                    });
                });
            menu.subrect = out.response.rect;
        }
        if let Some(selected) = action {
            if selected == "export" {
                menu.submenu = true;
                menu.subindex = 0;
                menu.focus = true;
            } else {
                close = true;
                restore = true;
                self.action(selected, menu.anchor, menu.origin, rt.now);
            }
        }
        let outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer.interact_pos().is_some_and(|p| {
                    !menu.rect.contains(p)
                        && (!menu.submenu || !menu.subrect.contains(p))
                        && !menu.anchor.contains(p)
                })
        });
        if outside {
            close = true;
            restore = false;
        }
        if close {
            self.queue_menu_exit(&menu, rt.now);
            if restore {
                crate::runtime::restore_focus(ctx, Some(menu.origin));
            }
        } else {
            self.menu = Some(menu);
        }
    }
    fn dialogs(&mut self, ctx: &Context, rt: &mut Runtime) {
        let Some(dialog) = self.dialog else {
            return;
        };
        let mut surface = std::mem::replace(
            &mut self.surface,
            Surface::new("ov-dialog", SurfaceKind::Dialog),
        );
        if self.drag.is_none() {
            surface.drag_y = self.spring.sample(rt.now);
            if self.spring.moving(rt.now) {
                rt.repaint();
            }
        }
        let sheet_height = if surface.rect.is_finite() {
            surface.rect.height()
        } else {
            surface.size.y
        };
        let mut close = false;
        let mut new_origin = None;
        let mut sheet_action = None;
        let mut focus_order = Vec::new();
        let t = rt.tokens.clone();
        let output=surface.show(ctx,rt.now,&t,rt.motion_scale(),rt.reduced(),|ui,first|{
            ui.horizontal_top(|ui|{ui.vertical(|ui|{ui.label(RichText::new("INTERACTIVE SPECIMEN").size(11.).color(if dialog==Dialog::Delete{rt.tokens.danger_text}else{rt.tokens.text3}));ui.heading(match dialog{Dialog::Project=>self.workspace.as_str(),Dialog::Rename=>"Rename workspace",Dialog::Delete=>"Remove draft?",Dialog::Drawer=>"Project inspector",Dialog::Sheet=>"Quick actions"});});
                let r=kit::icon_button(ui,rt,"ov-dialog-close","x","Close dialog");focus_order.push(r.id);if first&&matches!(dialog,Dialog::Project|Dialog::Drawer){r.request_focus();}
if r.clicked(){close=true;}
            });ui.add_space(12.);
            match dialog{
                Dialog::Project=>{
                    kit::muted(ui,rt,"A reference kit for interfaces that feel calm, connected, and clear.");let(rect,_)=ui.allocate_exact_size(vec2(ui.available_width(),140.),Sense::hover());project_art(ui,rt,rect,1.);
                    ui.horizontal(|ui|{ui.vertical(|ui|{kit::muted(ui,rt,"COMPONENTS");ui.strong("12");});ui.add_space(24.);ui.vertical(|ui|{kit::muted(ui,rt,"STATUS");ui.strong("In review");});ui.add_space(24.);ui.vertical(|ui|{kit::muted(ui,rt,"UPDATED");ui.strong("Today");});});
                    kit::muted(ui,rt,"The small card and this surface share an origin. Closing returns the surface to the card, preserving your place in the workspace.");let r=kit::button(ui,rt,"ov-project-back","Back to workspace",Some("arrow-left"),ButtonKind::Primary);focus_order.push(r.id);close|=r.clicked();
                },
                Dialog::Rename=>{
                    kit::muted(ui,rt,"Choose a name that your team will recognize. This changes only the local demo.");ui.label("Workspace name");
                    let mut output=egui::TextEdit::singleline(&mut self.rename).id(Id::new("ov-workspace-input")).char_limit(40).desired_width(f32::INFINITY).show(ui);rt.record("ov-workspace-input",&output.response);focus_order.push(output.response.id);
                    if first{output.response.request_focus();let range=egui::text::CCursorRange::two(egui::text::CCursor::new(0),egui::text::CCursor::new(self.rename.chars().count()));output.state.cursor.set_char_range(Some(range));output.state.store(ctx,output.response.id);}
                    if output.response.changed(){self.rename_error=false;}kit::muted(ui,rt,"1–40 characters. Stored only in this page.");if self.rename_error{ui.colored_label(rt.tokens.danger_text,"Enter a workspace name.");}
                    ui.separator();ui.horizontal(|ui|{let cancel=kit::button(ui,rt,"ov-rename-cancel","Cancel",None,ButtonKind::Secondary);focus_order.push(cancel.id);if cancel.clicked(){close=true;}
                        let r=kit::button(ui,rt,"ov-rename-save","Save name",Some("check"),ButtonKind::Primary);focus_order.push(r.id);let submit=r.clicked()||((output.response.has_focus()||output.response.lost_focus())&&ui.input(|i|i.key_pressed(Key::Enter)));if submit{if self.rename_workspace(){close=true;}else{output.response.request_focus();}}
                    });
                },
                Dialog::Delete=>{
                    kit::muted(ui,rt,"Remove the example draft from this specimen. This does not delete any real files.");ui.add_space(12.);ui.strong("Exploration draft");kit::muted(ui,rt,"1 local demo draft · you can restore it anytime");ui.separator();
                    ui.horizontal(|ui|{let safe=kit::button(ui,rt,"ov-delete-cancel","Keep draft",None,ButtonKind::Secondary);focus_order.push(safe.id);if first{safe.request_focus();}close|=safe.clicked();let remove=kit::button(ui,rt,"ov-delete-confirm","Remove draft",Some("trash"),ButtonKind::Danger);focus_order.push(remove.id);if remove.clicked(){self.removed=true;self.status="Example draft removed. Restore draft is available.".into();new_origin=Some(Id::new("ov-restore-draft"));close=true;}});
                },
                Dialog::Drawer=>{
                    kit::muted(ui,rt,"Edit a local preview preference. Your saved choice stays with this page.");ui.add_space(16.);ui.label(RichText::new("WORKSPACE").size(11.).color(rt.tokens.text3));ui.strong(&self.workspace);kit::muted(ui,rt,"12 components · design system");ui.separator();ui.label(RichText::new("PREVIEW DENSITY").size(11.).color(rt.tokens.text3));
                    for(n,(label,desc))in [("Comfortable","More room to scan and compare."),("Compact","More components in the same space.")].iter().enumerate(){let r=ui.radio_value(&mut self.draft_density,n,*label);focus_order.push(r.id);rt.record(&format!("ov-density-{n}"),&r);kit::muted(ui,rt,desc);}
                    let annotations=kit::checkbox(ui,rt,"ov-annotations","Show motion annotations",&mut self.draft_annotations);focus_order.push(annotations.id);kit::muted(ui,rt,&self.inspector_result);ui.add_space(20.);ui.separator();
                    ui.horizontal(|ui|{let r=kit::button(ui,rt,"ov-drawer-cancel","Cancel",None,ButtonKind::Secondary);focus_order.push(r.id);close|=r.clicked();let apply=kit::button(ui,rt,"ov-inspector-apply","Apply to demo",Some("check"),ButtonKind::Primary);focus_order.push(apply.id);if apply.clicked(){self.density=self.draft_density;self.annotations=self.draft_annotations;let choice=format!("{} · annotations {}",if self.density==0{"Comfortable"}else{"Compact"},if self.annotations{"on"}else{"off"});self.inspector_result=format!("Current: {choice}");self.status=format!("Inspector preference saved: {choice}.");close=true;}});
                },
                Dialog::Sheet=>{
                    let(handle,r)=ui.allocate_exact_size(vec2(ui.available_width(),24.),Sense::click_and_drag());let bar=Rect::from_center_size(handle.center(),vec2(48.,4.));ui.painter().rect_filled(bar,3.,rt.tokens.border_strong);r.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,true,"Close bottom sheet, or drag this handle down"));rt.record("ov-sheet-handle",&r);focus_order.push(r.id);
                    if r.drag_started(){let start=ctx.input(|i|i.pointer.press_origin()).map_or(handle.center().y,|p|p.y);self.drag=Some(SheetDrag{start,samples:vec![(rt.now,start)],..Default::default()});}
                    if let Some(drag)=&mut self.drag&& let Some(pos)=ctx.pointer_interact_pos(){drag.move_to(rt.now,pos.y);if r.drag_stopped(){let velocity=drag.velocity(rt.now,pos.y);self.suppress_sheet_click=drag.moved;if sheet_dismiss(drag.y,sheet_height,velocity){close=true;}else{let displacement=sheet_resistance(drag.y);self.spring.snap(displacement);self.spring.to(0.,rt.now,if rt.reduced(){0.}else{0.6*rt.motion_scale()},Curve::SpringSmooth);}self.drag=None;}else if !ctx.input(|i|i.pointer.primary_down()){self.spring.snap(sheet_resistance(drag.y));self.spring.to(0.,rt.now,if rt.reduced(){0.}else{0.6*rt.motion_scale()},Curve::SpringSmooth);self.drag=None;}}
                    if r.clicked(){if !self.suppress_sheet_click{close=true;}self.suppress_sheet_click=false;}
                    kit::muted(ui,rt,"A tactile surface with a keyboard alternative. Drag down or flick the handle; pull up to feel resistance.");
                    for(icon,title,desc,action)in [("copy","Duplicate specimen","Keep the original, make a local copy.","duplicate"),("star","Mark as favorite","A little emphasis for this example.","favorite"),("eye","Inspect details","Read the project without leaving the page.","inspect")]{let r=kit::button(ui,rt,("ov-sheet-action",action),title,Some(icon),ButtonKind::Ghost);focus_order.push(r.id);kit::muted(ui,rt,desc);if r.clicked(){sheet_action=Some(action);close=true;}}
                    ui.separator();kit::muted(ui,rt,"Drag handle · Escape · Close");let r=kit::button(ui,rt,"ov-sheet-close","Close sheet",None,ButtonKind::Secondary);focus_order.push(r.id);if first{r.request_focus();}close|=r.clicked();
                },
            }
        });
        surface.set_focus_order(&focus_order);
        if let Some(output) = output {
            if output.should_close {
                close = true;
            }
            rt.record("ov-dialog-surface", &output.response);
        }
        if let Some(id) = new_origin {
            surface.set_origin_focus(id);
        }
        if self.drag.is_some()
            && (ctx.pointer_interact_pos().is_none() || !ctx.input(|i| i.focused))
        {
            self.drag = None;
            self.suppress_sheet_click = true;
            self.spring.snap(surface.drag_y);
            self.spring.to(
                0.,
                rt.now,
                if rt.reduced() {
                    0.
                } else {
                    0.6 * rt.motion_scale()
                },
                Curve::SpringSmooth,
            );
            rt.repaint();
        }
        if let Some(drag) = &self.drag {
            surface.drag_y = if rt.reduced() {
                0.
            } else {
                sheet_resistance(drag.y)
            };
            rt.repaint();
        }
        if self.spring.moving(rt.now) {
            rt.repaint();
        }
        if close {
            surface.close_at(rt.now, rt.motion_scale(), rt.reduced());
            self.drag = None;
            rt.repaint();
        }
        let still_visible = surface.is_visible();
        let origin_rect = surface.origin.unwrap_or(Rect::NOTHING);
        let origin_id = surface.origin_focus.unwrap_or(surface.id);
        self.surface = surface;
        if !still_visible {
            self.dialog = None;
        }
        if let Some(action) = sheet_action {
            self.surface.close_immediately(ctx, false);
            self.dialog = None;
            self.action(action, origin_rect, origin_id, rt.now);
        }
    }
    fn loading(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        let complete = self.loading_value() >= 100;
        let value = self.loading_value();
        let phase = self.loading_phase();
        if !complete {
            rt.animate("ov-loaded-opacity", 0., 0.32, Curve::Out);
        }
        let width = ui.available_width();
        let workspace = self.workspace.clone();
        let running = self.loading_running;
        let preview = |ui: &mut Ui, rt: &mut Runtime| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width().min(320.), 250.), Sense::hover());
            ui.painter()
                .rect_filled(rect, rt.tokens.radius * 10., rt.tokens.surface2);
            if complete {
                let p = rt.animate("ov-loaded-opacity", 1., 0.32, Curve::Out);
                let art =
                    Rect::from_min_size(rect.min + vec2(16., 16.), vec2(rect.width() - 32., 128.));
                ui.painter()
                    .rect_filled(art, rt.tokens.radius * 8., rt.tokens.success_soft);
                crate::icons::paint(
                    ui.painter(),
                    "check-circle",
                    Rect::from_center_size(art.center() - vec2(0., 12.), vec2(40., 40.)),
                    rt.tokens.success_text,
                    2.,
                    p,
                );
                ui.painter().text(
                    art.center() + vec2(0., 32.),
                    egui::Align2::CENTER_CENTER,
                    "ATLAS / 001",
                    egui::FontId::monospace(11.),
                    rt.tokens.success_text,
                );
                ui.painter().text(
                    rect.min + vec2(16., 170.),
                    egui::Align2::LEFT_CENTER,
                    &workspace,
                    egui::FontId::proportional(16.),
                    rt.tokens.text,
                );
                ui.painter().text(
                    rect.min + vec2(16., 196.),
                    egui::Align2::LEFT_CENTER,
                    "12 components. One coherent system.",
                    egui::FontId::proportional(12.),
                    rt.tokens.text2,
                );
            } else {
                let base = rt.tokens.surface3;
                let art =
                    Rect::from_min_size(rect.min + vec2(16., 16.), vec2(rect.width() - 32., 116.));
                ui.painter().rect_filled(art, rt.tokens.radius * 6., base);
                for (n, w) in [0.6, 0.9, 0.75].iter().enumerate() {
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            rect.min + vec2(16., 150. + n as f32 * 20.),
                            vec2((rect.width() - 32.) * w, 9.),
                        ),
                        4.,
                        base,
                    );
                }
                if running && !rt.reduced() {
                    let x = ((rt.now % 1.8) / 1.8) as f32;
                    let scan = Rect::from_min_size(
                        pos2(art.left() + x * art.width(), art.top()),
                        vec2(40., art.height()),
                    )
                    .intersect(art);
                    ui.painter().rect_filled(
                        scan,
                        0.,
                        rt.tokens.surface_raised.gamma_multiply(0.45),
                    );
                    rt.repaint();
                }
            }
        };
        if width > 640. {
            ui.columns(2, |columns| {
                preview(&mut columns[0], rt);
                self.loading_controls(&mut columns[1], rt, value, phase);
            });
        } else {
            preview(ui, rt);
            self.loading_controls(ui, rt, value, phase);
        }
    }
    fn loading_controls(&mut self, ui: &mut Ui, rt: &mut Runtime, value: u32, phase: usize) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PREVIEW PIPELINE")
                    .size(11.)
                    .color(rt.tokens.text3),
            );
            kit::badge(ui, rt, "Simulation", "");
        });
        ui.horizontal(|ui| {
            ui.strong(if value == 0 && !self.loading_running {
                "Ready when you are"
            } else {
                [
                    "Reading specimen",
                    "Resolving tokens",
                    "Composing preview",
                    "Preview ready",
                ][phase]
            });
            ui.label(
                RichText::new(format!("{value}%"))
                    .monospace()
                    .color(rt.tokens.accent_text),
            );
        });
        let r = kit::progress(ui, rt, value as f32 / 100., 6.);
        r.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Other,
                true,
                format!("Simulated preview progress {value}%"),
            )
        });
        ui.ctx().accesskit_node_builder(r.id, |node| {
            node.set_role(egui::accesskit::Role::ProgressIndicator);
            node.set_numeric_value(value as f64);
            node.set_min_numeric_value(0.);
            node.set_max_numeric_value(100.);
        });
        rt.record("ov-progress", &r);
        ui.add_space(12.);
        for (n, label) in ["Read specimen", "Resolve tokens", "Compose preview"]
            .iter()
            .enumerate()
        {
            ui.horizontal(|ui| {
                let color = if n < phase {
                    rt.tokens.success_text
                } else if n == phase {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text3
                };
                if n < phase {
                    let (r, _) = ui.allocate_exact_size(vec2(16., 16.), Sense::hover());
                    crate::icons::paint(ui.painter(), "check", r, color, 1.6, 0.);
                } else {
                    ui.colored_label(color, (n + 1).to_string());
                }
                ui.colored_label(color, *label);
            });
        }
        ui.add_space(12.);
        ui.horizontal_wrapped(|ui| {
            let r = kit::button(
                ui,
                rt,
                "ov-progress-toggle",
                if value >= 100 {
                    "Run again"
                } else if self.loading_running {
                    "Pause"
                } else if self.loading_elapsed > 0. {
                    "Resume"
                } else {
                    "Run simulation"
                },
                None,
                ButtonKind::Primary,
            );
            if r.clicked() {
                if self.loading_running {
                    self.pause_loading("Paused · resume when ready.");
                } else {
                    self.start_loading(rt.now, false);
                }
            }
            if kit::button(
                ui,
                rt,
                "ov-progress-replay",
                "Replay",
                Some("refresh"),
                ButtonKind::Secondary,
            )
            .clicked()
            {
                self.start_loading(rt.now, true);
            }
            let mut selected = if self.loading_speed == 0.5 {
                0
            } else if self.loading_speed == 2. {
                2
            } else {
                1
            };
            let previous = selected;
            egui::ComboBox::from_id_salt("ov-progress-speed")
                .selected_text(["0.5×", "1×", "2×"][selected])
                .show_ui(ui, |ui| {
                    for (n, label) in ["0.5×", "1×", "2×"].iter().enumerate() {
                        ui.selectable_value(&mut selected, n, *label);
                    }
                });
            if previous != selected {
                self.loading_speed = [0.5, 1., 2.][selected];
                self.last_tick = Some(rt.now);
                self.loading_status = format!(
                    "{} at {}× · local simulation.",
                    if self.loading_running {
                        "Running"
                    } else {
                        "Paused"
                    },
                    self.loading_speed
                );
            }
        });
        kit::muted(ui, rt, &self.loading_status);
    }
}
impl Page for State {
    fn enter(&mut self, _rt: &mut Runtime) {
        self.visible = true;
        self.last_tick = None;
    }
    fn leave(&mut self, rt: &mut Runtime) {
        self.visible = false;
        self.suspend(
            rt.context(),
            rt.now,
            "Paused · page was left. Resume when ready.",
        );
    }
    fn show(&mut self, ui: &mut Ui, rt: &mut Runtime) {
        self.visible = true;
        let ctx = ui.ctx().clone();
        let active = ctx.input(|i| !i.viewport().minimized.unwrap_or(false));
        if active {
            self.advance(rt.now);
        } else {
            self.suspend(
                &ctx,
                rt.now,
                "Paused · window was hidden. Resume when ready.",
            );
        }
        if self.loading_running {
            rt.repaint();
        }
        if let Some((started, _)) = self.promise {
            rt.repaint_after((1.8 - (rt.now - started)).max(0.001));
        }
        let screen = ctx.content_rect();
        if self.last_screen != Rect::NOTHING && screen != self.last_screen {
            self.menu = None;
            self.card_open = false;
        }
        self.last_screen = screen;
        if ctx.input(|i| {
            i.raw_scroll_delta != Vec2::ZERO
                && i.pointer.hover_pos().is_none_or(|p| {
                    !self.card_rect.contains(p)
                        && self
                            .menu
                            .as_ref()
                            .is_none_or(|m| !m.rect.contains(p) && !m.subrect.contains(p))
                })
        }) {
            self.menu = None;
            self.card_open = false;
        }
        kit::page_head(
            ui,
            rt,
            "window",
            "Build / 07",
            "Overlays & Feedback",
            "Dialogs, drawers, sheets, tooltips and toasts.",
        );
        ui.horizontal_wrapped(|ui| {
            kit::badge(ui, rt, "Native dialogs", "accent");
            kit::badge(ui, rt, "Keyboard first", "");
            kit::badge(ui, rt, "Local simulations", "");
        });
        kit::section(ui, rt, "ov-dialogs", "01", "Dialogs", "");
        kit::demo(
            ui,
            rt,
            "overlays-0",
            "Origin morph",
            "A project card becomes its detail surface. Closing returns to its origin.",
            280.,
            |ui, rt| self.origin_card(ui, rt),
        );
        let wide = ui.available_width() > 720.;
        ui.columns(if wide { 2 } else { 1 }, |columns| {
            {
                let ui = &mut columns[0];
                kit::demo(
                    ui,
                    rt,
                    "overlays-1",
                    "Modal dialog",
                    "A focused form with native validation and a persistent local result.",
                    180.,
                    |ui, rt| {
                        ui.strong(&self.workspace);
                        let r = kit::button(
                            ui,
                            rt,
                            "ov-open-rename",
                            "Rename workspace",
                            Some("pencil"),
                            ButtonKind::Secondary,
                        );
                        if r.clicked() {
                            self.open(Dialog::Rename, r.rect, r.id, rt.now);
                        }
                        kit::muted(ui, rt, &self.rename_result);
                    },
                );
            }
            {
                let ui = &mut columns[if wide { 1 } else { 0 }];
                kit::demo(
                    ui,
                    rt,
                    "overlays-2",
                    "Destructive confirmation",
                    "The safe action receives focus first. Removal affects only this specimen.",
                    180.,
                    |ui, rt| {
                        ui.strong(if self.removed {
                            "Draft removed"
                        } else {
                            "Exploration draft"
                        });
                        if self.removed {
                            let r = kit::button(
                                ui,
                                rt,
                                "ov-restore-draft",
                                "Restore draft",
                                Some("refresh"),
                                ButtonKind::Secondary,
                            );
                            if self.surface.is_closing() {
                                self.surface.set_origin_focus(r.id);
                            }
                            if r.clicked() {
                                self.removed = false;
                                self.status = "Example draft restored.".into();
                                self.focus_delete = true;
                            }
                            kit::muted(ui, rt, "No demo drafts · restore anytime");
                        } else {
                            let r = kit::button(
                                ui,
                                rt,
                                "ov-open-delete",
                                "Remove draft",
                                Some("trash"),
                                ButtonKind::Secondary,
                            );
                            if self.focus_delete {
                                r.request_focus();
                                self.focus_delete = false;
                            }
                            if r.clicked() {
                                self.open(Dialog::Delete, r.rect, r.id, rt.now);
                            }
                            kit::muted(ui, rt, "1 local demo draft");
                        }
                    },
                );
            }
        });
        kit::section(ui, rt, "ov-surfaces", "02", "Drawers and sheets", "");
        let wide = ui.available_width() > 720.;
        ui.columns(if wide { 2 } else { 1 }, |columns| {
            {
                let ui = &mut columns[0];
                kit::demo(
                    ui,
                    rt,
                    "overlays-3",
                    "Side drawer",
                    "Edit preferences, then apply or dismiss without committing.",
                    180.,
                    |ui, rt| {
                        edge_preview(ui, rt, false);
                        let r = kit::button(
                            ui,
                            rt,
                            "ov-open-drawer",
                            "Open inspector",
                            Some("sidebar"),
                            ButtonKind::Secondary,
                        );
                        if r.clicked() {
                            self.open(Dialog::Drawer, r.rect, r.id, rt.now);
                        }
                    },
                );
            }
            {
                let ui = &mut columns[if wide { 1 } else { 0 }];
                kit::demo(
                    ui,
                    rt,
                    "overlays-4",
                    "Bottom sheet",
                    "Drag down or flick the handle. Pull upward against a soft rubberband.",
                    180.,
                    |ui, rt| {
                        edge_preview(ui, rt, true);
                        let r = kit::button(
                            ui,
                            rt,
                            "ov-open-sheet",
                            "Open bottom sheet",
                            Some("chevrons-up-down"),
                            ButtonKind::Secondary,
                        );
                        if r.clicked() {
                            self.open(Dialog::Sheet, r.rect, r.id, rt.now);
                        }
                    },
                );
            }
        });
        kit::section(ui, rt, "ov-menus", "03", "Menus and popovers", "");
        let wide = ui.available_width() > 720.;
        ui.columns(if wide{2}else{1},|columns|{
            {let ui=&mut columns[0];
kit::demo(
            ui,
            rt,
            "overlays-5",
            "Dropdown + submenu",
            "Up / Down cycle items. Right opens the submenu; Left returns. Type an initial to jump.",
            160.,
            |ui, rt| {
                self.menu_trigger(ui, rt, false);
                kit::muted(ui, rt, &self.result);
            },
        );
            }
            {let ui=&mut columns[if wide{1}else{0}];
kit::demo(
            ui,
            rt,
            "overlays-6",
            "Context menu",
            "Right-click the canvas, press Shift + F10, or use its menu button.",
            200.,
            |ui, rt| {
                let frame = Frame::new()
                    .fill(rt.tokens.surface2)
                    .stroke(Stroke::new(1., rt.tokens.border))
                    .inner_margin(20.)
                    .corner_radius(rt.tokens.radius * 8.)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width().min(360.));
                        ui.monospace("atlas / specimen.svg");
                        kit::muted(ui, rt, "Right-click anywhere in this canvas");
                        self.menu_trigger(ui, rt, true);
                    });
                let r = ui.interact(
                    frame.response.rect,
                    Id::new("ov-context-target"),
                    Sense::click(),
                );
                r.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,ui.is_enabled(),"Atlas specimen canvas. Press Shift F10 for its context menu."));
                ui.ctx().accesskit_node_builder(r.id,|node|node.set_has_popup(egui::accesskit::HasPopup::Menu));
                rt.record("ov-context-target", &r);
                let key =
                    r.has_focus() && ui.input(|i| i.modifiers.shift && i.key_pressed(Key::F10));
                if r.secondary_clicked() || key {
                    self.open_menu(
                        true,
                        &r,
                        if key {
                            None
                        } else {
                            ui.input(|i| i.pointer.interact_pos())
                        },
                        false,
                        rt.now,
                    );
                }
            },
        );
            }
        });
        let wide = ui.available_width() > 720.;
        ui.columns(if wide{2}else{1},|columns|{
            {let ui=&mut columns[0];
kit::demo(
            ui,
            rt,
            "overlays-7",
            "Intent hovercard",
            "Wait briefly on the name or focus it with Tab. Escape dismisses the card.",
            150.,
            |ui, rt| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("MC").size(22.).color(rt.tokens.accent_text));
                    ui.label("Reviewed by");
                    let r = kit::button(ui, rt, "ov-person", "Maya Chen", None, ButtonKind::Ghost);
                    self.card_anchor = r.rect;
                    self.card_origin = Some(r.id);
                    self.card_focus = r.has_focus();
                    if r.gained_focus() {
                        if self.card_suppress {
                            self.card_suppress = false;
                        } else {
                            self.card_open = true;
                            self.card_since = Some(rt.now);
                            self.menu = None;
                        }
                    }
                    if r.clicked() {
                        self.card_open = true;
                        self.card_since = Some(rt.now);
                        self.menu = None;
                    }
                    if r.hovered() {
                        if self.card_since.is_none() {
                            self.card_since = Some(rt.now);
                        }
                        if let Some(at) = self.card_since {
                            if rt.now - at >= 0.45 {
                                self.card_open = true;
                                self.menu = None;
                            } else {
                                rt.repaint_after(0.45 - (rt.now - at));
                            }
                        }
                        self.card_exit = None;
                    } else if !self.card_open && !r.has_focus() {
                        self.card_since = None;
                    }
                });
                kit::muted(ui, rt, "Motion designer · example profile");
            },
        );
            }
            {let ui=&mut columns[if wide{1}else{0}];
kit::demo(
            ui,
            rt,
            "overlays-8",
            "Tooltip toolbar",
            "Intent on hover; immediate on keyboard focus. Left / Right navigate the tools.",
            150.,
            |ui, rt| self.toolbar(ui, rt),
        );
            }
        });
        kit::section(ui, rt, "ov-notifications", "04", "Notifications", "");
        let wide = ui.available_width() > 720.;
        ui.columns(if wide{2}else{1},|columns|{
            {let ui=&mut columns[0];
kit::demo(
            ui,
            rt,
            "overlays-9",
            "Toast tones",
            "Trigger several, then hover the stack to fan it out. Timers pause while reading.",
            130.,
            |ui, rt| {
                ui.horizontal_wrapped(|ui| {
                    for (tone, icon, title, desc) in TONES {
                        let label = format!("{}{}", tone[..1].to_uppercase(), &tone[1..]);
                        if kit::button(
                            ui,
                            rt,
                            ("ov-toast", tone),
                            &label,
                            Some(icon),
                            ButtonKind::Secondary,
                        )
                        .clicked()
                        {
                            self.feedback.toast(title, desc, tone, rt.now);
                        }
                    }
                });
            },
        );
            }
            {let ui=&mut columns[if wide{1}else{0}];
kit::demo(
            ui,
            rt,
            "overlays-10",
            "Promise feedback",
            "A local 1.8-second delay. Pending is dismissed before success appears.",
            160.,
            |ui, rt| {
                ui.add_enabled_ui(self.promise.is_none(), |ui| {
                    if kit::button(
                        ui,
                        rt,
                        "ov-promise",
                        "Simulate save",
                        Some("upload"),
                        ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        self.start_promise(rt.now);
                    }
                });
                kit::muted(ui, rt, &self.promise_status);
            },
        );
            }
        });
        let count = if ui.available_width() > 720. { 2 } else { 1 };
        ui.columns(count, |columns| {
            for (n, (tone, icon, title, desc)) in TONES.into_iter().enumerate() {
                let ui = &mut columns[n % count];
                let color = match tone {
                    "success" => rt.tokens.success_soft,
                    "warning" => rt.tokens.warning_soft,
                    "danger" => rt.tokens.danger_soft,
                    _ => rt.tokens.info_soft,
                };
                Frame::new()
                    .fill(color)
                    .corner_radius(rt.tokens.radius * 8.)
                    .inner_margin(14.)
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(20., 20.), Sense::hover());
                            crate::icons::paint(ui.painter(), icon, r, rt.tokens.text2, 1.6, 0.);
                            ui.vertical(|ui| {
                                ui.strong(title);
                                kit::muted(ui, rt, desc);
                            });
                        });
                    });
                ui.add_space(8.);
            }
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("COLLAPSING ANNOUNCEMENT")
                    .size(11.)
                    .color(rt.tokens.text3),
            );
            let restore = kit::button(
                ui,
                rt,
                "ov-banner-restore",
                "Replay banner",
                Some("refresh"),
                ButtonKind::Ghost,
            );
            self.banner_restore_id = Some(restore.id);
            ui.ctx()
                .accesskit_node_builder(restore.id, |node| node.set_expanded(self.banner));
            if restore.clicked() {
                self.banner = true;
            }
        });
        let p = rt.animate(
            "ov-banner-height",
            if self.banner { 1. } else { 0. },
            0.36,
            Curve::Out,
        );
        let mut banner_close = false;
        collapse(ui, rt, "ov-banner", p, self.banner, |ui, rt| {
            Frame::new()
                .fill(rt.tokens.accent_soft)
                .corner_radius(rt.tokens.radius * 8.)
                .inner_margin(16.)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong("A smaller interruption.");
                            kit::muted(ui, rt, "Dismiss this banner and the page closes the gap.");
                        });
                        if kit::icon_button(ui, rt, "ov-banner-close", "x", "Dismiss announcement")
                            .clicked()
                        {
                            banner_close = true;
                        }
                    });
                });
        });
        if banner_close {
            self.banner = false;
            self.status = "Announcement dismissed.".into();
            if let Some(id) = self.banner_restore_id {
                crate::runtime::restore_focus(&ctx, Some(id));
            }
        }
        kit::section(ui, rt, "ov-disclosure", "05", "Accordion", "");
        let wide = ui.available_width() > 720.;
        ui.columns(if wide{2}else{1},|columns|{
            {let ui=&mut columns[0];
kit::demo(
            ui,
            rt,
            "overlays-11",
            "Single accordion",
            "Opening a new answer closes the previous one. Up / Down move between headings.",
            270.,
            |ui, rt| self.accordion(ui, rt, true),
        );
            }
            {let ui=&mut columns[if wide{1}else{0}];
kit::demo(
            ui,
            rt,
            "overlays-12",
            "Multiple accordion",
            "Answers remain open independently. State survives route changes.",
            270.,
            |ui, rt| self.accordion(ui, rt, false),
        );
            }
        });
        kit::demo(
            ui,
            rt,
            "overlays-13",
            "Show-more disclosure",
            "A compact summary with an optional deeper explanation. No content is fetched.",
            160.,
            |ui, rt| {
                ui.strong("Quiet by default. Complete on demand.");
                kit::muted(
                    ui,
                    rt,
                    "Keep the essential information visible, and let the reader choose the rest.",
                );
                let p = rt.animate(
                    "ov-more-height",
                    if self.more { 1. } else { 0. },
                    0.36,
                    Curve::Out,
                );
                collapse(ui, rt, "ov-more", p, self.more, |ui, rt| {
                    kit::muted(
                        ui,
                        rt,
                        "A disclosure belongs in the reading flow. It can hold rationale, technical notes, or a longer explanation without asking the reader to change context. The control remains in place, keeps focus, and describes its next action.",
                    );
                });
                if kit::button(
                    ui,
                    rt,
                    "ov-show-more",
                    if self.more { "Show less" } else { "Show more" },
                    Some("chevron-down"),
                    ButtonKind::Ghost,
                )
                .clicked()
                {
                    self.more = !self.more;
                }
            },
        );
        kit::section(ui, rt, "ov-loading", "06", "Loading states", "");
        kit::demo(
            ui,
            rt,
            "overlays-14",
            "Skeleton → content",
            "Eight-second local simulation at 1×. Pause, replay, or change speed. Leaving pauses it.",
            340.,
            |ui, rt| self.loading(ui, rt),
        );
        if !self.status.is_empty() {
            let status = ui.label(RichText::new(&self.status).size(12.).color(rt.tokens.text3));
            ui.ctx().accesskit_node_builder(status.id, |node| {
                node.set_role(egui::accesskit::Role::Status);
                node.set_live(egui::accesskit::Live::Polite);
            });
        }
        // Areas render after specimens, retaining native modality and global focus order.
        self.paint_floating_exits(&ctx, rt);
        self.show_card(&ctx, rt);
        self.menus(&ctx, rt);
        self.dialogs(&ctx, rt);
        self.feedback
            .show(&ctx, rt.now, &rt.tokens, rt.motion_scale(), rt.reduced());
    }
}
fn frame(rt: &Runtime) -> Frame {
    Frame::new()
        .fill(rt.tokens.surface_raised)
        .stroke(Stroke::new(1., rt.tokens.border))
        .corner_radius(rt.tokens.radius * 10.)
        .inner_margin(16.)
}
fn take_navigation(ui: &mut Ui, keys: &[Key]) -> Option<Key> {
    for key in keys {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, *key)) {
            return Some(*key);
        }
    }
    None
}
fn consume(ctx: &Context, key: Key, modifiers: egui::Modifiers) {
    ctx.input_mut(|i| {
        i.consume_key(modifiers, key);
    });
}
fn navigate_index(key: Key, index: usize, len: usize, forward: Key) -> usize {
    if key == Key::Home {
        0
    } else if key == Key::End {
        len - 1
    } else if key == forward {
        (index + 1) % len
    } else {
        (index + len - 1) % len
    }
}
/// Collision-aware placement for both root menus and submenus.
pub fn place_popup(
    screen: Rect,
    anchor: Rect,
    size: Vec2,
    point: Option<Pos2>,
    submenu: bool,
) -> Pos2 {
    let mut pos = point.unwrap_or_else(|| {
        if submenu {
            pos2(anchor.right() + 8., anchor.top())
        } else {
            pos2(anchor.left(), anchor.bottom() + 8.)
        }
    });
    if submenu && pos.x + size.x > screen.right() - 12. {
        pos.x = anchor.left() - size.x - 8.;
    }
    if point.is_none() && !submenu && pos.y + size.y > screen.bottom() - 12. {
        pos.y = anchor.top() - size.y - 8.;
    }
    pos.x = pos.x.clamp(
        screen.left() + 12.,
        (screen.right() - size.x - 12.).max(screen.left() + 12.),
    );
    pos.y = pos.y.clamp(
        screen.top() + 12.,
        (screen.bottom() - size.y - 12.).max(screen.top() + 12.),
    );
    pos
}
fn collapse(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl std::hash::Hash,
    p: f32,
    open: bool,
    body: impl FnOnce(&mut Ui, &mut Runtime),
) {
    if p <= 0.001 {
        return;
    }
    let id = ui.make_persistent_id(id);
    let width = ui.available_width();
    let previous = ui.ctx().data_mut(|d| d.get_temp::<f32>(id)).unwrap_or(84.);
    let start = ui.cursor().min;
    let available = Rect::from_min_size(start, vec2(width, previous * p));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(Rect::from_min_size(start, vec2(width, 1000.)))
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(ui.clip_rect().intersect(available));
    child.set_opacity(p);
    if !open {
        child.disable();
    }
    body(&mut child, rt);
    let height = child.min_rect().height().max(1.);
    ui.ctx().data_mut(|d| d.insert_temp(id, height));
    ui.allocate_space(vec2(width, height * p));
}
fn edge_preview(ui: &mut Ui, rt: &Runtime, sheet: bool) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width().min(300.), 76.), Sense::hover());
    ui.painter()
        .rect_filled(rect, rt.tokens.radius * 8., rt.tokens.surface2);
    let panel = if sheet {
        Rect::from_min_max(
            pos2(rect.left() + 30., rect.top() + 32.),
            rect.max - vec2(30., 0.),
        )
    } else {
        Rect::from_min_max(
            pos2(rect.right() - 90., rect.top() + 8.),
            rect.max - vec2(8., 0.),
        )
    };
    ui.painter()
        .rect_filled(panel, rt.tokens.radius * 6., rt.tokens.surface_raised);
    for n in 0..3 {
        let y = if sheet {
            panel.top() + 14. + n as f32 * 12.
        } else {
            rect.top() + 18. + n as f32 * 16.
        };
        ui.painter().line_segment(
            [
                pos2(rect.left() + 16., y),
                pos2(
                    if sheet {
                        rect.right() - 16.
                    } else {
                        panel.left() - 16.
                    },
                    y,
                ),
            ],
            Stroke::new(3., rt.tokens.border),
        );
    }
}
fn project_art(ui: &mut Ui, rt: &Runtime, rect: Rect, engaged: f32) {
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, rt.tokens.radius * 10., rt.tokens.accent_soft);
    for n in 1..8 {
        let x = rect.left() + rect.width() * n as f32 / 8.;
        painter.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(0.5, rt.tokens.accent_ring),
        );
    }
    for n in 1..4 {
        let y = rect.top() + rect.height() * n as f32 / 4.;
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(0.5, rt.tokens.accent_ring),
        );
    }
    let center = rect.center();
    let back = Rect::from_center_size(
        center + vec2(18., -10. - engaged * 3.),
        vec2(rect.width() * 0.36, rect.height() * 0.58),
    );
    let front = back.translate(vec2(-28., 18. + engaged * 3.));
    painter.rect_filled(back, 8., rt.tokens.accent_soft2);
    painter.rect_stroke(
        back,
        8.,
        Stroke::new(1., rt.tokens.accent_ring),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(front, 8., rt.tokens.surface_raised);
    painter.rect_stroke(
        front,
        8.,
        Stroke::new(1., rt.tokens.accent_ring),
        egui::StrokeKind::Inside,
    );
    for (n, width) in [0.45, 0.75, 0.58].iter().enumerate() {
        painter.line_segment(
            [
                front.min + vec2(14., 20. + n as f32 * 12.),
                front.min + vec2(front.width() * width, 20. + n as f32 * 12.),
            ],
            Stroke::new(2., rt.tokens.border_strong),
        );
    }
    crate::icons::paint(
        &painter,
        "check",
        Rect::from_center_size(front.max - vec2(16., 16.), vec2(20., 20.)),
        rt.tokens.accent_text,
        2.,
        engaged,
    );
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn loading_is_eight_real_seconds_and_pause_preserves_progress() {
        let mut s = State::default();
        s.start_loading(10., false);
        s.advance(12.4);
        assert_eq!(s.loading_value(), 32);
        assert_eq!(s.loading_phase(), 1);
        s.pause_loading("paused");
        s.advance(100.);
        assert_eq!(s.loading_value(), 32);
        s.start_loading(100., false);
        s.advance(105.6);
        assert_eq!(s.loading_value(), 100);
        assert!(!s.loading_running);
    }
    #[test]
    fn replay_and_speed_affect_only_simulation_time() {
        let mut s = State::default();
        s.loading_speed = 2.;
        s.start_loading(0., true);
        s.advance(4.);
        assert_eq!(s.loading_value(), 100);
        s.start_loading(5., true);
        assert_eq!(s.loading_value(), 0);
        s.advance(5.56);
        assert_eq!(s.loading_value(), 16);
    }
    #[test]
    fn empty_names_fail_and_trimmed_names_persist() {
        let mut s = State::default();
        s.rename = "   ".into();
        assert!(!s.rename_workspace());
        assert_eq!(s.workspace, "Atlas workspace");
        s.rename = "  New name  ".into();
        assert!(s.rename_workspace());
        assert_eq!(s.workspace, "New name");
    }
    #[test]
    fn pending_save_cannot_be_duplicated_and_needs_real_delay() {
        let mut s = State::default();
        s.start_promise(0.);
        s.start_promise(0.5);
        assert_eq!(s.feedback.len(), 1);
        s.advance(1.79);
        assert!(s.promise.is_some());
        s.advance(1.8);
        assert!(s.promise.is_none());
        assert_eq!(s.feedback.len(), 1);
        assert!(s.promise_status.contains("complete"));
    }
    #[test]
    fn route_cleanup_cancels_pending_without_success_and_retains_disclosures() {
        let mut s = State::default();
        s.start_promise(0.);
        s.start_loading(0., false);
        s.advance(1.);
        s.multi[2] = true;
        s.suspend(&Context::default(), 1., "away");
        s.advance(50.);
        assert!(s.promise.is_none());
        assert!(s.promise_status.contains("canceled"));
        assert_eq!(s.feedback.len(), 0);
        assert!(!s.loading_running);
        assert!(s.multi[2]);
        assert_eq!(s.loading_elapsed, 1.);
    }
    #[test]
    fn sheet_resistance_and_velocity_thresholds() {
        assert_eq!(sheet_resistance(100.), 100.);
        assert!(sheet_resistance(-100.) > -48.);
        assert!(sheet_resistance(-100.) < 0.);
        assert!(sheet_dismiss(150., 500., 0.));
        assert!(sheet_dismiss(20., 500., 0.7));
        assert!(!sheet_dismiss(10., 500., 2.));
        assert!(!sheet_dismiss(80., 500., 0.1));
    }
    #[test]
    fn menu_placement_flips_and_stays_inside_viewport() {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(800., 600.));
        let a = Rect::from_min_size(pos2(730., 550.), vec2(50., 20.));
        let pos = place_popup(screen, a, vec2(240., 160.), None, true);
        assert!(pos.x < a.left());
        assert!(pos.y + 160. <= 588.);
        let root = place_popup(screen, a, vec2(240., 160.), None, false);
        assert!(root.y < a.top());
        assert!(root.x + 240. <= 788.);
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod native_frames {
    use super::*;
    fn frame(
        state: &mut State,
        rt: &mut Runtime,
        time: f64,
        size: Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let ctx = rt.context().clone();
        ctx.enable_accesskit();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(time),
            events,
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| {
            rt.begin_frame();
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| state.show(ui, rt));
            });
        });
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native accessibility tree");
        assert!(
            update.nodes.iter().any(|(id, _)| *id == update.focus),
            "overlay focus {:?} has no current-frame node at {time}",
            update.focus
        );
        output
    }
    fn key(key: Key, shift: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                shift,
                ..Default::default()
            },
        }
    }
    #[test]
    fn all_specimens_and_dialogs_tessellate_in_all_appearances_and_narrow_windows() {
        for dark in [false, true] {
            for mono in [false, true] {
                for width in [420., 1100.] {
                    let ctx = Context::default();
                    let mut prefs = crate::runtime::Preferences::default();
                    prefs.dark = dark;
                    prefs.mono = mono;
                    let mut rt = Runtime::new(ctx.clone(), prefs);
                    let mut state = State::default();
                    let out = frame(&mut state, &mut rt, 0., vec2(width, 900.), vec![]);
                    assert!(out.shapes.len() > 30);
                    for (n, dialog) in [
                        Dialog::Project,
                        Dialog::Rename,
                        Dialog::Delete,
                        Dialog::Drawer,
                        Dialog::Sheet,
                    ]
                    .iter()
                    .enumerate()
                    {
                        let now = n as f64 * 3. + 1.;
                        state.open(
                            *dialog,
                            Rect::from_min_size(pos2(80., 180.), vec2(260., 160.)),
                            Id::new("test-origin"),
                            now,
                        );
                        frame(&mut state, &mut rt, now, vec2(width, 900.), vec![]);
                        let out = frame(&mut state, &mut rt, now + 0.2, vec2(width, 900.), vec![]);
                        let primitives = ctx.tessellate(out.shapes, out.pixels_per_point);
                        assert!(!primitives.is_empty());
                        for p in primitives {
                            if let egui::epaint::Primitive::Mesh(mesh) = p.primitive {
                                assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
                            }
                        }
                        frame(&mut state, &mut rt, now + 1., vec2(width, 900.), vec![]);
                        assert!(state.surface.rect.is_finite());
                        assert!(state.surface.rect.bottom() <= 900. + 1.);
                        if matches!(dialog, Dialog::Drawer | Dialog::Sheet) {
                            assert!(
                                (state.surface.rect.bottom() - 900.).abs() <= 1.,
                                "{:?} {:?}",
                                dialog,
                                state.surface.rect
                            );
                        }
                        if *dialog == Dialog::Drawer {
                            assert!(state.surface.rect.top().abs() <= 1.);
                            assert!((state.surface.rect.right() - width).abs() <= 1.);
                        }
                        assert!(
                            state.surface.rect.width() <= width + 2.,
                            "{:?} {:?}",
                            dialog,
                            state.surface.rect
                        );
                        state.surface.close_immediately(&ctx, false);
                        state.dialog = None;
                    }
                }
            }
        }
    }
    #[test]
    fn rename_focus_is_contained_and_escape_closes_without_modifying_name() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut state = State::default();
        let size = vec2(1000., 800.);
        frame(&mut state, &mut rt, 0., size, vec![]);
        state.open(
            Dialog::Rename,
            Rect::from_min_size(pos2(40., 200.), vec2(160., 34.)),
            Id::new("origin"),
            0.1,
        );
        frame(&mut state, &mut rt, 0.1, size, vec![]);
        frame(&mut state, &mut rt, 0.8, size, vec![]);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(Id::new("ov-workspace-input"))
        );
        for n in 0..8 {
            frame(
                &mut state,
                &mut rt,
                1. + n as f64 * 0.1,
                size,
                vec![key(Key::Tab, n % 2 == 0)],
            );
            let id = ctx.memory(|m| m.focused());
            assert!(id.is_some(), "Tab must remain on an enabled modal control");
            if let Some(id) = id {
                let response = ctx.read_response(id).unwrap();
                assert_eq!(response.layer_id.id, state.surface.id);
            }
        }
        frame(&mut state, &mut rt, 2., size, vec![key(Key::Escape, false)]);
        frame(&mut state, &mut rt, 3., size, vec![]);
        assert!(state.dialog.is_none());
        assert_eq!(state.workspace, "Atlas workspace");
    }
    #[test]
    fn native_pointer_opens_rename_from_its_actual_button_rect() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut state = State::default();
        let size = vec2(1100., 1300.);
        frame(&mut state, &mut rt, 0., size, vec![]);
        let r = rt
            .records
            .iter()
            .find(|r| r.id == "button:Rename workspace")
            .unwrap();
        let p = pos2((r.rect[0] + r.rect[2]) * 0.5, (r.rect[1] + r.rect[3]) * 0.5);
        frame(
            &mut state,
            &mut rt,
            0.1,
            size,
            vec![egui::Event::PointerMoved(p)],
        );
        frame(
            &mut state,
            &mut rt,
            0.2,
            size,
            vec![egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }],
        );
        frame(
            &mut state,
            &mut rt,
            0.3,
            size,
            vec![egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        assert_eq!(state.dialog, Some(Dialog::Rename));
        assert!(
            state.surface.open,
            "newly opened modal must not consume its trigger click as backdrop dismissal"
        );
        let origin = state.surface.origin_focus.unwrap();
        frame(
            &mut state,
            &mut rt,
            0.8,
            size,
            vec![key(Key::Escape, false)],
        );
        frame(&mut state, &mut rt, 1.4, size, vec![]);
        assert!(state.dialog.is_none());
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(origin),
            "Escape returns focus to the exact initiating control"
        );
    }
    #[test]
    fn rename_enter_submits_after_native_text_edit_surrenders_focus() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut state = State::default();
        let size = vec2(1000., 800.);
        frame(&mut state, &mut rt, 0., size, vec![]);
        state.open(
            Dialog::Rename,
            Rect::from_min_size(pos2(40., 200.), vec2(160., 34.)),
            Id::new("origin"),
            0.1,
        );
        frame(&mut state, &mut rt, 0.1, size, vec![]);
        frame(
            &mut state,
            &mut rt,
            0.8,
            size,
            vec![egui::Event::Text("Native rename".into())],
        );
        frame(&mut state, &mut rt, 0.9, size, vec![key(Key::Enter, false)]);
        frame(&mut state, &mut rt, 1.5, size, vec![]);
        assert_eq!(state.workspace, "Native rename");
        assert!(state.dialog.is_none());
    }
    #[test]
    fn menu_keyboard_enters_submenu_returns_and_runs_export() {
        let ctx = Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut state = State::default();
        let size = vec2(1000., 800.);
        state.menu = Some(Menu {
            context: false,
            anchor: Rect::from_min_size(pos2(50., 100.), vec2(100., 34.)),
            point: None,
            origin: Id::new("origin"),
            index: 2,
            submenu: false,
            subindex: 0,
            rect: Rect::NOTHING,
            subrect: Rect::NOTHING,
            subanchor: Rect::NOTHING,
            opened: 0.,
            focus: true,
        });
        frame(&mut state, &mut rt, 0., size, vec![]);
        frame(
            &mut state,
            &mut rt,
            0.1,
            size,
            vec![key(Key::ArrowRight, false)],
        );
        assert!(state.menu.as_ref().unwrap().submenu);
        frame(
            &mut state,
            &mut rt,
            0.2,
            size,
            vec![key(Key::ArrowLeft, false)],
        );
        assert!(!state.menu.as_ref().unwrap().submenu);
        frame(
            &mut state,
            &mut rt,
            0.3,
            size,
            vec![key(Key::ArrowRight, false)],
        );
        frame(
            &mut state,
            &mut rt,
            0.4,
            size,
            vec![key(Key::ArrowDown, false)],
        );
        frame(&mut state, &mut rt, 0.5, size, vec![key(Key::Enter, false)]);
        assert!(state.menu.is_none());
        assert!(state.result.contains("Design tokens selected"));
    }
}
