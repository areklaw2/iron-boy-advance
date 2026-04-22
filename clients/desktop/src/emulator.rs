use std::{
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
};

use ironboyadvance_core::{CYCLES_PER_FRAME, GameBoyAdvance};

use crate::{frame::FrameTimer, input::KEYPAD_IDLE};

pub enum EmulatorCommand {
    TogglePause,
    ToggleMaxSpeed,
}

pub struct EmulatorHandle {
    pub keypad: Arc<AtomicU16>,
    pub frames: Receiver<Vec<u32>>,
    pub commands: Sender<EmulatorCommand>,
}

pub fn spawn(rom_buffer: Vec<u8>, bios_buffer: Box<[u8]>, show_logs: bool) -> EmulatorHandle {
    let keypad = Arc::new(AtomicU16::new(KEYPAD_IDLE));
    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();
    let (command_tx, command_rx) = mpsc::channel::<EmulatorCommand>();

    let emu_keypad = keypad.clone();
    thread::spawn(move || {
        let mut gba = match GameBoyAdvance::new(rom_buffer, bios_buffer, show_logs) {
            Ok(gba) => gba,
            Err(e) => panic!("failed to initialize GBA: {e}"),
        };
        let mut overshoot = 0;
        let mut frame_timer = FrameTimer::new();
        let mut paused = false;
        let mut turbo = false;

        loop {
            loop {
                match command_rx.try_recv() {
                    Ok(EmulatorCommand::TogglePause) => {
                        paused = !paused;
                        tracing::info!("emulator {}", if paused { "paused" } else { "resumed" });
                    }
                    Ok(EmulatorCommand::ToggleMaxSpeed) => {
                        turbo = !turbo;
                        tracing::info!("max_speed {}", if turbo { "on" } else { "off" });
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            if !paused {
                gba.handle_pressed_buttons(emu_keypad.load(Ordering::Relaxed));
                overshoot = gba.run(CYCLES_PER_FRAME, overshoot);
                if frame_tx.send(gba.frame_buffer().to_vec()).is_err() {
                    break;
                }
            }

            if !turbo || paused {
                frame_timer.slow_frame();
            }
            frame_timer.count_frame();
        }
    });

    EmulatorHandle {
        keypad,
        frames: frame_rx,
        commands: command_tx,
    }
}
