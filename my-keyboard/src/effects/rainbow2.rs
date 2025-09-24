use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_WIDTH};

use super::{Effect, MatrixInput};

#[derive(Debug)]
pub struct EffectRainbow2 {
    start: std::time::Instant,
}

impl EffectRainbow2 {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Effect for EffectRainbow2 {
    fn identifier(&self) -> &str {
        "effect_rainbow_2"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        _inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        let hue_rot = time.as_secs_f32() * 100.0;
        matrix.iter_mut().for_each(|(x, y, color)| {
            let hue = (x as f32 / MATRIX_WIDTH as f32) * 360.0;
            let hue = if y % 2 == 0 { hue } else { -hue };
            *color = Color::from_hsl(hue + hue_rot, 1.0, 0.5);
        });

        matrix.send_update()?;
        Ok(())
    }
}
