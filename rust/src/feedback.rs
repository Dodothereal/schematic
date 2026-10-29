//! Native shared feedback. This module owns no Runtime and requires no web assets.
//!
//! Integration: keep one `State` in the application; enqueue with `toast` or `push`,
//! then call `show(ctx, now, &tokens, motion_scale, reduced)` after the page. A page
//! can own another State when its notifications must be canceled on route leave.
//! `Surface` is the reusable focus-contained dialog/drawer/sheet primitive.
use crate::{
    motion::{self, Curve},
    tokens::Tokens,
};
use eframe::egui::{
    self, Area, Color32, Context, Frame, Id, Key, Order, Pos2, Rect, Response, RichText, Sense,
    Stroke, Ui, Vec2, pos2, vec2,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Tween {
    pub value: f32,
    from: f32,
    target: f32,
    start: f64,
    seconds: f32,
    curve: Curve,
}
impl Tween {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            from: value,
            target: value,
            start: 0.,
            seconds: 0.,
            curve: Curve::OutExpo,
        }
    }
    pub fn sample(&mut self, now: f64) -> f32 {
        let t = if self.seconds <= 0. {
            1.
        } else {
            ((now - self.start) / self.seconds as f64).clamp(0., 1.) as f32
        };
        self.value = self.from + (self.target - self.from) * motion::ease(self.curve, t);
        self.value
    }
    pub fn to(&mut self, target: f32, now: f64, seconds: f32, curve: Curve) {
        self.sample(now);
        if self.target != target {
            self.from = self.value;
            self.target = target;
            self.start = now;
            self.seconds = seconds;
            self.curve = curve;
        }
        if seconds <= 0. {
            self.value = target;
            self.from = target;
            self.seconds = 0.;
        }
    }
    pub fn moving(&self, now: f64) -> bool {
        self.seconds > 0. && now - self.start < self.seconds as f64
    }
    pub fn snap(&mut self, value: f32) {
        *self = Self::new(value);
    }
}
fn duration(seconds: f32, scale: f32, reduced: bool) -> f32 {
    if reduced { 0. } else { seconds * scale }
}
fn alpha(color: Color32, value: f32) -> Color32 {
    color.gamma_multiply(value.clamp(0., 1.))
}
fn panel_frame(t: &Tokens) -> Frame {
    Frame::new()
        .fill(t.surface_raised)
        .stroke(Stroke::new(1., t.border))
        .corner_radius(t.radius * 12.)
        .inner_margin(16.)
}
fn tone_color(t: &Tokens, tone: &str) -> Color32 {
    match tone {
        "success" => t.success,
        "warning" => t.warning,
        "danger" => t.danger,
        "info" => t.info,
        _ => t.accent_text,
    }
}

// Feedback is rendered without a Runtime borrow, but its native controls share
// the same current-frame focus registry as Runtime::record.
fn record_focus(response: &Response, ids: &mut std::collections::HashSet<Id>) {
    if response.enabled() {
        ids.insert(response.id);
        response.ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<std::collections::HashSet<Id>>(Id::new(
                "schematic-live-focus-ids",
            ))
            .insert(response.id);
        });
    }
}

