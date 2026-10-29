use crate::{
    feedback::{Surface, SurfaceKind},
    icons,
    motion::{Curve, Tween},
    runtime::Runtime,
    ui::{self, ButtonKind},
};
use eframe::egui::{self, *};

const TITLES: [&str; 4] = [
    "Browse the library",
    "Choose a palette",
    "Preview motion",
    "Reuse Rust components",
];
const DESCRIPTIONS: [&str; 4] = [
    "Schematic brings spacing, type, components and motion into one place. Every example runs locally.",
    "Color pairs warm paper with a forest accent. Mono keeps the interface grayscale. Try either: this changes the whole app.",
    "A selection should travel to its new home. Try the sample, then use Motion Lab to inspect curves and springs.",
    "Use the native Rust tokens, controls, vector icons and state models in your own application.",
];
#[derive(Clone)]
struct Panel {
    opacity: Tween,
    x: Tween,
}
pub struct Guide {
    pub surface: Surface,
    pub step: usize,
    panels: [Panel; 4],
    sample: bool,
    sample_status: String,
    palette: usize,
    pending_focus: bool,
}
impl Default for Guide {
    fn default() -> Self {
        Self {
            surface: Surface::new("schematic-guide", SurfaceKind::Dialog),
            step: 0,
            panels: std::array::from_fn(|i| Panel {
                opacity: Tween::new(if i == 0 { 1.0 } else { 0.0 }, 0.0),
                x: Tween::new(0.0, 0.0),
            }),
            sample: false,
            sample_status: "Ready to move.".into(),
            palette: 0,
            pending_focus: false,
        }
    }
}
impl Guide {
    pub fn open(&mut self, rt: &mut Runtime, origin: Option<Rect>, focus: Option<Id>) {
        self.step = 0;
        self.palette = usize::from(rt.prefs.mono);
        self.sample = false;
        self.sample_status = "Ready to move.".into();
        for (i, p) in self.panels.iter_mut().enumerate() {
            p.opacity = Tween::new(if i == 0 { 1.0 } else { 0.0 }, rt.now);
            p.x = Tween::new(0.0, rt.now);
        }
        self.surface.size = vec2(560.0, 600.0);
        self.surface.show_dialog(origin, focus, rt.now);
        self.pending_focus = true;
    }
    pub fn close(&mut self) {
        self.surface.close();
    }
    pub fn visible(&self) -> bool {
        self.surface.is_visible()
    }
    pub fn change_step(&mut self, next: usize, rt: &mut Runtime) {
        let next = next.min(3);
        if next == self.step {
            return;
        }
        let direction = if next > self.step { 1.0 } else { -1.0 };
        self.step = next;
        let scale = rt.motion_scale();
        for (i, p) in self.panels.iter_mut().enumerate() {
            let active = i == next;
            let current = p.opacity.value(rt.now);
            let already_visible = current > 0.001;
            if active && !already_visible {
                p.x = Tween::new(direction * 24.0, rt.now);
                p.opacity = Tween::new(0.0, rt.now);
            }
            let start = rt.now
                + if active && !already_visible {
                    0.14 * scale as f64
                } else {
                    0.0
                };
            p.opacity.retarget(
                if active { 1.0 } else { 0.0 },
                start,
                if active { 0.46 } else { 0.14 } * scale,
                if active { Curve::Out } else { Curve::In },
            );
            p.x.retarget(
                if active { 0.0 } else { -direction * 18.0 },
                start,
                if active { 0.46 } else { 0.14 } * scale,
                if active { Curve::Out } else { Curve::In },
            );
        }
        if next == 2 {
            self.sample = false;
            self.sample_status = "Ready to move.".into();
        }
        self.pending_focus = true;
        crate::runtime::restore_focus(rt.context(), None);
        rt.repaint();
    }
    pub fn show(&mut self, ctx: &Context, rt: &mut Runtime) {
        if !self.visible() {
            return;
        }
        self.palette = usize::from(rt.prefs.mono);
        let scale = rt.motion_scale();
        let reduced = rt.reduced();
        let t = rt.tokens.clone();
        let now = rt.now;
        let width = (ctx.content_rect().width() - 32.0).min(560.0);
        let height = (ctx.content_rect().height() - 32.0).min(600.0);
        self.surface.size = vec2(width, height);
        let step = self.step;
        let current_panels = self.panels.clone();
        let sample = &mut self.sample;
        let status = &mut self.sample_status;
        let palette = &mut self.palette;
        let pending_focus = self.pending_focus;
        let mut focus_applied = false;
        let control_start = rt.records.len();
        let mut action = 0_i32;
        self.surface.scroll_body = false;
        self.surface.padding = 16.0;
        let response = self
            .surface
            .show(ctx, now, &t, scale, reduced, |ui, first| {
                let inner_width = (width - 32.0).max(224.0);
                let inner_height = (height - 32.0).max(320.0);
                let bounds = Rect::from_min_size(ui.cursor().min, vec2(inner_width, inner_height));
                let compact = inner_width < 440.0;
                let footer_height = if compact { 72.0 } else { 34.0 };
                let footer_top = bounds.bottom() - footer_height;
                let mut head = ui.new_child(
                    UiBuilder::new()
                        .id_salt("guide-head")
                        .max_rect(Rect::from_min_size(bounds.min, vec2(inner_width, 34.0)))
                        .layout(Layout::left_to_right(Align::Center)),
                );
                head.spacing_mut().item_spacing.x = 8.0;
                head.label(
                    RichText::new("SCHEMATIC / QUICK GUIDE")
                        .size(10.0)
                        .color(t.text3),
                );
                head.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = ui::button(
                        ui,
                        rt,
                        "guide-close",
                        "Close guide",
                        Some("x"),
                        ButtonKind::Ghost,
                    );
                    rt.record("guide-close", &response);
                    if response.clicked() {
                        action = 3;
                    }
                });
                let mut caption = ui.new_child(
                    UiBuilder::new()
                        .id_salt("guide-caption")
                        .max_rect(Rect::from_min_size(
                            bounds.min + vec2(8.0, 46.0),
                            vec2(inner_width - 16.0, 18.0),
                        ))
                        .layout(Layout::left_to_right(Align::Center)),
                );
                caption.label(
                    RichText::new("Explore, configure and reuse")
                        .size(12.0)
                        .color(t.text3),
                );
                caption.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} / 4", step + 1))
                            .monospace()
                            .size(12.0)
                            .color(t.text3),
                    );
                });
                let bar_width = (inner_width - 16.0 - 24.0) / 4.0;
                for (i, label) in ["Explore", "Palette", "Motion", "Reuse"].iter().enumerate() {
                    let x = bounds.left() + 8.0 + i as f32 * (bar_width + 8.0);
                    let top = bounds.top() + 74.0;
                    let bar = Rect::from_min_size(pos2(x, top), vec2(bar_width, 2.0));
                    ui.painter().rect_filled(bar, 1.0, t.border);
                    let fill = rt.animate(
                        ("guide-progress", i),
                        if i <= step { 1.0 } else { 0.0 },
                        0.46,
                        Curve::InOut,
                    );
                    ui.painter().rect_filled(
                        Rect::from_min_size(bar.min, vec2(bar_width * fill, 2.0)),
                        1.0,
                        t.accent,
                    );
                    let mut label_ui =
                        ui.new_child(UiBuilder::new().id_salt(("guide-step-label", i)).max_rect(
                            Rect::from_min_size(pos2(x, top + 8.0), vec2(bar_width, 18.0)),
                        ));
                    label_ui.label(RichText::new(*label).size(11.0).color(if i == step {
                        t.accent_text
                    } else {
                        t.text3
                    }));
                }
                let stage = Rect::from_min_max(
                    bounds.min + vec2(8.0, 114.0),
                    pos2(bounds.right() - 8.0, footer_top - 20.0),
                );
                rt.anchor("guide-stage", "Guide reading surface", stage);
                for (i, panel) in current_panels.iter().enumerate() {
                    let opacity = if reduced {
                        if i == step { 1.0 } else { 0.0 }
                    } else {
                        panel.opacity.value(now)
                    };
                    if opacity < 0.001 {
                        continue;
                    }
                    if panel.opacity.moving(now) || panel.x.moving(now) {
                        rt.repaint();
                    }
                    let dx = if reduced { 0.0 } else { panel.x.value(now) };
                    let panel_rect = stage.translate(vec2(dx, 0.0));
                    let mut content = ui.new_child(
                        UiBuilder::new()
                            .id_salt(("guide-panel", i))
                            .max_rect(panel_rect)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    content.set_clip_rect(stage.intersect(ui.clip_rect()));
                    content.set_opacity(opacity);
                    if i != step {
                        content.disable();
                    }
                    content.spacing_mut().item_spacing.y = 8.0;
                    let mut scrolling = ScrollArea::vertical()
                        .id_salt(("guide-scroll", i))
                        .max_height(stage.height())
                        .auto_shrink([false, false]);
                    if pending_focus && i == step {
                        scrolling = scrolling.vertical_scroll_offset(0.0);
                    }
                    scrolling.show(&mut content, |ui| {
                        let title = ui.add(
                            Label::new(
                                RichText::new(TITLES[i])
                                    .font(ui::strong_font(ui.ctx(), 22.0))
                                    .color(t.text),
                            )
                            .wrap(),
                        );
                        let heading = ui.interact(
                            title.rect,
                            Id::new(("guide-title", i)),
                            Sense::focusable_noninteractive(),
                        );
                        heading.widget_info(|| {
                            WidgetInfo::labeled(WidgetType::Label, true, TITLES[i])
                        });
                        ui.ctx().accesskit_node_builder(heading.id, |node| {
                            node.set_role(egui::accesskit::Role::Heading);
                            node.set_level(2);
                        });
                        if i == step && (first || pending_focus) {
                            heading.request_focus();
                            focus_applied = true;
                        }
                        rt.record(&format!("guide-title:{i}"), &heading);
                        ui.add_space(4.0);
                        ui.label(RichText::new(DESCRIPTIONS[i]).size(14.0).color(t.text2));
                        ui.add_space(16.0);
                        draw_step(ui, rt, i, sample, status, palette);
                    });
                }
                ui.painter().line_segment(
                    [
                        pos2(bounds.left(), footer_top - 10.0),
                        pos2(bounds.right(), footer_top - 10.0),
                    ],
                    Stroke::new(1.0, t.border_soft),
                );
                let footer = Rect::from_min_max(pos2(bounds.left(), footer_top), bounds.max);
                rt.anchor("guide-footer", "Guide controls", footer);
                let mut foot = ui.new_child(
                    UiBuilder::new()
                        .id_salt("guide-footer")
                        .max_rect(footer)
                        .layout(Layout::top_down(Align::Min)),
                );
                foot.spacing_mut().item_spacing.y = 6.0;
                if compact {
                    foot.label(
                        RichText::new("Close anytime. Reopen from Guide.")
                            .size(11.0)
                            .color(t.text3),
                    );
                }
                foot.horizontal(|ui| {
                    if !compact {
                        ui.label(
                            RichText::new("Close anytime. Reopen from Guide.")
                                .size(11.0)
                                .color(t.text3),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let next = ui::button(
                            ui,
                            rt,
                            "guide-next",
                            if step == 3 { "Finish guide" } else { "Next" },
                            Some("arrow-right"),
                            ButtonKind::Primary,
                        );
                        rt.record("guide-next", &next);
                        if next.clicked() {
                            action = 2;
                        }
                        ui.add_enabled_ui(step > 0, |ui| {
                            let back = ui::button(
                                ui,
                                rt,
                                "guide-back",
                                "Back",
                                Some("arrow-left"),
                                ButtonKind::Secondary,
                            );
                            rt.record("guide-back", &back);
                            if back.clicked() {
                                action = 1;
                            }
                        });
                    });
                });
                ui.advance_cursor_after_rect(bounds);
            });
        if focus_applied {
            self.pending_focus = false;
        }
        self.surface.set_focus_order(
            &rt.records[control_start..]
                .iter()
                .filter(|r| r.enabled)
                .map(|r| r.widget_id)
                .collect::<Vec<_>>(),
        );
        if response.is_some_and(|r| r.should_close) {
            action = 3;
        }
        match action {
            1 => self.change_step(self.step.saturating_sub(1), rt),
            2 => {
                if self.step == 3 {
                    rt.prefs.guide_completed = true;
                    self.close();
                } else {
                    self.change_step(self.step + 1, rt)
                }
            }
            3 => self.close(),
            _ => {}
        }
    }
}
fn draw_step(
    ui: &mut Ui,
    rt: &mut Runtime,
    step: usize,
    sample: &mut bool,
    status: &mut String,
    palette: &mut usize,
) {
    match step {
        0 => {
            ui::card(ui, rt, "guide-map", |ui, rt| {
                ui.spacing_mut().item_spacing.y = 6.0;
                for (index, (title, desc)) in [
                    (
                        "Find your foundation",
                        "Tokens, type and color in Foundations.",
                    ),
                    (
                        "Try the real interactions",
                        "Primitives, navigation, forms, charts and overlays.",
                    ),
                    (
                        "Jump straight there",
                        "Search Overview, or use Cmd / Ctrl K.",
                    ),
                ]
                .iter()
                .enumerate()
                {
                    if index > 0 {
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("0{}", index + 1))
                                .monospace()
                                .color(rt.tokens.accent_text),
                        );
                        ui.vertical(|ui| {
                            ui.label(RichText::new(*title).font(ui::strong_font(ui.ctx(), 13.0)));
                            ui::muted(ui, rt, desc);
                        });
                    });
                }
            });
        }
        1 => {
            if ui::segmented(ui, rt, "guide-palette", &["Color", "Mono"], palette) {
                rt.set_palette(*palette == 1);
            }
            ui.add_space(16.0);
            ui::card(ui, rt, "guide-preview", |ui, rt| {
                ui::badge(ui, rt, "Your workspace", "accent");
                ui.add_space(10.0);
                ui.label(RichText::new("Release progress").font(ui::strong_font(ui.ctx(), 16.0)));
                ui::muted(ui, rt, "18 of 25 tasks completed");
                ui.add_space(12.0);
                ui::progress(ui, rt, 0.72, 4.0);
            });
            ui.add_space(12.0);
            ui::muted(
                ui,
                rt,
                "Your choice is saved. Charts keep their own colors. Tune tokens also offers light and dark appearances.",
            );
        }
        2 => {
            let t = rt.tokens.clone();
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::hover());
            let left = rect.left() + 22.0;
            let right = rect.right() - 22.0;
            ui.painter().line_segment(
                [pos2(left, rect.center().y), pos2(right, rect.center().y)],
                Stroke::new(2.0, t.border),
            );
            let p = rt.animate(
                "guide-sample",
                if *sample { 1.0 } else { 0.0 },
                0.6,
                Curve::SpringSmooth,
            );
            let marker = Rect::from_center_size(
                pos2(left + (right - left) * p, rect.center().y),
                vec2(34.0, 34.0),
            );
            ui.painter()
                .rect_filled(marker, ui::rounded(10.0 * t.radius), t.accent);
            icons::paint(
                ui.painter(),
                "check",
                marker.shrink(8.0),
                t.accent_contrast,
                1.8,
                0.0,
            );
            let response = ui::button(
                ui,
                rt,
                "guide-try",
                "Try motion",
                Some("arrow-right"),
                ButtonKind::Secondary,
            );
            rt.record("guide-try", &response);
            if response.clicked() {
                *sample = !*sample;
                *status = if *sample {
                    "Selected. Try again to return."
                } else {
                    "Ready. Try again to select."
                }
                .into();
            }
            ui::muted(ui, rt, status);
            ui.add_space(12.0);
            ui::muted(
                ui,
                rt,
                if rt.reduced() {
                    "Reduced motion is on. The sample changes position instantly."
                } else {
                    "Tune tokens offers Full, Slow-mo and Reduced. System reduced motion is respected unless Full is selected."
                },
            );
        }
        _ => {
            ui::card(ui, rt, "guide-code", |ui, _| {
                ui.monospace("ui::button(\n    ui, runtime, \"save\", \"Save\",\n    Some(\"check\"), ButtonKind::Primary,\n);");
            });
            ui.add_space(12.0);
            ui::muted(
                ui,
                rt,
                "The native library is authored in Rust. Tokens, controls, vector icons and animation are public modules.",
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guide_is_optional_and_steps_are_bounded() {
        let mut guide = Guide::default();
        let mut rt = Runtime::new(Context::default(), Default::default());
        assert!(!guide.visible());
        guide.open(&mut rt, None, None);
        assert!(guide.visible());
        assert_eq!(guide.step, 0);
        guide.change_step(99, &mut rt);
        assert_eq!(guide.step, 3);
        guide.close();
        assert!(!guide.surface.open);
    }
}
