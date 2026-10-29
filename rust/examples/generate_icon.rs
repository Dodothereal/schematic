//! Render the application icon from Rust vector coordinates.
use image::{ImageBuffer, Rgba, RgbaImage};
fn distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / (d[0] * d[0] + d[1] * d[1]))
        .clamp(0.0, 1.0);
    ((p[0] - a[0] - t * d[0]).powi(2) + (p[1] - a[1] - t * d[1]).powi(2)).sqrt()
}
fn main() {
    let size = 1024u32;
    let mut image: RgbaImage = ImageBuffer::new(size, size);
    let edges = [
        ([0.5, 0.24], [0.73, 0.375]),
        ([0.73, 0.375], [0.73, 0.645]),
        ([0.73, 0.645], [0.5, 0.78]),
        ([0.5, 0.78], [0.27, 0.645]),
        ([0.27, 0.645], [0.27, 0.375]),
        ([0.27, 0.375], [0.5, 0.24]),
        ([0.27, 0.375], [0.5, 0.51]),
        ([0.5, 0.51], [0.73, 0.375]),
        ([0.5, 0.51], [0.5, 0.78]),
    ];
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let p = [
            (x as f32 + 0.5) / size as f32,
            (y as f32 + 0.5) / size as f32,
        ];
        let q = [(p[0] - 0.5).abs() - 0.29, (p[1] - 0.5).abs() - 0.29];
        let sdf =
            (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2)).sqrt() + q[0].max(q[1]).min(0.0) - 0.16;
        let alpha = (-sdf * size as f32 + 0.5).clamp(0.0, 1.0);
        let line = edges
            .iter()
            .map(|(a, b)| distance(p, *a, *b))
            .fold(f32::INFINITY, f32::min);
        let blend = ((0.009 - line) * size as f32 + 0.5).clamp(0.0, 1.0);
        let background = [24.0, 28.0, 27.0];
        let ink = [235.0, 244.0, 237.0];
        *pixel = Rgba([
            (background[0] + (ink[0] - background[0]) * blend) as u8,
            (background[1] + (ink[1] - background[1]) * blend) as u8,
            (background[2] + (ink[2] - background[2]) * blend) as u8,
            (alpha * 255.0) as u8,
        ]);
    }
    std::fs::create_dir_all("rust/assets").unwrap();
    image.save("rust/assets/icon.png").unwrap();
}
