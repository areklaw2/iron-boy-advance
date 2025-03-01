use crate::{
    arm7tdmi::{Condition, CpuAction},
    memory::{MemoryAccess, MemoryInterface},
};

use super::{arm::ArmInstruction, psr::ProgramStatusRegister, CpuMode, CpuState};

const SP_INDEX: usize = 13;
const LR_INDEX: usize = 14;
const PC_INDEX: usize = 15;

pub trait Instruction {
    type Size;
    fn decode(value: Self::Size, address: u32) -> Self;
    fn disassamble(&self) -> String;
    fn value(&self) -> Self::Size;
}

pub struct Arm7tdmiCpu<I: MemoryInterface> {
    general_registers: [u32; 15],
    general_registers_fiq: [u32; 7], //r8 to r12
    general_registers_svc: [u32; 2], //r13 to r14
    general_registers_abt: [u32; 2], //r13 to r14
    general_registers_irq: [u32; 2], //r13 to r14
    general_registers_und: [u32; 2], //r13 to r14
    pc: u32,
    cpsr: ProgramStatusRegister,
    spsrs: [ProgramStatusRegister; 6],
    fetched_instruction: u32,
    decoded_instruction: u32,
    bus: I, // May need to make this shared
    next_memory_access: MemoryAccess,
}

impl<I: MemoryInterface> MemoryInterface for Arm7tdmiCpu<I> {
    fn load_8(&self, address: u32) -> u8 {
        self.bus.load_8(address)
    }

    fn load_16(&self, address: u32) -> u16 {
        self.bus.load_16(address)
    }

    fn load_32(&self, address: u32) -> u32 {
        self.bus.load_32(address)
    }

    fn store_8(&mut self, address: u32, value: u8) {
        self.bus.store_8(address, value);
    }

    fn store_16(&mut self, address: u32, value: u16) {
        self.bus.store_16(address, value);
    }

    fn store_32(&mut self, address: u32, value: u32) {
        self.bus.store_32(address, value);
    }
}

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn new(bus: I, skip_bios: bool) -> Self {
        let mut cpu = Arm7tdmiCpu {
            general_registers: [0; 15],
            general_registers_fiq: [0; 7], //r8 to r12
            general_registers_svc: [0; 2], //r13 to r14
            general_registers_abt: [0; 2], //r13 to r14
            general_registers_irq: [0; 2], //r13 to r14
            general_registers_und: [0; 2], //r13 to r14
            pc: 0,
            cpsr: ProgramStatusRegister::from_bits(0x13),
            spsrs: [ProgramStatusRegister::from_bits(0x13); 6],
            fetched_instruction: 0,
            decoded_instruction: 0,
            bus,
            next_memory_access: MemoryAccess::NonSequential,
        };

        match skip_bios {
            true => {
                cpu.general_registers[LR_INDEX] = 0x08000000;
                cpu.general_registers[SP_INDEX] = 0x03007F00;
                cpu.general_registers_svc[0] = 0x3007FE0;
                cpu.general_registers_irq[0] = 0x3007FA0;
                cpu.cpsr.set_cpu_mode(CpuMode::System);
                cpu.cpsr.set_irq_disable(false);
                cpu.pc = 0x08000000;
            }
            false => {
                cpu.cpsr.set_cpu_mode(CpuMode::Supervisor);
                cpu.cpsr.set_irq_disable(true);
                cpu.pc = 0;
            }
        }

        cpu
    }

    pub fn cycle(&mut self) {
        match self.cpsr.cpu_state() {
            CpuState::Arm => {
                let pc = self.pc & !0b11;
                let executed_instruction = self.decoded_instruction;
                self.decoded_instruction = self.fetched_instruction;
                self.fetched_instruction = self.load_32(pc);

                let instruction = ArmInstruction::decode(executed_instruction, pc);
                //TODO log this
                println!("{}", instruction.disassamble());

                let condtion = instruction.cond();
                if condtion != Condition::AL && !self.is_condition_met(condtion) {
                    self.pc = self.pc.wrapping_add(4);
                    self.next_memory_access = MemoryAccess::NonSequential;
                    return;
                }

                match self.arm_execute(instruction) {
                    CpuAction::Advance(memory_access) => {
                        self.next_memory_access = memory_access;
                        self.pc = self.pc.wrapping_add(4);
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
            CpuState::Thumb => {
                let pc = self.pc & !0b01;
                self.pc = self.pc.wrapping_add(2);
            }
        }
    }

    fn is_condition_met(&self, condition: Condition) -> bool {
        use Condition::*;
        match condition {
            EQ => self.cpsr.zero(),
            NE => !self.cpsr.zero(),
            CS => self.cpsr.carry(),
            CC => !self.cpsr.carry(),
            MI => self.cpsr.negative(),
            PL => !self.cpsr.negative(),
            VS => self.cpsr.overflow(),
            VC => !self.cpsr.overflow(),
            HI => self.cpsr.carry() && !self.cpsr.zero(),
            LS => !self.cpsr.carry() || self.cpsr.zero(),
            GE => self.cpsr.negative() == self.cpsr.overflow(),
            LT => self.cpsr.negative() != self.cpsr.overflow(),
            GT => !self.cpsr.zero() && (self.cpsr.negative() == self.cpsr.overflow()),
            LE => self.cpsr.zero() || (self.cpsr.negative() != self.cpsr.overflow()),
            AL => true,
        }
    }
}
