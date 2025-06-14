use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom};

use super::Effect;

pub struct EffectRainbow1<'a, 'b> {
    matrix: &'b mut DeviceMatrixCustom<'a>,
    start: std::time::Instant,
}

impl<'a, 'b> Effect<'a, 'b> for EffectRainbow1<'a, 'b> {
    fn attach(matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<Self, Error> {
        Ok(Self {
            matrix,
            start: std::time::Instant::now(),
        })
    }

    fn update(&mut self) -> Result<(), Error> {
        let time = std::time::Instant::now().duration_since(self.start);

        let hue_rot = time.as_secs_f32() * 100.0;
        self.matrix.iter_mut().for_each(|(_x, _y, color)| {
            *color = Color::from_hsl(hue_rot, 1.0, 0.5);
        });

        self.matrix.send_update()?;
        Ok(())
    }
}
