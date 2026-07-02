use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use clap::Parser;
use openrazer::{
    query_razer_devices, DeviceMatrixCustom, EvdevDeviceNonblocking, EventSummary, KeyCode,
    RazerDevice, MATRIX_HEIGHT, MATRIX_WIDTH,
};

use crate::{
    effect_finder::EffectFinder, lua::KeyboardLuaRunner,
    matrix_mapper::create_default_matrix_mapper,
};

mod effect_finder;
mod lua;
mod matrix_mapper;
mod util;

const DEFAULT_FPS: u64 = 20;
const DEFAULT_EFFECT_CYCLE_TIME: u64 = 60 * 5;

const NEXT_EFFECT_KEY: Option<KeyCode> = Some(KeyCode::KEY_PAUSE);

#[derive(Parser, Debug)]
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
    #[arg(short, long, default_value_t = DEFAULT_EFFECT_CYCLE_TIME)]
    cycle_time_secs: u64,
    /// Visualize the keyboard matrix in the terminal
    #[arg(short, long, default_value_t = false)]
    visualize: bool,
}

#[derive(Debug)]
struct App<'a> {
    visualize: bool,
    watch: bool,
    fps: u64,
    cycle_time_secs: u64,
    path: PathBuf,
    razer_device: Option<&'a RazerDevice>,
    evdev_device: Option<&'a EvdevDeviceNonblocking>,
}

impl<'a> App<'a> {
    pub fn run(mut self) -> Result<()> {
        let mut terminal = self.visualize.then(ratatui::init);

        let update_rate = std::time::Duration::from_secs_f64(1.0 / (self.fps as f64));
        let cycle_time = std::time::Duration::from_secs(self.cycle_time_secs);

        let mut matrix_manager = self
            .razer_device
            .ok_or(anyhow!("No razer device"))?
            .get_matrix_manager()?
            .ok_or(anyhow!("Razer device has no custom lighting"))?;
        let mut matrix = matrix_manager.effect_custom()?;
        let matrix_mapper = create_default_matrix_mapper();

        let mut finder = EffectFinder::new(&self.path, self.watch)?;
        let mut lua_runner = KeyboardLuaRunner::new_with_code_path(finder.next_effect()?)?;

        let term = if terminal.is_none() {
            let term = Arc::new(std::sync::atomic::AtomicBool::new(false));
            signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
            signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
            Some(term)
        } else {
            None
        };

        let mut cycle_next_effect_time = std::time::Instant::now() + cycle_time;
        'outer: loop {
            let next_frame_time = std::time::Instant::now() + update_rate;

            #[allow(clippy::collapsible_if)]
            if let Some(term) = term.as_ref() {
                if term.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
            }

            lua_runner.step_start()?;

            if let Some(evdev_device) = self.evdev_device.as_mut() {
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

            if let Some(terminal) = terminal.as_mut() {
                while crossterm::event::poll(std::time::Duration::ZERO)? {
                    if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Char('c'),
                        modifiers: crossterm::event::KeyModifiers::CONTROL,
                        kind: crossterm::event::KeyEventKind::Press,
                        state: _,
                    }) = crossterm::event::read()?
                    {
                        break 'outer;
                    }
                }
                terminal.draw(|frame| {
                    const SCALE_X: u16 = 4;
                    const SCALE_Y: u16 = 2;

                    let chunks = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Vertical)
                        .constraints([
                            ratatui::layout::Constraint::Min(MATRIX_HEIGHT as u16 * SCALE_Y + 2),
                            ratatui::layout::Constraint::Fill(u16::MAX),
                        ])
                        .split(frame.area());

                    let block = ratatui::widgets::Block::bordered().title(" Visualization ");
                    let inner = block.inner(chunks[0]);
                    frame.render_widget(block, chunks[0]);

                    #[derive(Debug)]
                    struct KeyboardWidget<'a> {
                        matrix: &'a DeviceMatrixCustom<'a>,
                    }
                    impl<'a> ratatui::widgets::Widget for KeyboardWidget<'a> {
                        fn render(
                            self,
                            area: ratatui::prelude::Rect,
                            buf: &mut ratatui::prelude::Buffer,
                        ) where
                            Self: Sized,
                        {
                            let mut tmp_cell = ratatui::buffer::Cell::EMPTY;
                            self.matrix.iter().for_each(|(mx, my, color)| {
                                let [r, g, b] = color.to_quantized();
                                tmp_cell.set_bg(ratatui::style::Color::Rgb(r, g, b));
                                for dx in 0..SCALE_X {
                                    for dy in 0..SCALE_Y {
                                        let x = area.x + mx as u16 * SCALE_X + dx;
                                        let y = area.y + my as u16 * SCALE_Y + dy;
                                        if let Some(cell) = buf.cell_mut((x, y)) {
                                            *cell = tmp_cell.clone();
                                        }
                                    }
                                }
                            });
                        }
                    }
                    let chunks2 = ratatui::layout::Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            ratatui::layout::Constraint::Min(MATRIX_WIDTH as u16 * SCALE_X),
                            ratatui::layout::Constraint::Length(1),
                            ratatui::layout::Constraint::Fill(u16::MAX),
                        ])
                        .split(inner);
                    frame.render_widget(KeyboardWidget { matrix: &matrix }, chunks2[0]);
                    frame.render_widget("CTRL+C to exit", chunks2[2]);
                    frame.render_widget(
                        format!("Effect: {:?}", lua_runner.code_path()),
                        chunks2[2] + ratatui::layout::Position::new(0, 1).into(),
                    );

                    let log_widget = tui_logger::TuiLoggerWidget::default()
                        .block(ratatui::widgets::Block::bordered().title(" Logs "))
                        .style_error(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Red),
                        )
                        .style_debug(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                        )
                        .style_warn(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                        )
                        .style_trace(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Magenta),
                        )
                        .style_info(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
                        )
                        .output_separator(':')
                        .output_timestamp(Some("%H:%M:%S".to_owned()))
                        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated));
                    frame.render_widget(log_widget, chunks[1]);
                })?;
            }

            if util::sleep_until(next_frame_time) == std::time::Duration::ZERO {
                log::warn!("Failed to update effect in time");
            }
        }

        log::info!("Exiting my-keyboard, setting keyboard matrix to spectrum");
        matrix_manager.effect_spectrum()?;

        if terminal.is_some() {
            ratatui::restore();
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.visualize {
        tui_logger::init_logger(tui_logger::LevelFilter::Debug)?;
        // env_logger::Builder::default()
        //     .format(move |_, record| {
        //         tui_logger::Drain.log(record);
        //         Ok(())
        //     })
        //     .init();
    } else {
        env_logger::init();
    }

    let razer_device = query_razer_devices()?
        .into_iter()
        .next()
        .expect("No razer device found.");

    let evdev_device = if cli.no_key_events {
        None
    } else {
        razer_device.get_evdev_device()?
    };

    let app = App {
        visualize: cli.visualize,
        watch: cli.watch,
        fps: cli.fps,
        cycle_time_secs: cli.cycle_time_secs,
        path: cli
            .path
            .unwrap_or(std::env::current_dir()?)
            .canonicalize()?,
        razer_device: Some(&razer_device),
        evdev_device: evdev_device.as_ref(),
    };

    app.run()?;

    Ok(())
}
