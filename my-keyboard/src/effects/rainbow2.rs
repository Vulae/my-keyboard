use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom, MATRIX_WIDTH};

use super::Effect;

pub struct EffectRainbow2<'a, 'b> {
    matrix: &'b mut DeviceMatrixCustom<'a>,
    start: std::time::Instant,
}

impl<'a, 'b> Effect<'a, 'b> for EffectRainbow2<'a, 'b> {
    fn attach(matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<Self, Error> {
        Ok(Self {
            matrix,
            start: std::time::Instant::now(),
        })
    }

    fn update(&mut self) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        let hue_rot = time.as_secs_f32() * 100.0;
        self.matrix.iter_mut().for_each(|(x, y, color)| {
            let hue = (x as f32 / MATRIX_WIDTH as f32) * 360.0;
            let hue = if y % 2 == 0 { hue } else { -hue };
            *color = Color::from_hsl(hue + hue_rot, 1.0, 0.5);
        });

        self.matrix.send_update()?;
        Ok(())
    }
}
