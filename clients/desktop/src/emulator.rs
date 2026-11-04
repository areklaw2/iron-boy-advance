use std::{
    fs, io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use chrono::Local;
use getset::{CopyGetters, Getters};
use ironboyadvance::{BootError, SystemInfo, Viewport, boot, detect_console, system_info};
use ringbuf::traits::{Observer, Producer};
use winit::event_loop::EventLoopProxy;

use crate::{
    DesktopError,
    app::UserEvent,
    audio::{self, AudioSink},
    config::Config,
    input::KEYPAD_IDLE,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FpsTarget {
    Fixed(u32),
    Max,
    Native,
}

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

pub(crate) struct SpawnFailure {
    pub(crate) error: DesktopError,
    pub(crate) audio_sink: Option<AudioSink>,
}

struct PanicNotifier {
    panicked: Arc<AtomicBool>,
    proxy: EventLoopProxy<UserEvent>,
}

impl Drop for PanicNotifier {
    fn drop(&mut self) {
        if thread::panicking() {
            self.panicked.store(true, Ordering::Relaxed);
            let _ = self.proxy.send_event(UserEvent::EmulatorExited);
        }
    }
}

pub(crate) enum EmulatorCommand {
    SetPaused(bool),
    SetFps(FpsTarget),
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
    rom_path: String,
    thread: JoinHandle<Option<AudioSink>>,
    panicked: Arc<AtomicBool>,
}

impl Emulator {
    pub(crate) fn spawn(
        rom_path: String,
        rom_buffer: Vec<u8>,
        config: &Config,
        show_logs: bool,
        proxy: EventLoopProxy<UserEvent>,
        mut audio_sink: Option<AudioSink>,
    ) -> Result<Emulator, SpawnFailure> {
        let kind = match detect_console(&rom_buffer) {
            Some(kind) => kind,
            None => {
                return Err(SpawnFailure {
                    error: BootError::UnknownFormat.into(),
                    audio_sink,
                });
            }
        };
        let bios_buffer = match read_bios(config.bios(kind)) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(SpawnFailure {
                    error: error.into(),
                    audio_sink,
                });
            }
        };

        let SystemInfo {
            viewport,
            native_fps,
            sample_rate,
            cycles_per_frame,
        } = system_info(kind);

        let keypad = Arc::new(AtomicU16::new(KEYPAD_IDLE));
        let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();
        let (command_tx, command_rx) = mpsc::channel::<EmulatorCommand>();

        let mut resampler = audio_sink
            .as_ref()
            .map(|sink| audio::Resampler::new(sample_rate, sink.output_sample_rate));
        let emu_keypad = keypad.clone();
        let emu_rom_path = rom_path.clone();
        let panicked = Arc::new(AtomicBool::new(false));
        let panic_notifier = PanicNotifier {
            panicked: panicked.clone(),
            proxy,
        };

        let (boot_tx, boot_rx) = mpsc::sync_channel::<Result<(), BootError>>(1);
        let thread = thread::spawn(move || {
            let _panic_notifier = panic_notifier;
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
                    return audio_sink;
                }
            };

            let mut overshoot = 0;
            let mut frame_limiter = FrameLimiter::new(native_fps);
            let mut paused = false;
            let mut is_max_fps = false;

            'frame: loop {
                'commands: loop {
                    match command_rx.try_recv() {
                        Ok(EmulatorCommand::SetPaused(value)) => {
                            paused = value;
                            tracing::info!("emulator {}", if paused { "paused" } else { "resumed" });
                        }
                        Ok(EmulatorCommand::SetFps(fps_target)) => {
                            is_max_fps = fps_target == FpsTarget::Max;
                            let fps = match fps_target {
                                FpsTarget::Fixed(fixed_fps) => fixed_fps as f32,
                                FpsTarget::Max | FpsTarget::Native => native_fps,
                            };
                            frame_limiter = FrameLimiter::new(fps);
                            if let Some(resampler) = resampler.as_mut() {
                                resampler.set_speed((fps / native_fps) as f64);
                            }
                            tracing::info!("fps target {fps_target:?}");
                        }
                        Ok(EmulatorCommand::Stop) => break 'frame,
                        Err(TryRecvError::Empty) => break 'commands,
                        Err(TryRecvError::Disconnected) => return audio_sink,
                    }
                }

                if !paused {
                    system.handle_pressed_buttons(emu_keypad.load(Ordering::Relaxed));
                    overshoot = system.run(cycles_per_frame, overshoot);

                    if !is_max_fps && let (Some(sink), Some(resampler)) = (audio_sink.as_mut(), resampler.as_mut()) {
                        for &sample in system.audio_buffer() {
                            resampler.push_sample(sample, |left, right| {
                                if sink.producer.vacant_len() >= 2 {
                                    let _ = sink.producer.try_push(left);
                                    let _ = sink.producer.try_push(right);
                                }
                            });
                        }
                    }
                    system.clear_audio_buffer();

                    if frame_tx.send(system.frame_buffer().to_vec()).is_err() {
                        break 'frame;
                    }
                }

                if !is_max_fps || paused {
                    frame_limiter.wait();
                }
            }

            audio_sink
        });

        match boot_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                return Err(SpawnFailure {
                    error: DesktopError::BootError(error),
                    audio_sink: thread.join().unwrap_or(None),
                });
            }
            Err(_) => {
                return Err(SpawnFailure {
                    error: DesktopError::EmulatorThreadExited,
                    audio_sink: thread.join().unwrap_or(None),
                });
            }
        }

        Ok(Emulator {
            keypad,
            frames: frame_rx,
            commands: command_tx,
            viewport,
            last_frame: None,
            fps_counter: FpsCounter::new(),
            paused: false,
            rom_path,
            thread,
            panicked,
        })
    }

    pub(crate) fn toggle_pause(&mut self) {
        self.paused ^= true;
        self.send(EmulatorCommand::SetPaused(self.paused));
    }

    pub(crate) fn set_fps(&self, fps_target: FpsTarget) {
        self.send(EmulatorCommand::SetFps(fps_target));
    }

    pub(crate) fn stop(self) -> Option<AudioSink> {
        self.send(EmulatorCommand::Stop);
        match self.thread.join() {
            Ok(audio_sink) => audio_sink,
            Err(e) => {
                tracing::error!("emulator thread panicked: {e:?}");
                None
            }
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

    pub(crate) fn panicked(&self) -> bool {
        self.panicked.load(Ordering::Relaxed)
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
