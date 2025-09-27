use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

use crate::util::simple_ease;

use super::{Effect, MatrixInput};

const RIPPLE_SPEED: f32 = 10.0;

#[derive(Debug)]
struct Ripple {
    color: Color,
    x: f32,
    y: f32,
    start: std::time::Instant,
}

impl Ripple {
    fn new(x: f32, y: f32) -> Self {
        Self {
            color: Color::from_hsl(rand::random::<f32>() * 360.0, 1.0, 0.5),
            x,
            y,
            start: std::time::Instant::now(),
        }
    }

    fn new_random() -> Self {
        Self::new(
            rand::random::<f32>() * MATRIX_WIDTH as f32,
            rand::random::<f32>() * MATRIX_HEIGHT as f32,
        )
    }

    fn dist(&self, x: f32, y: f32) -> f32 {
        f32::sqrt((x - self.x).powi(2) + (y - self.y).powi(2))
    }
}

#[derive(Debug)]
pub struct EffectRipple {
    ripples: Vec<Ripple>,
}

impl EffectRipple {
    pub fn new() -> Self {
        Self {
            ripples: vec![Ripple::new_random()],
        }
    }
}

impl Effect for EffectRipple {
    fn identifier(&self) -> &str {
        "effect_ripple"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let now = std::time::Instant::now();

        for input in inputs {
            if let MatrixInput::Pressed { x, y } = input {
                self.ripples.push(Ripple::new(*x as f32, *y as f32));
            }
            // self.ripples
            //     .push(Ripple::new(input.x() as f32, input.y() as f32));
        }

        self.ripples.retain(|ripple| {
            now.duration_since(ripple.start).as_secs_f32()
                <= (MATRIX_WIDTH as f32 + MATRIX_HEIGHT as f32) / RIPPLE_SPEED
        });

        if self.ripples.is_empty() {
            self.ripples.push(Ripple::new_random());
        }

        matrix.iter_mut().for_each(|(x, y, color)| {
            let x = x as f32;
            let y = y as f32;

            *color = self
                .ripples
                .iter()
                .map(|ripple| {
                    let t = now.duration_since(ripple.start);
                    let ripple_radius = t.as_secs_f32() * RIPPLE_SPEED;
                    let dist = ripple.dist(x, y) - ripple_radius;
                    let amount = 1.0 - simple_ease((dist.abs() / 2.0).clamp(0.0, 1.0));
                    ripple.color * amount
                })
                .fold(Color::new(0.0, 0.0, 0.0), |acc, col| acc + col)
        });

        matrix.send_update()?;

        Ok(())
    }
}
