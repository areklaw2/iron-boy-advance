use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::{
    arm7tdmi::cpu::Arm7tdmiCpu, bios::Bios, cartridge::Cartridge, memory::system_bus::SystemBus, scheduler::Scheduler,
    sharp_sm83::cpu::SharpSm83Cpu, GbaError,
};

pub struct GameBoyAdvance {
    arm7tdmi: Arm7tdmiCpu<SystemBus>,
    // may end making a common cpu trait
    // sharp_sm83: SharpSm83Cpu<SystemBus>,
    scheduler: Rc<RefCell<Scheduler>>,
    rom_name: String,
}

impl GameBoyAdvance {
    pub fn new(rom_path: PathBuf, bios_path: PathBuf, show_logs: bool) -> Result<GameBoyAdvance, GbaError> {
        let rom_name = rom_path.file_name().unwrap().to_str().unwrap().to_string();
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let cartridge = Cartridge::load(rom_path)?;
        let bios = Bios::load(bios_path)?;
        let gba = GameBoyAdvance {
            arm7tdmi: Arm7tdmiCpu::new(SystemBus::new(cartridge, bios, scheduler.clone())),
            scheduler,
            rom_name,
        };
        Ok(gba)
    }
}
