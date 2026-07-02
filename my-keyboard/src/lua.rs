use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use mlua::{FromLua, Function, Lua, UserData};
use openrazer::{Color, DeviceMatrixCustom, MATRIX_HEIGHT, MATRIX_WIDTH};

#[derive(Default, UserData)]
struct Log;

#[mlua::userdata_impl]
impl Log {
    #[lua(infallible)]
    fn debug(message: &str) {
        log::debug!("{message}");
    }
    #[lua(infallible)]
    fn info(message: &str) {
        log::info!("{message}");
    }
    #[lua(infallible)]
    fn warn(message: &str) {
        log::warn!("{message}");
    }
    #[lua(infallible)]
    fn error(message: &str) {
        log::error!("{message}");
    }
}

#[derive(Debug, Clone, Copy, UserData)]
#[allow(clippy::upper_case_acronyms)]
struct RGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl RGB {
    pub fn as_openrazer_color(&self) -> Color {
        Color::new(self.r as f32, self.g as f32, self.b as f32)
    }
}

#[mlua::userdata_impl]
impl RGB {
    #[lua(infallible)]
    fn new(r: f64, g: f64, b: f64) -> RGB {
        RGB { r, g, b }
    }

    #[lua(meta, infallible)]
    fn __call(r: f64, g: f64, b: f64) -> RGB {
        RGB::new(r, g, b)
    }

    #[lua(meta, infallible)]
    fn __add(&self, other: &RGB) -> RGB {
        RGB {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }

    #[lua(meta, infallible)]
    fn __sub(&self, other: &RGB) -> RGB {
        RGB {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }

    #[lua(meta, infallible)]
    fn __mul(&self, k: f64) -> RGB {
        RGB {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
        }
    }

    #[lua(meta, infallible)]
    fn __div(&self, k: f64) -> RGB {
        RGB {
            r: self.r / k,
            g: self.g / k,
            b: self.b / k,
        }
    }

    #[lua(infallible)]
    fn clamp01(&self) -> RGB {
        RGB {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
        }
    }

    #[lua(infallible)]
    fn lerp(a: &RGB, b: &RGB, t: f64) -> RGB {
        fn lerp(a: f64, b: f64, t: f64) -> f64 {
            a * (1.0 - t) + b * t
        }
        RGB {
            r: lerp(a.r, b.r, t),
            g: lerp(a.g, b.g, t),
            b: lerp(a.b, b.b, t),
        }
    }

    #[lua]
    fn from_hex(hex: &str) -> mlua::Result<RGB> {
        fn inner(hex: &str) -> Option<RGB> {
            let hex = hex.strip_prefix('#').unwrap_or(hex);
            match hex.chars().collect::<Box<[char]>>().as_ref() {
                [c_r, c_g, c_b] | [c_r, c_g, c_b, _] => Some(RGB::new(
                    (c_r.to_digit(16)? as f64) / 15.0,
                    (c_g.to_digit(16)? as f64) / 15.0,
                    (c_b.to_digit(16)? as f64) / 15.0,
                )),
                [c1_r, c2_r, c1_g, c2_g, c1_b, c2_b]
                | [c1_r, c2_r, c1_g, c2_g, c1_b, c2_b, _, _] => Some(RGB::new(
                    (((c1_r.to_digit(16)? << 4) | c2_r.to_digit(16)?) as f64) / 255.0,
                    (((c1_g.to_digit(16)? << 4) | c2_g.to_digit(16)?) as f64) / 255.0,
                    (((c1_b.to_digit(16)? << 4) | c2_b.to_digit(16)?) as f64) / 255.0,
                )),
                _ => None,
            }
        }
        inner(hex).ok_or_else(|| mlua::Error::FromLuaConversionError {
            from: "String",
            to: "RGB".to_owned(),
            message: Some(format!("Could not convert hex RGB string \"{hex}\"")),
        })
    }

    #[lua(infallible)]
    fn from_hsl(h: f64, s: f64, l: f64) -> RGB {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            300.0..=360.0 => (c, 0.0, x),
            _ => unreachable!(),
        };

        RGB {
            r: r + m,
            g: g + m,
            b: b + m,
        }
    }
}

impl FromLua for RGB {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let mlua::Value::UserData(data) = value else {
            return Err(mlua::Error::FromLuaConversionError {
                from: "LuaMultiValues",
                to: "RGB".to_owned(),
                message: None,
            });
        };
        data.take()
    }
}

#[derive(Default)]
struct Keyboard {
    #[allow(clippy::type_complexity)]
    on_recieve_key_callbackfn: Option<Box<dyn Fn(String, usize, usize) -> mlua::Result<()>>>,
    #[allow(clippy::type_complexity)]
    on_matrix_before_frame: Option<Box<dyn Fn() -> mlua::Result<()>>>,
    #[allow(clippy::type_complexity)]
    on_matrix_update_callbackfn: Option<Box<dyn Fn(usize, usize) -> mlua::Result<Option<RGB>>>>,
}

impl std::fmt::Debug for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyboard").finish()
    }
}

#[derive(UserData, Default)]
struct KeyboardWrapper {
    #[lua(skip)]
    pub inner: Arc<Mutex<Keyboard>>,
}

