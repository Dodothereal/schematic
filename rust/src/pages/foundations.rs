//! Native semantic-token inspector and resolved exports.
use crate::{
    icons,
    motion::Curve,
    runtime::{Page, Runtime},
    ui::{self, ButtonKind},
};
use eframe::egui::{self, Align2, Color32, FontId, RichText, Sense, Stroke, StrokeKind, Vec2};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct State {
    fields: [String; 4],
    status: String,
}

fn color_string(color: Color32) -> String {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a == 255 {
        format!("rgb({r} {g} {b})")
    } else {
        format!("rgb({r} {g} {b} / {:.4})", a as f32 / 255.0)
    }
}

fn control_button(ui: &mut egui::Ui, rt: &mut Runtime, size: &str, height: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(100.0, height), Sense::click());
    rt.record(&format!("fd-create-{size}"), &response);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            format!("Create, size {size}"),
        )
    });
    let engaged = response.hovered() || response.has_focus();
    let amount = rt.animate(
        ("fd-control", size),
        if engaged { 1.0 } else { 0.0 },
        0.14,
        Curve::Out,
    );
    ui.painter().rect(
        rect,
        8.0 * rt.prefs.radius,
        crate::ui::mix(rt.tokens.surface, rt.tokens.surface2, amount),
        Stroke::new(
            1.0_f32,
            if engaged {
                rt.tokens.border_strong
            } else {
                rt.tokens.border
            },
        ),
        StrokeKind::Inside,
    );
    let icon_rect =
        egui::Rect::from_center_size(rect.left_center() + Vec2::new(20.0, 0.0), Vec2::splat(16.0));
    let progress = rt.icon_progress(("fd-create-icon", size), engaged);
    icons::paint(
        ui.painter(),
        "plus",
        icon_rect,
        rt.tokens.text,
        1.6,
        progress,
    );
    ui.painter().text(
        rect.left_center() + Vec2::new(36.0, 0.0),
        Align2::LEFT_CENTER,
        "Create",
        FontId::proportional(12.0),
        rt.tokens.text,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            8.0 * rt.prefs.radius,
            Stroke::new(2.0_f32, rt.tokens.accent_ring),
            StrokeKind::Outside,
        );
    }
    response
}

