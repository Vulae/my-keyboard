use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Error;
use cycler::EffectCycler;
use openrazer::query_razer_devices;

pub mod cycler;
mod effects;

const TARGET_FPS: u64 = 15;
const TARGET_UPDATE_RATE: std::time::Duration = std::time::Duration::from_millis(1000 / TARGET_FPS);

const EFFECT_CHANGE_TIME: std::time::Duration = std::time::Duration::from_secs(60 * 5);

pub fn main() -> Result<(), Error> {
    env_logger::init();

    let mut device = query_razer_devices()?
        .into_iter()
        .next()
        .expect("No razer device found.");

    let matrix_manager = device.matrix_manager();

    let mut effect_cycler = EffectCycler::new(matrix_manager.effect_custom()?);
    effects::add_effects_to_cycler(&mut effect_cycler);

    let mut cycle_next_effect_time = std::time::Instant::now();

    let term = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term));

    loop {
        let next_frame_time = std::time::Instant::now() + TARGET_UPDATE_RATE;

        if term.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        if std::time::Instant::now() >= cycle_next_effect_time {
            cycle_next_effect_time = std::time::Instant::now() + EFFECT_CHANGE_TIME;

            effect_cycler.next_effect();
            log::info!(
                "Playing effect: {:?}",
                effect_cycler.current_effect_identifier(),
            );
        }

        effect_cycler.update()?;

        let mut waited = false;
        while std::time::Instant::now() < next_frame_time {
            std::thread::sleep(std::time::Duration::from_nanos(100));
            waited = true;
        }
        if !waited {
            log::warn!("Failed to update effect in time");
        }
    }

    log::info!("Exiting my-keyboard, setting keyboard matrix to spectrum");
    matrix_manager.effect_spectrum()?;

    Ok(())
}
