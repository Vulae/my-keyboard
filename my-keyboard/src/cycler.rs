use anyhow::Error;
use openrazer::DeviceMatrixCustom;

use crate::effects::{Effect, MatrixInput};

pub struct EffectCycler<'a> {
    matrix: DeviceMatrixCustom<'a>,
    effect: Option<Box<dyn Effect>>,
    effect_creators: Vec<Box<dyn Fn() -> Box<dyn Effect>>>,
}

impl<'a> std::fmt::Debug for EffectCycler<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectCycler")
            .field("matrix", &self.matrix)
            .field("effect", &self.effect)
            // .field("effect_creators", &self.effect_creators)
            .finish()
    }
}

impl<'a> EffectCycler<'a> {
    pub fn new(matrix: DeviceMatrixCustom<'a>) -> Self {
        Self {
            matrix,
            effect: None,
            effect_creators: Vec::new(),
        }
    }

    pub fn add_effect<F>(&mut self, creator: F)
    where
        F: Fn() -> Box<dyn Effect> + 'static,
    {
        self.effect_creators.push(Box::new(creator));
    }

    pub fn update(&mut self, events: &[MatrixInput]) -> Result<(), Error> {
        if let Some(effect) = self.effect.as_mut() {
            effect.update(&mut self.matrix, events)?;
        }
        Ok(())
    }

    pub fn next_effect(&mut self) {
        assert!(!self.effect_creators.is_empty());
        let ident = self
            .effect
            .as_ref()
            .map(|effect| effect.identifier().to_owned());
        loop {
            let creator = &self.effect_creators[rand::random_range(0..self.effect_creators.len())];
            self.effect = Some(creator());
            if self.effect_creators.len() == 1
                || self.effect.as_ref().map(|effect| effect.identifier())
                    != ident.as_ref().map(|s| s.as_str())
            {
                break;
            }
        }
    }

    pub fn set_no_effect(&mut self) {
        self.effect = None;
    }

    pub fn set_effect(&mut self, effect_identifier: &str) -> bool {
        for creator in self.effect_creators.iter() {
            let effect = creator();
            if effect.identifier() == effect_identifier {
                self.effect = Some(effect);
                return true;
            }
        }
        false
    }

    pub fn current_effect_identifier(&self) -> Option<&str> {
        self.effect.as_ref().map(|effect| effect.identifier())
    }
}
