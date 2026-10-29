use eframe::egui::Color32;

/// Semantic colors in the reference's perceptual OKLCH space.
#[derive(Clone, Debug)]
pub struct Tokens {
    pub bg: Color32,
    pub bg_sunken: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub surface3: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub border_soft: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub text3: Color32,
    pub text4: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_press: Color32,
    pub accent_text: Color32,
    pub accent_contrast: Color32,
    pub accent_soft: Color32,
    pub accent_soft2: Color32,
    pub accent_ring: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub success_text: Color32,
    pub warning: Color32,
    pub warning_soft: Color32,
    pub warning_text: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub danger_text: Color32,
    pub info: Color32,
    pub info_soft: Color32,
    pub info_text: Color32,
    pub overlay: Color32,
    pub charts: [Color32; 5],
    pub radius: f32,
}
pub fn oklch(l: f32, c: f32, h: f32, alpha: f32) -> Color32 {
    let angle = h.to_radians();
    let a = c * angle.cos();
    let b = c * angle.sin();
    let ll = (l + 0.39633778 * a + 0.21580376 * b).powi(3);
    let mm = (l - 0.105561346 * a - 0.06385417 * b).powi(3);
    let ss = (l - 0.08948418 * a - 1.2914855 * b).powi(3);
    let encode = |v: f32| {
        let n = if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.max(0.0).powf(1.0 / 2.4) - 0.055
        };
        (n.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Color32::from_rgba_unmultiplied(
        encode(4.0767417 * ll - 3.3077116 * mm + 0.23096994 * ss),
        encode(-1.268438 * ll + 2.6097574 * mm - 0.34131938 * ss),
        encode(-0.0041960863 * ll - 0.7034186 * mm + 1.7076147 * ss),
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}
impl Tokens {
    pub fn new(dark: bool, mono: bool, hue: f32, radius: f32) -> Self {
        let n: f32 = if mono { 0.0 } else { 0.008 };
        let a: f32 = if mono { 0.0 } else { 0.115 };
        let neutral = |l: f32, c: f32| oklch(l, n * c, 80.0, 1.0);
        let accent = |l: f32, c: f32, alpha: f32| oklch(l, a * c, hue, alpha);
        if dark {
            Self {
                bg: neutral(0.17, 1.0),
                bg_sunken: neutral(0.15, 1.0),
                surface: neutral(0.205, 1.0),
                surface2: neutral(0.23, 1.0),
                surface3: neutral(0.265, 1.0),
                surface_raised: neutral(0.24, 1.0),
                border: neutral(0.3, 1.0),
                border_soft: neutral(0.255, 1.0),
                border_strong: neutral(0.38, 1.0),
                text: neutral(0.965, 0.6),
                text2: neutral(0.77, 1.0),
                text3: neutral(0.68, 1.0),
                text4: neutral(0.47, 1.0),
                accent: accent(0.73, 1.0, 1.0),
                accent_hover: accent(0.78, 1.0, 1.0),
                accent_press: accent(0.69, 1.0, 1.0),
                accent_text: accent(0.8, 0.8, 1.0),
                accent_contrast: oklch(0.14, 0.0, 0.0, 1.0),
                accent_soft: accent(0.73, 1.0, 0.12),
                accent_soft2: accent(0.73, 1.0, 0.18),
                accent_ring: accent(0.73, 1.0, 0.45),
                success: oklch(0.7, 0.15, 155.0, 1.0),
                success_soft: oklch(0.7, 0.15, 155.0, 0.14),
                success_text: oklch(0.78, 0.14, 155.0, 1.0),
                warning: oklch(0.8, 0.15, 75.0, 1.0),
                warning_soft: oklch(0.8, 0.15, 75.0, 0.14),
                warning_text: oklch(0.84, 0.13, 80.0, 1.0),
                danger: oklch(0.69, 0.18, 25.0, 1.0),
                danger_soft: oklch(0.66, 0.2, 25.0, 0.15),
                danger_text: oklch(0.76, 0.16, 25.0, 1.0),
                info: oklch(0.7, 0.13, 235.0, 1.0),
                info_soft: oklch(0.7, 0.13, 235.0, 0.15),
                info_text: oklch(0.8, 0.11, 235.0, 1.0),
                overlay: oklch(0.08, n, 80.0, 0.6),
                charts: [
                    oklch(0.73, 0.13, 255.0, 1.0),
                    oklch(0.76, 0.11, 185.0, 1.0),
                    oklch(0.8, 0.13, 80.0, 1.0),
                    oklch(0.74, 0.15, 340.0, 1.0),
                    oklch(0.75, 0.13, 295.0, 1.0),
                ],
                radius,
            }
        } else {
            Self {
                bg: neutral(0.982, 1.0),
                bg_sunken: neutral(0.958, 1.0),
                surface: neutral(0.995, 0.35),
                surface2: neutral(0.972, 0.8),
                surface3: neutral(0.945, 1.0),
                surface_raised: neutral(0.995, 0.35),
                border: neutral(0.895, 1.2),
                border_soft: neutral(0.93, 1.0),
                border_strong: neutral(0.82, 1.4),
                text: neutral(0.23, 1.0),
                text2: neutral(0.41, 1.0),
                text3: neutral(0.49, 1.0),
                text4: neutral(0.67, 1.0),
                accent: accent(0.45, 1.0, 1.0),
                accent_hover: accent(0.42, 1.0, 1.0),
                accent_press: accent(0.39, 1.0, 1.0),
                accent_text: accent(0.41, 1.0, 1.0),
                accent_contrast: oklch(0.995, 0.0, 0.0, 1.0),
                accent_soft: accent(0.45, 1.0, 0.08),
                accent_soft2: accent(0.45, 1.0, 0.13),
                accent_ring: accent(0.45, 1.0, 0.42),
                success: oklch(0.6, 0.15, 155.0, 1.0),
                success_soft: oklch(0.6, 0.15, 155.0, 0.12),
                success_text: oklch(0.44, 0.11, 155.0, 1.0),
                warning: oklch(0.75, 0.16, 70.0, 1.0),
                warning_soft: oklch(0.75, 0.16, 70.0, 0.16),
                warning_text: oklch(0.47, 0.11, 60.0, 1.0),
                danger: oklch(0.51, 0.19, 25.0, 1.0),
                danger_soft: oklch(0.6, 0.21, 25.0, 0.1),
                danger_text: oklch(0.47, 0.18, 25.0, 1.0),
                info: oklch(0.62, 0.14, 235.0, 1.0),
                info_soft: oklch(0.62, 0.14, 235.0, 0.12),
                info_text: oklch(0.44, 0.11, 235.0, 1.0),
                overlay: oklch(0.2, n, 80.0, 0.28),
                charts: [
                    oklch(0.53, 0.15, 255.0, 1.0),
                    oklch(0.58, 0.1, 185.0, 1.0),
                    oklch(0.65, 0.14, 75.0, 1.0),
                    oklch(0.58, 0.17, 340.0, 1.0),
                    oklch(0.57, 0.14, 295.0, 1.0),
                ],
                radius,
            }
        }
    }
}
impl Default for Tokens {
    fn default() -> Self {
        Self::new(false, false, 168.0, 1.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_endpoints() {
        assert_eq!(oklch(0.0, 0.0, 0.0, 1.0), Color32::BLACK);
        assert_eq!(oklch(1.0, 0.0, 0.0, 1.0), Color32::WHITE);
    }
    #[test]
    fn mono_changes_interface_but_keeps_chart_identity() {
        let a = Tokens::new(false, false, 168.0, 1.0);
        let b = Tokens::new(false, true, 168.0, 1.0);
        assert_eq!(a.charts, b.charts);
        assert_eq!(b.accent.r(), b.accent.g());
        assert_eq!(b.accent.g(), b.accent.b());
        assert_ne!(a.accent, b.accent);
    }
}
