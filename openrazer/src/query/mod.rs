//! Query devices

mod devices;

pub use devices::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Malformed query result: {0}")]
    Malformed(&'static str),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
