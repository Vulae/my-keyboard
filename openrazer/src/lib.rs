use thiserror::Error;

mod color;
mod device;
mod matrix;

pub use color::*;
pub use device::*;
pub use matrix::*;

#[derive(Debug, Error)]
pub enum OpenRazerError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse matrix effect brightness")]
    MatrixEffectBrightnessParseError,
}
