use getset::{Getters, MutGetters, Setters};
use tracing::debug;

use crate::{
    Condition, GbMode, R16Memory, Register8, Register16, Register16Stack,
    instruction::{Instruction, SharpSm83InstructionFactory, generate_lut},
    memory::MemoryInterface,
    registers::Registers,
};

#[derive(Getters, MutGetters, Setters)]
#[getset(get = "pub(crate)", set = "pub(crate)")]
pub struct Sm83<I: MemoryInterface> {
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    registers: Registers,
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    bus: I,
    show_logs: bool,
    halted: bool,
    interrupt_master_enable: bool,
    enable_interrupt_delay: u8,
    disable_interrupt_delay: u8,
    lut: [SharpSm83InstructionFactory; 256],
}

impl<I: MemoryInterface> MemoryInterface for Sm83<I> {
    fn load_8(&self, address: u16) -> u8 {
        self.bus.load_8(address)
    }

    fn load_16(&self, address: u16) -> u16 {
        self.bus.load_16(address)
    }

    fn store_8(&mut self, address: u16, value: u8) {
        self.bus.store_8(address, value);
    }

    fn store_16(&mut self, address: u16, value: u16) {
        self.bus.store_16(address, value);
    }

    fn idle_cycle(&mut self) {
        self.bus.idle_cycle();
    }

    fn change_speed(&mut self) {
        self.bus.change_speed();
    }
}

impl<I: MemoryInterface> Sm83<I> {
    pub fn new(bus: I, show_logs: bool, skip_boot: bool, mode: GbMode) -> Self {
        Self {
            registers: Registers::new(skip_boot, mode),
            bus,
            show_logs,
            halted: false,
            interrupt_master_enable: false,
            enable_interrupt_delay: 0,
            disable_interrupt_delay: 0,
            lut: generate_lut(),
        }
    }

    pub fn cycle(&mut self) {
        let opcode = self.bus.load_8(self.registers.pc());
        let instruction = (self.lut[opcode as usize])(opcode);
        if self.show_logs {
            debug!("{}", instruction.disassemble(self));
        }
        self.advance_pc();
        instruction.execute(self);
    }

    #[inline]
    pub(crate) fn is_condition_met(&self, condition: Condition) -> bool {
        use Condition::*;
        match condition {
            NZ => !self.registers.f().zero(),
            Z => self.registers.f().zero(),
            NC => !self.registers.f().carry(),
            C => self.registers.f().carry(),
        }
    }

    pub(crate) fn pc(&self) -> u16 {
        self.registers.pc()
    }

    pub(crate) fn set_pc(&mut self, value: u16) {
        self.registers.set_pc(value);
    }

    pub(crate) fn advance_pc(&mut self) {
        self.registers.set_pc(self.registers.pc().wrapping_add(1));
    }

    pub(crate) fn register_8(&self, reg: Register8) -> u8 {
        match reg {
            Register8::A => self.registers.a(),
            Register8::B => self.registers.b(),
            Register8::C => self.registers.c(),
            Register8::D => self.registers.d(),
            Register8::E => self.registers.e(),
            Register8::H => self.registers.h(),
            Register8::L => self.registers.l(),
            Register8::HLMem => self.bus.load_8(self.registers.hl()),
        }
    }

    pub(crate) fn set_register_8(&mut self, reg: Register8, value: u8) {
        match reg {
            Register8::A => {
                self.registers.set_a(value);
            }
            Register8::B => {
                self.registers.set_b(value);
            }
            Register8::C => {
                self.registers.set_c(value);
            }
            Register8::D => {
                self.registers.set_d(value);
            }
            Register8::E => {
                self.registers.set_e(value);
            }
            Register8::H => {
                self.registers.set_h(value);
            }
            Register8::L => {
                self.registers.set_l(value);
            }
            Register8::HLMem => self.bus.store_8(self.registers.hl(), value),
        }
    }

    pub(crate) fn register_16(&self, reg: Register16) -> u16 {
        match reg {
            Register16::BC => self.registers.bc(),
            Register16::DE => self.registers.de(),
            Register16::HL => self.registers.hl(),
            Register16::SP => self.registers.sp(),
        }
    }

    pub(crate) fn set_register_16(&mut self, reg: Register16, value: u16) {
        match reg {
            Register16::BC => self.registers.set_bc(value),
            Register16::DE => self.registers.set_de(value),
            Register16::HL => self.registers.set_hl(value),
            Register16::SP => {
                self.registers.set_sp(value);
            }
        }
    }

    pub(crate) fn register_16_stack(&self, reg: Register16Stack) -> u16 {
        match reg {
            Register16Stack::BC => self.registers.bc(),
            Register16Stack::DE => self.registers.de(),
            Register16Stack::HL => self.registers.hl(),
            Register16Stack::AF => self.registers.af(),
        }
    }

    pub(crate) fn set_register_16_stack(&mut self, reg: Register16Stack, value: u16) {
        match reg {
            Register16Stack::BC => self.registers.set_bc(value),
            Register16Stack::DE => self.registers.set_de(value),
            Register16Stack::HL => self.registers.set_hl(value),
            Register16Stack::AF => self.registers.set_af(value),
        }
    }

    pub(crate) fn register_16_memory(&mut self, reg: R16Memory) -> u16 {
        match reg {
            R16Memory::BC => self.registers.bc(),
            R16Memory::DE => self.registers.de(),
            R16Memory::HLI => self.registers.increment_hl(),
            R16Memory::HLD => self.registers.decrement_hl(),
        }
    }

    pub(crate) fn fetch_byte(&mut self) -> u8 {
        let byte = self.bus.load_8(self.registers.pc());
        self.registers.set_pc(self.registers.pc().wrapping_add(1));
        byte
    }

    pub(crate) fn fetch_word(&mut self) -> u16 {
        let word = self.bus.load_16(self.registers.pc());
        self.registers.set_pc(self.registers.pc().wrapping_add(2));
        word
    }

    pub(crate) fn pop_stack(&mut self) -> u16 {
        let value = self.bus.load_16(self.registers.sp());
        self.registers.set_sp(self.registers.sp().wrapping_add(2));
        value
    }

    pub(crate) fn push_stack(&mut self, value: u16) {
        self.bus.idle_cycle();
        self.registers.set_sp(self.registers.sp().wrapping_sub(1));
        self.bus.store_8(self.registers.sp(), (value >> 8) as u8);
        self.registers.set_sp(self.registers.sp().wrapping_sub(1));
        self.bus.store_8(self.registers.sp(), value as u8);
    }
}
