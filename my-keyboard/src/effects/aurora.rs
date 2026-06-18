use super::{Effect, MatrixInput};
use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom};

const TIME_SCALE: f32 = 1.2;
const WAVE_FREQUENCY: f32 = 0.15;

#[derive(Debug)]
struct AuroraImpulse {
    x: f32,
    y: f32,
    intensity: f32,
}

#[derive(Debug)]
pub struct EffectAurora {
    start_time: std::time::Instant,
    impulses: Vec<AuroraImpulse>,
}

impl EffectAurora {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            impulses: Vec::new(),
        }
    }
}

impl Effect for EffectAurora {
    fn identifier(&self) -> &str {
        "effect_aurora"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let elapsed = self.start_time.elapsed().as_secs_f32() * TIME_SCALE;

        for input in inputs {
            if let MatrixInput::Pressed { x, y } = input {
                self.impulses.push(AuroraImpulse {
                    x: *x as f32,
                    y: *y as f32,
                    intensity: 1.0,
                });
            }
        }

        self.impulses.iter_mut().for_each(|imp| {
            imp.intensity -= 0.02;
        });
        self.impulses.retain(|imp| imp.intensity > 0.0);

        matrix.iter_mut().for_each(|(x, y, color)| {
            let x_f = x as f32;
            let y_f = y as f32;

            let wave_1 = f32::sin(x_f * WAVE_FREQUENCY + elapsed) * 2.0;
            let wave_2 = f32::cos(y_f * 0.3 - elapsed * 0.7) * 1.5;
            let wave_3 = f32::sin((x_f + y_f) * 0.1 + elapsed * 0.5);

            let curtain = wave_1 + wave_2 + wave_3;

            let hue = 120.0 + (curtain * 25.0) + 40.0;

            let mut brightness = (f32::sin(curtain) * 0.5 + 0.5).powi(3) * 0.6;

            let mut impulse_contribution = 0.0;
            let mut impulse_hue_shift = 0.0;

            for imp in &self.impulses {
                let dist = f32::sqrt((x_f - imp.x).powi(2) + (y_f - imp.y).powi(2));
                if dist < 4.0 {
                    let falloff = (1.0 - (dist / 4.0)).clamp(0.0, 1.0);
                    impulse_contribution += falloff * imp.intensity * 0.4;
                    impulse_hue_shift += falloff * imp.intensity * 30.0;
                }
            }

            brightness = (brightness + impulse_contribution).clamp(0.0, 1.0);

            let final_hue = (hue + impulse_hue_shift) % 360.0;
            *color = Color::from_hsl(final_hue, 0.9, brightness);
        });

        matrix.send_update()?;
        Ok(())
    }
}
