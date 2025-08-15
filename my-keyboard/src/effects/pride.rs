use std::sync::LazyLock;

use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_WIDTH};

use super::Effect;

const SCALE: f32 = 0.15;

#[rustfmt::skip]
static COLORS: LazyLock<Box<[Color]>> = LazyLock::new(|| {
    [
        "#E50000", "#FF8D00", "#FFEE00", "#028121", "#004CFF", "#770088", // Pride
        "#55CDFD", "#F6AAB7", "#FFFFFF", "#F6AAB7", "#55CDFD", // Transgender
        "#FCF431", "#FCFCFC", "#9D59D2", "#282828", // Nonbinary
        "#FE76A2", "#FFFFFF", "#BF12D7", "#000000", "#303CBE", // Genderfluid
        "#D60270", "#9B4F96", "#0038A8", // Bisexual
        "#FF1C8D", "#FFD700", "#1AB3FF", // Pansexual
        "#078D70", "#98E8C1", "#FFFFFF", "#7BADE2", "#3D1A78", // Gay
        "#D62800", "#FF9B56", "#FFFFFF", "#D462A6", "#A40062", // Lesbian
        "#000000", "#A4A4A4", "#FFFFFF", "#810081", // Asexual
        "#3BA740", "#A8D47A", "#FFFFFF", "#ABABAB", "#000000", // Aromantic
        "#F714BA", "#01D66A", "#1594F6", // Polysexual
    ]
    .into_iter()
    .map(|hex| Color::from_hex(hex).unwrap())
    .collect()
});

fn get_color(percent: f32) -> Color {
    let percent = percent % 1.0;

    let index = (percent * COLORS.len() as f32) as usize;
    let next_index = (index + 1) % COLORS.len();
    let color = COLORS[index];
    let next_color = COLORS[next_index];

    fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
        (1.0 - t) * v0 + t * v1
    }

    fn simple_ease(x: f32) -> f32 {
        if x > 0.5 {
            1.0 - (2.0 * (1.0 - x)).powi(4) / 2.0
        } else {
            (2.0 * x).powi(4) / 2.0
        }
    }

    let interpolation = percent % (1.0 / COLORS.len() as f32) * COLORS.len() as f32;
    let interpolation = simple_ease(interpolation);

    Color::new(
        lerp(color.r, next_color.r, interpolation),
        lerp(color.g, next_color.g, interpolation),
        lerp(color.b, next_color.b, interpolation),
    )
}

#[derive(Debug)]
pub struct EffectPride {
    start: std::time::Instant,
}

impl EffectPride {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Effect for EffectPride {
    fn identifier(&self) -> &str {
        "effect_pride"
    }

    fn update<'a, 'b>(&mut self, matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        matrix.iter_mut().for_each(|(x, _y, color)| {
            let pos = (x as f32 / MATRIX_WIDTH as f32) * SCALE;
            *color = get_color(pos + time.as_secs_f32() * 0.02);
        });

        matrix.send_update()?;
        Ok(())
    }
}
