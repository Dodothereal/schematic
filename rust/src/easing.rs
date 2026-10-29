//! Platform-independent motion math. UI runtimes consume the same samples.

pub fn cubic_bezier(progress: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if progress <= 0.0 {
        return 0.0;
    }
    if progress >= 1.0 {
        return 1.0;
    }
    let sample = |t: f64, a: f64, b: f64| {
        3.0 * (1.0 - t).powi(2) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
    };
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if sample(mid, x1, x2) < progress {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    sample((lo + hi) / 2.0, y1, y2)
}

pub fn sampled_linear(values: &[f64], progress: f64) -> f64 {
    if values.is_empty() {
        return progress.clamp(0.0, 1.0);
    }
    if values.len() == 1 {
        return values[0];
    }
    let position = progress.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let index = (position as usize).min(values.len() - 2);
    values[index] + (values[index + 1] - values[index]) * (position - index as f64)
}

#[derive(Clone, Debug)]
pub struct Spring {
    pub duration_ms: u32,
    pub values: Vec<f64>,
}

impl Spring {
    /// Same 240 Hz semi-implicit integrator and stop conditions as Motion Lab.
    pub fn simulate(stiffness: f64, damping: f64, mass: f64) -> Self {
        let dt = 1.0 / 240.0;
        let (mut x, mut velocity, mut time) = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut samples = vec![0.0];
        while time < 4.0 {
            velocity += (-stiffness.max(0.0) * (x - 1.0) - damping.max(0.0) * velocity)
                / mass.max(0.001)
                * dt;
            x += velocity * dt;
            time += dt;
            samples.push(x);
            if time > 0.2 && (x - 1.0).abs() < 0.001 && velocity.abs() < 0.01 {
                break;
            }
        }
        let values = (0..=80)
            .map(|index| {
                if index == 80 {
                    1.0
                } else {
                    samples[((index as f64 / 80.0 * (samples.len() - 1) as f64).round() as usize)
                        .min(samples.len() - 1)]
                }
            })
            .collect();
        Self {
            duration_ms: (time * 1000.0).round() as u32,
            values,
        }
    }

    pub fn at(&self, progress: f64) -> f64 {
        sampled_linear(&self.values, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn travel_curve_is_symmetric_and_has_exact_endpoints() {
        assert_eq!(cubic_bezier(0.0, 0.65, 0.0, 0.35, 1.0), 0.0);
        assert_eq!(cubic_bezier(1.0, 0.65, 0.0, 0.35, 1.0), 1.0);
        for step in 1..100 {
            let p = step as f64 / 100.0;
            let a = cubic_bezier(p, 0.65, 0.0, 0.35, 1.0);
            let b = cubic_bezier(1.0 - p, 0.65, 0.0, 0.35, 1.0);
            assert!((a + b - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn confirmation_curve_keeps_its_overshoot() {
        assert!(cubic_bezier(0.7, 0.34, 1.56, 0.64, 1.0) > 1.0);
    }

    #[test]
    fn spring_has_shared_sampling_and_settles() {
        let spring = Spring::simulate(300.0, 26.0, 1.0);
        assert_eq!(spring.values.len(), 81);
        assert_eq!(spring.at(0.0), 0.0);
        assert_eq!(spring.at(1.0), 1.0);
        assert!(spring.values.iter().copied().fold(0.0_f64, f64::max) > 1.0);
        assert!((200..=4005).contains(&spring.duration_ms));
        assert!(Spring::simulate(300.0, 8.0, 1.0).duration_ms > spring.duration_ms);
    }

    #[test]
    fn sampling_interpolates_and_bounds_time_without_clamping_spring_values() {
        let values = [0.0, 1.2, 1.0];
        assert_eq!(sampled_linear(&values, -0.1), 0.0);
        assert_eq!(sampled_linear(&values, 0.25), 0.6);
        assert_eq!(sampled_linear(&values, 0.5), 1.2);
        assert_eq!(sampled_linear(&values, 1.1), 1.0);
    }
}
