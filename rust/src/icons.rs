use crate::{
    icon_data::{Geometry, ICONS},
    motion::{self, Curve},
};
use eframe::egui::*;
use std::{collections::HashMap, sync::LazyLock};
use svgtypes::{PathParser, PathSegment};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    pub rotation: f32,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Key {
    pub t: f32,
    pub transform: Option<Transform>,
    pub opacity: Option<f32>,
    pub trace: Option<f32>,
    pub curve: Option<Curve>,
    pub translate: Option<[f32; 2]>,
}
pub struct Rule {
    pub name: &'static str,
    pub group: &'static str,
    pub keyframes: &'static str,
    pub duration: f32,
    pub delay: f32,
    pub curve: Curve,
    pub origin: Option<[f32; 2]>,
    pub fill: bool,
}
#[path = "icon_motion_data.rs"]
mod data;
#[derive(Clone)]
struct Part {
    group: &'static str,
    paths: Vec<(Vec<Pos2>, bool)>,
    fill: bool,
    stroke: bool,
    width: Option<f32>,
}
static NAMES: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| ICONS.iter().map(|i| i.name).collect());
static CACHE: LazyLock<HashMap<&'static str, Vec<Part>>> = LazyLock::new(|| {
    ICONS
        .iter()
        .map(|icon| {
            (
                icon.name,
                icon.parts
                    .iter()
                    .map(|part| Part {
                        group: part.group,
                        paths: flatten(part.geometry),
                        fill: part.fill,
                        stroke: part.stroke,
                        width: part.stroke_width,
                    })
                    .collect(),
            )
        })
        .collect()
});
pub fn names() -> &'static [&'static str] {
    &NAMES
}
fn cubic(a: Pos2, b: Pos2, c: Pos2, d: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    pos2(
        u * u * u * a.x + 3.0 * u * u * t * b.x + 3.0 * u * t * t * c.x + t * t * t * d.x,
        u * u * u * a.y + 3.0 * u * u * t * b.y + 3.0 * u * t * t * c.y + t * t * t * d.y,
    )
}
fn arc(
    from: Pos2,
    to: Pos2,
    rx: f32,
    ry: f32,
    rotation: f32,
    large: bool,
    sweep: bool,
) -> Vec<Pos2> {
    let mut rx = rx.abs();
    let mut ry = ry.abs();
    if rx <= 0.0 || ry <= 0.0 || from == to {
        return vec![to];
    }
    let phi = rotation.to_radians();
    let (cos, sin) = (phi.cos(), phi.sin());
    let dx = (from.x - to.x) / 2.0;
    let dy = (from.y - to.y) / 2.0;
    let xp = cos * dx + sin * dy;
    let yp = -sin * dx + cos * dy;
    let lambda = xp * xp / (rx * rx) + yp * yp / (ry * ry);
    if lambda > 1.0 {
        let v = lambda.sqrt();
        rx *= v;
        ry *= v;
    }
    let denom = rx * rx * yp * yp + ry * ry * xp * xp;
    let numerator = (rx * rx * ry * ry - denom).max(0.0);
    let factor = if denom > 0.0 {
        (numerator / denom).sqrt() * if large == sweep { -1.0 } else { 1.0 }
    } else {
        0.0
    };
    let cxp = factor * rx * yp / ry;
    let cyp = -factor * ry * xp / rx;
    let cx = cos * cxp - sin * cyp + (from.x + to.x) / 2.0;
    let cy = sin * cxp + cos * cyp + (from.y + to.y) / 2.0;
    let start = ((yp - cyp) / ry).atan2((xp - cxp) / rx);
    let end = ((-yp - cyp) / ry).atan2((-xp - cxp) / rx);
    let mut delta = end - start;
    if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    }
    let steps = (delta.abs() * rx.max(ry) * 1.6).ceil().max(4.0) as usize;
    (1..=steps)
        .map(|i| {
            let a = start + delta * i as f32 / steps as f32;
            pos2(
                cx + cos * rx * a.cos() - sin * ry * a.sin(),
                cy + sin * rx * a.cos() + cos * ry * a.sin(),
            )
        })
        .collect()
}
fn flatten(geometry: Geometry) -> Vec<(Vec<Pos2>, bool)> {
    match geometry {
        Geometry::Circle { cx, cy, r } => vec![(
            (0..=48)
                .map(|i| {
                    let a = i as f32 / 48.0 * std::f32::consts::TAU;
                    pos2(cx + a.cos() * r, cy + a.sin() * r)
                })
                .collect(),
            true,
        )],
        Geometry::Ellipse { cx, cy, rx, ry } => vec![(
            (0..=48)
                .map(|i| {
                    let a = i as f32 / 48.0 * std::f32::consts::TAU;
                    pos2(cx + a.cos() * rx, cy + a.sin() * ry)
                })
                .collect(),
            true,
        )],
        Geometry::Line { x1, y1, x2, y2 } => vec![(vec![pos2(x1, y1), pos2(x2, y2)], false)],
        Geometry::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => {
            let r = rx.min(width / 2.0).min(height / 2.0).max(0.0);
            if r == 0.0 {
                return vec![(
                    vec![
                        pos2(x, y),
                        pos2(x + width, y),
                        pos2(x + width, y + height),
                        pos2(x, y + height),
                        pos2(x, y),
                    ],
                    true,
                )];
            }
            let mut points = vec![];
            for (cx, cy, start) in [
                (x + width - r, y + r, -90.0_f32),
                (x + width - r, y + height - r, 0.0),
                (x + r, y + height - r, 90.0),
                (x + r, y + r, 180.0),
            ] {
                for i in 0..=8 {
                    let a = (start + i as f32 / 8.0 * 90.0).to_radians();
                    points.push(pos2(cx + a.cos() * r, cy + a.sin() * r));
                }
            }
            points.push(points[0]);
            vec![(points, true)]
        }
        Geometry::Path(source) => {
            let (mut paths, mut points) = (Vec::new(), Vec::new());
            let (mut current, mut first, mut control) = (Pos2::ZERO, Pos2::ZERO, Pos2::ZERO);
            let mut prev_cubic = false;
            let mut prev_quad = false;
            for segment in PathParser::from(source).flatten() {
                let relative = |abs: bool, x: f64, y: f64, origin: Pos2| {
                    if abs {
                        pos2(x as f32, y as f32)
                    } else {
                        origin + vec2(x as f32, y as f32)
                    }
                };
                match segment {
                    PathSegment::MoveTo { abs, x, y } => {
                        if !points.is_empty() {
                            paths.push((std::mem::take(&mut points), false));
                        }
                        current = relative(abs, x, y, current);
                        first = current;
                        points.push(current);
                        prev_cubic = false;
                        prev_quad = false;
                    }
                    PathSegment::LineTo { abs, x, y } => {
                        current = relative(abs, x, y, current);
                        points.push(current);
                        prev_cubic = false;
                        prev_quad = false;
                    }
                    PathSegment::HorizontalLineTo { abs, x } => {
                        current.x = if abs { x as f32 } else { current.x + x as f32 };
                        points.push(current);
                        prev_cubic = false;
                        prev_quad = false;
                    }
                    PathSegment::VerticalLineTo { abs, y } => {
                        current.y = if abs { y as f32 } else { current.y + y as f32 };
                        points.push(current);
                        prev_cubic = false;
                        prev_quad = false;
                    }
                    PathSegment::CurveTo {
                        abs,
                        x1,
                        y1,
                        x2,
                        y2,
                        x,
                        y,
                    } => {
                        let b = relative(abs, x1, y1, current);
                        let c = relative(abs, x2, y2, current);
                        let end = relative(abs, x, y, current);
                        for i in 1..=24 {
                            points.push(cubic(current, b, c, end, i as f32 / 24.0));
                        }
                        current = end;
                        control = c;
                        prev_cubic = true;
                        prev_quad = false;
                    }
                    PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                        let b = if prev_cubic {
                            current + (current - control)
                        } else {
                            current
                        };
                        let c = relative(abs, x2, y2, current);
                        let end = relative(abs, x, y, current);
                        for i in 1..=24 {
                            points.push(cubic(current, b, c, end, i as f32 / 24.0));
                        }
                        current = end;
                        control = c;
                        prev_cubic = true;
                        prev_quad = false;
                    }
                    PathSegment::Quadratic { abs, x1, y1, x, y } => {
                        let c = relative(abs, x1, y1, current);
                        let end = relative(abs, x, y, current);
                        for i in 1..=20 {
                            let t = i as f32 / 20.0;
                            let u = 1.0 - t;
                            points.push(pos2(
                                u * u * current.x + 2.0 * u * t * c.x + t * t * end.x,
                                u * u * current.y + 2.0 * u * t * c.y + t * t * end.y,
                            ));
                        }
                        current = end;
                        control = c;
                        prev_quad = true;
                        prev_cubic = false;
                    }
                    PathSegment::SmoothQuadratic { abs, x, y } => {
                        let c = if prev_quad {
                            current + (current - control)
                        } else {
                            current
                        };
                        let end = relative(abs, x, y, current);
                        for i in 1..=20 {
                            let t = i as f32 / 20.0;
                            let u = 1.0 - t;
                            points.push(pos2(
                                u * u * current.x + 2.0 * u * t * c.x + t * t * end.x,
                                u * u * current.y + 2.0 * u * t * c.y + t * t * end.y,
                            ));
                        }
                        current = end;
                        control = c;
                        prev_quad = true;
                        prev_cubic = false;
                    }
                    PathSegment::EllipticalArc {
                        abs,
                        rx,
                        ry,
                        x_axis_rotation,
                        large_arc,
                        sweep,
                        x,
                        y,
                    } => {
                        let end = relative(abs, x, y, current);
                        points.extend(arc(
                            current,
                            end,
                            rx as f32,
                            ry as f32,
                            x_axis_rotation as f32,
                            large_arc,
                            sweep,
                        ));
                        current = end;
                        prev_cubic = false;
                        prev_quad = false;
                    }
                    PathSegment::ClosePath { .. } => {
                        points.push(first);
                        paths.push((std::mem::take(&mut points), true));
                        current = first;
                        prev_cubic = false;
                        prev_quad = false;
                    }
                }
            }
            if !points.is_empty() {
                paths.push((points, false));
            }
            paths
        }
    }
}
fn has_group(value: &str, group: &str) -> bool {
    group.is_empty() || value.split_whitespace().any(|v| v == group)
}
fn sample(
    keys: &[Key],
    t: f32,
    default: f32,
    curve: Curve,
    property: impl Fn(&Key) -> Option<f32>,
) -> f32 {
    let mut previous = (0.0, default, curve);
    let mut next = None;
    for key in keys {
        let Some(value) = property(key) else { continue };
        if key.t <= t {
            previous = (key.t, value, key.curve.unwrap_or(curve));
        } else {
            next = Some((key.t, value));
            break;
        }
    }
    let next = next.unwrap_or((1.0, default));
    if next.0 <= previous.0 {
        return previous.1;
    }
    let p = ((t - previous.0) / (next.0 - previous.0)).clamp(0.0, 1.0);
    previous.1 + (next.1 - previous.1) * motion::ease(previous.2, p)
}
pub fn paint(
    painter: &Painter,
    name: &str,
    rect: Rect,
    color: Color32,
    stroke: f32,
    progress: f32,
) {
    let Some(parts) = CACHE.get(name) else { return };
    let scale = rect.width().min(rect.height()) / 24.0;
    let offset = rect.center() - vec2(12.0, 12.0) * scale;
    for part in parts {
        let mut transform = Transform::default();
        let mut opacity = if name == "cursor" && has_group(part.group, "b") {
            0.0
        } else {
            1.0
        };
        let mut trace = 1.0;
        let mut origin = pos2(12.0, 12.0);
        if part.group.split_whitespace().any(|g| {
            g.starts_with('d')
                || g.starts_with('s')
                || g == "fb"
                || g.starts_with('b') && g.chars().last().is_some_and(|c| c.is_ascii_digit())
        }) || name == "window" && has_group(part.group, "a")
        {
            let bbox = part
                .paths
                .iter()
                .flat_map(|(points, _)| points.iter())
                .fold(Rect::NOTHING, |r, p| r.union(Rect::from_min_max(*p, *p)));
            origin = bbox.center();
        }
        if name == "chart-bar" {
            let bbox = part
                .paths
                .iter()
                .flat_map(|(points, _)| points.iter())
                .fold(Rect::NOTHING, |r, p| r.union(Rect::from_min_max(*p, *p)));
            origin = pos2(bbox.center().x, bbox.bottom());
        }
        if name == "menu" {
            origin = pos2(4.0, 12.0);
        }
        if progress > 0.0 {
            for rule in data::RULES
                .iter()
                .filter(|rule| rule.name == name && has_group(part.group, rule.group))
            {
                let time = progress * 1200.0;
                let p = (time - rule.delay) / rule.duration;
                if !rule.fill && !(0.0..1.0).contains(&p) {
                    continue;
                }
                let p = p.clamp(0.0, 1.0);
                let keys = data::keys(rule.keyframes);
                if let Some(o) = rule.origin {
                    origin = pos2(o[0], o[1]);
                }
                transform.x += sample(keys, p, 0.0, rule.curve, |k| k.transform.map(|t| t.x))
                    + sample(keys, p, 0.0, rule.curve, |k| k.translate.map(|t| t[0]));
                transform.y += sample(keys, p, 0.0, rule.curve, |k| k.transform.map(|t| t.y))
                    + sample(keys, p, 0.0, rule.curve, |k| k.translate.map(|t| t[1]));
                transform.sx *= sample(keys, p, 1.0, rule.curve, |k| k.transform.map(|t| t.sx));
                transform.sy *= sample(keys, p, 1.0, rule.curve, |k| k.transform.map(|t| t.sy));
                transform.rotation += sample(keys, p, 0.0, rule.curve, |k| {
                    k.transform.map(|t| t.rotation)
                });
                if keys.iter().any(|k| k.opacity.is_some()) {
                    opacity = sample(keys, p, opacity, rule.curve, |k| k.opacity);
                }
                if keys.iter().any(|k| k.trace.is_some()) {
                    trace = sample(keys, p, 1.0, rule.curve, |k| k.trace);
                }
            }
        }
        if opacity <= 0.001 {
            continue;
        }
        let c = color.gamma_multiply(opacity.clamp(0.0, 1.0));
        let rotation = transform.rotation.to_radians();
        let transformed = |p: &Pos2| {
            let v = (*p - origin) * vec2(transform.sx, transform.sy);
            let local = origin
                + vec2(
                    v.x * rotation.cos() - v.y * rotation.sin(),
                    v.x * rotation.sin() + v.y * rotation.cos(),
                )
                + vec2(transform.x, transform.y);
            offset + local.to_vec2() * scale
        };
        for (points, closed) in &part.paths {
            let points: Vec<Pos2> = points.iter().map(transformed).collect();
            if part.fill && *closed && points.len() > 2 {
                painter.add(Shape::convex_polygon(points.clone(), c, Stroke::NONE));
            }
            if part.stroke {
                let line = Stroke::new(part.width.unwrap_or(stroke) * scale, c);
                if trace >= 0.999 {
                    painter.add(Shape::line(points, line));
                } else {
                    crate::ui::paint_trace(painter, &points, trace, line);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_icon_has_finite_native_geometry() {
        assert_eq!(names().len(), 82);
        for (name, parts) in CACHE.iter() {
            assert!(!parts.is_empty(), "{name}");
            for part in parts {
                assert!(!part.paths.is_empty(), "{name}");
                for (points, _) in &part.paths {
                    assert!(
                        points.iter().all(|p| p.x.is_finite() && p.y.is_finite()),
                        "{name}"
                    );
                }
            }
        }
    }
    #[test]
    fn every_choreography_resolves_to_native_keyframes() {
        for rule in data::RULES {
            assert!(!data::keys(rule.keyframes).is_empty(), "{}", rule.keyframes);
            assert!(rule.duration > 0.0);
        }
    }
}
