use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Error;
use cycler::EffectCycler;
use effects::MatrixInput;
use openrazer::{query_razer_devices, EventSummary, KeyCode, MatrixMapper};

pub mod cycler;
mod effects;
pub mod util;

const TARGET_FPS: u64 = 15;
const TARGET_UPDATE_RATE: std::time::Duration = std::time::Duration::from_millis(1000 / TARGET_FPS);

const EFFECT_CHANGE_TIME: std::time::Duration = std::time::Duration::from_secs(60 * 5);

const FORCED_EFFECT: Option<&str> = None;

pub fn main() -> Result<(), Error> {
    env_logger::init();

    let device = query_razer_devices()?
        .into_iter()
        .next()
        .expect("No Razer device found.");

    let Some(mut matrix_manager) = device.get_matrix_manager()? else {
        panic!("Razer device has no custom lighting.");
    };

    let mut matrix_mapper = MatrixMapper::default();
    // NOTE: This is the mapping for my keyboard (Razer Ornata Chroma)
    matrix_mapper.add_mappings([
        // Other
        (KeyCode::KEY_ESC, (1, 0)),
        // Function keys
        (KeyCode::KEY_F1, (3, 0)),
        (KeyCode::KEY_F2, (4, 0)),
        (KeyCode::KEY_F3, (5, 0)),
        (KeyCode::KEY_F4, (6, 0)),
        (KeyCode::KEY_F5, (7, 0)),
        (KeyCode::KEY_F6, (8, 0)),
        (KeyCode::KEY_F7, (9, 0)),
        (KeyCode::KEY_F8, (10, 0)),
        (KeyCode::KEY_F9, (11, 0)),
        (KeyCode::KEY_F10, (12, 0)),
        (KeyCode::KEY_F11, (13, 0)),
        (KeyCode::KEY_F12, (14, 0)),
        // Control keys
        (KeyCode::KEY_SYSRQ, (15, 0)),
        (KeyCode::KEY_SCROLLLOCK, (16, 0)),
        (KeyCode::KEY_PAUSE, (17, 0)),
        (KeyCode::KEY_INSERT, (15, 1)),
        (KeyCode::KEY_HOME, (16, 1)),
        (KeyCode::KEY_PAGEUP, (17, 1)),
        (KeyCode::KEY_DELETE, (15, 2)),
        (KeyCode::KEY_END, (16, 2)),
        (KeyCode::KEY_PAGEDOWN, (17, 2)),
        // Arrow keys
        (KeyCode::KEY_UP, (16, 4)),
        (KeyCode::KEY_LEFT, (15, 5)),
        (KeyCode::KEY_DOWN, (16, 5)),
        (KeyCode::KEY_RIGHT, (17, 5)),
        // Numpad
        (KeyCode::KEY_NUMLOCK, (18, 1)),
        (KeyCode::KEY_KPSLASH, (19, 1)),
        (KeyCode::KEY_KPASTERISK, (20, 1)),
        (KeyCode::KEY_KPMINUS, (21, 1)),
        (KeyCode::KEY_KP7, (18, 2)),
        (KeyCode::KEY_KP8, (19, 2)),
        (KeyCode::KEY_KP9, (20, 2)),
        (KeyCode::KEY_KPPLUS, (21, 2)),
        (KeyCode::KEY_KP4, (18, 3)),
        (KeyCode::KEY_KP5, (19, 3)),
        (KeyCode::KEY_KP6, (20, 3)),
        (KeyCode::KEY_KP1, (18, 4)),
        (KeyCode::KEY_KP2, (19, 4)),
        (KeyCode::KEY_KP3, (20, 4)),
        (KeyCode::KEY_KPENTER, (21, 4)),
        (KeyCode::KEY_KP0, (19, 5)),
        (KeyCode::KEY_KPDOT, (20, 5)),
        // Typewriter A
        (KeyCode::KEY_GRAVE, (1, 1)),
        (KeyCode::KEY_1, (2, 1)),
        (KeyCode::KEY_2, (3, 1)),
        (KeyCode::KEY_3, (4, 1)),
        (KeyCode::KEY_4, (5, 1)),
        (KeyCode::KEY_5, (6, 1)),
        (KeyCode::KEY_6, (7, 1)),
        (KeyCode::KEY_7, (8, 1)),
        (KeyCode::KEY_8, (9, 1)),
        (KeyCode::KEY_9, (10, 1)),
        (KeyCode::KEY_0, (11, 1)),
        (KeyCode::KEY_MINUS, (12, 1)),
        (KeyCode::KEY_EQUAL, (13, 1)),
        (KeyCode::KEY_BACKSPACE, (14, 1)),
        // Typewriter B
        (KeyCode::KEY_TAB, (1, 2)),
        (KeyCode::KEY_Q, (2, 2)),
        (KeyCode::KEY_W, (3, 2)),
        (KeyCode::KEY_E, (4, 2)),
        (KeyCode::KEY_R, (5, 2)),
        (KeyCode::KEY_T, (6, 2)),
        (KeyCode::KEY_Y, (7, 2)),
        (KeyCode::KEY_U, (8, 2)),
        (KeyCode::KEY_I, (9, 2)),
        (KeyCode::KEY_O, (10, 2)),
        (KeyCode::KEY_P, (11, 2)),
        (KeyCode::KEY_LEFTBRACE, (12, 2)),
        (KeyCode::KEY_RIGHTBRACE, (13, 2)),
        (KeyCode::KEY_BACKSLASH, (14, 2)),
        // Typewriter C
        (KeyCode::KEY_CAPSLOCK, (1, 3)),
        (KeyCode::KEY_A, (2, 3)),
        (KeyCode::KEY_S, (3, 3)),
        (KeyCode::KEY_D, (4, 3)),
        (KeyCode::KEY_F, (5, 3)),
        (KeyCode::KEY_G, (6, 3)),
        (KeyCode::KEY_H, (7, 3)),
        (KeyCode::KEY_J, (8, 3)),
        (KeyCode::KEY_K, (9, 3)),
        (KeyCode::KEY_L, (10, 3)),
        (KeyCode::KEY_SEMICOLON, (11, 3)),
        (KeyCode::KEY_APOSTROPHE, (12, 3)),
        (KeyCode::KEY_ENTER, (14, 3)),
        // Typewriter D
        (KeyCode::KEY_LEFTSHIFT, (1, 4)),
        (KeyCode::KEY_Z, (3, 4)),
        (KeyCode::KEY_X, (4, 4)),
        (KeyCode::KEY_C, (5, 4)),
        (KeyCode::KEY_V, (6, 4)),
        (KeyCode::KEY_B, (7, 4)),
        (KeyCode::KEY_N, (8, 4)),
        (KeyCode::KEY_M, (9, 4)),
        (KeyCode::KEY_COMMA, (10, 4)),
        (KeyCode::KEY_DOT, (11, 4)),
        (KeyCode::KEY_SLASH, (12, 4)),
        (KeyCode::KEY_RIGHTSHIFT, (14, 4)),
        // Typewriter E
        (KeyCode::KEY_LEFTCTRL, (1, 5)),
        (KeyCode::KEY_LEFTMETA, (2, 5)),
        (KeyCode::KEY_LEFTALT, (3, 5)),
        (KeyCode::KEY_SPACE, (7, 5)),
        (KeyCode::KEY_RIGHTALT, (11, 5)),
        // NOTE: Not actually accessible with my keyboard, it seems to do some special stuff with
        // it where it allows me to do something like fn+f6 to press the pause/play media button.
        (KeyCode::KEY_FN, (12, 5)),
        (KeyCode::KEY_COMPOSE, (13, 5)),
        (KeyCode::KEY_RIGHTCTRL, (14, 5)),
    ]);

    let mut evdev_device = device.get_evdev_device()?;

    let mut effect_cycler = EffectCycler::new(matrix_manager.effect_custom()?);
    effects::add_effects_to_cycler(&mut effect_cycler);

    if let Some(forced_effect) = FORCED_EFFECT {
        if !effect_cycler.set_effect(forced_effect) {
            log::warn!("Invalid forced effect");
        } else {
            log::info!(
                "Playing effect: {:?}",
                effect_cycler.current_effect_identifier(),
            );
        }
    }

    let mut cycle_next_effect_time = std::time::Instant::now();

    let term = Arc::new(AtomicBool::new(false));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term));

    loop {
        let next_frame_time = std::time::Instant::now() + TARGET_UPDATE_RATE;

        if term.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        if FORCED_EFFECT.is_none() && std::time::Instant::now() >= cycle_next_effect_time {
            cycle_next_effect_time = std::time::Instant::now() + EFFECT_CHANGE_TIME;

            effect_cycler.next_effect();
            log::info!(
                "Playing effect: {:?}",
                effect_cycler.current_effect_identifier(),
            );
        }

        let mut matrix_events = Vec::new();

        if let Some(evdev_device) = evdev_device.as_mut() {
            while let Some(event) = evdev_device.try_next()? {
                match event.destructure() {
                    EventSummary::Key(_, key, 0) => {
                        if let Some((x, y)) = matrix_mapper.map(key) {
                            matrix_events.push(MatrixInput::Released { x, y });
                        }
                    }
                    EventSummary::Key(_, key, 1) => {
                        if let Some((x, y)) = matrix_mapper.map(key) {
                            matrix_events.push(MatrixInput::Pressed { x, y });
                        } else {
                            log::warn!("Unknown mapping: {key:?}");
                        }
                    }
                    EventSummary::Key(_, key, 2) => {
                        if let Some((x, y)) = matrix_mapper.map(key) {
                            matrix_events.push(MatrixInput::Repeat { x, y });
                        }
                    }
                    _ => {}
                }
            }
        }

        effect_cycler.update(&matrix_events)?;

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
