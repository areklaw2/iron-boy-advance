use ironboyadvance_utils::get_set;

use crate::{
    CpuAction,
    arm::Condition,
    memory::{MemoryAccess, MemoryInterface},
};

use super::{CpuMode, CpuState, arm::ArmInstruction, psr::ProgramStatusRegister};

pub const SP: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 15;

pub trait Instruction {
    type Size;
    fn decode(value: Self::Size, pc: u32) -> Self;
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction;
    fn disassamble(&self) -> String;
    fn value(&self) -> Self::Size;
}

pub struct Arm7tdmiCpu<I: MemoryInterface> {
    general_registers: [u32; 16],
    general_registers_fiq: [u32; 7], //r8 to r12
    general_registers_svc: [u32; 2], //r13 to r14
    general_registers_abt: [u32; 2], //r13 to r14
    general_registers_irq: [u32; 2], //r13 to r14
    general_registers_und: [u32; 2], //r13 to r14
    cpsr: ProgramStatusRegister,
    spsrs: [ProgramStatusRegister; 5],
    pipeline: [u32; 2],
    bus: I, // May need to make this shared
    next_memory_access: u8,
}

impl<I: MemoryInterface> MemoryInterface for Arm7tdmiCpu<I> {
    fn load_8(&mut self, address: u32, access: u8) -> u32 {
        self.bus.load_8(address, access)
    }

    fn load_16(&mut self, address: u32, access: u8) -> u32 {
        self.bus.load_16(address, access)
    }

    fn load_32(&mut self, address: u32, access: u8) -> u32 {
        self.bus.load_32(address, access)
    }

    fn store_8(&mut self, address: u32, value: u8, access: u8) {
        self.bus.store_8(address, value, access);
    }

    fn store_16(&mut self, address: u32, value: u16, access: u8) {
        self.bus.store_16(address, value, access);
    }

