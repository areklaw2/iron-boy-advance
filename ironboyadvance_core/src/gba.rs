use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ironboyadvance_arm7tdmi::cpu::Arm7tdmiCpu;

use crate::{
    GbaError,
    bios::Bios,
    cartridge::Cartridge,
    ppu::CYCLES_PER_FRAME,
    scheduler::{Scheduler, event::EventType},
    system_bus::SystemBus,
    system_control::HaltMode,
};

pub struct GameBoyAdvance {
    arm7tdmi: Arm7tdmiCpu<SystemBus>,
    // may end making a common cpu trait
    // sharp_sm83: SharpSm83Cpu<SystemBus>,
    scheduler: Rc<RefCell<Scheduler>>,
    rom_name: String,
}

impl GameBoyAdvance {
    pub fn new(rom_path: PathBuf, bios_path: PathBuf, show_logs: bool, skip_bios: bool) -> Result<GameBoyAdvance, GbaError> {
        let rom_name = rom_path.file_name().unwrap().to_str().unwrap().to_string();
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path)?;
        let bios = Bios::load(bios_path)?;
        let gba = GameBoyAdvance {
            arm7tdmi: Arm7tdmiCpu::new(SystemBus::new(cartridge, bios, scheduler.clone()), skip_bios),
            scheduler,
            rom_name,
        };
        Ok(gba)
    }

    pub fn cycle(&mut self) {
        match self.arm7tdmi.bus().halt_mode() {
            HaltMode::Stopped => todo!(),
            HaltMode::Halted => {
                if self.arm7tdmi.bus().interrupt_pending() {
                    self.arm7tdmi.irq();
                    self.arm7tdmi.bus().un_halt();
                } else {
                    self.scheduler.borrow_mut().update_to_next_event()
                }
            }
            HaltMode::Running => {
                if self.arm7tdmi.bus().interrupt_pending() {
                    self.arm7tdmi.irq();
                }
                self.arm7tdmi.cycle();
            }
        }
    }

    pub fn run(&mut self, overshoot: usize) -> usize {
        let start_time = self.scheduler.borrow().timestamp();
        let end_time = start_time + CYCLES_PER_FRAME - overshoot;

        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(EventType::FrameComplete, end_time);

        'events: loop {
            while self.scheduler.borrow().timestamp() <= self.scheduler.borrow().timestamp_of_next_event() {
                self.cycle();
            }

            if self.handle_events() {
                break 'events;
            }
        }

        self.scheduler.borrow().timestamp() - start_time
    }

    fn handle_events(&mut self) -> bool {
        let mut scheduler = self.scheduler.borrow_mut();
        while let Some((event, timestamp)) = scheduler.pop() {
            let future_event: Option<(EventType, usize)> = match event {
                EventType::FrameComplete => return true,
                //TODO: write handlers for events
                EventType::Timer(_timer_event) => None,
                EventType::Ppu(_ppu_event) => None,
                EventType::Apu(_apu_event) => None,
            };

            if let Some((event_type, time)) = future_event {
                scheduler.schedule_at_timestamp(event_type, timestamp + time);
            }
        }
        false
    }
}
