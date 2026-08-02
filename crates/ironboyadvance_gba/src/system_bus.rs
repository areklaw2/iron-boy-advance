use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_arm7tdmi::{
    CpuState,
    memory::{CpuContext, MemoryInterface},
};
use ironboyadvance_common::{
    memory::{MemoryAccess, MemoryAccessWidth, SystemMemoryAccess},
    scheduler::Scheduler,
};
use tracing::debug;

use crate::{
    bios::Bios,
    cartridge::Cartridge,
    dma_control::ChunkSize,
    events::{ApuEvent, CartridgeEvent, DmaEvent, GbaEvent, InterruptEvent, PpuEvent, TimerEvent},
    io_registers::IoRegisters,
    memory::Memory,
    system_control::HaltMode,
};

pub const BIOS_BASE: u32 = 0x0000_0000;
pub const WRAM_BOARD_BASE: u32 = 0x0200_0000;
pub const WRAM_CHIP_BASE: u32 = 0x0300_0000;
pub const IO_REGISTERS_BASE: u32 = 0x0400_0000;
pub const PALETTE_RAM_BASE: u32 = 0x0500_0000;
pub const VRAM_BASE: u32 = 0x0600_0000;
pub const OAM_BASE: u32 = 0x0700_0000;
pub const ROM_WS0_LO: u32 = 0x0800_0000;
pub const ROM_WS0_HI: u32 = 0x0900_0000;
pub const ROM_WS1_LO: u32 = 0x0A00_0000;
pub const ROM_WS1_HI: u32 = 0x0B00_0000;
pub const ROM_WS2_LO: u32 = 0x0C00_0000;
pub const ROM_WS2_HI: u32 = 0x0D00_0000;
pub const SRAM_LO: u32 = 0x0E00_0000;
pub const SRAM_HI: u32 = 0x0F00_0000;

#[derive(Getters, MutGetters)]
pub struct SystemBus {
    bios: Bios,
    memory: Memory,
    #[getset(get = "pub", get_mut = "pub")]
    io_registers: IoRegisters,
    cartridge: Cartridge,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    cpu_context: CpuContext,
    dma_open_bus_value: u32,
    last_access: u8,
}

impl MemoryInterface for SystemBus {
    fn load_8(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::Byte);
        let value = self.read_8(address) as u32;
        self.latch_dma_open_bus(access, value);
        value
    }

    fn load_16(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::HalfWord);
        let value = self.read_16(address) as u32;
        self.latch_dma_open_bus(access, (value << 16) | value);
        value
    }

    fn load_32(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::Word);
        let value = self.read_32(address);
        self.latch_dma_open_bus(access, value);
        value
    }

    fn store_8(&mut self, address: u32, value: u8, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::Byte);
        self.write_8(address, value);
        self.latch_dma_open_bus(access, u32::from(value));
    }

    fn store_16(&mut self, address: u32, value: u16, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::HalfWord);
        self.write_16(address, value);
        let value = u32::from(value);
        self.latch_dma_open_bus(access, (value << 16) | value);
    }

    fn store_32(&mut self, address: u32, value: u32, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::Word);
        self.write_32(address, value);
        self.latch_dma_open_bus(access, value);
    }

    fn idle_cycle(&mut self) {
        if self.io_registers.dma_controller().is_active() {
            self.run_dma();
        }
        self.scheduler.borrow_mut().step(1);
        self.handle_events();
    }

    fn cpu_context_mut(&mut self) -> &mut CpuContext {
        &mut self.cpu_context
    }
}