    fn store_32(&mut self, address: u32, value: u32, access: u8) {
        self.bus.store_32(address, value, access);
    }
}

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn new(bus: I, skip_bios: bool) -> Self {
        let mut cpu = Arm7tdmiCpu {
            general_registers: [0; 16],
            general_registers_fiq: [0; 7], //r8 to r12
            general_registers_svc: [0; 2], //r13 to r14
            general_registers_abt: [0; 2], //r13 to r14
            general_registers_irq: [0; 2], //r13 to r14
            general_registers_und: [0; 2], //r13 to r14
            cpsr: ProgramStatusRegister::from_bits(0x13),
            spsrs: [ProgramStatusRegister::from_bits(0x13); 5],
            pipeline: [0; 2],
            bus,
            next_memory_access: MemoryAccess::Instruction | MemoryAccess::Nonsequential,
        };

        match skip_bios {
            true => {
                cpu.general_registers[SP] = 0x03007F00;
                cpu.general_registers[LR] = 0x08000000;
                cpu.general_registers[PC] = 0x08000000;
                cpu.general_registers_svc[0] = 0x3007FE0;
                cpu.general_registers_irq[0] = 0x3007FA0;
                cpu.cpsr.set_cpu_mode(CpuMode::System);
                cpu.cpsr.set_irq_disable(false);
            }
            false => {
                cpu.cpsr.set_cpu_mode(CpuMode::Supervisor);
                cpu.cpsr.set_irq_disable(true);
            }
        }

        //TODO: not sure if i need this forever
        //cpu.refill_pipeline();
        cpu
    }

    get_set!(general_registers, set_general_registers, [u32; 16]);
    get_set!(general_registers_fiq, set_general_registers_fiq, [u32; 7]);
    get_set!(general_registers_svc, set_general_registers_svc, [u32; 2]);
    get_set!(general_registers_abt, set_general_registers_abt, [u32; 2]);
    get_set!(general_registers_irq, set_general_registers_irq, [u32; 2]);
    get_set!(general_registers_und, set_general_registers_und, [u32; 2]);
    get_set!(cpsr, set_cpsr, ProgramStatusRegister);
    get_set!(spsrs, set_spsrs, [ProgramStatusRegister; 5]);
    get_set!(pipeline, set_pipeline, [u32; 2]);

    pub fn cycle(&mut self) {
        let pc = self.general_registers[PC] & !0x1;

        match self.cpsr.cpu_state() {
            CpuState::Arm => {
                let instruction = self.pipeline[0];
                self.pipeline[0] = self.pipeline[1];
                self.pipeline[1] = self.load_32(pc, self.next_memory_access);
                let instruction = ArmInstruction::decode(instruction, pc - 8);

                //TODO log this
                println!("{}", instruction);
                println!("{}", instruction.disassamble());

                let condition = instruction.cond();
                if condition != Condition::AL && !self.is_condition_met(condition) {
                    self.advance_pc_arm();
                    self.next_memory_access = MemoryAccess::Instruction | MemoryAccess::Sequential;
                    return;
                }
                match instruction.execute(self) {
                    CpuAction::Advance(memory_access) => {
                        self.advance_pc_arm();
                        self.next_memory_access = memory_access;
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
            CpuState::Thumb => {
                let instruction = self.pipeline[0];
                self.pipeline[0] = self.pipeline[1];
                self.pipeline[1] = self.load_32(pc, self.next_memory_access);
                self.advance_pc_thumb();

                // TODO
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

    pub fn set_cpu_state(&mut self, state: CpuState) {
        self.cpsr.set_cpu_state(state);
    }

    pub fn pc(&self) -> u32 {
        self.general_registers[PC]
    }

    pub fn set_pc(&mut self, value: u32) {
        self.general_registers[PC] = value;
    }

    pub fn advance_pc_thumb(&mut self) {
        self.general_registers[PC] = self.general_registers[PC].wrapping_add(2);
    }

    pub fn advance_pc_arm(&mut self) {
        self.general_registers[PC] = self.general_registers[PC].wrapping_add(4);
    }

    pub fn refill_pipeline(&mut self) {
        match self.cpsr.cpu_state() {
            CpuState::Arm => {
                self.pipeline[0] = self.load_32(
                    self.general_registers[PC],
                    MemoryAccess::Instruction | MemoryAccess::Nonsequential,
                );
                self.advance_pc_arm();
                self.pipeline[1] = self.load_32(
                    self.general_registers[PC],
                    MemoryAccess::Instruction | MemoryAccess::Sequential,
                );
                self.advance_pc_arm();
                self.next_memory_access = MemoryAccess::Instruction | MemoryAccess::Sequential;
            }
            CpuState::Thumb => {
                self.pipeline[0] = self.load_16(
                    self.general_registers[PC],
                    MemoryAccess::Instruction | MemoryAccess::Sequential,
                );
                self.advance_pc_thumb();
                self.pipeline[1] = self.load_16(
                    self.general_registers[PC],
                    MemoryAccess::Instruction | MemoryAccess::Sequential,
                );
                self.advance_pc_thumb();
                self.next_memory_access = MemoryAccess::Instruction | MemoryAccess::Sequential;
            }
        }
    }

    pub fn get_register(&self, index: usize) -> u32 {
        match index {
            0..=7 | 15 => self.general_registers[index],
            8..=12 => match self.cpsr.cpu_mode() == CpuMode::Fiq {
                true => self.general_registers_fiq[index - 8],
                false => self.general_registers[index],
            },
            13 | 14 => match self.cpsr.cpu_mode() {
                CpuMode::System | CpuMode::User => self.general_registers[index],
                CpuMode::Fiq => self.general_registers_fiq[index - 8],
                CpuMode::Irq => self.general_registers_irq[index - 13],
                CpuMode::Supervisor => self.general_registers_svc[index - 13],
                CpuMode::Abort => self.general_registers_abt[index - 13],
                CpuMode::Undefined => self.general_registers_und[index - 13],
            },
            _ => panic!("Index out of range"),
        }
    }

    pub fn set_register(&mut self, index: usize, value: u32) {
        match index {
            0..=7 | 15 => self.general_registers[index] = value,
            8..=12 => match self.cpsr.cpu_mode() == CpuMode::Fiq {
                true => self.general_registers_fiq[index - 8] = value,
                false => self.general_registers[index] = value,
            },
            13 | 14 => match self.cpsr.cpu_mode() {
                CpuMode::System | CpuMode::User => self.general_registers[index] = value,
                CpuMode::Fiq => self.general_registers_fiq[index - 8] = value,
                CpuMode::Irq => self.general_registers_irq[index - 13] = value,
                CpuMode::Supervisor => self.general_registers_svc[index - 13] = value,
                CpuMode::Abort => self.general_registers_abt[index - 13] = value,
                CpuMode::Undefined => self.general_registers_und[index - 13] = value,
            },
            _ => panic!("Index out of range"),
        }
    }
}
