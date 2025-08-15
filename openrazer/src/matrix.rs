use std::{fs::File, io::Write, path::PathBuf};

use crate::{Color, OpenRazerError};

#[derive(Debug)]
pub struct DeviceMatrixEffectManager {
    path: PathBuf,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum EffectWaveDirection {
    #[default]
    Left,
    Right,
}

impl DeviceMatrixEffectManager {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn subpath(&self, file: &str) -> PathBuf {
        let mut path = self.path.clone();
        path.push(file);
        path
    }

    fn write_to(&self, file: &str, data: &[u8]) -> Result<(), OpenRazerError> {
        std::fs::write(self.subpath(file), data)?;
        Ok(())
    }

    pub fn get_brightness(&self) -> Result<u8, OpenRazerError> {
        let data = std::fs::read_to_string(self.subpath("matrix_brightness"))?;
        data.trim()
            .parse::<u8>()
            .map_err(|_| OpenRazerError::MatrixEffectBrightnessParseError)
    }

    /// Matrix may have transition time.
    pub fn set_brightness(&self, brightness: u8) -> Result<(), OpenRazerError> {
        self.write_to("matrix_brightness", brightness.to_string().as_bytes())
    }

    /// Matrix may have transition time.
    pub fn effect_none(&self) -> Result<(), OpenRazerError> {
        self.write_to("matrix_effect_none", &[0])
    }

    /// Matrix may have transition time.
    pub fn effect_static(&self, color: Color) -> Result<(), OpenRazerError> {
        self.write_to("matrix_effect_static", &color.to_quantized())
    }

    /// Matrix may have transition time.
    pub fn effect_spectrum(&self) -> Result<(), OpenRazerError> {
        self.write_to("matrix_effect_spectrum", &[0])
    }

    /// Matrix may have transition time.
    pub fn effect_wave(&self, direction: EffectWaveDirection) -> Result<(), OpenRazerError> {
        self.write_to(
            "matrix_effect_spectrum",
            &[match direction {
                EffectWaveDirection::Left => 2,
                EffectWaveDirection::Right => 1,
            }],
        )
    }

    /// Display a custom matrix frame.
    pub fn effect_custom(&mut self) -> Result<DeviceMatrixCustom<'_>, OpenRazerError> {
        DeviceMatrixCustom::new(self)
    }
}

pub const MATRIX_WIDTH: usize = 22;
pub const MATRIX_HEIGHT: usize = 6;
const MATRIX_SIZE: usize = MATRIX_WIDTH * MATRIX_HEIGHT;

#[derive(Debug)]
pub struct DeviceMatrixCustom<'a> {
    #[allow(unused)]
    matrix_manager: &'a mut DeviceMatrixEffectManager,

    file_matrix: File,
    file_update: File,

    matrix: [Color; MATRIX_SIZE],
}

impl<'a> DeviceMatrixCustom<'a> {
    fn new(matrix_manager: &'a mut DeviceMatrixEffectManager) -> Result<Self, OpenRazerError> {
        Ok(Self {
            file_matrix: std::fs::File::options()
                .append(true)
                .open(matrix_manager.subpath("matrix_custom_frame"))?,
            file_update: std::fs::File::options()
                .append(true)
                .open(matrix_manager.subpath("matrix_effect_custom"))?,
            matrix_manager,
            matrix: [Color::new(0.0, 0.0, 0.0); MATRIX_SIZE],
        })
    }
}

impl DeviceMatrixCustom<'_> {
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
            return None;
        }
        Some(x + y * MATRIX_WIDTH)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Color> {
        self.matrix.get(self.index(x, y)?)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Color> {
        let index = self.index(x, y)?;
        self.matrix.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &Color)> {
        self.matrix
            .iter()
            .enumerate()
            .map(|(i, c)| (i % MATRIX_WIDTH, i / MATRIX_WIDTH, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut Color)> {
        self.matrix
            .iter_mut()
            .enumerate()
            .map(|(i, c)| (i % MATRIX_WIDTH, i / MATRIX_WIDTH, c))
    }

    pub fn send_update(&mut self) -> Result<(), OpenRazerError> {
        // There are 2 streams:
        // - 'matrix_effect_custom': Update matrix with new frame data.
        // - 'matrix_custom_frame': Frame data for a packet.
        //
        // 'matrix_effect_custom' has any byte written to it when to display the new frame.
        //
        // 'matrix_custom_frame' has packets of data:
        // row: u8
        // col_start: u8
        // col_end: u8
        // colors: (u8, u8, u8)[col_end - col_start + 1]
        //
        // NOTE:
        //     I have tried to send less data by only sending the colors that have had a visually
        //     perceptive change, but atleast for my keyboard the start_col must always be 0.

        let mut data = Vec::new();

        for y in 0..MATRIX_HEIGHT {
            data.write_all(&[y as u8, 0, MATRIX_WIDTH as u8 - 1])?;
            for x in 0..MATRIX_WIDTH {
                let color = self.matrix[x + y * MATRIX_WIDTH];
                data.write_all(&color.to_quantized())?;
            }
        }

        if !data.is_empty() {
            self.file_matrix.write_all(&data)?;
            self.file_matrix.flush()?;
            self.file_update.write_all(&[1])?;
            self.file_update.flush()?;
        }

        Ok(())
    }
}
