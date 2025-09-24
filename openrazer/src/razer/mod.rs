//! Manage Razer devices & their lighting
use thiserror::Error;

mod device;
mod matrix;

pub use device::*;
pub use matrix::*;

use crate::QueryError;

pub const RAZER_DEVICE_VENDOR_ID: u16 = 0x1532;

#[derive(Debug, Error)]
pub enum OpenRazerError {
    #[error("Failed to parse matrix effect brightness")]
    MatrixEffectBrightnessParseError,
    #[error(transparent)]
    QueryError(#[from] QueryError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
