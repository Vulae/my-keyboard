use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom};

use super::{Effect, MatrixInput};

#[derive(Debug)]
pub struct EffectRainbow1 {
    start: std::time::Instant,
}

impl EffectRainbow1 {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Effect for EffectRainbow1 {
    fn identifier(&self) -> &str {
        "effect_rainbow_1"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        _inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        let hue_rot = time.as_secs_f32() * 100.0;
        matrix.iter_mut().for_each(|(_x, _y, color)| {
            *color = Color::from_hsl(hue_rot, 1.0, 0.5);
        });

        matrix.send_update()?;
        Ok(())
    }
}