pub type ToastId = u64;
#[derive(Clone, Debug)]
pub struct ToastOptions {
    pub title: String,
    pub desc: String,
    pub tone: String,
    pub icon: String,
    /// Real seconds of reading time; None remains until explicitly dismissed.
    pub duration: Option<f64>,
    pub action: Option<String>,
    pub owner: Option<String>,
}
impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            desc: String::new(),
            tone: "neutral".into(),
            icon: String::new(),
            duration: Some(4.2),
            action: None,
            owner: None,
        }
    }
}
#[derive(Clone, Debug)]
struct Toast {
    id: ToastId,
    options: ToastOptions,
    remaining: Option<f64>,
    last: f64,
    leaving: bool,
    right: bool,
    reveal: Tween,
    y: Tween,
    x: Tween,
    scale: Tween,
    height: f32,
    drag: Vec2,
    drag_started: f64,
    created: f64,
}
pub struct State {
    scope: u64,
    next: ToastId,
    toasts: Vec<Toast>,
    expanded: bool,
    hovered: bool,
    focused: bool,
    focus_ids: std::collections::HashSet<Id>,
    /// Action events are returned rather than holding closures or a Runtime.
    pub actions: Vec<ToastId>,
}
impl Default for State {
    fn default() -> Self {
        static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);
        Self {
            scope: NEXT_SCOPE.fetch_add(1, Ordering::Relaxed),
            next: 0,
            toasts: Vec::new(),
            expanded: false,
            hovered: false,
            focused: false,
            focus_ids: Default::default(),
            actions: Vec::new(),
        }
    }
}
impl State {
    pub fn toast(&mut self, title: &str, desc: &str, tone: &str, now: f64) -> ToastId {
        self.push(
            ToastOptions {
                title: title.into(),
                desc: desc.into(),
                tone: tone.into(),
                ..Default::default()
            },
            now,
        )
    }
    pub fn push(&mut self, options: ToastOptions, now: f64) -> ToastId {
        self.next += 1;
        let id = self.next;
        self.toasts.insert(
            0,
            Toast {
                id,
                remaining: options.duration,
                options,
                last: now,
                leaving: false,
                right: false,
                reveal: Tween::new(0.),
                y: Tween::new(80.),
                x: Tween::new(0.),
                scale: Tween::new(0.98),
                height: 92.,
                drag: Vec2::ZERO,
                drag_started: now,
                created: now,
            },
        );
        id
    }
    pub fn dismiss(&mut self, id: ToastId) {
        if let Some(t) = self.toasts.iter_mut().find(|t| t.id == id) {
            t.leaving = true;
        }
    }
    pub fn dismiss_owner(&mut self, owner: &str) {
        for t in &mut self.toasts {
            if t.options.owner.as_deref() == Some(owner) {
                t.leaving = true;
            }
        }
    }
    pub fn clear(&mut self) {
        self.toasts.clear();
        self.expanded = false;
        self.hovered = false;
        self.focused = false;
    }
    /// Use when removing this queue without rendering another frame of it.
    pub fn clear_with_focus(&mut self, ctx: &Context) {
        if ctx
            .memory(|m| m.focused())
            .is_some_and(|id| self.focus_ids.contains(&id))
        {
            crate::runtime::restore_focus(ctx, None);
        }
        self.clear();
    }
    pub fn len(&self) -> usize {
        self.toasts.iter().filter(|t| !t.leaving).count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn contains(&self, id: ToastId) -> bool {
        self.toasts.iter().any(|t| t.id == id && !t.leaving)
    }
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }
    /// Pure time progression, also used by tests. `paused` includes hover/focus and inactivity.
    pub fn advance(&mut self, now: f64, paused: bool) {
        for toast in &mut self.toasts {
            let delta = (now - toast.last).max(0.);
            toast.last = now;
            if !paused
                && !toast.leaving
                && let Some(left) = toast.remaining.as_mut()
            {
                *left = (*left - delta).max(0.);
                if *left == 0. {
                    toast.leaving = true;
                }
            }
        }
    }
    pub fn show(&mut self, ctx: &Context, now: f64, t: &Tokens, scale: f32, reduced: bool) {
        let previous_focus_ids = std::mem::take(&mut self.focus_ids);
        let mut focus_ids = std::collections::HashSet::new();
        let focused = ctx.input(|i| i.focused);
        self.advance(now, self.expanded || !focused);
        let screen = ctx.content_rect();
        let width = 360_f32.min(screen.width() - 32.).max(160.);
        let front = self
            .toasts
            .iter()
            .find(|t| !t.leaving)
            .map_or(0., |t| t.height);
        let mut offset = 0.;
        let mut live_index = 0;
        let mut hovered = false;
        let mut has_focus = false;
        let mut dismiss = Vec::new();
        let mut actions = Vec::new();
        let mut stack_rect = Rect::NOTHING;
        let mut layers = Vec::new();
        for toast in &mut self.toasts {
            let visible = toast.leaving || live_index < if self.expanded { 6 } else { 3 };
            let target_y = if self.expanded {
                -offset
            } else {
                -(live_index as f32) * 10.
            };
            let target_scale = if self.expanded {
                1.
            } else {
                1. - live_index as f32 * 0.05
            };
            if !toast.leaving {
                offset += toast.height + 10.;
                live_index += 1;
            }
            let leave_y = if toast.leaving {
                target_y + 14.
            } else {
                target_y
            };
            toast.y.to(
                leave_y,
                now,
                duration(if toast.leaving { 0.24 } else { 0.6 }, scale, reduced),
                if toast.leaving {
                    Curve::In
                } else {
                    Curve::SpringSmooth
                },
            );
            toast.scale.to(
                if toast.leaving { 0.96 } else { target_scale },
                now,
                duration(0.6, scale, reduced),
                Curve::SpringSmooth,
            );
            toast.reveal.to(
                if toast.leaving { 0. } else { 1. },
                now,
                duration(if toast.leaving { 0.24 } else { 0.32 }, scale, reduced),
                if toast.leaving { Curve::In } else { Curve::Out },
            );
            let opacity = toast.reveal.sample(now);
            let y = toast.y.sample(now);
            let s = toast.scale.sample(now);
            if toast.y.moving(now) || toast.scale.moving(now) || toast.reveal.moving(now) {
                ctx.request_repaint();
            }
            if !visible {
                continue;
            }
            let behind = !self.expanded && live_index > 1 && !toast.leaving;
            let h = if behind { front } else { toast.height };
            if toast.right {
                toast
                    .x
                    .to(width + 32., now, duration(0.24, scale, reduced), Curve::In);
            }
            let x_shift = toast.x.sample(now);
            if toast.x.moving(now) {
                ctx.request_repaint();
            }
            let pos = pos2(
                screen.right() - 24. - width + (width * (1. - s) / 2.) + x_shift,
                screen.bottom() - 24. - h + y,
            );
            let id = Id::new(("schematic-toast", self.scope, toast.id));
            let response = Area::new(id)
                .fixed_pos(pos)
                .order(Order::Tooltip)
                .fade_in(false)
                .movable(false)
                .constrain(false)
                .interactable(!toast.leaving)
                .show(ctx, |ui| {
                    if toast.leaving {
                        ui.disable();
                    }
                    ui.ctx().accesskit_node_builder(ui.id(), |node| {
                        node.set_role(egui::accesskit::Role::Status);
                        node.set_live(egui::accesskit::Live::Polite);
                        node.set_live_atomic();
                    });
                    ui.set_width(width * s);
                    ui.set_opacity(opacity * (1. - (x_shift / 260.).clamp(0., 0.6)));
                    if behind {
                        let (r, _) = ui.allocate_exact_size(vec2(width * s, h), Sense::hover());
                        ui.painter()
                            .rect_filled(r, t.radius * 12., t.surface_raised);
                        ui.painter().rect_stroke(
                            r,
                            t.radius * 12.,
                            Stroke::new(1., t.border),
                            egui::StrokeKind::Inside,
                        );
                        return None;
                    }
                    let panel = panel_frame(t).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let (icon_rect, _) =
                                ui.allocate_exact_size(vec2(20., 20.), Sense::hover());
                            let icon = if !toast.options.icon.is_empty() {
                                toast.options.icon.as_str()
                            } else {
                                match toast.options.tone.as_str() {
                                    "success" => "check-circle",
                                    "warning" => "alert",
                                    "danger" => "x-circle",
                                    "info" => "info",
                                    _ => "bell",
                                }
                            };
                            crate::icons::paint(
                                ui.painter(),
                                icon,
                                icon_rect,
                                tone_color(t, &toast.options.tone),
                                1.6,
                                if reduced {
                                    0.
                                } else {
                                    ((now - toast.created) / (1.2 * scale.max(0.001)) as f64)
                                        .clamp(0., 1.) as f32
                                },
                            );
                            if !reduced && now - toast.created < (1.2 * scale) as f64 {
                                ctx.request_repaint();
                            }
                            ui.vertical(|ui| {
                                ui.set_width((width - 108.).max(80.));
                                ui.label(
                                    RichText::new(&toast.options.title).strong().color(t.text),
                                );
                                if !toast.options.desc.is_empty() {
                                    ui.label(
                                        RichText::new(&toast.options.desc).size(12.).color(t.text2),
                                    );
                                }
                            });
                            ui.vertical(|ui| {
                                let close = ui
                                    .button(RichText::new("×").size(17.))
                                    .on_hover_text("Dismiss notification");
                                record_focus(&close, &mut focus_ids);
                                if close.clicked() {
                                    dismiss.push(toast.id);
                                }
                                if let Some(action) = &toast.options.action {
                                    let action_button = ui.small_button(action);
                                    record_focus(&action_button, &mut focus_ids);
                                    if action_button.clicked() {
                                        actions.push(toast.id);
                                        dismiss.push(toast.id);
                                    }
                                }
                            });
                        });
                    });
                    let drag_rect = Rect::from_min_max(
                        panel.response.rect.min,
                        pos2(
                            panel.response.rect.right() - 48.,
                            panel.response.rect.bottom(),
                        ),
                    );
                    let drag = ui.interact(drag_rect, id.with("drag"), Sense::drag());
                    record_focus(&drag, &mut focus_ids);
                    Some(drag)
                });
            if !toast.leaving {
                layers.push(response.response.layer_id);
            }
            toast.height = response.response.rect.height();
            stack_rect = stack_rect.union(response.response.rect);
            hovered |= response.response.contains_pointer();
            let focus_id = ctx.memory(|m| m.focused());
            has_focus |= focus_id.is_some_and(|id| {
                ctx.read_response(id)
                    .is_some_and(|r| r.layer_id == response.response.layer_id)
            });
            // Drag only the non-control body, preserving buttons and keyboard activation.
            let Some(drag) = response.inner else {
                continue;
            };
            if drag.drag_started() {
                toast.drag_started = now;
            }
            if drag.dragged() {
                let dx = drag.drag_delta().x;
                toast.drag = vec2(if dx < 0. { dx * 0.2 } else { dx }, 0.);
                toast.x.snap(toast.drag.x);
                ctx.request_repaint();
            }
            if drag.drag_stopped() {
                let dx = drag.drag_delta().x;
                if dx > 90. || dx / (((now - toast.drag_started) * 1000.) as f32).max(1.) > 0.6 {
                    toast.right = true;
                    dismiss.push(toast.id);
                } else {
                    toast
                        .x
                        .to(0., now, duration(0.6, scale, reduced), Curve::SpringSmooth);
                }
                toast.drag = Vec2::ZERO;
                ctx.request_repaint();
            }
        }
        for layer in layers.into_iter().rev() {
            ctx.move_to_top(layer);
        }
        if ctx
            .memory(|m| m.focused())
            .is_some_and(|id| previous_focus_ids.contains(&id) && !focus_ids.contains(&id))
        {
            crate::runtime::restore_focus(ctx, None);
        }
        self.focus_ids = focus_ids;
        // Include the gaps between fanned notifications in the reading region.
        hovered |= ctx
            .pointer_hover_pos()
            .is_some_and(|p| stack_rect.contains(p));
        self.hovered = hovered;
        self.focused = has_focus;
        self.expanded = hovered || has_focus;
        for id in dismiss {
            self.dismiss(id);
        }
        self.actions.extend(actions);
        self.toasts.retain(|toast| {
            !toast.leaving || toast.reveal.value > 0.001 || toast.reveal.moving(now)
        });
        if !self.expanded
            && focused
            && let Some(remaining) = self
                .toasts
                .iter()
                .filter(|t| !t.leaving)
                .filter_map(|t| t.remaining)
                .min_by(f64::total_cmp)
        {
            ctx.request_repaint_after(Duration::from_secs_f64(remaining.max(0.001)));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    Dialog,
    Morph,
    Drawer,
    Sheet,
}
#[derive(Clone, Debug)]
pub struct Surface {
    pub id: Id,
    pub kind: SurfaceKind,
    pub open: bool,
    pub origin: Option<Rect>,
    pub origin_focus: Option<Id>,
    reveal: Tween,
    pub rect: Rect,
    pub size: Vec2,
    /// Wrap the body in a vertical scroll area. Disable for fixed header/stage/footer layouts.
    pub scroll_body: bool,
    /// Inner spacing between the surface border and its content, in points.
    pub padding: f32,
    pub drag_y: f32,
    first_frame: bool,
    restore: bool,
    closed: bool,
    focus_order: Vec<Id>,
}
pub struct SurfaceResponse<R> {
    pub inner: R,
    pub response: Response,
    pub should_close: bool,
    pub first_frame: bool,
}
impl Surface {
    pub fn new(id: impl std::hash::Hash, kind: SurfaceKind) -> Self {
        Self {
            id: Id::new(id),
            kind,
            open: false,
            origin: None,
            origin_focus: None,
            reveal: Tween::new(0.),
            rect: Rect::NOTHING,
            size: vec2(480., 400.),
            scroll_body: true,
            padding: 24.,
            drag_y: 0.,
            first_frame: false,
            restore: true,
            closed: true,
            focus_order: Vec::new(),
        }
    }
    pub fn show_dialog(&mut self, origin: Option<Rect>, origin_focus: Option<Id>, now: f64) {
        self.open = true;
        self.closed = false;
        self.origin = origin;
        self.origin_focus = origin_focus;
        self.first_frame = true;
        self.focus_order.clear();
        self.restore = true;
        self.reveal.sample(now);
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn close_at(&mut self, now: f64, scale: f32, reduced: bool) {
        self.open = false;
        self.reveal
            .to(0., now, duration(0.32, scale, reduced), Curve::In);
    }
    pub fn close_immediately(&mut self, ctx: &Context, restore: bool) {
        self.open = false;
        self.closed = true;
        self.reveal.snap(0.);
        self.drag_y = 0.;
        if restore {
            self.restore_focus(ctx);
        }
    }
    pub fn is_visible(&self) -> bool {
        self.open || !self.closed
    }
    pub fn is_closing(&self) -> bool {
        !self.open && !self.closed
    }
    /// Supply enabled controls in their rendered order; Tab wraps at both ends.
    pub fn set_focus_order(&mut self, controls: &[Id]) {
        self.focus_order = controls.to_vec();
    }
    pub fn set_origin_focus(&mut self, id: Id) {
        self.origin_focus = Some(id);
    }
    fn restore_focus(&self, ctx: &Context) {
        crate::runtime::restore_focus(ctx, self.origin_focus);
    }
    pub fn show<R>(
        &mut self,
        ctx: &Context,
        now: f64,
        t: &Tokens,
        scale: f32,
        reduced: bool,
        body: impl FnOnce(&mut Ui, bool) -> R,
    ) -> Option<SurfaceResponse<R>> {
        if !self.is_visible() {
            return None;
        }
        self.reveal.to(
            if self.open { 1. } else { 0. },
            now,
            duration(
                if self.open {
                    if self.kind == SurfaceKind::Morph {
                        0.6
                    } else {
                        0.46
                    }
                } else {
                    0.32
                },
                scale,
                reduced,
            ),
            if self.open {
                if self.kind == SurfaceKind::Morph {
                    Curve::SpringSmooth
                } else {
                    Curve::OutExpo
                }
            } else {
                Curve::In
            },
        );
        let p = self.reveal.sample(now);
        if self.reveal.moving(now) {
            ctx.request_repaint();
        }
        if !self.open && p <= 0.001 && !self.reveal.moving(now) {
            self.closed = true;
            self.drag_y = 0.;
            if self.restore {
                self.restore_focus(ctx);
            }
            return None;
        }
        let screen = ctx.content_rect();
        let size = vec2(
            self.size.x.min(screen.width() - 32.).max(160.),
            self.size.y.min(screen.height() - 32.).max(120.),
        );
        let target = match self.kind {
            SurfaceKind::Drawer => Rect::from_min_size(
                pos2(screen.right() - size.x, screen.top()),
                vec2(size.x, screen.height()),
            ),
            SurfaceKind::Sheet => Rect::from_min_size(
                pos2(screen.center().x - size.x / 2., screen.bottom() - size.y),
                size,
            ),
            _ => Rect::from_center_size(screen.center(), size),
        };
        let from = match self.kind {
            SurfaceKind::Morph => self.origin.unwrap_or(target.shrink(12.)),
            SurfaceKind::Drawer => target.translate(vec2(size.x, 0.)),
            SurfaceKind::Sheet => target.translate(vec2(0., size.y)),
            _ => target.translate(vec2(0., 16.)).shrink(6.),
        };
        let animated = if reduced {
            target
        } else {
            Rect::from_min_max(from.min.lerp(target.min, p), from.max.lerp(target.max, p))
        }
        .translate(vec2(0., self.drag_y));
        // Consume Escape before TextEdit can use it to surrender focus.
        let escape = self.open
            && !egui::Popup::is_any_open(ctx)
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape));
        let mut tab_focus = None;
        if self.open && !self.focus_order.is_empty() {
            let backwards = ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, Key::Tab));
            let forwards =
                !backwards && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Tab));
            if forwards || backwards {
                let current = ctx.memory(|m| m.focused());
                let len = self.focus_order.len();
                let next = if let Some(index) =
                    self.focus_order.iter().position(|id| Some(*id) == current)
                {
                    if backwards {
                        (index + len - 1) % len
                    } else {
                        (index + 1) % len
                    }
                } else if backwards {
                    len - 1
                } else {
                    0
                };
                tab_focus = Some(self.focus_order[next]);
            }
        }
        let first = self.first_frame;
        self.first_frame = false;
        let width = animated.width().max(80.);
        let height = animated.height().max(80.);
        let mut modal = egui::Modal::new(self.id)
            .backdrop_color(alpha(t.overlay, p))
            .area(
                Area::new(self.id)
                    .kind(egui::UiKind::Modal)
                    .sense(Sense::hover())
                    .order(Order::Foreground)
                    .interactable(true)
                    .fixed_pos(animated.min)
                    .constrain(false)
                    .fade_in(false),
            );
        modal.frame = Some(Frame::NONE);
        let response = modal.show(ctx, |ui| {
            ui.set_width(width);
            ui.set_opacity(p);
            if !self.open {
                ui.disable();
            }
            let padding = self.padding.max(0.);
            let inset = padding * 2. + 2.;
            let mut frame = panel_frame(t).inner_margin(padding);
            if self.kind == SurfaceKind::Drawer {
                frame = frame.corner_radius(0.);
            }
            if self.kind == SurfaceKind::Sheet {
                frame = frame.corner_radius(egui::CornerRadius {
                    nw: (t.radius * 16.) as u8,
                    ne: (t.radius * 16.) as u8,
                    sw: 0,
                    se: 0,
                });
            }
            frame
                .show(ui, |ui| {
                    ui.set_min_width((width - inset).max(30.));
                    ui.set_min_height((height - inset).max(30.));
                    if self.scroll_body {
                        egui::ScrollArea::vertical()
                            .id_salt(self.id.with("scroll"))
                            .max_height((height - inset).max(30.))
                            .show(ui, |ui| body(ui, first))
                            .inner
                    } else {
                        body(ui, first)
                    }
                })
                .inner
        });
        if tab_focus.is_some() {
            // The body has now recorded its controls for this frame. Never
            // request one of last frame's IDs before validating it is still live.
            crate::runtime::restore_focus(ctx, tab_focus);
        }
        self.rect = response.response.rect;
        let should_close = escape || response.should_close();
        Some(SurfaceResponse {
            inner: response.inner,
            response: response.response,
            should_close,
            first_frame: first,
        })
    }
}

