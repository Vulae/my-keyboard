use std::fmt::Debug;

use anyhow::Error;
use openrazer::DeviceMatrixCustom;

use crate::cycler::EffectCycler;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixInput {
    Pressed { x: usize, y: usize },
    Released { x: usize, y: usize },
    Repeat { x: usize, y: usize },
}

impl MatrixInput {
    pub fn x(&self) -> usize {
        match self {
            MatrixInput::Pressed { x, .. } => *x,
            MatrixInput::Released { x, .. } => *x,
            MatrixInput::Repeat { x, .. } => *x,
        }
    }

    pub fn y(&self) -> usize {
        match self {
            MatrixInput::Pressed { y, .. } => *y,
            MatrixInput::Released { y, .. } => *y,
            MatrixInput::Repeat { y, .. } => *y,
        }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.x(), self.y())
    }
}

pub trait Effect: Debug {
    fn identifier(&self) -> &str;

    #[allow(unused)]
    fn update<'a, 'b>(
        &mut self,
        matrix: &'b mut DeviceMatrixCustom<'a>,
        inputs: &[MatrixInput],
    ) -> Result<(), Error> {
        Ok(())
    }
}

mod line;
mod particles;
mod pride;
mod rainbow1;
mod rainbow2;
mod rainbow3;
mod random;
mod ripple;

pub use line::EffectLine;
pub use particles::EffectParticles;
pub use pride::EffectPride;
pub use rainbow1::EffectRainbow1;
pub use rainbow2::EffectRainbow2;
pub use rainbow3::EffectRainbow3;
pub use random::EffectRandom;
pub use ripple::EffectRipple;

pub fn add_effects_to_cycler(effect_cycler: &mut EffectCycler<'_>) {
    effect_cycler.add_effect(|| Box::new(EffectRainbow1::new()));
    effect_cycler.add_effect(|| Box::new(EffectRainbow2::new()));
    effect_cycler.add_effect(|| Box::new(EffectRainbow3::new()));
    effect_cycler.add_effect(|| Box::new(EffectPride::new()));
    effect_cycler.add_effect(|| Box::new(EffectRandom::new()));
    effect_cycler.add_effect(|| Box::new(EffectRipple::new()));
    effect_cycler.add_effect(|| Box::new(EffectLine::new()));
    effect_cycler.add_effect(|| Box::new(EffectParticles::new()));
}
