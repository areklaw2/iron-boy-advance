use std::{
    fs, io,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};

use chrono::Local;
use getset::CopyGetters;
use ironboyadvance::{BootError, boot, detect_console, system_info};
use ringbuf::traits::Producer;

use crate::{DesktopError, audio, config::Config, input::KEYPAD_IDLE};

struct FrameLimiter {
    duration: Duration,
    clock: Instant,
}

impl FrameLimiter {
    fn new(target_fps: f32) -> Self {
        let duration_nanos = 1_000_000_000.0 / target_fps;
        Self {
            duration: Duration::from_nanos(duration_nanos as u64),
            clock: Instant::now(),
        }
    }

    fn wait(&mut self) {
        let target = self.clock + self.duration;
        if let Some(remaining) = target.checked_duration_since(Instant::now()) {
            thread::sleep(remaining);
        }
        self.clock = target;
    }
}

#[derive(CopyGetters)]
pub(crate) struct FpsCounter {
    frame_count: u16,
    clock: Instant,
    #[getset(get_copy = "pub(crate)")]
    fps: f64,
}

impl FpsCounter {
    pub(crate) fn new() -> Self {
        Self {
            frame_count: 0,
            clock: Instant::now(),
            fps: 0.0,
        }
    }

    pub(crate) fn count_frame(&mut self) {
        self.frame_count += 1;
        let time_elapsed = self.clock.elapsed();
        if time_elapsed.as_secs() >= 1 {
            self.fps = self.frame_count as f64 / time_elapsed.as_secs_f64();
            self.frame_count = 0;
            self.clock = Instant::now();
        }
    }
}

fn current_unix_seconds() -> u64 {
    Local::now().naive_local().and_utc().timestamp().max(0) as u64
}

pub(crate) enum EmulatorCommand {
    Reset,
    SetPaused(bool),
    SetMaxSpeed(bool),
}

pub(crate) struct EmulatorHandle {
    pub keypad: Arc<AtomicU16>,
    pub frames: Receiver<Vec<u32>>,
    pub commands: Sender<EmulatorCommand>,
    pub viewport_width: usize,
    pub viewport_height: usize,
    _audio_stream: Option<cpal::Stream>,
}

impl EmulatorHandle {
    pub(crate) fn send(&self, command: EmulatorCommand) {
        if let Err(e) = self.commands.send(command) {
            tracing::error!("emulator command dropped (thread gone?): {e}");
        }
    }
}

pub(crate) fn read_rom(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

fn read_bios(path: Option<&str>) -> io::Result<Vec<u8>> {
    match path {
        Some(p) => fs::read(p),
        None => Ok(Vec::new()),
    }
}

pub(crate) fn spawn(
    rom_path: String,
    rom_buffer: Vec<u8>,
    config: Config,
    show_logs: bool,
) -> Result<EmulatorHandle, DesktopError> {
    let kind = detect_console(&rom_buffer).ok_or(BootError::UnknownFormat)?;
    let bios_buffer = read_bios(config.bios(kind))?;

    let (viewport_width, viewport_height, fps, sample_rate, cycles_per_frame) = system_info(kind);

    let keypad = Arc::new(AtomicU16::new(KEYPAD_IDLE));
    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();
    let (command_tx, command_rx) = mpsc::channel::<EmulatorCommand>();

    let (audio_stream, mut audio_producer, mut resampler) = match audio::start() {
        Some(output) => {
            let resampler = audio::Resampler::new(sample_rate, output.output_sample_rate);
            (Some(output.stream), Some(output.producer), Some(resampler))
        }
        None => (None, None, None),
    };

    let emu_keypad = keypad.clone();
    thread::spawn(move || {
        let mut system = boot(kind, &rom_path, rom_buffer, bios_buffer, current_unix_seconds(), show_logs)
            .unwrap_or_else(|e| panic!("failed to initialize emulator: {e}"));
        let mut overshoot = 0;
        let mut frame_limiter = FrameLimiter::new(fps);
        let mut paused = false;
        let mut turbo = false;

        'frame: loop {
            'commands: loop {
                match command_rx.try_recv() {
                    Ok(EmulatorCommand::SetPaused(value)) => {
                        paused = value;
                        tracing::info!("emulator {}", if paused { "paused" } else { "resumed" });
                    }
                    Ok(EmulatorCommand::SetMaxSpeed(value)) => {
                        turbo = value;
                        frame_limiter = FrameLimiter::new(fps);
                        tracing::info!("max_speed {}", if turbo { "on" } else { "off" });
                    }
                    Ok(EmulatorCommand::Reset) => {
                        let rom = match read_rom(&rom_path) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                tracing::error!("reset failed reading rom {rom_path}: {e}");
                                continue 'commands;
                            }
                        };
                        let Some(reset_kind) = detect_console(&rom) else {
                            tracing::error!("reset failed: unrecognized rom format");
                            continue 'commands;
                        };
                        let bios = match read_bios(config.bios(reset_kind)) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                tracing::error!("reset failed reading boot rom: {e}");
                                continue 'commands;
                            }
                        };
                        match boot(reset_kind, &rom_path, rom, bios, current_unix_seconds(), show_logs) {
                            Ok(new_system) => {
                                system = new_system;
                                overshoot = 0;
                                frame_limiter = FrameLimiter::new(fps);
                                tracing::info!("emulator reset");
                            }
                            Err(e) => tracing::error!("reset failed building emulator: {e}"),
                        }
                    }
                    Err(TryRecvError::Empty) => break 'commands,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            if !paused {
                system.handle_pressed_buttons(emu_keypad.load(Ordering::Relaxed));
                overshoot = system.run(cycles_per_frame, overshoot);

                if let (Some(producer), Some(resampler)) = (audio_producer.as_mut(), resampler.as_mut()) {
                    for &sample in system.audio_buffer() {
                        resampler.push_sample(sample, |left, right| {
                            let _ = producer.try_push(left);
                            let _ = producer.try_push(right);
                        });
                    }
                }
                system.clear_audio_buffer();

                if frame_tx.send(system.frame_buffer().to_vec()).is_err() {
                    break 'frame;
                }
            }

            if !turbo || paused {
                frame_limiter.wait();
            }
        }
    });

    Ok(EmulatorHandle {
        keypad,
        frames: frame_rx,
        commands: command_tx,
        viewport_width,
        viewport_height,
        _audio_stream: audio_stream,
    })
}
