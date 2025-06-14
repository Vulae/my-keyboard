use anyhow::Error;
use openrazer::DeviceMatrixCustom;

pub trait Effect<'a, 'b> {
    fn attach(matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<Self, Error>
    where
        Self: Sized;
    fn update(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

mod frozen;
mod pride;
mod rainbow1;
mod rainbow2;
mod rainbow3;

pub use frozen::EffectFrozen;
pub use pride::EffectPride;
pub use rainbow1::EffectRainbow1;
pub use rainbow2::EffectRainbow2;
pub use rainbow3::EffectRainbow3;
