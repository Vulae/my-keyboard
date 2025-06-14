use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

use super::Effect;

struct Metaball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

pub struct EffectRainbow3<'a, 'b> {
    matrix: &'b mut DeviceMatrixCustom<'a>,
    start: std::time::Instant,
    last_frame: Option<std::time::Instant>,
    balls: Vec<Metaball>,
}

impl<'a, 'b> Effect<'a, 'b> for EffectRainbow3<'a, 'b> {
    fn attach(matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<Self, Error> {
        Ok(Self {
            matrix,
            start: std::time::Instant::now(),
            last_frame: None,
            balls: std::array::from_fn::<_, 3, _>(|_| Metaball {
                x: rand::random_range((0.0)..=(MATRIX_WIDTH as f32 - 1.0)),
                y: rand::random_range((0.0)..=(MATRIX_HEIGHT as f32 - 1.0)),
                vx: rand::random_range((-4.0)..=4.0),
                vy: rand::random_range((-4.0)..=4.0),
            })
            .into_iter()
            .collect(),
        })
    }

    fn update(&mut self) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        let dt = if let Some(last_frame) = self.last_frame {
            std::time::Instant::now()
                .duration_since(last_frame)
                .as_secs_f32()
        } else {
            0.0
        };
        self.last_frame = Some(std::time::Instant::now());

        self.balls.iter_mut().for_each(|ball| {
            if ball.x < 0.0 {
                ball.x = 0.0;
                ball.vx = f32::abs(ball.vx);
            }
            if ball.y < 0.0 {
                ball.y = 0.0;
                ball.vy = f32::abs(ball.vy);
            }
            if ball.x > MATRIX_WIDTH as f32 - 1.0 {
                ball.x = MATRIX_WIDTH as f32 - 1.0;
                ball.vx = -f32::abs(ball.vx);
            }
            if ball.y > MATRIX_HEIGHT as f32 - 1.0 {
                ball.y = MATRIX_HEIGHT as f32 - 1.0;
                ball.vy = -f32::abs(ball.vy);
            }
            ball.x += ball.vx * dt;
            ball.y += ball.vy * dt;
        });

        self.matrix.iter_mut().for_each(|(x, y, color)| {
            let x = x as f32;
            let y = y as f32;

            let dist_sum = self.balls.iter().fold(0.0, |sum, ball| {
                sum + ((ball.x - x).powi(2) + (ball.y - y).powi(2)).sqrt()
            });

            *color = Color::from_hsl(dist_sum * 20.0 + time.as_secs_f32() * 10.0, 1.0, 0.5);
        });

        self.matrix.send_update()?;
        Ok(())
    }
}
