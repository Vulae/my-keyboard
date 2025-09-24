use anyhow::{anyhow, Error};
use openrazer::DeviceMatrixCustom;

use crate::effects::{Effect, MatrixInput};

#[derive(Debug)]
pub struct EffectCycler<'a> {
    matrix: DeviceMatrixCustom<'a>,
    effects: Vec<Box<dyn Effect>>,
    effect_index: Option<usize>,
}

impl<'a> EffectCycler<'a> {
    pub fn new(matrix: DeviceMatrixCustom<'a>) -> Self {
        Self {
            matrix,
            effects: Vec::new(),
            effect_index: None,
        }
    }

    pub fn add_effect(&mut self, effect: impl Effect + 'static) {
        if self
            .effects
            .iter()
            .any(|e| e.identifier() == effect.identifier())
        {
            return;
        }
        self.effects.push(Box::new(effect));
    }

    pub fn update(&mut self, events: &[MatrixInput]) -> Result<(), Error> {
        let Some(effect_index) = self.effect_index else {
            return Err(anyhow!("Invalid selected effect"));
        };
        let effect = self
            .effects
            .get_mut(effect_index)
            .expect("Invalid effect index");
        effect.update(&mut self.matrix, events)?;
        Ok(())
    }

    pub fn next_effect(&mut self) {
        assert!(!self.effects.is_empty());
        loop {
            let effect_index = Some(rand::random_range(0..self.effects.len()));
            if effect_index != self.effect_index || self.effects.len() == 1 {
                self.effect_index = effect_index;
                break;
            }
        }
    }

    pub fn set_no_effect(&mut self) {
        self.effect_index = None;
    }

    pub fn set_effect(&mut self, effect_identifier: &str) -> bool {
        let Some(effect_index) = self
            .effects
            .iter()
            .enumerate()
            .find_map(|(i, effect)| (effect.identifier() == effect_identifier).then_some(i))
        else {
            return false;
        };
        self.effect_index = Some(effect_index);
        true
    }

    pub fn iter_effect_identifiers(&self) -> impl Iterator<Item = &str> {
        self.effects.iter().map(|effect| effect.identifier())
    }

    pub fn current_effect_identifier(&self) -> Option<&str> {
        Some(
            self.effects
                .get(self.effect_index?)
                .expect("Invalid effect index")
                .identifier(),
        )
    }
}