impl SystemMemoryAccess for SystemBus {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_8(address),
            WRAM_BOARD_BASE => self.memory.read_8(address),
            WRAM_CHIP_BASE => self.memory.read_8(address),
            IO_REGISTERS_BASE => self.io_registers.read_8(address),
            PALETTE_RAM_BASE => self.io_registers.read_8(address),
            VRAM_BASE => self.io_registers.read_8(address),
            OAM_BASE => self.io_registers.read_8(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_8(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_8(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_8(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                self.open_bus_read(address, MemoryAccessWidth::Byte) as u8
            }
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        let address = self.align(address, MemoryAccessWidth::HalfWord);
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_16(address),
            WRAM_BOARD_BASE => self.memory.read_16(address),
            WRAM_CHIP_BASE => self.memory.read_16(address),
            IO_REGISTERS_BASE => self.io_registers.read_16(address),
            PALETTE_RAM_BASE => self.io_registers.read_16(address),
            VRAM_BASE => self.io_registers.read_16(address),
            OAM_BASE => self.io_registers.read_16(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_16(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_16(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_16(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_16(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                self.open_bus_read(address, MemoryAccessWidth::HalfWord) as u16
            }
        }
    }

    fn read_32(&self, address: u32) -> u32 {
        let address = self.align(address, MemoryAccessWidth::Word);
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_32(address),
            WRAM_BOARD_BASE => self.memory.read_32(address),
            WRAM_CHIP_BASE => self.memory.read_32(address),
            IO_REGISTERS_BASE => self.io_registers.read_32(address),
            PALETTE_RAM_BASE => self.io_registers.read_32(address),
            VRAM_BASE => self.io_registers.read_32(address),
            OAM_BASE => self.io_registers.read_32(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_32(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_32(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_32(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_32(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                self.open_bus_read(address, MemoryAccessWidth::Word)
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address, value),
            WRAM_BOARD_BASE => self.memory.write_8(address, value),
            WRAM_CHIP_BASE => self.memory.write_8(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_8(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_8(address, value),
            VRAM_BASE => self.io_registers.write_8(address, value),
            OAM_BASE => self.io_registers.write_8(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_8(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_8(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_8(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_8(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        let address = self.align(address, MemoryAccessWidth::HalfWord);
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_16(address, value),
            WRAM_BOARD_BASE => self.memory.write_16(address, value),
            WRAM_CHIP_BASE => self.memory.write_16(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_16(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_16(address, value),
            VRAM_BASE => self.io_registers.write_16(address, value),
            OAM_BASE => self.io_registers.write_16(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_16(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_16(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_16(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_16(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
        }
    }

    fn write_32(&mut self, address: u32, value: u32) {
        let address = self.align(address, MemoryAccessWidth::Word);
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_32(address, value),
            WRAM_BOARD_BASE => self.memory.write_32(address, value),
            WRAM_CHIP_BASE => self.memory.write_32(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_32(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_32(address, value),
            VRAM_BASE => self.io_registers.write_32(address, value),
            OAM_BASE => self.io_registers.write_32(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_32(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_32(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_32(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_32(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
        }
    }
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, bios: Bios, scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        SystemBus {
            bios,
            memory: Memory::new(),
            io_registers: IoRegisters::new(scheduler.clone()),
            cartridge,
            scheduler,
            cpu_context: CpuContext::default(),
            dma_open_bus_value: 0,
            last_access: 0,
        }
    }

    fn latch_dma_open_bus(&mut self, access: u8, value: u32) {
        if MemoryAccess::Dma.is_set(access) {
            self.dma_open_bus_value = value;
        }
    }

    fn align(&self, address: u32, width: MemoryAccessWidth) -> u32 {
        match address & 0xFF000000 {
            SRAM_LO | SRAM_HI => address,
            _ => match width {
                MemoryAccessWidth::Byte => address,
                MemoryAccessWidth::HalfWord => address & !1,
                MemoryAccessWidth::Word => address & !3,
            },
        }
    }

    fn open_bus_read(&self, address: u32, width: MemoryAccessWidth) -> u32 {
        let value = match MemoryAccess::Dma.is_set(self.last_access) {
            true => self.dma_open_bus_value,
            false => match self.cpu_context.cpu_state {
                CpuState::Arm => self.cpu_context.pipeline[1],
                CpuState::Thumb => {
                    let decoded = self.cpu_context.pipeline[0] & 0xFFFF;
                    let fetched = self.cpu_context.pipeline[1] & 0xFFFF;
                    let pc = self.cpu_context.pc;
                    match pc & 0xFF00_0000 {
                        // Approximation, cant get to $+6 for aligned and $+2 for unaligned
                        // See GBATEK - GBA Unpredictable Things.
                        BIOS_BASE | OAM_BASE => match pc & 2 == 0 {
                            true => (fetched << 16) | decoded,
                            false => (fetched << 16) | fetched,
                        },
                        WRAM_CHIP_BASE => match pc & 2 == 0 {
                            true => (decoded << 16) | fetched,
                            false => (fetched << 16) | decoded,
                        },
                        _ => (fetched << 16) | fetched,
                    }
                }
            },
        };

        match width {
            MemoryAccessWidth::Byte => value >> ((address & 3) * 8),
            MemoryAccessWidth::HalfWord => value >> ((address & 2) * 8),
            MemoryAccessWidth::Word => value,
        }
    }

    fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        if self.io_registers.dma_controller().is_active() && (access_pattern & (MemoryAccess::Dma | MemoryAccess::Lock)) == 0
        {
            self.run_dma();
        }

        self.last_access = access_pattern;
        self.bios.set_pc(self.cpu_context.pc);

        let access = match MemoryAccess::Sequential.is_set(access_pattern) {
            true => MemoryAccess::Sequential,
            false => MemoryAccess::NonSequential,
        };

        let index = ((address >> 24) & 0xF) as usize;
        let cycles = self.io_registers.system_controller().cycles(index, width, access);
        self.scheduler.borrow_mut().step(cycles);
        self.handle_events();
    }

    pub fn handle_events(&mut self) {
        loop {
            let Some((event, timestamp)) = self.scheduler.borrow_mut().pop() else {
                return;
            };

            match event {
                GbaEvent::Interrupt(interrupt_event) => self.raise_interrupt(interrupt_event),
                GbaEvent::Timer(timers_event) => self.handle_timer_event(timers_event),
                GbaEvent::Ppu(ppu_event) => self.handle_ppu_event(ppu_event, timestamp),
                GbaEvent::Apu(apu_event) => self.handle_apu_event(apu_event),
                GbaEvent::Dma(dma_event) => self.handle_dma_event(dma_event),
                GbaEvent::Cartridge(cartridge_event) => self.handle_cartridge_event(cartridge_event),
            }
        }
    }

    pub fn interrupt_pending(&self) -> bool {
        self.io_registers.interrupt_controller().interrupt_pending()
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.io_registers.interrupt_controller_mut().raise_interrupt(interrupt_event);
    }

    pub fn halt_mode(&self) -> HaltMode {
        self.io_registers.system_controller().halt_mode()
    }

    pub fn un_halt(&mut self) {
        self.io_registers.system_controller_mut().set_halt_mode(HaltMode::Running);
    }

    pub fn handle_ppu_event(&mut self, ppu_event: PpuEvent, timestamp: usize) {
        self.io_registers.ppu_mut().handle_event(ppu_event, timestamp);
    }

    pub fn handle_apu_event(&mut self, apu_event: ApuEvent) {
        self.io_registers.apu_mut().handle_event(apu_event);
    }

    pub fn handle_timer_event(&mut self, timer_event: TimerEvent) {
        self.io_registers.timer_controller_mut().handle_event(timer_event);
    }

    pub fn handle_dma_event(&mut self, dma_event: DmaEvent) {
        self.io_registers.dma_controller_mut().handle_event(dma_event);
    }

    pub fn handle_cartridge_event(&mut self, cartridge_event: CartridgeEvent) {
        self.cartridge.handle_event(cartridge_event);
    }

    pub fn run_dma(&mut self) {
        self.scheduler.borrow_mut().step(1);

        while let Some(transfer) = self.io_registers.dma_controller().next_transfer() {
            match transfer.chunk_size {
                ChunkSize::Size16 => {
                    let value = self.load_16(transfer.source, transfer.source_access) as u16;
                    self.store_16(transfer.destination, value, transfer.destination_access);
                }
                ChunkSize::Size32 => {
                    let value = self.load_32(transfer.source, transfer.source_access);
                    self.store_32(transfer.destination, value, transfer.destination_access);
                }
            }

            self.io_registers.dma_controller_mut().complete_transfer();

            while let Some(channel_id) = self.io_registers.dma_controller().pending_dma_request() {
                self.io_registers
                    .dma_controller_mut()
                    .handle_event(DmaEvent::Activate { channel_id });
            }
        }

        self.scheduler.borrow_mut().step(1);
    }

    pub fn dma_active(&self) -> bool {
        self.io_registers.dma_controller().is_active()
    }
}
