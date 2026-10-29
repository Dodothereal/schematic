use crate::easing::{cubic_bezier, sampled_linear};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Curve {
    Linear,
    Out,
    OutExpo,
    In,
    InOut,
    Overshoot,
    SpringSmooth,
    SpringSnappy,
    SpringBouncy,
    Default,
    Steps(u8),
    Custom(f32, f32, f32, f32),
}
pub fn ease(curve: Curve, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) as f64;
    (match curve {
        Curve::Linear => t,
        Curve::Out => cubic_bezier(t, 0.22, 1.0, 0.36, 1.0),
        Curve::OutExpo => cubic_bezier(t, 0.16, 1.0, 0.3, 1.0),
        Curve::In => cubic_bezier(t, 0.64, 0.0, 0.78, 0.0),
        Curve::InOut => cubic_bezier(t, 0.65, 0.0, 0.35, 1.0),
        Curve::Overshoot => cubic_bezier(t, 0.34, 1.56, 0.64, 1.0),
        Curve::Default => cubic_bezier(t, 0.25, 0.1, 0.25, 1.0),
        Curve::Steps(n) => (t * n.max(1) as f64).floor() / n.max(1) as f64,
        Curve::Custom(a, b, c, d) => cubic_bezier(t, a as f64, b as f64, c as f64, d as f64),
        Curve::SpringSmooth => sampled_linear(SMOOTH, t),
        Curve::SpringSnappy => sampled_linear(SNAPPY, t),
        Curve::SpringBouncy => sampled_linear(BOUNCY, t),
    }) as f32
}
pub const SMOOTH: &[f64] = &[
    0.0, 0.0198, 0.0597, 0.1221, 0.1858, 0.2639, 0.3334, 0.4115, 0.477, 0.5388, 0.604, 0.656,
    0.7095, 0.7512, 0.7933, 0.8255, 0.8537, 0.8814, 0.9021, 0.9222, 0.937, 0.9511, 0.9613, 0.9709,
    0.9778, 0.9834, 0.9885, 0.992, 0.9952, 0.9973, 0.9991, 1.0003, 1.0012, 1.0018, 1.0021, 1.0024,
    1.0025, 1.0024, 1.0024, 1.0022, 1.0021, 1.0019, 1.0017, 1.0015, 1.0014, 1.0012, 1.001, 1.0009,
    1.0,
];
pub const SNAPPY: &[f64] = &[
    0.0, 0.0203, 0.0681, 0.1343, 0.212, 0.2954, 0.3803, 0.4635, 0.5425, 0.6159, 0.6721, 0.733,
    0.7868, 0.8334, 0.8732, 0.9067, 0.9345, 0.9571, 0.9752, 0.9893, 0.9984, 1.0067, 1.0126, 1.0165,
    1.0188, 1.0199, 1.02, 1.0195, 1.0184, 1.0172, 1.0157, 1.014, 1.0123, 1.0106, 1.009, 1.0076,
    1.0062, 1.0051, 1.004, 1.0033, 1.0025, 1.0018, 1.0013, 1.0008, 1.0005, 1.0002, 1.0, 0.9998,
    1.0,
];
pub const BOUNCY: &[f64] = &[
    0.0, 0.0584, 0.1975, 0.3806, 0.5604, 0.7473, 0.9084, 1.0253, 1.116, 1.1696, 1.1903, 1.1879,
    1.1669, 1.1375, 1.1004, 1.0633, 1.0325, 1.0048, 0.9842, 0.9718, 0.9647, 0.9632, 0.9655, 0.9708,
    0.9776, 0.9843, 0.9911, 0.997, 1.0012, 1.0045, 1.0063, 1.007, 1.0069, 1.0061, 1.005, 1.0036,
    1.0022, 1.0011, 1.0001, 0.9994, 0.9989, 0.9987, 0.9986, 0.9987, 0.9989, 0.9992, 0.9994, 0.9997,
    1.0,
];
#[derive(Clone, Debug)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub start: f64,
    pub duration: f32,
    pub curve: Curve,
}
impl Tween {
    pub fn new(value: f32, now: f64) -> Self {
        Self {
            from: value,
            to: value,
            start: now,
            duration: 0.0,
            curve: Curve::Out,
        }
    }
    pub fn value(&self, now: f64) -> f32 {
        if self.duration <= 0.0 {
            return self.to;
        }
        self.from
            + (self.to - self.from)
                * ease(
                    self.curve,
                    ((now - self.start) / self.duration as f64) as f32,
                )
    }
    pub fn retarget(&mut self, target: f32, now: f64, duration: f32, curve: Curve) {
        if self.to != target {
            self.from = self.value(now);
            self.to = target;
            self.start = now;
            self.duration = duration;
            self.curve = curve;
        }
    }
    pub fn moving(&self, now: f64) -> bool {
        self.from != self.to && now - self.start < self.duration as f64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retarget_preserves_displayed_value() {
        let mut t = Tween::new(0.0, 0.0);
        t.retarget(1.0, 0.0, 1.0, Curve::Out);
        let middle = t.value(0.3);
        t.retarget(0.0, 0.3, 0.5, Curve::Out);
        assert_eq!(t.value(0.3), middle);
        assert_eq!(t.value(0.8), 0.0);
    }
}
