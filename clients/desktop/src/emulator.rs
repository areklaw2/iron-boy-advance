use std::{
    fs, io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
};

use ironboyadvance_common::emulator::{Emulator, System, detect_system};
use ironboyadvance_gba::{APU_SAMPLING_FREQUENCY, CYCLES_PER_FRAME, GameBoyAdvance, GbaError};
use ironboyadvance_gbc::{GameBoyColor, GbcError};
use ringbuf::traits::Producer;
use thiserror::Error;

use crate::{DesktopError, audio, frame::FrameTimer, input::KEYPAD_IDLE};

#[derive(Error, Debug)]
enum BootError {
    #[error("failed to initialize GBA: {0}")]
    Gba(#[from] GbaError),
    #[error("failed to initialize GBC: {0}")]
    Gbc(#[from] GbcError),
    #[error("unrecognized rom format")]
    UnknownFormat,
}

pub enum EmulatorCommand {
    Reset,
    TogglePause,
    ToggleMaxSpeed,
}

pub struct EmulatorHandle {
    pub keypad: Arc<AtomicU16>,
    pub frames: Receiver<Vec<u32>>,
    pub commands: Sender<EmulatorCommand>,
    _audio_stream: Option<cpal::Stream>,
}

fn read_rom(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

fn read_bios(path: Option<&str>) -> io::Result<Vec<u8>> {
    match path {
        Some(p) => fs::read(p),
        None => Ok(Vec::new()),
    }
}

fn boot(rom_path: &str, rom: Vec<u8>, bios: Vec<u8>, show_logs: bool) -> Result<Box<dyn Emulator>, BootError> {
    match detect_system(&rom) {
        Some(System::Gba) => Ok(Box::new(GameBoyAdvance::new(PathBuf::from(rom_path), rom, bios, show_logs)?)),
        Some(System::Gb) | Some(System::Gbc) => Ok(Box::new(GameBoyColor::new(PathBuf::from(rom_path), rom, show_logs)?)),
        None => Err(BootError::UnknownFormat),
    }
}

pub fn spawn(rom_path: String, bios_path: Option<String>, show_logs: bool) -> Result<EmulatorHandle, DesktopError> {
    let rom_buffer = read_rom(&rom_path)?;
    let bios_buffer = read_bios(bios_path.as_deref())?;

    let keypad = Arc::new(AtomicU16::new(KEYPAD_IDLE));
    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();
    let (command_tx, command_rx) = mpsc::channel::<EmulatorCommand>();

    let (audio_stream, mut audio_producer, mut resampler) = match audio::start() {
        Some(output) => {
            let resampler = audio::Resampler::new(APU_SAMPLING_FREQUENCY as u32, output.output_sample_rate);
            (Some(output.stream), Some(output.producer), Some(resampler))
        }
        None => (None, None, None),
    };

    let emu_keypad = keypad.clone();
    thread::spawn(move || {
        let mut system = boot(&rom_path, rom_buffer, bios_buffer, show_logs)
            .unwrap_or_else(|e| panic!("failed to initialize emulator: {e}"));
        let mut overshoot = 0;
        let mut frame_timer = FrameTimer::new();
        let mut paused = false;
        let mut turbo = false;

        'frame: loop {
            'commands: loop {
                match command_rx.try_recv() {
                    Ok(EmulatorCommand::TogglePause) => {
                        paused = !paused;
                        tracing::info!("emulator {}", if paused { "paused" } else { "resumed" });
                    }
                    Ok(EmulatorCommand::ToggleMaxSpeed) => {
                        turbo = !turbo;
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
                        let bios = match read_bios(bios_path.as_deref()) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                tracing::error!("reset failed reading bios {bios_path:?}: {e}");
                                continue 'commands;
                            }
                        };
                        match boot(&rom_path, rom, bios, show_logs) {
                            Ok(new_system) => {
                                system = new_system;
                                overshoot = 0;
                                frame_timer = FrameTimer::new();
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
                overshoot = system.run(CYCLES_PER_FRAME, overshoot);

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
                frame_timer.slow_frame();
            }
            frame_timer.count_frame();
        }
    });

    Ok(EmulatorHandle {
        keypad,
        frames: frame_rx,
        commands: command_tx,
        _audio_stream: audio_stream,
    })
}
