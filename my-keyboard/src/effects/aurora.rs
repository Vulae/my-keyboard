// TODO: Actually finish this!

#![allow(unused)]

use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

use crate::util::{lerp, simple_ease};

use super::Effect;

fn aurora_color_bg(x: f32, y: f32, time: f32) -> Color {
    Color::new(0.4, 0.0, 0.8)
}

fn aurora_color_main(x: f32, y: f32, time: f32) -> Color {
    Color::new(0.3, 0.3, 0.7)
}

fn aurora_dist(x: f32, y: f32, time: f32) -> f32 {
    let x = x - (MATRIX_WIDTH as f32) / 2.0;
    let y = y - (MATRIX_HEIGHT as f32) / 2.0;
    (x * x + y * y).sqrt() / 8.0
}

fn aurora_color(x: f32, y: f32, time: f32) -> Color {
    let main = aurora_color_main(x, y, time);
    let bg = aurora_color_bg(x, y, time);
    let dist = aurora_dist(x, y, time);

    let interpolation = dist.clamp(0.0, 1.0);
    let interpolation = simple_ease(interpolation);

    Color::new(
        lerp(main.r, bg.r, interpolation),
        lerp(main.g, bg.g, interpolation),
        lerp(main.b, bg.b, interpolation),
    )
}

#[derive(Debug)]
pub struct EffectAurora {
    start: std::time::Instant,
}

impl EffectAurora {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Effect for EffectAurora {
    fn identifier(&self) -> &str {
        "effect_aurora"
    }

    fn update<'a, 'b>(&mut self, matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        matrix.iter_mut().for_each(|(x, y, color)| {
            let x = x as f32;
            let y = y as f32;

            *color = aurora_color(x, y, time.as_secs_f32());
        });

        matrix.send_update()?;
        Ok(())
    }
}
