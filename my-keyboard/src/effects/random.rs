use anyhow::Error;
use openrazer::{Color, DeviceMatrixCustom};

use super::{Effect, MatrixInput};

#[derive(Debug)]
pub struct EffectRandom {
    initialized: bool,
}

impl EffectRandom {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}

impl Effect for EffectRandom {
    fn identifier(&self) -> &str {
        "effect_random"
    }

    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        let mut updated: bool = false;

        if !self.initialized {
            matrix.iter_mut().for_each(|(_x, _y, color)| {
                *color = Color::from_hsl(rand::random::<f32>() * 360.0, 1.0, 0.5);
            });
            self.initialized = true;
            updated = true;
        }

        for input in inputs {
            if let Some(color) = matrix.get_mut(input.x(), input.y()) {
                *color = Color::from_hsl(rand::random::<f32>() * 360.0, 1.0, 0.5);
                updated = true;
            }
        }

        if updated {
            matrix.send_update()?;
        }

        Ok(())
    }
}
