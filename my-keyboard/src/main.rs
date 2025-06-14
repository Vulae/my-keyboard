use anyhow::Error;
use effects::Effect;
use openrazer::query_razer_devices;

mod effects;

const TARGET_FPS: std::time::Duration = std::time::Duration::from_millis(1000 / 15);

pub fn main() -> Result<(), Error> {
    let mut device = query_razer_devices()?
        .into_iter()
        .next()
        .expect("No razer device found.");

    let matrix_manager = device.matrix_manager();
    let mut matrix_frame = matrix_manager.effect_custom()?;

    let mut cycle_next_effect_time = std::time::Instant::now();
    let mut effect_index = 0;
    let mut effect: Box<dyn Effect> = Box::new(effects::EffectFrozen::attach(&mut matrix_frame)?);

    loop {
        let next_frame_time = std::time::Instant::now() + TARGET_FPS;

        if std::time::Instant::now() >= cycle_next_effect_time {
            cycle_next_effect_time =
                std::time::Instant::now() + std::time::Duration::from_secs_f64(60.0 * 10.0);

            let last_effect_index = effect_index;
            while effect_index == last_effect_index {
                effect_index = rand::random_range(0..=3);
            }
            // effect_index = 2;

            drop(effect);
            effect = match effect_index {
                0 => Box::new(effects::EffectRainbow1::attach(&mut matrix_frame)?),
                1 => Box::new(effects::EffectRainbow2::attach(&mut matrix_frame)?),
                2 => Box::new(effects::EffectRainbow3::attach(&mut matrix_frame)?),
                3 => Box::new(effects::EffectPride::attach(&mut matrix_frame)?),
                _ => unreachable!(),
            };
        }

        effect.update()?;

        let mut waited = false;
        while std::time::Instant::now() < next_frame_time {
            std::thread::sleep(std::time::Duration::from_nanos(100));
            waited = true;
        }
        if !waited {
            println!("WARNING: Failed to update in time!");
        }
    }
}