/// Tooltip placement is native geometry; no browser metadata is used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}
/// Observe tooltip controls with `show`, then render once with `finish` each frame.
/// One native area preserves the warm group glide, text swap and exit animation.
pub struct TooltipState {
    current: Option<Id>,
    suppressed: Option<Id>,
    since: f64,
    warm_until: f64,
    visible: bool,
    last_frame: u64,
    position: Option<Pos2>,
    anchor: Rect,
    side: TooltipSide,
    label: String,
    keys: Vec<String>,
    reveal: Tween,
    x: Tween,
    y: Tween,
    swap: Tween,
}
impl Default for TooltipState {
    fn default() -> Self {
        Self {
            current: None,
            suppressed: None,
            since: 0.,
            warm_until: 0.,
            visible: false,
            last_frame: 0,
            position: None,
            anchor: Rect::NOTHING,
            side: TooltipSide::Top,
            label: String::new(),
            keys: Vec::new(),
            reveal: Tween::new(0.),
            x: Tween::new(0.),
            y: Tween::new(0.),
            swap: Tween::new(1.),
        }
    }
}
impl TooltipState {
    pub fn hide(&mut self, now: f64) {
        let was_warm = self.visible || self.reveal.value > 0. || now < self.warm_until;
        self.current = None;
        self.visible = false;
        self.warm_until = if was_warm { now + 0.4 } else { now };
    }
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ctx: &Context,
        response: &Response,
        label: &str,
        keys: &[&str],
        now: f64,
        _t: &Tokens,
        _reduced: bool,
    ) {
        self.show_side(ctx, response, label, keys, TooltipSide::Top, now);
    }
    pub fn show_side(
        &mut self,
        ctx: &Context,
        response: &Response,
        label: &str,
        keys: &[&str],
        side: TooltipSide,
        now: f64,
    ) {
        let engaged = response.hovered() || response.has_focus();
        if ctx.input(|i| {
            i.pointer.any_pressed()
                || i.raw_scroll_delta != Vec2::ZERO
                || i.key_pressed(Key::Escape)
        }) {
            if let Some(current) = self.current {
                self.suppressed = Some(current);
            } else if engaged && self.suppressed.is_none() {
                self.suppressed = Some(response.id);
            }
            self.hide(now);
            return;
        }
        if self.suppressed == Some(response.id) {
            if engaged {
                return;
            } else {
                self.suppressed = None;
            }
        }
        if !engaged {
            if self.current == Some(response.id) {
                self.hide(now);
            }
            return;
        }
        if self.current != Some(response.id) {
            self.current = Some(response.id);
            self.since = now;
        }
        self.last_frame = ctx.cumulative_frame_nr();
        if !response.has_focus()
            && !self.visible
            && now >= self.warm_until
            && now - self.since < 0.45
        {
            ctx.request_repaint_after(Duration::from_secs_f64(
                (0.45 - (now - self.since)).max(0.001),
            ));
            return;
        }
        if self.label != label {
            self.swap.snap(0.);
        }
        self.visible = true;
        self.anchor = response.rect;
        self.side = side;
        self.label = label.into();
        self.keys = keys.iter().map(|k| (*k).to_string()).collect();
    }
    pub fn finish(&mut self, ctx: &Context, now: f64, t: &Tokens, scale: f32, reduced: bool) {
        if self.current.is_some() && self.last_frame != ctx.cumulative_frame_nr() {
            self.hide(now);
        }
        self.reveal.to(
            if self.visible { 1. } else { 0. },
            now,
            duration(0.14, scale, reduced),
            if self.visible { Curve::Out } else { Curve::In },
        );
        let opacity = self.reveal.sample(now);
        if self.reveal.moving(now) {
            ctx.request_repaint();
        }
        if opacity <= 0.001 && !self.visible {
            return;
        }
        let painter = ctx.debug_painter();
        let label_width = painter
            .layout_no_wrap(self.label.clone(), egui::FontId::proportional(12.), t.bg)
            .size()
            .x;
        let key_width = self
            .keys
            .iter()
            .map(|key| {
                painter
                    .layout_no_wrap(key.clone(), egui::FontId::monospace(11.), t.bg)
                    .size()
                    .x
                    + 12.
            })
            .sum::<f32>();
        let screen = ctx.content_rect();
        let size = vec2(
            (label_width + key_width + 16.).min(screen.width() - 16.),
            30.,
        );
        let mut side = self.side;
        if side == TooltipSide::Top && self.anchor.top() - size.y - 8. < screen.top() + 8. {
            side = TooltipSide::Bottom;
        }
        if side == TooltipSide::Bottom && self.anchor.bottom() + size.y + 8. > screen.bottom() - 8.
        {
            side = TooltipSide::Top;
        }
        let mut target = match side {
            TooltipSide::Top => pos2(
                self.anchor.center().x - size.x / 2.,
                self.anchor.top() - size.y - 8.,
            ),
            TooltipSide::Bottom => pos2(
                self.anchor.center().x - size.x / 2.,
                self.anchor.bottom() + 8.,
            ),
            TooltipSide::Left => pos2(
                self.anchor.left() - size.x - 8.,
                self.anchor.center().y - size.y / 2.,
            ),
            TooltipSide::Right => pos2(
                self.anchor.right() + 8.,
                self.anchor.center().y - size.y / 2.,
            ),
        };
        target.x = target.x.clamp(
            screen.left() + 8.,
            (screen.right() - size.x - 8.).max(screen.left() + 8.),
        );
        target.y = target.y.clamp(
            screen.top() + 8.,
            (screen.bottom() - size.y - 8.).max(screen.top() + 8.),
        );
        if self.position.is_none() {
            self.x.snap(target.x);
            self.y.snap(target.y);
        }
        self.x
            .to(target.x, now, duration(0.36, scale, reduced), Curve::Out);
        self.y
            .to(target.y, now, duration(0.36, scale, reduced), Curve::Out);
        self.swap
            .to(1., now, duration(0.22, scale, reduced), Curve::Out);
        let text_alpha = self.swap.sample(now);
        let position = pos2(self.x.sample(now), self.y.sample(now));
        self.position = Some(position);
        if self.x.moving(now) || self.y.moving(now) || self.swap.moving(now) {
            ctx.request_repaint();
        }
        Area::new(Id::new("schematic-shared-tooltip"))
            .order(Order::Tooltip)
            .interactable(false)
            .fixed_pos(
                position
                    + vec2(
                        0.,
                        (1. - opacity)
                            * if side == TooltipSide::Top {
                                3.
                            } else if side == TooltipSide::Bottom {
                                -3.
                            } else {
                                0.
                            },
                    ),
            )
            .fade_in(false)
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                ui.ctx().accesskit_node_builder(ui.id(), |node| {
                    node.set_role(egui::accesskit::Role::Tooltip)
                });
                Frame::new()
                    .fill(t.text)
                    .corner_radius(4.)
                    .inner_margin(6.)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&self.label)
                                    .size(12.)
                                    .color(t.bg.gamma_multiply(text_alpha)),
                            );
                            for key in &self.keys {
                                Frame::new()
                                    .fill(t.bg.gamma_multiply(0.1))
                                    .corner_radius(3.)
                                    .inner_margin(2.)
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(key)
                                                .monospace()
                                                .size(11.)
                                                .color(t.bg.gamma_multiply(text_alpha)),
                                        );
                                    });
                            }
                        });
                    });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reading_time_pauses_and_persistent_notifications_remain() {
        let mut s = State::default();
        let id = s.toast("saved", "", "success", 0.);
        let persistent = s.push(
            ToastOptions {
                duration: None,
                ..Default::default()
            },
            0.,
        );
        s.advance(2., false);
        s.advance(20., true);
        assert!(s.contains(id));
        s.advance(22.3, false);
        assert!(!s.contains(id));
        assert!(s.contains(persistent));
    }
    #[test]
    fn owner_cleanup_does_not_dismiss_other_notifications() {
        let mut s = State::default();
        let a = s.push(
            ToastOptions {
                owner: Some("overlays".into()),
                ..Default::default()
            },
            0.,
        );
        let b = s.toast("other", "", "info", 0.);
        s.dismiss_owner("overlays");
        assert!(!s.contains(a));
        assert!(s.contains(b));
    }
    #[test]
    fn tween_retarget_starts_at_displayed_value() {
        let mut t = Tween::new(0.);
        t.to(1., 0., 1., Curve::Linear);
        let halfway = t.sample(0.5);
        t.to(0., 0.5, 1., Curve::Linear);
        assert_eq!(t.sample(0.5), halfway);
        assert!((t.sample(1.) - 0.25).abs() < 0.0001);
        t.to(1., 1., 0., Curve::Linear);
        assert_eq!(t.sample(1.), 1.);
    }
}

