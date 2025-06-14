use anyhow::Error;
use openrazer::DeviceMatrixCustom;

use super::Effect;

pub struct EffectFrozen<'a, 'b> {
    #[allow(unused)]
    matrix: &'b mut DeviceMatrixCustom<'a>,
}

impl<'a, 'b> Effect<'a, 'b> for EffectFrozen<'a, 'b> {
    fn attach(matrix: &'b mut DeviceMatrixCustom<'a>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self { matrix })
    }
}
