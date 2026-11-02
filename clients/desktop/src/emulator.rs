use std::{
    fs, io,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use chrono::Local;
use getset::{CopyGetters, Getters};
use ironboyadvance::{BootError, SystemInfo, Viewport, boot, detect_console, system_info};
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
    SetPaused(bool),
    SetMaxSpeed(bool),
    Stop,
}

#[derive(Getters, CopyGetters)]
pub(crate) struct Emulator {
    #[getset(get = "pub(crate)")]
    keypad: Arc<AtomicU16>,
    frames: Receiver<Vec<u32>>,
    commands: Sender<EmulatorCommand>,
    #[getset(get_copy = "pub(crate)")]
    viewport: Viewport,
    last_frame: Option<Vec<u32>>,
    fps_counter: FpsCounter,
    #[getset(get_copy = "pub(crate)")]
    paused: bool,
    max_speed: bool,
    _audio_stream: Option<cpal::Stream>,
    rom_path: String,
    thread: JoinHandle<()>,
}

impl Emulator {
    pub(crate) fn spawn(
        rom_path: String,
        rom_buffer: Vec<u8>,
        config: &Config,
        show_logs: bool,
    ) -> Result<Emulator, DesktopError> {
        let kind = detect_console(&rom_buffer).ok_or(BootError::UnknownFormat)?;
        let bios_buffer = read_bios(config.bios(kind))?;

        let SystemInfo {
            viewport,
            fps,
            sample_rate,
            cycles_per_frame,
        } = system_info(kind);

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
        let emu_rom_path = rom_path.clone();

        let (boot_tx, boot_rx) = mpsc::sync_channel::<Result<(), BootError>>(1);
        let thread = thread::spawn(move || {
            let mut system = match boot(
                kind,
                &emu_rom_path,
                rom_buffer,
                bios_buffer,
                current_unix_seconds(),
                show_logs,
            ) {
                Ok(system) => {
                    let _ = boot_tx.send(Ok(()));
                    system
                }
                Err(error) => {
                    let _ = boot_tx.send(Err(error));
                    return;
                }
            };

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
                        Ok(EmulatorCommand::Stop) => break 'frame,
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

        match boot_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = thread.join();
                return Err(DesktopError::BootError(error));
            }
            Err(_) => return Err(DesktopError::EmulatorThreadExited),
        }

        Ok(Emulator {
            keypad,
            frames: frame_rx,
            commands: command_tx,
            viewport,
            _audio_stream: audio_stream,
            last_frame: None,
            fps_counter: FpsCounter::new(),
            paused: false,
            max_speed: false,
            rom_path,
            thread,
        })
    }

    pub(crate) fn toggle_pause(&mut self) {
        self.paused ^= true;
        self.send(EmulatorCommand::SetPaused(self.paused));
    }

    pub(crate) fn toggle_max_speed(&mut self) {
        self.max_speed ^= true;
        self.send(EmulatorCommand::SetMaxSpeed(self.max_speed));
    }

    pub(crate) fn stop(self) {
        self.send(EmulatorCommand::Stop);
        if let Err(e) = self.thread.join() {
            tracing::error!("emulator thread panicked: {e:?}");
        }
    }

    pub(crate) fn drain_frames(&mut self) {
        while let Ok(frame) = self.frames.try_recv() {
            self.last_frame = Some(frame);
            self.fps_counter.count_frame();
        }
    }

    pub(crate) fn last_frame(&self) -> Option<&[u32]> {
        self.last_frame.as_deref()
    }

    pub(crate) fn fps(&self) -> f64 {
        self.fps_counter.fps
    }

    pub(crate) fn rom_path(&self) -> &str {
        &self.rom_path
    }

    fn send(&self, command: EmulatorCommand) {
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