#[cfg(test)]
mod native_frames {
    use super::*;
    fn render(state: &mut State, ctx: &Context, now: f64, pointer: Option<Pos2>) {
        let events = pointer
            .map(|p| vec![egui::Event::PointerMoved(p)])
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                time: Some(now),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800., 600.))),
                events,
                ..Default::default()
            },
            |ctx| {
                state.show(ctx, now, &Tokens::new(false, false, 168., 1.), 1., false);
            },
        );
    }
    #[test]
    fn native_toast_fans_pauses_reading_and_resumes_its_remaining_time() {
        let ctx = Context::default();
        let mut state = State::default();
        let a = state.toast("first", "context", "info", 0.);
        state.toast("second", "more context", "success", 0.);
        render(&mut state, &ctx, 0., Some(pos2(100., 100.)));
        render(&mut state, &ctx, 0.4, Some(pos2(100., 100.)));
        assert!(!state.expanded);
        render(&mut state, &ctx, 0.5, Some(pos2(600., 550.)));
        assert!(state.expanded);
        let before = state.toasts.iter().find(|t| t.id == a).unwrap().remaining;
        render(&mut state, &ctx, 20., Some(pos2(600., 550.)));
        assert!(state.contains(a));
        assert_eq!(
            state.toasts.iter().find(|t| t.id == a).unwrap().remaining,
            before
        );
        render(&mut state, &ctx, 20.1, Some(pos2(100., 100.)));
        assert!(!state.expanded);
        render(&mut state, &ctx, 25., Some(pos2(100., 100.)));
        assert!(!state.contains(a));
    }
    #[test]
    fn independent_global_and_page_queues_have_distinct_native_ids() {
        let a = State::default();
        let b = State::default();
        assert_ne!(a.scope, b.scope);
    }
    #[test]
    fn origin_morph_has_intermediate_geometry_and_close_can_interrupt_opening() {
        let ctx = Context::default();
        let tokens = Tokens::new(false, false, 168., 1.);
        let origin = Rect::from_min_size(pos2(30., 100.), vec2(220., 120.));
        let mut s = Surface::new("surface-test", SurfaceKind::Morph);
        s.size = vec2(500., 300.);
        s.show_dialog(Some(origin), None, 0.);
        let draw = |surface: &mut Surface, now: f64| {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(now),
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(900., 700.))),
                    ..Default::default()
                },
                |ctx| {
                    surface.show(ctx, now, &tokens, 1., false, |ui, _| {
                        ui.label("Native content");
                    });
                },
            );
        };
        draw(&mut s, 0.);
        let start = s.rect;
        draw(&mut s, 0.12);
        let mid = s.rect;
        assert!(mid.left() > start.left());
        assert!(mid.width() > start.width());
        assert!(mid.width() < 510.);
        s.close_at(0.12, 1., false);
        draw(&mut s, 0.12);
        assert!((s.rect.left() - mid.left()).abs() <= 1.);
        draw(&mut s, 0.3);
        assert!(s.rect.left() < mid.left());
        draw(&mut s, 0.6);
        assert!(!s.is_visible());
    }
}

