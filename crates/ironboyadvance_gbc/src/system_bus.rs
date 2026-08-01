use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::{
    GbMode, GbSpeed,
    memory::{InterruptContext, MemoryInterface},
};
use tracing::debug;

use crate::{
    DOUBLE_SPEED_T_CYCLES, NORMAL_SPEED_T_CYCLES,
    boot_rom::BootRom,
    cartridge::Cartridge,
    events::{DmaEvent, GbcEvent, InterruptEvent, PpuEvent, SerialEvent, TimerEvent},
    io_registers::IoRegisters,
    memory::Memory,
};

const SPEED_SWITCH_T_CYCLES: usize = 8200;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SystemBus {
    boot_rom: BootRom,
    cartridge: Cartridge,
    memory: Memory,
    io_registers: IoRegisters,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
    #[getset(skip)]
    cpu_halted: bool,
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, boot_rom: BootRom, mode: GbMode, scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        let skip_boot = !boot_rom.loaded();

        SystemBus {
            boot_rom,
            cartridge,
            memory: Memory::new(),
            io_registers: IoRegisters::new(mode, skip_boot, scheduler.clone()),
            scheduler,
            cpu_halted: false,
        }
    }

    fn m_cycle(&mut self) {
        if self.io_registers.dma_controller().vram_dma_active() {
            self.run_vram_dma();
        }

        let t_cycles = match self.speed() {
            GbSpeed::Normal => NORMAL_SPEED_T_CYCLES,
            GbSpeed::Double => DOUBLE_SPEED_T_CYCLES,
        };

        self.scheduler.borrow_mut().step(t_cycles);
        self.handle_events();
    }

    fn handle_events(&mut self) {
        loop {
            let Some((event, timestamp)) = self.scheduler.borrow_mut().pop() else {
                return;
            };

            match event {
                GbcEvent::Interrupt(interrupt_event) => self.raise_interrupt(interrupt_event),
                GbcEvent::Serial(serial_event) => self.handle_serial_event(serial_event, timestamp),
                GbcEvent::Ppu(ppu_event) => self.handle_ppu_event(ppu_event, timestamp),
                GbcEvent::Timer(timer_event) => self.handle_timer_event(timer_event),
                GbcEvent::Apu(_) => todo!(),
                GbcEvent::Dma(dma_event) => self.handle_dma_event(dma_event, timestamp),
            }
        }
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.io_registers.interrupt_controller_mut().raise_interrupt(interrupt_event);
    }

    pub fn interrupts_pending(&self) -> bool {
        self.io_registers.interrupt_controller().interrupts_pending()
    }

    pub fn handle_serial_event(&mut self, serial_event: SerialEvent, timestamp: usize) {
        self.io_registers.serial_transfer_mut().handle_event(serial_event, timestamp);
    }

    pub fn handle_timer_event(&mut self, timer_event: TimerEvent) {
        self.io_registers.timer_mut().handle_event(timer_event);
    }

    pub fn handle_dma_event(&mut self, dma_event: DmaEvent, timestamp: usize) {
        match dma_event {
            DmaEvent::OamTransfer => self.run_oam_dma(timestamp),
        }
    }

    fn run_oam_dma(&mut self, timestamp: usize) {
        self.io_registers.dma_controller_mut().activate_oam_dma();

        let Some(transfer) = self.io_registers.dma_controller().next_oam_transfer() else {
            return;
        };

        let value = self.read_8(transfer.source);
        self.write_8(transfer.destination, value);
        self.io_registers.dma_controller_mut().complete_oam_transfer(timestamp);
    }

    fn run_vram_dma(&mut self) {
        while let Some(transfer) = self.io_registers.dma_controller().next_vram_transfer() {
            let value = self.read_8(transfer.source);
            self.write_8(transfer.destination, value);
            self.io_registers.dma_controller_mut().complete_vram_transfer();
        }
    }

    pub fn handle_ppu_event(&mut self, ppu_event: PpuEvent, timestamp: usize) {
        self.io_registers.ppu_mut().handle_event(ppu_event, timestamp);

        if ppu_event == PpuEvent::DrawingPixels {
            let cpu_halted = self.cpu_halted;
            self.io_registers.dma_controller_mut().start_h_blank_block(cpu_halted);
        }
    }

    pub fn set_cpu_halted(&mut self, cpu_halted: bool) {
        self.cpu_halted = cpu_halted;
    }

    pub fn speed(&self) -> GbSpeed {
        self.io_registers.speed_controller().speed()
    }
}

impl SystemMemoryAccess for SystemBus {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            _ if self.boot_rom.contains(address) => self.boot_rom.read_8(address),
            0xFF50 => self.boot_rom.read_8(address),
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_8(address),
            0xC000..=0xFDFF | 0xFF70 | 0xFF80..=0xFFFE => self.memory.read_8(address),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF00..=0xFF7F | 0xFFFF => self.io_registers.read_8(address),
            _ => {
                debug!("Read byte not implemented for address: {:#06X}", address);
                0xFF
            }
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF50 => self.boot_rom.write_8(address, value),
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_8(address, value),
            0xC000..=0xFDFF | 0xFF70 | 0xFF80..=0xFFFE => self.memory.write_8(address, value),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF00..=0xFF7F | 0xFFFF => self.io_registers.write_8(address, value),
            _ => debug!("Write byte not implemented for address: {:#06X}", address),
        }
    }
}

impl MemoryInterface for SystemBus {
    fn load_8(&mut self, address: u16) -> u8 {
        self.m_cycle();

        match self.io_registers.dma_controller().oam_dma_conflict(address) {
            true => 0xFF,
            false => self.read_8(address),
        }
    }

    fn load_16(&mut self, address: u16) -> u16 {
        let low = self.load_8(address) as u16;
        let high = self.load_8(address.wrapping_add(1)) as u16;
        high << 8 | low
    }

    fn store_8(&mut self, address: u16, value: u8) {
        self.m_cycle();

        if !self.io_registers.dma_controller().oam_dma_conflict(address) {
            self.write_8(address, value);
        }
    }

    fn store_16(&mut self, address: u16, value: u16) {
        self.store_8(address, value as u8);
        self.store_8(address.wrapping_add(1), (value >> 8) as u8);
    }

    fn idle_cycle(&mut self) {
        self.m_cycle();
    }

    fn change_speed(&mut self) -> bool {
        if !self.io_registers.speed_controller_mut().change_speed() {
            return false;
        }

        let speed = self.speed();
        self.io_registers.serial_transfer_mut().set_speed(speed);
        self.io_registers.timer_mut().set_speed(speed);
        self.io_registers.dma_controller_mut().set_speed(speed);

        self.scheduler.borrow_mut().step(SPEED_SWITCH_T_CYCLES);
        self.handle_events();
        true
    }

    fn interrupt_context(&self) -> &InterruptContext {
        self.io_registers.interrupt_controller().interrupt_context()
    }

    fn interrupt_context_mut(&mut self) -> &mut InterruptContext {
        self.io_registers.interrupt_controller_mut().interrupt_context_mut()
    }
}