/// Values come from the native appearance, never from parsed CSS or a browser.
pub fn resolved_tokens(rt: &Runtime) -> BTreeMap<String, String> {
    let t = &rt.tokens;
    let mut values = BTreeMap::new();
    for (name, value) in [
        ("bg", t.bg),
        ("bg-sunken", t.bg_sunken),
        ("surface", t.surface),
        ("surface-2", t.surface2),
        ("surface-3", t.surface3),
        ("surface-raised", t.surface_raised),
        ("border", t.border),
        ("border-soft", t.border_soft),
        ("border-strong", t.border_strong),
        ("text", t.text),
        ("text-2", t.text2),
        ("text-3", t.text3),
        ("text-4", t.text4),
        (
            "text-inverse",
            if rt.prefs.dark { t.bg } else { Color32::WHITE },
        ),
        ("overlay", t.overlay),
        ("accent", t.accent),
        ("accent-hover", t.accent_hover),
        ("accent-press", t.accent_press),
        ("accent-text", t.accent_text),
        ("accent-contrast", t.accent_contrast),
        ("accent-soft", t.accent_soft),
        ("accent-soft-2", t.accent_soft2),
        ("accent-ring", t.accent_ring),
        ("success", t.success),
        ("success-soft", t.success_soft),
        ("success-text", t.success_text),
        ("warning", t.warning),
        ("warning-soft", t.warning_soft),
        ("warning-text", t.warning_text),
        ("danger", t.danger),
        ("danger-soft", t.danger_soft),
        ("danger-text", t.danger_text),
        ("danger-hover", t.danger),
        ("danger-contrast", t.accent_contrast),
        ("info", t.info),
        ("info-soft", t.info_soft),
        ("info-text", t.info_text),
        ("chart-grid", t.border_soft),
    ] {
        values.insert(name.into(), color_string(value));
    }
    for (i, color) in t.charts.iter().enumerate() {
        values.insert(format!("chart-{}", i + 1), color_string(*color));
    }
    for (name, n) in [
        ("0", 0.0),
        ("px", 1.0),
        ("0_5", 2.0),
        ("1", 4.0),
        ("1_5", 6.0),
        ("2", 8.0),
        ("2_5", 10.0),
        ("3", 12.0),
        ("4", 16.0),
        ("5", 20.0),
        ("6", 24.0),
        ("8", 32.0),
        ("10", 40.0),
        ("12", 48.0),
        ("16", 64.0),
        ("20", 80.0),
        ("24", 96.0),
    ] {
        values.insert(format!("s-{name}"), format!("{n}px"));
    }
    for (name, n) in [
        ("xs", 4.0),
        ("sm", 6.0),
        ("md", 8.0),
        ("lg", 12.0),
        ("xl", 16.0),
        ("2xl", 22.0),
    ] {
        values.insert(format!("r-{name}"), format!("{}px", n * rt.prefs.radius));
    }
    values.insert("r-full".into(), "999px".into());
    for (name, size, height) in [
        ("2xs", 11, 16),
        ("xs", 12, 16),
        ("sm", 13, 20),
        ("md", 14, 22),
        ("lg", 16, 24),
        ("xl", 18, 26),
        ("2xl", 22, 30),
        ("3xl", 28, 34),
        ("4xl", 36, 42),
        ("5xl", 48, 52),
        ("6xl", 64, 66),
    ] {
        values.insert(format!("t-{name}"), format!("{size}px"));
        values.insert(format!("lh-{name}"), format!("{height}px"));
    }
    for (name, value) in [
        ("font-sans", "system-ui, sans-serif"),
        ("font-display", "system-ui, sans-serif"),
        ("font-mono", "ui-monospace, monospace"),
        ("tracking-tight", "-0.022em"),
        ("tracking-display", "-0.035em"),
        ("tracking-wide", "0.06em"),
        ("w-regular", "420"),
        ("w-medium", "520"),
        ("w-semibold", "600"),
        ("w-bold", "680"),
        ("ic-stroke", "1.6"),
        ("neutral-h", "80"),
    ] {
        values.insert(name.into(), value.into());
    }
    for (name, n) in [
        ("control-xs", 24),
        ("control-sm", 28),
        ("control-md", 34),
        ("control-lg", 40),
        ("control-xl", 48),
        ("icon-sm", 16),
        ("icon-md", 18),
        ("icon-lg", 20),
        ("icon-xl", 24),
        ("sidebar-w", 248),
        ("topbar-h", 56),
        ("content-max", 1080),
        ("rail-w", 196),
    ] {
        values.insert(name.into(), format!("{n}px"));
    }
    for (name, n) in [
        ("instant", 80),
        ("fast", 140),
        ("base", 220),
        ("moderate", 320),
        ("slow", 460),
        ("slower", 650),
        ("glacial", 1000),
        ("spring-smooth", 600),
        ("spring-snappy", 470),
        ("spring-bouncy", 930),
    ] {
        values.insert(
            format!("d-{name}"),
            format!("{}ms", n as f32 * rt.motion_scale()),
        );
    }
    values.insert("stagger".into(), format!("{}ms", 32.0 * rt.motion_scale()));
    for (name, value) in [
        ("out", "cubic-bezier(0.22, 1, 0.36, 1)"),
        ("out-expo", "cubic-bezier(0.16, 1, 0.3, 1)"),
        ("out-soft", "cubic-bezier(0.25, 0.8, 0.25, 1)"),
        ("in", "cubic-bezier(0.64, 0, 0.78, 0)"),
        ("in-out", "cubic-bezier(0.65, 0, 0.35, 1)"),
        ("overshoot", "cubic-bezier(0.34, 1.56, 0.64, 1)"),
        ("anticipate", "cubic-bezier(0.68, -0.4, 0.32, 1.4)"),
    ] {
        values.insert(format!("ease-{name}"), value.into());
    }
    for (name, curve) in [
        ("smooth", Curve::SpringSmooth),
        ("snappy", Curve::SpringSnappy),
        ("bouncy", Curve::SpringBouncy),
    ] {
        values.insert(
            format!("spring-{name}"),
            format!(
                "linear({})",
                (0..=48)
                    .map(|i| format!("{:.4}", crate::motion::ease(curve, i as f32 / 48.0)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    for (name, n) in [
        ("sticky", 10),
        ("sidebar", 20),
        ("popover", 50),
        ("toast", 70),
        ("dialog", 80),
        ("tooltip", 90),
    ] {
        values.insert(format!("z-{name}"), n.to_string());
    }
    values.insert("accent-h".into(), rt.prefs.hue.to_string());
    values.insert(
        "accent-c".into(),
        if rt.prefs.mono { "0" } else { "0.115" }.into(),
    );
    values.insert(
        "neutral-c".into(),
        if rt.prefs.mono { "0" } else { "0.008" }.into(),
    );
    values.insert("radius-scale".into(), rt.prefs.radius.to_string());
    values.insert("motion-scale".into(), rt.motion_scale().to_string());
    for (name, offset, blur, alpha) in [
        ("xs", 1, 2, 0.06),
        ("sm", 2, 6, 0.06),
        ("md", 6, 16, 0.1),
        ("lg", 16, 40, 0.16),
        ("xl", 32, 72, 0.24),
    ] {
        values.insert(
            format!("shadow-{name}"),
            format!(
                "0 {offset}px {blur}px rgb(0 0 0 / {})",
                if rt.prefs.dark { alpha + 0.3 } else { alpha }
            ),
        );
    }
    values.insert(
        "inset-highlight".into(),
        format!(
            "inset 0 1px 0 rgb(255 255 255 / {})",
            if rt.prefs.dark { 0.05 } else { 0.7 }
        ),
    );
    values.insert(
        "btn-primary-highlight".into(),
        "inset 0 1px 0 rgb(255 255 255 / 0.22), inset 0 -1px 0 rgb(0 0 0 / 0.12)".into(),
    );
    values
}

pub fn css_snapshot(values: &BTreeMap<String, String>) -> String {
    format!(
        "/* Schematic: resolved snapshot of the current native theme. */\n:root {{\n{}\n}}\n",
        values
            .iter()
            .map(|(name, value)| format!("  --{name}: {value};"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

impl Page for State {
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let viewport_width = (ui.clip_rect().right() - ui.cursor().min.x - 16.0).max(100.0);
        ui.set_max_width(ui.available_width().min(viewport_width));
        ui::page_head(
            ui,
            rt,
            "droplet",
            "Foundations / 01",
            "Foundations",
            "Colors, typography, spacing, borders and shadows.",
        );
        ui::section(ui, rt, "foundations-color", "01", "Colour has a job", "");
        let colors = [
            ("bg", rt.tokens.bg),
            ("bg-sunken", rt.tokens.bg_sunken),
            ("surface", rt.tokens.surface),
            ("surface-2", rt.tokens.surface2),
            ("surface-3", rt.tokens.surface3),
            ("border", rt.tokens.border),
            ("border-strong", rt.tokens.border_strong),
            ("text", rt.tokens.text),
            ("text-2", rt.tokens.text2),
            ("accent", rt.tokens.accent),
            ("accent-soft", rt.tokens.accent_soft),
            ("accent-ring", rt.tokens.accent_ring),
            ("success", rt.tokens.success),
            ("warning", rt.tokens.warning),
            ("danger", rt.tokens.danger),
            ("info", rt.tokens.info),
            ("chart-1", rt.tokens.charts[0]),
            ("chart-2", rt.tokens.charts[1]),
            ("chart-3", rt.tokens.charts[2]),
            ("chart-4", rt.tokens.charts[3]),
        ];
        let columns = (ui.available_width() / 160.0).floor().clamp(2.0, 5.0) as usize;
        let swatch_width = (ui.available_width() - (columns - 1) as f32 * 12.0) / columns as f32;
        egui::Grid::new("fd-palette")
            .num_columns(columns)
            .spacing(Vec2::splat(12.0))
            .show(ui, |ui| {
                for (i, (name, color)) in colors.iter().enumerate() {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(swatch_width, 100.0), Sense::click());
                    rt.record(&format!("fd-token-{name}"), &response);
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            format!("Copy {name} token"),
                        )
                    });
                    let engaged = response.hovered() || response.has_focus();
                    let p = rt.animate(
                        ("fd-swatch", name),
                        if engaged { 1.0 } else { 0.0 },
                        0.14,
                        Curve::Out,
                    );
                    let painter = ui.painter();
                    painter.rect(
                        rect,
                        8.0 * rt.prefs.radius,
                        rt.tokens.surface,
                        Stroke::new(
                            1.0_f32,
                            if engaged {
                                rt.tokens.accent
                            } else {
                                rt.tokens.border_soft
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    painter.rect(
                        rect.shrink2(Vec2::new(8.0, 8.0))
                            .with_max_y(rect.top() + 72.0),
                        6.0 * rt.prefs.radius,
                        *color,
                        Stroke::new(1.0_f32, rt.tokens.border_soft),
                        StrokeKind::Inside,
                    );
                    painter.text(
                        rect.left_bottom() + Vec2::new(8.0, -12.0),
                        Align2::LEFT_CENTER,
                        format!("--{name}"),
                        FontId::monospace(11.0),
                        rt.tokens.text2,
                    );
                    if p > 0.0 {
                        let copy_rect = egui::Rect::from_min_size(
                            rect.right_top() + Vec2::new(-30.0, 12.0),
                            Vec2::splat(18.0),
                        );
                        painter.rect_filled(
                            copy_rect.expand(4.0),
                            4.0,
                            rt.tokens.surface.gamma_multiply(p),
                        );
                        icons::paint(
                            painter,
                            "copy",
                            copy_rect,
                            rt.tokens.text.gamma_multiply(p),
                            1.5,
                            p,
                        );
                    }
                    if response.clicked() {
                        let ok = rt.copy(ui.ctx(), format!("var(--{name})"));
                        rt.toast(
                            if ok {
                                "Token copied"
                            } else {
                                "Clipboard unavailable"
                            },
                            &format!("var(--{name})"),
                            if ok { "success" } else { "warning" },
                        );
                    }
                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ui::button(
                ui,
                rt,
                "fd-light",
                "Light",
                Some("sun"),
                ButtonKind::Secondary,
            )
            .clicked()
            {
                rt.set_theme(false)
            }
            if ui::button(
                ui,
                rt,
                "fd-dark",
                "Dark",
                Some("moon"),
                ButtonKind::Secondary,
            )
            .clicked()
            {
                rt.set_theme(true)
            }
            ui::muted(ui, rt, "Both themes consume the same semantic names.");
        });
        ui::section(
            ui,
            rt,
            "foundations-type",
            "02",
            "A voice at every size",
            "",
        );
        ui::card(ui, rt, "fd-type", |ui, rt| {
            for (i, (token, size, title, label)) in [
                ("6xl", 64.0, "One language.", "Display"),
                ("4xl", 36.0, "Designed to belong.", "Page title"),
                ("2xl", 22.0, "A clear hierarchy", "Section title"),
                ("lg", 16.0, "Motion follows intention.", "Lead / heading"),
                (
                    "md",
                    14.0,
                    "A shared rhythm for every screen, every state and every platform.",
                    "Body",
                ),
                (
                    "sm",
                    13.0,
                    "The details should feel inevitable.",
                    "Supporting",
                ),
                ("xs", 12.0, "Schematic / reference edition", "Caption"),
            ]
            .iter()
            .enumerate()
            {
                if i > 0 {
                    ui.separator();
                }
                ui.add_space(16.0);
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(105.0);
                        ui.monospace(format!("--t-{token}"));
                        ui::muted(ui, rt, label);
                    });
                    ui.add(
                        egui::Label::new(
                            RichText::new(*title)
                                .size(if ui.available_width() < 400.0 {
                                    f32::min(*size, 40.0)
                                } else {
                                    *size
                                })
                                .color(rt.tokens.text),
                        )
                        .wrap(),
                    );
                });
                ui.add_space(16.0);
            }
        });
        ui::section(
            ui,
            rt,
            "foundations-space",
            "03",
            "Space is the structure",
            "",
        );
        ui::demo(
            ui,
            rt,
            "fd-spaces",
            "Spacing scale",
            "The same scale governs every page.",
            0.0,
            |ui, rt| {
                for n in [1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24] {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [80.0, 24.0],
                            egui::Label::new(RichText::new(format!("--s-{n}")).monospace()),
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new((n * 4) as f32, 12.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, rt.tokens.accent);
                        ui::muted(ui, rt, &format!("{}px", n * 4));
                    });
                }
            },
        );
        ui::demo(
            ui,
            rt,
            "fd-controls",
            "Control rhythm",
            "A shared height aligns buttons, inputs and selectors.",
            0.0,
            |ui, rt| {
                for (i, (size, height)) in [("sm", 28.0), ("md", 34.0), ("lg", 40.0), ("xl", 48.0)]
                    .iter()
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        ui.monospace(*size);
                        let width = (ui.available_width() - 110.0).max(80.0);
                        let response = ui.add_sized(
                            [width, *height],
                            egui::TextEdit::singleline(&mut self.fields[i])
                                .hint_text("Project name"),
                        );
                        rt.record(&format!("fd-field-{size}"), &response);
                        ui.scope(|ui| {
                            if control_button(ui, rt, size, *height).clicked() {
                                rt.toast(
                                    "Control selected",
                                    &format!(
                                        "Size {} shares its height with the adjacent field.",
                                        size.to_uppercase()
                                    ),
                                    "info",
                                );
                            }
                        });
                    });
                }
            },
        );
        ui::section(
            ui,
            rt,
            "foundations-surfaces",
            "04",
            "Depth, with restraint",
            "",
        );
        ui.horizontal_wrapped(|ui| {
            for (i, name) in ["SM", "MD", "LG"].iter().enumerate() {
                egui::Frame::new()
                    .fill(rt.tokens.surface)
                    .stroke(Stroke::new(1.0_f32, rt.tokens.border))
                    .corner_radius(((6 + i * 3) as f32 * rt.prefs.radius) as u8)
                    .inner_margin(24.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, (i * 4 + 2) as i8],
                        blur: (i * 6 + 4) as u8,
                        spread: 0,
                        color: Color32::BLACK.gamma_multiply(if rt.prefs.dark {
                            0.35
                        } else {
                            0.09
                        }),
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(145.0);
                        ui.vertical_centered(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                            icons::paint(
                                ui.painter(),
                                "layers",
                                rect,
                                rt.tokens.accent_text,
                                1.6,
                                0.0,
                            );
                            ui.strong(format!("Elevation {name}"));
                            ui.monospace(format!("--shadow-{}", name.to_lowercase()));
                            ui::muted(ui, rt, &format!("Radius: --r-{}", name.to_lowercase()));
                        });
                    });
            }
        });
        ui.add_space(16.0);
        ui.horizontal_wrapped(|ui| {
            ui::muted(ui, rt, "Try a different character");
            let mut selected = if rt.prefs.radius < 0.5 {
                0
            } else if rt.prefs.radius > 1.2 {
                2
            } else {
                1
            };
            if ui::segmented(
                ui,
                rt,
                "fd-radius",
                &["Sharp", "Default", "Round"],
                &mut selected,
            ) {
                rt.prefs.radius = [0.0, 1.0, 1.6][selected];
            }
        });
        ui::section(
            ui,
            rt,
            "foundations-export",
            "05",
            "Take the system with you",
            "",
        );
        ui::card(ui, rt, "fd-export", |ui, rt| {
            ui::badge(ui, rt, "Native Rust", "success");
            ui.strong("Resolved token snapshot");
            ui.monospace("color: var(--text);\nbackground: var(--surface);\npadding: var(--s-3) var(--s-4);\nborder-radius: var(--r-md);");
            ui.horizontal_wrapped(|ui| {
                for (kind,label,icon,style) in [("css","Download CSS snapshot","download",ButtonKind::Primary),("json","Export JSON","code",ButtonKind::Secondary),("copy","Copy CSS","copy",ButtonKind::Ghost)] {
                    if ui::button(ui,rt,("fd-export",kind),label,Some(icon),style).clicked(){let tokens=resolved_tokens(rt);if kind=="copy"{let ok=rt.copy(ui.ctx(),css_snapshot(&tokens));self.status=if ok{"CSS snapshot copied."}else{"Clipboard unavailable. Use Download CSS instead."}.into();}else{let (name,contents,mime)=if kind=="css"{("schematic-theme.css",css_snapshot(&tokens),"text/css")}else{("schematic-tokens.json",serde_json::to_string_pretty(&serde_json::json!({"theme":if rt.prefs.dark{"dark"}else{"light"},"tokens":tokens})).unwrap_or_default(),"application/json")};self.status=match rt.download(name,&contents,mime){Ok(Some(path))=>format!("Snapshot saved to {}.",path.display()),Ok(None)=>"Save cancelled.".into(),Err(error)=>format!("Export failed: {error}")};}}
                }
            });
            ui::muted(ui, rt, &self.status);
        });
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_keeps_names_and_resolved_values() {
        let values = BTreeMap::from([
            ("text".into(), "rgb(20 20 20)".into()),
            ("s-4".into(), "16px".into()),
        ]);
        let css = css_snapshot(&values);
        assert!(css.contains("--s-4: 16px;"));
        assert!(css.contains("--text: rgb(20 20 20);"));
        assert!(css.ends_with("}\n"));
    }
    #[test]
    fn native_foundations_render_and_export_every_semantic_group() {
        use crate::runtime::Preferences;
        for width in [390.0, 1200.0] {
            for dark in [false, true] {
                for mono in [false, true] {
                    let ctx = egui::Context::default();
                    let mut rt = Runtime::new(
                        ctx.clone(),
                        Preferences {
                            dark,
                            mono,
                            ..Default::default()
                        },
                    );
                    let mut state = State::default();
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                Vec2::new(width, 950.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| state.show(ui, &mut rt));
                            });
                        },
                    );
                    assert!(
                        !ctx.tessellate(output.shapes, output.pixels_per_point)
                            .is_empty()
                    );
                    let values = resolved_tokens(&rt);
                    for name in [
                        "bg",
                        "accent",
                        "chart-5",
                        "s-24",
                        "r-2xl",
                        "t-6xl",
                        "control-xl",
                        "spring-snappy",
                        "ease-overshoot",
                        "d-glacial",
                        "shadow-lg",
                    ] {
                        assert!(values.contains_key(name), "{name}");
                    }
                    assert!(values.len() > 140);
                    assert!(serde_json::to_string(&values).is_ok());
                    for (size, height) in [("sm", 28.0), ("md", 34.0), ("lg", 40.0), ("xl", 48.0)] {
                        let rect = rt
                            .records
                            .iter()
                            .find(|r| r.id == format!("fd-create-{size}"))
                            .unwrap()
                            .rect;
                        assert_eq!(rect[3] - rect[1], height);
                    }
                }
            }
        }
    }
}