#[cfg(test)]
mod tooltip_tests {
    use super::*;
    #[test]
    fn tooltip_intent_keyboard_warm_glide_and_escape_follow_native_input() {
        let ctx = Context::default();
        let t = Tokens::new(false, false, 168., 1.);
        let mut tip = TooltipState::default();
        let a = Id::new("tip-a");
        let b = Id::new("tip-b");
        let mut render = |now: f64, pos: Pos2, events: Vec<egui::Event>, focus: Option<Id>| {
            let mut events = events;
            events.insert(0, egui::Event::PointerMoved(pos));
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(now),
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(500., 300.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    if let Some(id) = focus {
                        ctx.memory_mut(|m| m.request_focus(id));
                    }
                    egui::CentralPanel::default().show(ctx, |ui| {
                        for (id, label, x) in [(a, "Copy", 20.), (b, "Favorite", 100.)] {
                            let r = ui.interact(
                                Rect::from_min_size(pos2(x, 80.), vec2(50., 32.)),
                                id,
                                Sense::click(),
                            );
                            tip.show(ctx, &r, label, &["Ctrl", "C"], now, &t, false);
                        }
                        tip.finish(ctx, now, &t, 1., false);
                    });
                },
            );
            (tip.visible, tip.current, tip.position)
        };
        render(-0.1, pos2(350., 250.), vec![], None);
        assert!(!render(0., pos2(30., 90.), vec![], None).0);
        assert!(!render(0.3, pos2(30., 90.), vec![], None).0);
        assert!(render(0.46, pos2(30., 90.), vec![], None).0);
        let old = render(0.6, pos2(30., 90.), vec![], None).2.unwrap();
        let warm = render(0.61, pos2(110., 90.), vec![], None);
        assert!(warm.0);
        assert_eq!(warm.1, Some(b));
        assert!((warm.2.unwrap().x - old.x).abs() < 1.);
        assert!(
            !render(
                0.7,
                pos2(110., 90.),
                vec![egui::Event::Key {
                    key: Key::Escape,
                    physical_key: Some(Key::Escape),
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default()
                }],
                None
            )
            .0
        );
        assert!(render(1., pos2(350., 250.), vec![], Some(a)).0);
    }
}

