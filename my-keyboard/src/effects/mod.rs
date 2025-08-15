use std::fmt::Debug;

use anyhow::Error;
use openrazer::DeviceMatrixCustom;

use crate::cycler::EffectCycler;

pub trait Effect: Debug {
    fn identifier(&self) -> &str;

    #[allow(unused)]
    fn update<'a, 'b>(&mut self, matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<(), Error> {
        Ok(())
    }
}

mod pride;
mod rainbow1;
mod rainbow2;
mod rainbow3;

pub use pride::EffectPride;
pub use rainbow1::EffectRainbow1;
pub use rainbow2::EffectRainbow2;
pub use rainbow3::EffectRainbow3;

pub fn add_effects_to_cycler(effect_cycler: &mut EffectCycler<'_>) {
    effect_cycler.add_effect(EffectRainbow1::new());
    effect_cycler.add_effect(EffectRainbow2::new());
    effect_cycler.add_effect(EffectRainbow3::new());
    effect_cycler.add_effect(EffectPride::new());
}
