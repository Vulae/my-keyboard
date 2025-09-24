pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

pub fn simple_ease(x: f32) -> f32 {
    if x > 0.5 {
        1.0 - (2.0 * (1.0 - x)).powi(4) / 2.0
    } else {
        (2.0 * x).powi(4) / 2.0
    }
}
