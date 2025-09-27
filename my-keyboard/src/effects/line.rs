use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

use crate::util::simple_ease;

use super::{Effect, MatrixInput};

/// Seconds until line starts decaying
const LINE_DECAY_OFFSET: f32 = 0.5;
/// Line decay seconds after initial wait
const LINE_DECAY_DURATION: f32 = 0.5;

#[derive(Debug)]
struct Line {
    color: Color,
    x: f32,
    y: f32,
    angle: f32,
    start: std::time::Instant,
}

impl Line {
    fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            color: Color::from_hsl(rand::random::<f32>() * 360.0, 1.0, 0.5),
            x,
            y,
            angle,
            start: std::time::Instant::now(),
        }
    }

    fn new_random_angle(x: f32, y: f32) -> Self {
        Self::new(x, y, rand::random::<f32>() * std::f32::consts::TAU)
    }

    fn new_random_pos_angle() -> Self {
        Self::new_random_angle(
            rand::random::<f32>() * MATRIX_WIDTH as f32,
            rand::random::<f32>() * MATRIX_HEIGHT as f32,
        )
    }

    fn dist(&self, x: f32, y: f32) -> f32 {
        let (ax, ay) = f32::sin_cos(self.angle);
        f32::abs(ax * (x - self.x) - ay * (y - self.y))
    }
}

#[derive(Debug)]
pub struct EffectLine {
    lines: Vec<Line>,
}

impl EffectLine {
    pub fn new() -> Self {
        Self {
            lines: vec![Line::new_random_pos_angle()],
        }
    }
}

impl Effect for EffectLine {
    fn identifier(&self) -> &str {
        "effect_line"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let now = std::time::Instant::now();

        for input in inputs {
            // if let MatrixInput::Pressed { x, y } = input {
            //     self.lines
            //         .push(Line::new_random_angle(*x as f32, *y as f32));
            // }
            self.lines
                .push(Line::new_random_angle(input.x() as f32, input.y() as f32));
        }

        self.lines.retain(|line| {
            now.duration_since(line.start).as_secs_f32() <= LINE_DECAY_OFFSET + LINE_DECAY_DURATION
        });

        if self.lines.is_empty() {
            self.lines.push(Line::new_random_pos_angle());
        }

        matrix.iter_mut().for_each(|(x, y, color)| {
            let x = x as f32;
            let y = y as f32;

            *color = self
                .lines
                .iter()
                .map(|line| {
                    let dist = line.dist(x, y);
                    let amount = 1.0 - simple_ease((dist / 1.5).clamp(0.0, 1.0));
                    let t = now.duration_since(line.start);
                    let amount = if t.as_secs_f32() <= LINE_DECAY_OFFSET {
                        amount
                    } else {
                        amount
                            * (1.0 - ((t.as_secs_f32() - LINE_DECAY_OFFSET) / LINE_DECAY_DURATION))
                    };
                    line.color * amount
                })
                .fold(Color::new(0.0, 0.0, 0.0), |acc, col| acc + col)
        });

        matrix.send_update()?;

        Ok(())
    }
}
