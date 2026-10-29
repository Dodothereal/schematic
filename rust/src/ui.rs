use crate::{icons, motion::Curve, runtime::Runtime};
use eframe::egui::*;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Soft,
    Danger,
}
pub fn rounded(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.clamp(0.0, 255.0).round() as u8)
}
pub fn strong_font(ctx: &Context, size: f32) -> FontId {
    if ctx
        .data(|d| d.get_temp::<bool>(Id::new("schematic-native-strong-font")))
        .unwrap_or(false)
    {
        FontId::new(size, FontFamily::Name("Schematic Semibold".into()))
    } else {
        FontId::proportional(size)
    }
}
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let ac = a.to_array();
    let bc = b.to_array();
    Color32::from_rgba_premultiplied(
        (ac[0] as f32 + (bc[0] as f32 - ac[0] as f32) * t) as u8,
        (ac[1] as f32 + (bc[1] as f32 - ac[1] as f32) * t) as u8,
        (ac[2] as f32 + (bc[2] as f32 - ac[2] as f32) * t) as u8,
        (ac[3] as f32 + (bc[3] as f32 - ac[3] as f32) * t) as u8,
    )
}
pub fn button(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    label: &str,
    icon: Option<&str>,
    kind: ButtonKind,
) -> Response {
    let id = ui.make_persistent_id(id);
    let t = rt.tokens.clone();
    let font = FontId::proportional(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.into(), font, Color32::PLACEHOLDER);
    let width = galley.size().x + 24.0 + if icon.is_some() { 26.0 } else { 0.0 };
    let (rect, response) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let response = response.with_new_rect(rect);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label));
    let h = rt.animate(
        (id, "hover"),
        if response.hovered() || response.has_focus() {
            1.0
        } else {
            0.0
        },
        0.14,
        Curve::Out,
    );
    let pressed = rt.animate(
        (id, "press"),
        if response.is_pointer_button_down_on() {
            1.0
        } else {
            0.0
        },
        0.08,
        Curve::Out,
    );
    let (bg, fg, border) = match kind {
        ButtonKind::Primary => (
            mix(t.accent, t.accent_hover, h),
            t.accent_contrast,
            t.accent,
        ),
        ButtonKind::Secondary => (
            mix(t.surface, t.surface2, h),
            t.text,
            mix(t.border, t.border_strong, h),
        ),
        ButtonKind::Ghost => (
            mix(Color32::TRANSPARENT, t.surface2, h),
            t.text2,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Soft => (
            mix(t.accent_soft, t.accent_soft2, h),
            t.accent_text,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Danger => (
            mix(t.danger, t.danger_text, h),
            if rt.prefs.dark { t.bg } else { Color32::WHITE },
            t.danger,
        ),
    };
    let mut paint = ui.painter().clone();
    if !ui.is_enabled() {
        paint.set_opacity(0.45);
    }
    let pr = rect.shrink2(vec2(
        rect.width() * 0.015 * pressed,
        rect.height() * 0.015 * pressed,
    ));
    paint.rect(
        pr,
        rounded(6.0 * t.radius),
        bg,
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );
    let mut x = pr.left() + 12.0;
    if let Some(name) = icon {
        let progress = rt.icon_progress((id, "icon"), response.hovered() || response.has_focus());
        icons::paint(
            &paint,
            name,
            Rect::from_center_size(pos2(x + 8.0, pr.center().y), vec2(16.0, 16.0)),
            fg,
            1.6,
            progress,
        );
        x += 26.0;
    }
    paint.galley(pos2(x, pr.center().y - galley.size().y / 2.0), galley, fg);
    if response.has_focus() {
        paint.rect_stroke(
            pr.expand(3.0),
            rounded(8.0 * t.radius),
            Stroke::new(2.0_f32, t.accent_ring),
            StrokeKind::Outside,
        );
    }
    rt.record(&format!("button:{label}"), &response);
    response
}
pub fn icon_button(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    icon: &str,
    label: &str,
) -> Response {
    let id = ui.make_persistent_id(id);
    let (rect, response) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label));
    let t = rt.tokens.clone();
    let hover = rt.animate(
        (id, "hover"),
        if response.hovered() || response.has_focus() {
            1.0
        } else {
            0.0
        },
        0.14,
        Curve::Out,
    );
    ui.painter().rect_filled(
        rect,
        rounded(6.0 * t.radius),
        mix(Color32::TRANSPARENT, t.surface2, hover),
    );
    let p = rt.icon_progress((id, "icon"), response.hovered() || response.has_focus());
    icons::paint(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), vec2(18.0, 18.0)),
        mix(t.text2, t.text, hover),
        1.6,
        p,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            rounded(8.0 * t.radius),
            Stroke::new(2.0_f32, t.accent_ring),
            StrokeKind::Outside,
        );
    }
    rt.record(&format!("icon:{label}"), &response);
    response.on_hover_text(label)
}
pub fn segmented(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    labels: &[&str],
    selected: &mut usize,
) -> bool {
    if labels.is_empty() {
        return false;
    }
    *selected = (*selected).min(labels.len() - 1);
    let id = ui.make_persistent_id(id);
    let t = rt.tokens.clone();
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap((*label).into(), FontId::proportional(12.0), t.text)
                .size()
                .x
                + 28.0
        })
        .collect();
    let natural = widths.iter().sum::<f32>() + 4.0;
    let width = natural.min(ui.available_width());
    let (rect, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, rounded(8.0 * t.radius), t.surface2);
    let each = (width - 4.0) / labels.len() as f32;
    let mut changed = false;
    for (i, label) in labels.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(rect.left() + 2.0 + each * i as f32, rect.top() + 2.0),
            vec2(each, 32.0),
        );
        let response = ui.interact(r, id.with(i), Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::SelectableLabel,
                ui.is_enabled(),
                i == *selected,
                *label,
            )
        });
        let keyboard = response.has_focus()
            && ui.input(|input| {
                input.key_pressed(Key::ArrowLeft)
                    || input.key_pressed(Key::ArrowRight)
                    || input.key_pressed(Key::Home)
                    || input.key_pressed(Key::End)
            });
        if response.clicked() {
            changed |= *selected != i;
            *selected = i;
        }
        if keyboard {
            let next = ui.input(|input| {
                if input.key_pressed(Key::Home) {
                    0
                } else if input.key_pressed(Key::End) {
                    labels.len() - 1
                } else if input.key_pressed(Key::ArrowRight) {
                    (*selected + 1) % labels.len()
                } else {
                    (*selected + labels.len() - 1) % labels.len()
                }
            });
            changed |= *selected != next;
            *selected = next;
            ui.memory_mut(|m| m.request_focus(id.with(next)));
        }
        rt.record(&format!("segment:{label}"), &response);
    }
    let x = rt.animate(
        (id, "thumb-x"),
        each * *selected as f32,
        0.47,
        Curve::SpringSnappy,
    );
    let thumb = Rect::from_min_size(
        pos2(rect.left() + 2.0 + x, rect.top() + 2.0),
        vec2(each, 32.0),
    );
    ui.painter().rect(
        thumb,
        rounded(6.0 * t.radius),
        t.surface,
        Stroke::new(1.0_f32, t.border_soft),
        StrokeKind::Inside,
    );
    for (i, label) in labels.iter().enumerate() {
        ui.painter().text(
            pos2(rect.left() + 2.0 + each * (i as f32 + 0.5), rect.center().y),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(12.0),
            if i == *selected { t.text } else { t.text2 },
        );
    }
    changed
}
pub fn checkbox(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    label: &str,
    value: &mut bool,
) -> Response {
    let id = ui.make_persistent_id(id);
    let t = rt.tokens.clone();
    let width = ui
        .painter()
        .layout_no_wrap(label.into(), FontId::proportional(13.0), t.text)
        .size()
        .x
        + 34.0;
    let (rect, mut response) =
        ui.allocate_exact_size(vec2(width.min(ui.available_width()), 28.0), Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), *value, label));
    let amount = rt.animate(
        (id, "checked"),
        if *value { 1.0 } else { 0.0 },
        0.32,
        Curve::Out,
    );
    let box_rect =
        Rect::from_center_size(pos2(rect.left() + 9.0, rect.center().y), vec2(18.0, 18.0));
    ui.painter().rect(
        box_rect,
        rounded(4.0 * t.radius),
        mix(t.surface, t.accent, amount),
        Stroke::new(1.0_f32, mix(t.border_strong, t.accent, amount)),
        StrokeKind::Inside,
    );
    if amount > 0.0 {
        let points = [
            box_rect.min + vec2(4.0, 9.0),
            box_rect.min + vec2(7.5, 12.0),
            box_rect.min + vec2(14.0, 5.5),
        ];
        paint_trace(
            ui.painter(),
            &points,
            amount,
            Stroke::new(1.8_f32, t.accent_contrast),
        );
    }
    ui.painter().text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        t.text,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            box_rect.expand(3.0),
            rounded(5.0 * t.radius),
            Stroke::new(2.0_f32, t.accent_ring),
            StrokeKind::Outside,
        );
    }
    rt.record(&format!("checkbox:{label}"), &response);
    response
}
pub fn toggle(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    label: &str,
    value: &mut bool,
) -> Response {
    let id = ui.make_persistent_id(id);
    let t = rt.tokens.clone();
    let width = ui
        .painter()
        .layout_no_wrap(label.into(), FontId::proportional(13.0), t.text)
        .size()
        .x
        + 56.0;
    let (rect, mut response) =
        ui.allocate_exact_size(vec2(width.min(ui.available_width()), 30.0), Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), *value, label));
    let p = rt.animate(
        (id, "toggle"),
        if *value { 1.0 } else { 0.0 },
        0.32,
        Curve::SpringSnappy,
    );
    let track = Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(38.0, 22.0));
    ui.painter().rect(
        track,
        rounded(12.0),
        mix(t.surface3, t.accent, p),
        Stroke::new(1.0_f32, mix(t.border, t.accent, p)),
        StrokeKind::Inside,
    );
    ui.painter().circle_filled(
        pos2(track.left() + 11.0 + 16.0 * p, track.center().y),
        8.0,
        if rt.prefs.dark && p > 0.5 {
            t.accent_contrast
        } else {
            t.surface
        },
    );
    ui.painter().text(
        pos2(rect.left() + 50.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        t.text,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            track.expand(3.0),
            rounded(15.0),
            Stroke::new(2.0_f32, t.accent_ring),
            StrokeKind::Outside,
        );
    }
    rt.record(&format!("toggle:{label}"), &response);
    response
}
pub fn badge(ui: &mut Ui, rt: &Runtime, label: &str, tone: &str) -> Response {
    let t = &rt.tokens;
    let (bg, fg) = match tone {
        "accent" => (t.accent_soft, t.accent_text),
        "success" => (t.success_soft, t.success_text),
        "warning" => (t.warning_soft, t.warning_text),
        "danger" | "error" => (t.danger_soft, t.danger_text),
        "info" => (t.info_soft, t.info_text),
        _ => (t.surface2, t.text3),
    };
    Frame::new()
        .fill(bg)
        .corner_radius(rounded(5.0 * t.radius))
        .inner_margin(Margin::symmetric(7, 3))
        .show(ui, |ui| ui.label(RichText::new(label).size(11.0).color(fg)))
        .response
}
pub fn muted(ui: &mut Ui, rt: &Runtime, value: &str) -> Response {
    ui.label(RichText::new(value).size(12.0).color(rt.tokens.text3))
}
pub fn page_head(
    ui: &mut Ui,
    rt: &mut Runtime,
    icon: &str,
    eyebrow: &str,
    title: &str,
    lead: &str,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
        ui.painter().rect(
            rect,
            rounded(12.0 * rt.tokens.radius),
            rt.tokens.surface,
            Stroke::new(1.0_f32, rt.tokens.border),
            StrokeKind::Inside,
        );
        icons::paint(
            ui.painter(),
            icon,
            rect.shrink(12.0),
            rt.tokens.accent_text,
            1.6,
            0.0,
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new(eyebrow.to_uppercase())
                    .size(10.0)
                    .color(rt.tokens.text3),
            );
            ui.label(
                RichText::new(title)
                    .font(strong_font(
                        ui.ctx(),
                        if ui.ctx().content_rect().width() > 900.0 {
                            60.0
                        } else {
                            40.0
                        },
                    ))
                    .color(rt.tokens.text),
            );
        });
    });
    if !lead.is_empty() {
        ui.add_space(16.0);
        ui.label(RichText::new(lead).size(16.0).color(rt.tokens.text2));
    }
    ui.add_space(40.0);
}
pub fn section(ui: &mut Ui, rt: &mut Runtime, id: &str, num: &str, title: &str, desc: &str) {
    ui.add_space(24.0);
    let response = ui
        .horizontal(|ui| {
            if !num.is_empty() {
                ui.label(
                    RichText::new(num)
                        .monospace()
                        .size(11.0)
                        .color(rt.tokens.text4),
                );
                ui.add_space(8.0);
            }
            ui.label(
                RichText::new(title)
                    .size(22.0)
                    .strong()
                    .color(rt.tokens.text),
            );
        })
        .response;
    rt.anchor(id, title, response.rect);
    if !desc.is_empty() {
        ui.add_space(6.0);
        muted(ui, rt, desc);
    }
    ui.add_space(16.0);
}
pub fn card<R>(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    body: impl FnOnce(&mut Ui, &mut Runtime) -> R,
) -> InnerResponse<R> {
    let t = rt.tokens.clone();
    ui.push_id(id, |ui| {
        Frame::new()
            .fill(t.surface)
            .stroke(Stroke::new(1.0_f32, t.border))
            .corner_radius(rounded(12.0 * t.radius))
            .inner_margin(20)
            .show(ui, |ui| body(ui, rt))
    })
    .inner
}
pub fn demo<R>(
    ui: &mut Ui,
    rt: &mut Runtime,
    id: impl Hash,
    title: &str,
    desc: &str,
    height: f32,
    body: impl FnOnce(&mut Ui, &mut Runtime) -> R,
) -> InnerResponse<R> {
    let t = rt.tokens.clone();
    let result = ui
        .push_id(id, |ui| {
            Frame::new()
                .fill(t.surface)
                .stroke(Stroke::new(1.0_f32, t.border))
                .corner_radius(rounded(16.0 * t.radius))
                .show(ui, |ui| {
                    let value = Frame::new()
                        .inner_margin(20)
                        .show(ui, |ui| {
                            ui.set_min_height((height - 40.0).max(0.0));
                            body(ui, rt)
                        })
                        .inner;
                    ui.add(Separator::default().spacing(0.0));
                    Frame::new().inner_margin(16).show(ui, |ui| {
                        ui.label(
                            RichText::new(title)
                                .font(strong_font(ui.ctx(), 13.0))
                                .color(t.text),
                        );
                        if !desc.is_empty() {
                            ui.add_space(5.0);
                            ui.label(RichText::new(desc).size(12.0).color(t.text3));
                        }
                    });
                    value
                })
        })
        .inner;
    if let Some(entry) = crate::catalog::ENTRIES
        .iter()
        .find(|entry| entry.page == rt.route.id() && entry.title == title)
    {
        rt.anchor(entry.id, title, result.response.rect);
    }
    result
}
pub fn progress(ui: &mut Ui, rt: &Runtime, fraction: f32, height: f32) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter()
        .rect_filled(rect, rounded(height / 2.0), rt.tokens.surface3);
    if fraction > 0.0 {
        ui.painter().rect_filled(
            Rect::from_min_size(
                rect.min,
                vec2(rect.width() * fraction.clamp(0.0, 1.0), height),
            ),
            rounded(height / 2.0),
            rt.tokens.accent,
        );
    }
    response
}
pub fn field(
    ui: &mut Ui,
    rt: &Runtime,
    id: impl Hash,
    label: &str,
    value: &mut String,
    hint: &str,
) -> Response {
    if !label.is_empty() {
        ui.label(RichText::new(label).size(12.0).color(rt.tokens.text2));
    }
    ui.add(
        TextEdit::singleline(value)
            .id_source(id)
            .hint_text(hint)
            .desired_width(f32::INFINITY)
            .margin(vec2(12.0, 9.0)),
    )
}
pub fn paint_trace(painter: &Painter, points: &[Pos2], fraction: f32, stroke: Stroke) {
    if points.len() < 2 || fraction <= 0.0 {
        return;
    }
    let total = points.windows(2).map(|p| p[0].distance(p[1])).sum::<f32>();
    let mut remain = total * fraction.clamp(0.0, 1.0);
    let mut traced = vec![points[0]];
    for pair in points.windows(2) {
        let length = pair[0].distance(pair[1]);
        if remain >= length {
            traced.push(pair[1]);
            remain -= length;
        } else {
            if length > 0.0 {
                traced.push(pair[0].lerp(pair[1], remain / length));
            }
            break;
        }
    }
    painter.add(Shape::line(traced, stroke));
}
