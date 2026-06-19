use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use openrazer::{query_razer_devices, EventSummary, KeyCode};

use crate::{
    effect_finder::EffectFinder, lua::KeyboardLuaRunner,
    matrix_mapper::create_default_matrix_mapper,
};

mod effect_finder;
mod lua;
mod matrix_mapper;
mod util;

const DEFAULT_FPS: u64 = 20;
const DEFAULT_EFFECT_CYCLE_TIME: std::time::Duration = std::time::Duration::from_secs(60 * 5);

const NEXT_EFFECT_KEY: Option<KeyCode> = Some(KeyCode::KEY_PAUSE);

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Effects directory path or file path
    #[arg(short, long)]
    path: Option<PathBuf>,
    /// Watch for file changes
    #[arg(short, long, default_value_t = false)]
    watch: bool,
    /// If to not listen for keyboard events
    #[arg(short, long, default_value_t = false)]
    no_key_events: bool,
    #[arg(short, long, default_value_t = DEFAULT_FPS)]
    fps: u64,
    #[arg(short, long)]
    cycle_time_secs: Option<u64>,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let path = cli
        .path
        .unwrap_or(std::env::current_dir()?)
        .canonicalize()?;
    log::info!("Effects Path: {path:?}");

    let update_rate = std::time::Duration::from_secs_f64(1.0 / (cli.fps as f64));

    let cycle_time = cli
        .cycle_time_secs
        .map(std::time::Duration::from_secs)
        .unwrap_or(DEFAULT_EFFECT_CYCLE_TIME);

    let device = query_razer_devices()?
        .into_iter()
        .next()
        .expect("No razer device found.");

    let Some(mut matrix_manager) = device.get_matrix_manager()? else {
        panic!("Razer device has no custom lighting.");
    };

    let mut matrix = matrix_manager.effect_custom()?;

    let matrix_mapper = create_default_matrix_mapper();

    let mut evdev_device = if cli.no_key_events {
        None
    } else {
        device.get_evdev_device()?
    };

    let mut finder = EffectFinder::new(&path, cli.watch)?;

    let mut lua_runner = KeyboardLuaRunner::new_with_code_path(finder.next_effect()?)?;

    let mut cycle_next_effect_time = std::time::Instant::now() + cycle_time;

    let term = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term));

    loop {
        let next_frame_time = std::time::Instant::now() + update_rate;

        if term.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        lua_runner.step_start()?;

        if let Some(evdev_device) = evdev_device.as_mut() {
            while let Some(event) = evdev_device.try_next()? {
                #[allow(clippy::collapsible_if)]
                if let EventSummary::Key(_, key, r#type) = event.destructure() {
                    if Some(key) == NEXT_EFFECT_KEY && r#type == 1 {
                        cycle_next_effect_time = std::time::Instant::now();
                    }
                    if let Some((x, y)) = matrix_mapper.map(key) {
                        lua_runner.key_event(
                            match r#type {
                                0 => lua::KeyboardLuaRunnerKeyEventType::Release,
                                1 => lua::KeyboardLuaRunnerKeyEventType::Press,
                                2 => lua::KeyboardLuaRunnerKeyEventType::Repeat,
                                _ => panic!(),
                            },
                            x,
                            y,
                        )?;
                    }
                }
            }
        }

        lua_runner.matrix_update(&mut matrix)?;

        lua_runner.step_end()?;

        if std::time::Instant::now() >= cycle_next_effect_time || finder.dir_changed()? {
            cycle_next_effect_time = std::time::Instant::now() + cycle_time;
            lua_runner = KeyboardLuaRunner::new_with_code_path(finder.next_effect()?)?;
        }

        if util::sleep_until(next_frame_time) == std::time::Duration::ZERO {
            log::warn!("Failed to update effect in time");
        }
    }

    log::info!("Exiting my-keyboard, setting keyboard matrix to spectrum");
    matrix_manager.effect_spectrum()?;

    Ok(())
}
