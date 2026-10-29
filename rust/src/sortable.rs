//! Native reversible list sorting. Stable item IDs own identity across lists.
//!
//! `Sortable::new(vec![vec![0,1,2]])`; call `show(ui,rt,id,&["Tasks"],64.,
//! |ui,rt,item| { /* row contents, excluding the drag handle */ None::<Action> })`.
//! Returned `(item, action)` values are ordinary row-button actions. `groups`
//! always contains live order. Call `cancel()` on route leave. Programmatic moves
//! use `move_item(item,group,index)`; the same FLIP path animates all movements.
use crate::{
    motion::{self, Curve},
    runtime::Runtime,
};
use eframe::egui::{self, Align, Id, Key, Layout, Pos2, Rect, Sense, Stroke, Vec2};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Pointer,
    Keyboard,
}
#[derive(Clone, Debug)]
struct Pickup {
    item: usize,
    mode: Mode,
    started: bool,
    original: Vec<Vec<usize>>,
    origin: Pos2,
    pointer: Pos2,
    source: Rect,
    touch: Option<(egui::TouchDeviceId, egui::TouchId)>,
}
#[derive(Clone, Copy, Debug)]
struct Travel {
    from: Vec2,
    start: f64,
    scale: f32,
}
#[derive(Clone, Copy, Debug)]
struct Landing {
    item: usize,
    from: Rect,
    start: f64,
    scale: f32,
}
#[derive(Clone, Debug)]
pub struct Sortable {
    pub groups: Vec<Vec<usize>>,
    pub announcement: String,
    pickup: Option<Pickup>,
    landing: Option<Landing>,
    enabled: bool,
    displayed: HashMap<usize, Rect>,
    final_rects: HashMap<usize, Rect>,
    flip_from: Option<HashMap<usize, Rect>>,
    travel: HashMap<usize, Travel>,
    focus: Option<usize>,
    frame_id: Id,
    names: HashMap<usize, String>,
    group_names: Vec<String>,
}
impl Default for Sortable {
    fn default() -> Self {
        Self::new(vec![vec![]])
    }
}
fn visual_progress(now: f64, start: f64, seconds: f32, scale: f32) -> f32 {
    if scale <= 0. {
        1.
    } else {
        ((now - start) as f32 / (seconds * scale)).clamp(0., 1.)
    }
}
impl Sortable {
    pub fn new(groups: Vec<Vec<usize>>) -> Self {
        Self {
            groups,
            announcement: String::new(),
            pickup: None,
            landing: None,
            enabled: true,
            displayed: HashMap::new(),
            final_rects: HashMap::new(),
            flip_from: None,
            travel: HashMap::new(),
            focus: None,
            frame_id: Id::NULL,
            names: HashMap::new(),
            group_names: vec![],
        }
    }
    pub fn active(&self) -> bool {
        self.pickup.as_ref().is_some_and(|p| p.started)
    }
    pub fn mode(&self) -> Option<Mode> {
        self.pickup.as_ref().filter(|p| p.started).map(|p| p.mode)
    }
    pub fn picked_item(&self) -> Option<usize> {
        self.pickup.as_ref().filter(|p| p.started).map(|p| p.item)
    }
    pub fn position(&self, item: usize) -> Option<(usize, usize)> {
        self.groups
            .iter()
            .enumerate()
            .find_map(|(g, rows)| rows.iter().position(|i| *i == item).map(|i| (g, i)))
    }
    pub fn set_labels(&mut self, labels: &[&str]) {
        self.names = labels
            .iter()
            .enumerate()
            .map(|(i, label)| (i, (*label).into()))
            .collect();
    }
    fn label(&self, item: usize) -> String {
        self.names
            .get(&item)
            .cloned()
            .unwrap_or_else(|| format!("Item {}", item + 1))
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.cancel();
            self.settle();
            self.focus = None;
        }
    }
    pub fn settle(&mut self) {
        self.travel.clear();
        self.flip_from = None;
        self.landing = None;
    }
    pub fn cancel(&mut self) {
        if let Some(p) = self.pickup.take()
            && p.started
        {
            self.flip_from = Some(self.displayed.clone());
            self.groups = p.original;
            self.focus = Some(p.item);
            self.announcement = format!("Cancelled. Restored {}", self.describe(p.item));
        }
        self.landing = None;
    }
    pub fn drop_item(&mut self) {
        if let Some(p) = self.pickup.take()
            && p.started
        {
            self.focus = Some(p.item);
            self.announcement = format!("Dropped {}", self.describe(p.item));
        }
    }
    pub fn begin_keyboard(&mut self, item: usize) -> bool {
        if !self.enabled || self.position(item).is_none() {
            return false;
        }
        self.cancel();
        self.settle();
        self.pickup = Some(Pickup {
            item,
            mode: Mode::Keyboard,
            started: true,
            original: self.groups.clone(),
            origin: Pos2::ZERO,
            pointer: Pos2::ZERO,
            source: Rect::NOTHING,
            touch: None,
        });
        self.focus = Some(item);
        self.announcement = format!("Picked up {}", self.describe(item));
        true
    }
    pub fn move_item(&mut self, item: usize, group: usize, index: usize) -> bool {
        self.cancel();
        self.settle();
        let moved = self.reorder(item, group, index);
        self.focus = None;
        moved
    }
    pub fn reset(&mut self, groups: Vec<Vec<usize>>) {
        self.cancel();
        self.focus = None;
        self.flip_from = Some(self.displayed.clone());
        self.groups = groups;
        self.announcement = "Original order restored.".into();
    }
    pub fn describe(&self, item: usize) -> String {
        self.position(item)
            .map(|(g, i)| {
                format!(
                    "{}, position {} of {}{}.",
                    self.label(item),
                    i + 1,
                    self.groups[g].len(),
                    if self.groups.len() > 1 {
                        format!(
                            ", {}",
                            self.group_names
                                .get(g)
                                .cloned()
                                .unwrap_or_else(|| format!("list {}", g + 1))
                        )
                    } else {
                        String::new()
                    }
                )
            })
            .unwrap_or_default()
    }
    fn reorder(&mut self, item: usize, group: usize, index: usize) -> bool {
        let Some((old_group, old_index)) = self.position(item) else {
            return false;
        };
        if group >= self.groups.len() {
            return false;
        }
        let max = self.groups[group].len() - usize::from(old_group == group);
        let index = index.min(max);
        if old_group == group && old_index == index {
            return false;
        }
        self.flip_from = Some(self.displayed.clone());
        self.groups[old_group].remove(old_index);
        self.groups[group].insert(index, item);
        self.focus = Some(item);
        true
    }
    pub fn key(&mut self, key: Key) -> bool {
        let Some(p) = self.pickup.as_ref().filter(|p| p.mode == Mode::Keyboard) else {
            return false;
        };
        let item = p.item;
        if key == Key::Escape {
            self.cancel();
            return true;
        }
        if key == Key::Enter || key == Key::Space || key == Key::Tab {
            self.drop_item();
            return true;
        }
        let Some((g, i)) = self.position(item) else {
            return false;
        };
        let len = self.groups[g].len();
        let (dest, pos) = match key {
            Key::Home => (g, 0),
            Key::End => (g, len.saturating_sub(1)),
            Key::ArrowUp if i > 0 => (g, i - 1),
            Key::ArrowDown if i + 1 < len => (g, i + 1),
            Key::ArrowUp | Key::ArrowLeft if g > 0 => (
                g - 1,
                if key == Key::ArrowUp {
                    self.groups[g - 1].len()
                } else {
                    i.min(self.groups[g - 1].len())
                },
            ),
            Key::ArrowDown | Key::ArrowRight if g + 1 < self.groups.len() => (
                g + 1,
                if key == Key::ArrowDown {
                    0
                } else {
                    i.min(self.groups[g + 1].len())
                },
            ),
            _ => return false,
        };
        if self.reorder(item, dest, pos) {
            self.announcement = self.describe(item)
        }
        true
    }
    fn handle_id(&self, item: usize) -> Id {
        self.frame_id.with(("grip", item))
    }
    fn rollback_input(&mut self, ui: &egui::Ui) {
        if self.mode() == Some(Mode::Keyboard) && ui.input(|i| i.key_pressed(Key::Tab)) {
            self.drop_item();
            self.focus = None;
        }
        let cancel = ui.input(|i| {
            i.events.iter().any(|e| match e {
                egui::Event::WindowFocused(false) => true,
                egui::Event::Touch {
                    device_id,
                    id,
                    phase: egui::TouchPhase::Cancel,
                    ..
                } => self
                    .pickup
                    .as_ref()
                    .is_some_and(|p| p.touch == Some((*device_id, *id))),
                _ => false,
            })
        });
        if cancel {
            self.cancel();
            self.settle();
        }
        if self.pickup.is_some()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape))
        {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            self.cancel()
        }
    }
    /// Labels are the list headings. Row contents use native egui widgets and
    /// retain IDs derived from the item, never the item's current position.
    pub fn show<A>(
        &mut self,
        ui: &mut egui::Ui,
        rt: &mut Runtime,
        id: impl std::hash::Hash,
        labels: &[&str],
        row_height: f32,
        mut row: impl FnMut(&mut egui::Ui, &mut Runtime, usize) -> Option<A>,
    ) -> Vec<(usize, A)> {
        self.frame_id = ui.make_persistent_id(id);
        self.group_names = labels.iter().map(|s| (*s).into()).collect();
        self.rollback_input(ui);
        if rt.reduced() {
            self.settle()
        }
        let flip = self.flip_from.take();
        let mut rects = HashMap::new();
        let mut displayed = HashMap::new();
        let mut groups_rect = vec![];
        let mut grips = vec![];
        let mut actions = vec![];
        for (g, items) in self.groups.clone().iter().enumerate() {
            ui.add_space(if g == 0 { 0. } else { 16. });
            if let Some(label) = labels.get(g) {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(*label).color(rt.tokens.text2).strong());
                    ui.label(
                        egui::RichText::new(format!("{:02}", items.len()))
                            .monospace()
                            .color(rt.tokens.text3),
                    );
                });
                ui.add_space(6.);
            }
            let start = ui.cursor().min;
            let width = ui.available_width();
            if items.is_empty() {
                let (r, _) = ui.allocate_exact_size(egui::vec2(width, row_height), Sense::hover());
                ui.painter().rect(
                    r,
                    crate::ui::rounded(8. * rt.tokens.radius),
                    rt.tokens.bg_sunken,
                    Stroke::new(1.0_f32, rt.tokens.border_soft),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop a document here",
                    egui::FontId::proportional(12.),
                    rt.tokens.text3,
                );
            }
            for &item in items {
                let (final_rect, _) =
                    ui.allocate_exact_size(egui::vec2(width, row_height), Sense::hover());
                rects.insert(item, final_rect);
                if let Some(before) = flip.as_ref().and_then(|f| f.get(&item)) {
                    let delta = before.min - final_rect.min;
                    if delta.length() > 0.5 && !rt.reduced() {
                        self.travel.insert(
                            item,
                            Travel {
                                from: delta,
                                start: rt.now,
                                scale: rt.motion_scale(),
                            },
                        );
                    }
                }
                let offset = self
                    .travel
                    .get(&item)
                    .map(|t| {
                        let p = visual_progress(rt.now, t.start, 0.32, t.scale);
                        t.from * (1. - motion::ease(Curve::Out, p))
                    })
                    .unwrap_or(Vec2::ZERO);
                if offset.length() > 0.1 {
                    rt.repaint()
                } else {
                    self.travel.remove(&item);
                }
                let painted = final_rect.translate(offset);
                displayed.insert(item, painted);
                let source = self
                    .pickup
                    .as_ref()
                    .is_some_and(|p| p.started && p.item == item);
                let pointer = source && self.mode() == Some(Mode::Pointer);
                let landing = self.landing.is_some_and(|p| p.item == item);
                ui.painter().rect(
                    painted,
                    crate::ui::rounded(8. * rt.tokens.radius),
                    if source {
                        rt.tokens.accent_soft
                    } else {
                        rt.tokens.surface
                    },
                    Stroke::new(
                        if source { 2. } else { 1.0_f32 },
                        if source {
                            rt.tokens.accent_ring
                        } else {
                            rt.tokens.border
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt((self.frame_id, item))
                        .global_scope(true)
                        .max_rect(painted.shrink2(egui::vec2(8., 8.)))
                        .layout(Layout::left_to_right(Align::Center)),
                );
                if pointer || landing {
                    child.set_opacity(0.22)
                }
                let grip_rect =
                    Rect::from_min_size(child.cursor().min, egui::vec2(28., row_height - 16.));
                let grip_id = self.handle_id(item);
                let grip = child.interact(grip_rect, grip_id, Sense::click_and_drag());
                grips.push(grip_rect);
                child.advance_cursor_after_rect(grip_rect);
                let color = if grip.hovered() || grip.has_focus() || source {
                    rt.tokens.accent_text
                } else {
                    rt.tokens.text3
                };
                crate::icons::paint(
                    child.painter(),
                    "grip",
                    Rect::from_center_size(grip_rect.center(), egui::vec2(16., 16.)),
                    color,
                    1.5,
                    0.,
                );
                if grip.has_focus() {
                    child.memory_mut(|m| {
                        m.set_focus_lock_filter(
                            grip.id,
                            egui::EventFilter {
                                horizontal_arrows: true,
                                vertical_arrows: true,
                                escape: true,
                                ..Default::default()
                            },
                        )
                    });
                    child.painter().rect_stroke(
                        grip_rect,
                        4.,
                        Stroke::new(2.0_f32, rt.tokens.accent_ring),
                        egui::StrokeKind::Outside,
                    );
                }
                let info = format!(
                    "Reorder {}. Space or Enter picks up and drops. Arrows move between positions and lists. Escape cancels.",
                    self.label(item)
                );
                grip.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::Button,
                        self.enabled,
                        source,
                        &info,
                    )
                });
                grip.clone().on_hover_text(info);
                if self.focus == Some(item) {
                    grip.request_focus();
                    if self.mode() == Some(Mode::Keyboard) {
                        grip.scroll_to_me(None);
                    }
                    self.focus = None;
                }
                rt.record(
                    &format!("sort-grip-{}-{item}", self.frame_id.value()),
                    &grip,
                );
                if self.enabled
                    && self.mode() != Some(Mode::Pointer)
                    && grip.is_pointer_button_down_on()
                    && ui.input(|i| i.pointer.primary_pressed())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    self.cancel();
                    self.settle();
                    let touch = ui.input(|i| {
                        i.events.iter().find_map(|event| {
                            if let egui::Event::Touch {
                                device_id,
                                id,
                                phase: egui::TouchPhase::Start,
                                pos: touch_pos,
                                ..
                            } = event
                            {
                                if touch_pos.distance(pos) < 2. {
                                    Some((*device_id, *id))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    });
                    self.pickup = Some(Pickup {
                        item,
                        mode: Mode::Pointer,
                        started: false,
                        original: self.groups.clone(),
                        origin: pos,
                        pointer: pos,
                        source: painted,
                        touch,
                    });
                    grip.request_focus();
                }
                if self.enabled && grip.has_focus() {
                    let keys = [
                        Key::Space,
                        Key::Enter,
                        Key::ArrowUp,
                        Key::ArrowDown,
                        Key::ArrowLeft,
                        Key::ArrowRight,
                        Key::Home,
                        Key::End,
                        Key::Tab,
                    ];
                    for key in keys {
                        let fresh=ui.input(|i|i.events.iter().any(|event|matches!(event,egui::Event::Key{key:event_key,pressed:true,repeat:false,..}if *event_key==key)));
                        let pressed = if key == Key::Tab {
                            ui.input(|i| i.key_pressed(key))
                        } else {
                            ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key))
                        };
                        if !pressed {
                            continue;
                        }
                        if (key == Key::Space || key == Key::Enter) && !fresh {
                            continue;
                        }
                        if key != Key::Tab {
                            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                        }
                        if self.mode() == Some(Mode::Keyboard) {
                            self.key(key);
                        } else if key == Key::Space || key == Key::Enter {
                            self.begin_keyboard(item);
                        }
                    }
                }
                let result = child
                    .add_enabled_ui(!pointer && !landing, |ui| row(ui, rt, item))
                    .inner;
                if let Some(action) = result {
                    actions.push((item, action))
                }
                ui.add_space(4.);
            }
            groups_rect.push(Rect::from_min_max(
                start,
                egui::pos2(start.x + width, ui.cursor().min.y.max(start.y + row_height)),
            ));
        }
        self.final_rects = rects;
        self.displayed = displayed;
        if self.mode() == Some(Mode::Keyboard)
            && ui.input(|i| {
                i.pointer.primary_pressed()
                    && i.pointer.interact_pos().is_some_and(|p| {
                        groups_rect.iter().any(|r| r.contains(p))
                            && !grips.iter().any(|r| r.contains(p))
                    })
            })
        {
            self.cancel();
            self.focus = None;
            rt.repaint();
        }
        self.pointer_frame(ui, rt, &groups_rect);
        if let Some(item) = self.picked_item()
            && self.mode() == Some(Mode::Pointer)
            && let Some(rect) = self.final_rects.get(&item)
        {
            ui.painter().line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(2.0_f32, rt.tokens.accent),
            );
        }
        let ghost = self
            .pickup
            .as_ref()
            .filter(|p| p.started && p.mode == Mode::Pointer)
            .map(|p| (p.item, p.source.translate(p.pointer - p.origin)));
        let landing = self.landing.and_then(|l| {
            let target = *self.final_rects.get(&l.item)?;
            let p = visual_progress(rt.now, l.start, 0.22, l.scale);
            if p >= 1. {
                None
            } else {
                Some((
                    l.item,
                    Rect::from_min_size(
                        l.from.min.lerp(target.min, motion::ease(Curve::Out, p)),
                        target.size(),
                    ),
                ))
            }
        });
        if self.landing.is_some() && landing.is_none() {
            self.landing = None;
        }
        if let Some((item, rect)) = ghost.or(landing) {
            if landing.is_some() {
                rt.repaint()
            }
            egui::Area::new(self.frame_id.with("ghost"))
                .order(egui::Order::Tooltip)
                .interactable(false)
                .movable(false)
                .fixed_pos(rect.min)
                .show(ui.ctx(), |ui| {
                    ui.set_width(rect.width());
                    egui::Frame::new()
                        .fill(rt.tokens.surface_raised)
                        .stroke(Stroke::new(1.0_f32, rt.tokens.accent_ring))
                        .corner_radius(crate::ui::rounded(10. * rt.tokens.radius))
                        .inner_margin(8.)
                        .show(ui, |ui| {
                            ui.set_height(row_height - 16.);
                            ui.horizontal(|ui| {
                                let (grip, _) = ui.allocate_exact_size(
                                    egui::vec2(28., row_height - 16.),
                                    Sense::hover(),
                                );
                                crate::icons::paint(
                                    ui.painter(),
                                    "grip",
                                    Rect::from_center_size(grip.center(), egui::vec2(16., 16.)),
                                    rt.tokens.accent_text,
                                    1.5,
                                    0.,
                                );
                                ui.add_enabled_ui(false, |ui| {
                                    let _ = row(ui, rt, item);
                                });
                            });
                        });
                });
        }
        if self.flip_from.is_some() || self.focus.is_some() {
            rt.repaint()
        }
        actions
    }
    fn pointer_frame(&mut self, ui: &mut egui::Ui, rt: &mut Runtime, groups: &[Rect]) {
        let Some(p) = self.pickup.as_mut().filter(|p| p.mode == Mode::Pointer) else {
            return;
        };
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            p.pointer = pos
        }
        if !p.started && p.pointer.distance(p.origin) >= 6. {
            p.started = true;
            self.announcement = format!(
                "Picked up {}.",
                self.names
                    .get(&p.item)
                    .cloned()
                    .unwrap_or_else(|| format!("Item {}", p.item + 1))
            );
        }
        let (item, started, pos, source, origin) =
            (p.item, p.started, p.pointer, p.source, p.origin);
        if started {
            let target = groups
                .iter()
                .enumerate()
                .filter(|(_, r)| pos.x >= r.left() - 24. && pos.x <= r.right() + 24.)
                .map(|(g, r)| (g, (r.top() - pos.y).max(pos.y - r.bottom()).max(0.)))
                .filter(|(_, d)| *d <= 56.)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((g, _)) = target {
                let others: Vec<_> = self.groups[g]
                    .iter()
                    .copied()
                    .filter(|i| *i != item)
                    .collect();
                let index = others
                    .iter()
                    .position(|i| {
                        self.final_rects
                            .get(i)
                            .is_some_and(|r| pos.y < r.center().y)
                    })
                    .unwrap_or(others.len());
                if self.reorder(item, g, index) {
                    self.announcement = self.describe(item);
                    rt.repaint();
                }
            }
            let clip = ui.clip_rect();
            let band = 56_f32.min(clip.height() / 4.);
            let speed = if pos.y < clip.top() + band {
                -((clip.top() + band - pos.y) / band).clamp(0., 1.)
            } else if pos.y > clip.bottom() - band {
                ((pos.y - clip.bottom() + band) / band).clamp(0., 1.)
            } else {
                0.
            };
            let content = groups
                .iter()
                .copied()
                .reduce(Rect::union)
                .unwrap_or(Rect::NOTHING);
            if (speed < 0. && content.top() < clip.top())
                || (speed > 0. && content.bottom() > clip.bottom())
            {
                let dt = ui.input(|i| i.stable_dt).min(0.032);
                ui.scroll_with_delta_animation(
                    egui::vec2(0., -speed * 960. * dt),
                    egui::style::ScrollAnimation::none(),
                );
                rt.repaint();
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        if ui.input(|i| i.pointer.primary_released()) {
            let p = self.pickup.take().unwrap();
            if p.started {
                self.focus = Some(item);
                self.announcement = format!("Dropped {}", self.describe(item));
                if !rt.reduced() {
                    self.landing = Some(Landing {
                        item,
                        from: source.translate(pos - origin),
                        start: rt.now,
                        scale: rt.motion_scale(),
                    });
                    rt.repaint()
                }
            }
        } else if !ui.input(|i| i.pointer.primary_down()) {
            self.cancel();
            self.settle()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input_frame(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut Sortable,
        time: f64,
        events: Vec<egui::Event>,
        scroll: bool,
    ) -> f32 {
        let mut offset = 0.;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(600., 700.))),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    if scroll {
                        offset = egui::ScrollArea::vertical()
                            .id_salt("sort-test-scroll")
                            .max_height(180.)
                            .show(ui, |ui| {
                                s.show(
                                    ui,
                                    rt,
                                    "test-sort",
                                    &["First", "Second"],
                                    60.,
                                    |ui, _, item| {
                                        ui.label(format!("Row {item}"));
                                        None::<()>
                                    },
                                );
                            })
                            .state
                            .offset
                            .y;
                    } else {
                        s.show(
                            ui,
                            rt,
                            "test-sort",
                            &["First", "Second"],
                            60.,
                            |ui, _, item| {
                                ui.label(format!("Row {item}"));
                                None::<()>
                            },
                        );
                    }
                });
            },
        );
        offset
    }
    fn pointer(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn key_event(key: Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn grip(rt: &Runtime, item: usize) -> Pos2 {
        let record = rt
            .records
            .iter()
            .find(|r| r.id.starts_with("sort-grip-") && r.id.ends_with(&format!("-{item}")))
            .unwrap();
        egui::pos2(
            (record.rect[0] + record.rect[2]) / 2.,
            (record.rect[1] + record.rect[3]) / 2.,
        )
    }
    #[test]
    fn native_pointer_threshold_flip_drop_and_focus() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = Sortable::new(vec![vec![0, 1, 2, 3]]);
        input_frame(&ctx, &mut rt, &mut s, 0., vec![], false);
        input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], false);
        let start = grip(&rt, 0);
        let end = egui::pos2(start.x, s.final_rects[&3].bottom() - 4.);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.02,
            vec![egui::Event::PointerMoved(start), pointer(start, true)],
            false,
        );
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.03,
            vec![egui::Event::PointerMoved(start + egui::vec2(1., 1.))],
            false,
        );
        assert!(!s.active());
        assert_eq!(s.groups[0], vec![0, 1, 2, 3]);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.04,
            vec![egui::Event::PointerMoved(end)],
            false,
        );
        assert!(s.active());
        assert_eq!(s.groups[0], vec![1, 2, 3, 0]);
        input_frame(&ctx, &mut rt, &mut s, 0.05, vec![], false);
        assert!(s.travel.values().any(|t| t.from.length() > 1.));
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.06,
            vec![pointer(end, false)],
            false,
        );
        assert!(!s.active());
        assert!(s.landing.is_some());
        input_frame(&ctx, &mut rt, &mut s, 1., vec![], false);
        assert!(s.landing.is_none());
        assert_eq!(ctx.memory(|m| m.focused()), Some(s.handle_id(0)));
        assert_eq!(s.groups[0], vec![1, 2, 3, 0]);
    }
    #[test]
    fn native_cross_group_touchcancel_escape_and_blur_restore_snapshot() {
        for cancel in 0..3 {
            let ctx = egui::Context::default();
            let mut rt = Runtime::new(ctx.clone(), Default::default());
            let mut s = Sortable::new(vec![vec![0, 1], vec![2, 3]]);
            let original = s.groups.clone();
            input_frame(&ctx, &mut rt, &mut s, 0., vec![], false);
            input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], false);
            let start = grip(&rt, 0);
            let end = s.final_rects[&3].center();
            let mut press = vec![egui::Event::PointerMoved(start), pointer(start, true)];
            if cancel == 1 {
                press.push(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::Start,
                    pos: start,
                    force: None,
                });
            }
            input_frame(&ctx, &mut rt, &mut s, 0.02, press, false);
            input_frame(
                &ctx,
                &mut rt,
                &mut s,
                0.03,
                vec![egui::Event::PointerMoved(end)],
                false,
            );
            assert_ne!(s.groups, original);
            assert!(s.groups[1].contains(&0));
            let event = match cancel {
                0 => key_event(Key::Escape),
                1 => egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(1),
                    phase: egui::TouchPhase::Cancel,
                    pos: end,
                    force: None,
                },
                _ => egui::Event::WindowFocused(false),
            };
            input_frame(&ctx, &mut rt, &mut s, 0.04, vec![event], false);
            assert_eq!(s.groups, original);
            assert!(!s.active());
            input_frame(
                &ctx,
                &mut rt,
                &mut s,
                0.05,
                vec![pointer(end, false)],
                false,
            );
            assert_eq!(s.groups, original);
        }
    }
    #[test]
    fn native_keyboard_events_move_and_escape_restores_focus() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = Sortable::new(vec![vec![0, 1], vec![2, 3]]);
        input_frame(&ctx, &mut rt, &mut s, 0., vec![], false);
        s.focus = Some(0);
        input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], false);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.02,
            vec![key_event(Key::Space)],
            false,
        );
        assert_eq!(s.mode(), Some(Mode::Keyboard));
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.03,
            vec![key_event(Key::ArrowRight)],
            false,
        );
        assert_eq!(s.groups, vec![vec![1], vec![0, 2, 3]]);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.04,
            vec![key_event(Key::Escape)],
            false,
        );
        assert_eq!(s.groups, vec![vec![0, 1], vec![2, 3]]);
        input_frame(&ctx, &mut rt, &mut s, 0.05, vec![], false);
        assert_eq!(ctx.memory(|m| m.focused()), Some(s.handle_id(0)));
    }
    #[test]
    fn native_edge_scroll_and_leave_cleanup() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = Sortable::new(vec![(0..20).collect()]);
        let initial = s.groups.clone();
        input_frame(&ctx, &mut rt, &mut s, 0., vec![], true);
        input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], true);
        let start = grip(&rt, 0);
        let edge = egui::pos2(start.x, 180.);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.02,
            vec![egui::Event::PointerMoved(start), pointer(start, true)],
            true,
        );
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.03,
            vec![egui::Event::PointerMoved(edge)],
            true,
        );
        let mut offset = 0.;
        for frame in 4..30 {
            offset = input_frame(&ctx, &mut rt, &mut s, frame as f64 / 60., vec![], true);
        }
        assert!(offset > 40., "edge scroll offset {offset}");
        assert!(s.active());
        s.set_enabled(false);
        assert_eq!(s.groups, initial);
        assert!(!s.active());
        assert!(s.landing.is_none() && s.travel.is_empty());
    }
    #[test]
    fn native_keyboard_repeat_does_not_drop_and_tab_preserves_next_focus() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        let mut s = Sortable::new(vec![vec![0, 1]]);
        input_frame(&ctx, &mut rt, &mut s, 0., vec![], false);
        s.focus = Some(0);
        input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], false);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.02,
            vec![key_event(Key::Space)],
            false,
        );
        assert!(s.active());
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.03,
            vec![egui::Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: true,
                modifiers: egui::Modifiers::NONE,
            }],
            false,
        );
        assert!(s.active());
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.04,
            vec![key_event(Key::Tab)],
            false,
        );
        assert!(!s.active());
        let focus = ctx.memory(|m| m.focused());
        assert_ne!(focus, Some(s.handle_id(0)));
        input_frame(&ctx, &mut rt, &mut s, 0.05, vec![], false);
        assert_eq!(ctx.memory(|m| m.focused()), focus);
    }
    #[test]
    fn reduced_drag_reaches_final_geometry_and_ignores_unrelated_touch_cancel() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let mut s = Sortable::new(vec![vec![0, 1], vec![]]);
        input_frame(&ctx, &mut rt, &mut s, 0., vec![], false);
        input_frame(&ctx, &mut rt, &mut s, 0.01, vec![], false);
        let start = grip(&rt, 0);
        let end = egui::pos2(start.x, 240.);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.02,
            vec![egui::Event::PointerMoved(start), pointer(start, true)],
            false,
        );
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.03,
            vec![egui::Event::PointerMoved(end)],
            false,
        );
        assert_eq!(s.groups, vec![vec![1], vec![0]]);
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.04,
            vec![egui::Event::Touch {
                device_id: egui::TouchDeviceId(9),
                id: egui::TouchId(9),
                phase: egui::TouchPhase::Cancel,
                pos: end,
                force: None,
            }],
            false,
        );
        assert!(s.active());
        input_frame(
            &ctx,
            &mut rt,
            &mut s,
            0.05,
            vec![pointer(end, false)],
            false,
        );
        assert!(s.landing.is_none() && s.travel.is_empty());
        assert_eq!(s.groups, vec![vec![1], vec![0]]);
    }
    #[test]
    fn keyboard_transaction_rolls_back_every_group() {
        let mut s = Sortable::new(vec![vec![0, 1], vec![2, 3]]);
        assert!(s.begin_keyboard(1));
        s.key(Key::ArrowRight);
        assert_eq!(s.groups, vec![vec![0], vec![2, 1, 3]]);
        s.key(Key::End);
        assert_eq!(s.groups[1], vec![2, 3, 1]);
        s.key(Key::Escape);
        assert_eq!(s.groups, vec![vec![0, 1], vec![2, 3]]);
        assert!(!s.active());
    }
    #[test]
    fn keyboard_boundaries_and_drop() {
        let mut s = Sortable::new(vec![vec![0, 1], vec![]]);
        s.begin_keyboard(0);
        s.key(Key::ArrowUp);
        assert_eq!(s.groups[0], vec![0, 1]);
        s.key(Key::End);
        s.key(Key::ArrowDown);
        assert_eq!(s.groups, vec![vec![1], vec![0]]);
        s.key(Key::Enter);
        s.cancel();
        assert_eq!(s.groups, vec![vec![1], vec![0]]);
    }
    #[test]
    fn programmatic_order_preserves_identity() {
        let mut s = Sortable::new(vec![vec![0], vec![1, 2, 3]]);
        assert!(s.move_item(2, 0, 1));
        assert_eq!(s.groups, vec![vec![0, 2], vec![1, 3]]);
        s.begin_keyboard(3);
        s.key(Key::ArrowLeft);
        s.set_enabled(false);
        assert_eq!(s.groups, vec![vec![0, 2], vec![1, 3]]);
        let mut all = s.groups.concat();
        all.sort();
        assert_eq!(all, vec![0, 1, 2, 3]);
    }
}