#[mlua::userdata_impl]
impl KeyboardWrapper {
    const WIDTH: usize = MATRIX_WIDTH;
    const HEIGHT: usize = MATRIX_HEIGHT;

    #[lua(infallible)]
    fn on_recieve_key(&mut self, callbackfn: Function) {
        self.inner.lock().unwrap().on_recieve_key_callbackfn =
            Some(Box::new(move |r#type, x, y| {
                callbackfn.call::<()>((r#type, x, y))
            }));
    }

    #[lua(infallible)]
    fn on_matrix_before_frame(&mut self, callbackfn: Function) {
        self.inner.lock().unwrap().on_matrix_before_frame =
            Some(Box::new(move || callbackfn.call::<()>(())));
    }

    #[lua(infallible)]
    fn on_matrix_update(&mut self, callbackfn: Function) {
        self.inner.lock().unwrap().on_matrix_update_callbackfn =
            Some(Box::new(move |x, y| callbackfn.call::<Option<RGB>>((x, y))));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyboardLuaRunnerKeyEventType {
    Release,
    Press,
    Repeat,
}

impl KeyboardLuaRunnerKeyEventType {
    fn lua_str(&self) -> String {
        match self {
            KeyboardLuaRunnerKeyEventType::Release => "release",
            KeyboardLuaRunnerKeyEventType::Press => "press",
            KeyboardLuaRunnerKeyEventType::Repeat => "repeat",
        }
        .to_owned()
    }
}

#[derive(Debug)]
pub struct KeyboardLuaRunner {
    start: std::time::Instant,
    code_path: PathBuf,
    lua: Lua,
    keyboard: Arc<Mutex<Keyboard>>,
    pub err: Option<mlua::Error>,
}

impl KeyboardLuaRunner {
    // 10MiB
    const MEMORY_LIMIT: usize = 1024 * 1024 * 10;

    pub fn code_path(&self) -> &PathBuf {
        &self.code_path
    }

    pub fn new_with_code_path<P: AsRef<Path>>(path: P) -> Result<KeyboardLuaRunner> {
        let lua = Lua::new_with(
            mlua::StdLib::MATH | mlua::StdLib::UTF8 | mlua::StdLib::TABLE | mlua::StdLib::STRING,
            mlua::LuaOptions::default(),
        )?;
        lua.set_memory_limit(KeyboardLuaRunner::MEMORY_LIMIT)?;

        lua.globals().set("Log", lua.create_proxy::<Log>()?)?;
        lua.globals().set("RGB", lua.create_proxy::<RGB>()?)?;

        let start = std::time::Instant::now();
        lua.globals().set(
            "curtime",
            std::time::Instant::now()
                .duration_since(start)
                .as_secs_f64(),
        )?;

        let keyboard_wrapper = KeyboardWrapper::default();
        let keyboard = keyboard_wrapper.inner.clone();
        lua.globals().set("Keyboard", keyboard_wrapper)?;

        let code_path = path.as_ref().to_path_buf();
        log::info!("KeyboardLuaRunner lua script: {code_path:#?}");
        let err = match lua.load(code_path.clone()).exec() {
            Ok(_) => None,
            Err(err) => {
                log::error!("{err}");
                Some(err)
            }
        };

        Ok(KeyboardLuaRunner {
            start,
            code_path,
            lua,
            keyboard,
            err,
        })
    }

    pub fn step_start(&mut self) -> Result<()> {
        if self.err.is_some() {
            return Ok(());
        }
        self.lua.globals().set(
            "curtime",
            std::time::Instant::now()
                .duration_since(self.start)
                .as_secs_f64(),
        )?;
        Ok(())
    }

    pub fn key_event(
        &mut self,
        r#type: KeyboardLuaRunnerKeyEventType,
        x: usize,
        y: usize,
    ) -> Result<()> {
        if self.err.is_some() {
            return Ok(());
        }
        if let Some(callbackfn) = &self.keyboard.lock().unwrap().on_recieve_key_callbackfn {
            match callbackfn(r#type.lua_str(), x, y) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{err}");
                    self.err = Some(err);
                }
            };
        }
        Ok(())
    }

    pub fn matrix_update(&mut self, matrix: &mut DeviceMatrixCustom<'_>) -> Result<()> {
        if self.err.is_some() {
            return Ok(());
        }
        if let Some(callbackfn) = &self.keyboard.lock().unwrap().on_matrix_before_frame {
            match callbackfn() {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{err}");
                    self.err = Some(err);
                }
            };
        }
        if self.err.is_some() {
            return Ok(());
        }
        if let Some(callbackfn) = &self.keyboard.lock().unwrap().on_matrix_update_callbackfn {
            for (x, y, c) in matrix.iter_mut() {
                match callbackfn(x, y) {
                    Ok(None) => {}
                    Ok(Some(color)) => *c = color.as_openrazer_color(),
                    Err(err) => {
                        log::error!("{err}");
                        self.err = Some(err);
                        break;
                    }
                }
            }
            matrix.send_update()?;
        }
        Ok(())
    }

    pub fn step_end(&mut self) -> Result<()> {
        if self.err.is_some() {
            return Ok(());
        }
        self.lua.gc_step()?;
        Ok(())
    }
}