#[cfg(test)]
mod accessible_focus_tests {
    use super::*;
    use crate::runtime::{Preferences, Runtime};

    fn input(time: f64) -> egui::RawInput {
        egui::RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(900., 700.))),
            ..Default::default()
        }
    }
    fn assert_focus_has_node(output: &egui::FullOutput) {
        let update = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("native accessibility tree");
        assert!(
            update.nodes.iter().any(|(id, _)| *id == update.focus),
            "focus {:?} has no node in this frame",
            update.focus
        );
    }
    #[test]
    fn modal_close_restores_only_a_return_control_recorded_in_the_current_frame() {
        for keep_origin in [false, true] {
            let ctx = Context::default();
            ctx.enable_accesskit();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            let mut surface = Surface::new(("focus-regression", keep_origin), SurfaceKind::Dialog);
            let mut origin = None;
            let output = ctx.run(input(0.), |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    let r = crate::ui::button(
                        ui,
                        &mut rt,
                        "return-control",
                        "Open dialog",
                        None,
                        crate::ui::ButtonKind::Secondary,
                    );
                    origin = Some(r.id);
                    surface.show_dialog(Some(r.rect), origin, rt.now);
                });
                let tokens = rt.tokens.clone();
                surface.show(ctx, rt.now, &tokens, 0., true, |ui, _| {
                    let r = ui.button("Dialog action");
                    rt.record("modal-action", &r);
                    r.request_focus();
                });
            });
            assert_focus_has_node(&output);
            let output = ctx.run(input(1.), |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    if keep_origin {
                        crate::ui::button(
                            ui,
                            &mut rt,
                            "return-control",
                            "Open dialog",
                            None,
                            crate::ui::ButtonKind::Secondary,
                        );
                    } else {
                        ui.label("The original control was removed.");
                    }
                });
                surface.close_immediately(ctx, true);
            });
            assert_focus_has_node(&output);
            assert_eq!(
                ctx.memory(|m| m.focused()),
                if keep_origin { origin } else { None }
            );
        }
    }
    #[test]
    fn removing_a_focused_toast_control_never_leaves_an_accesskit_focus_id() {
        for clear_queue in [false, true] {
            let ctx = Context::default();
            ctx.enable_accesskit();
            let mut rt = Runtime::new(ctx.clone(), Preferences::default());
            let mut state = State::default();
            let toast = state.push(
                ToastOptions {
                    title: "Pending".into(),
                    duration: None,
                    ..Default::default()
                },
                0.,
            );
            // New Areas perform a disabled sizing pass before controls become live.
            let initial = ctx.run(input(-0.1), |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Page content");
                });
                state.show(ctx, rt.now, &rt.tokens, 0., true);
            });
            assert_focus_has_node(&initial);
            let output = ctx.run(input(0.), |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Page content");
                });
                state.show(ctx, rt.now, &rt.tokens, 0., true);
                let close = state
                    .focus_ids
                    .iter()
                    .copied()
                    .find(|id| {
                        ctx.read_response(*id)
                            .is_some_and(|r| !r.sense.senses_drag())
                    })
                    .expect("toast dismiss control");
                // The request targets a real control emitted in this frame.
                crate::runtime::restore_focus(ctx, Some(close));
            });
            assert_focus_has_node(&output);
            assert!(ctx.memory(|m| m.focused()).is_some());
            let output = ctx.run(input(1.), |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Page content");
                });
                if clear_queue {
                    state.clear_with_focus(ctx);
                } else {
                    state.dismiss(toast);
                }
                state.show(ctx, rt.now, &rt.tokens, 0., true);
            });
            assert_focus_has_node(&output);
            assert!(ctx.memory(|m| m.focused()).is_none());
        }
    }
}
