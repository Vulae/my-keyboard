use std::ops::{Add, Mul};

pub fn lerp<V>(v0: V, v1: V, t: f32) -> V
where
    V: Mul<f32, Output = V> + Add<V, Output = V>,
{
    v0 * (1.0 - t) + v1 * t
}

pub fn simple_ease(x: f32) -> f32 {
    if x > 0.5 {
        1.0 - (2.0 * (1.0 - x)).powi(4) / 2.0
    } else {
        (2.0 * x).powi(4) / 2.0
    }
}
