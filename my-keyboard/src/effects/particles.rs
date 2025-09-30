use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

use crate::util::simple_ease;

use super::{Effect, MatrixInput};

/// Seconds until the particle starts decaying
const PARTICLE_DECAY_OFFSET: f32 = 3.0;
/// Particle decay seconds after initial wait
const PARTICLE_DECAY_DURATION: f32 = 0.5;

const NO_INPUT_AUTOSPAWN_PARTICLES_TIME: f32 = 5.0;
const NO_INPUT_AUTOSPAWN_PARTICLES_DELAY: f32 = 0.5;

#[derive(Debug)]
struct Particle {
    color: Color,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    start: std::time::Instant,
}

impl Particle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            color: Color::from_hsl(rand::random::<f32>() * 360.0, 1.0, 0.5),
            x,
            y,
            vx: rand::random::<f32>() * 20.0 - 10.0,
            vy: rand::random::<f32>() * 20.0 - 10.0,
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
pub struct EffectParticles {
    particles: Vec<Particle>,
    last_update: Option<std::time::Instant>,
    time_since_last_input: std::time::Instant,
    time_since_last_autospawn: std::time::Instant,
}

impl EffectParticles {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            last_update: None,
            time_since_last_input: std::time::Instant::now(),
            time_since_last_autospawn: std::time::Instant::now(),
        }
    }
}

impl Effect for EffectParticles {
    fn identifier(&self) -> &str {
        "effect_particles"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let now = std::time::Instant::now();
        let dt = if let Some(last_update) = self.last_update {
            now.duration_since(last_update).as_secs_f32()
        } else {
            0.0
        };

        self.last_update = Some(now);

        for input in inputs {
            if let MatrixInput::Pressed { x, y } = input {
                self.particles.push(Particle::new(*x as f32, *y as f32));
            }
        }

        if !inputs.is_empty() {
            self.time_since_last_input = now;
        }

        if now.duration_since(self.time_since_last_input).as_secs_f32()
            >= NO_INPUT_AUTOSPAWN_PARTICLES_TIME
            && now
                .duration_since(self.time_since_last_autospawn)
                .as_secs_f32()
                >= NO_INPUT_AUTOSPAWN_PARTICLES_DELAY
        {
            self.particles.push(Particle::new_random());
            self.time_since_last_autospawn = now;
        }

        self.particles.retain(|particle| {
            now.duration_since(particle.start).as_secs_f32()
                <= PARTICLE_DECAY_OFFSET + PARTICLE_DECAY_DURATION
        });

        self.particles.iter_mut().for_each(|particle| {
            if particle.x < 0.0 {
                particle.x = 0.0;
                particle.vx = f32::abs(particle.vx);
            }
            if particle.y < 0.0 {
                particle.y = 0.0;
                particle.vy = f32::abs(particle.vy);
            }
            if particle.x > MATRIX_WIDTH as f32 - 1.0 {
                particle.x = MATRIX_WIDTH as f32 - 1.0;
                particle.vx = -f32::abs(particle.vx);
            }
            if particle.y > MATRIX_HEIGHT as f32 - 1.0 {
                particle.y = MATRIX_HEIGHT as f32 - 1.0;
                particle.vy = -f32::abs(particle.vy);
            }
            particle.x += particle.vx * dt;
            particle.y += particle.vy * dt;
        });

        matrix.iter_mut().for_each(|(x, y, color)| {
            let x = x as f32;
            let y = y as f32;

            *color = self
                .particles
                .iter()
                .map(|particle| {
                    let dist = particle.dist(x, y);
                    let amount = 1.0 - simple_ease((dist / 1.5).clamp(0.0, 1.0));
                    let t = now.duration_since(particle.start);
                    let amount = if t.as_secs_f32() <= PARTICLE_DECAY_OFFSET {
                        amount
                    } else {
                        amount
                            * (1.0
                                - ((t.as_secs_f32() - PARTICLE_DECAY_OFFSET)
                                    / PARTICLE_DECAY_DURATION))
                    };
                    particle.color * amount
                })
                .fold(Color::new(0.0, 0.0, 0.0), |acc, col| acc + col)
        });

        matrix.send_update()?;

        Ok(())
    }
}
