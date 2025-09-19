use ironboyadvance_utils::get_set;

use crate::{
    Condition, CpuAction, Exception,
    arm::{ArmInstructionKind, lut::generate_arm_lut},
    memory::{MemoryAccess, MemoryInterface},
    thumb::{ThumbInstruction, ThumbInstructionKind, lut::generate_thumb_lut},
};

use super::{CpuMode, CpuState, arm::ArmInstruction, psr::ProgramStatusRegister};

pub const SP: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 15;

pub trait Instruction {
    type Size;
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction;
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String;
    fn value(&self) -> Self::Size;
}

pub struct Arm7tdmiCpu<I: MemoryInterface> {
    general_registers: [u32; 16],
    banked_registers_fiq: [u32; 7], //r8 to r14
    banked_registers_svc: [u32; 2], //r13 to r14
    banked_registers_abt: [u32; 2], //r13 to r14
    banked_registers_irq: [u32; 2], //r13 to r14
    banked_registers_und: [u32; 2], //r13 to r14
    spsrs: [ProgramStatusRegister; 5],
    cpsr: ProgramStatusRegister,
    pipeline: [u32; 2],
    bus: I, // May need to make this shared TODO:make getter
    next_memory_access: u8,
    arm_lut: [ArmInstructionKind; 4096],
    thumb_lut: [ThumbInstructionKind; 1024],
    pub(super) dissassembled_instruction: String,
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

    fn idle_cycle(&mut self) {
        self.bus.idle_cycle();
    }
}

impl<I: MemoryInterface> Arm7tdmiCpu<I> {
    pub fn new(bus: I, skip_bios: bool) -> Self {
        let mut cpu = Arm7tdmiCpu {
            general_registers: [0; 16],
            banked_registers_fiq: [0; 7], //r8 to r14
            banked_registers_svc: [0; 2], //r13 to r14
            banked_registers_abt: [0; 2], //r13 to r14
            banked_registers_irq: [0; 2], //r13 to r14
            banked_registers_und: [0; 2], //r13 to r14
            spsrs: [ProgramStatusRegister::from_bits(0x13); 5],
            cpsr: ProgramStatusRegister::from_bits(0x13),
            pipeline: [0; 2],
            bus,
            next_memory_access: MemoryAccess::Instruction | MemoryAccess::NonSequential,
            arm_lut: [ArmInstructionKind::Undefined; 4096],
            thumb_lut: [ThumbInstructionKind::Undefined; 1024],
            dissassembled_instruction: String::new(),
        };

        cpu.arm_lut = generate_arm_lut();
        cpu.thumb_lut = generate_thumb_lut();

        match skip_bios {
            true => {
                cpu.general_registers[SP] = 0x03007F00;
                cpu.general_registers[LR] = 0x08000000;
                cpu.general_registers[PC] = 0x08000000;
                cpu.banked_registers_svc[0] = 0x3007FE0;
                cpu.banked_registers_irq[0] = 0x3007FA0;
                cpu.cpsr.set_mode(CpuMode::System);
                cpu.cpsr.set_irq_disable(false);
            }
            false => {
                cpu.cpsr.set_mode(CpuMode::Supervisor);
                cpu.cpsr.set_irq_disable(true);
            }
        }

        //TODO: not sure if i need this forever
        //cpu.refill_pipeline();
        cpu
    }

    get_set!(general_registers, set_general_registers, [u32; 16]);
    get_set!(banked_registers_fiq, set_banked_registers_fiq, [u32; 7]);
    get_set!(banked_registers_svc, set_banked_registers_svc, [u32; 2]);
    get_set!(banked_registers_abt, set_banked_registers_abt, [u32; 2]);
    get_set!(banked_registers_irq, set_banked_registers_irq, [u32; 2]);
    get_set!(banked_registers_und, set_banked_registers_und, [u32; 2]);
    get_set!(spsrs, set_spsrs, [ProgramStatusRegister; 5]);
    get_set!(cpsr, set_cpsr, ProgramStatusRegister);
    get_set!(pipeline, set_pipeline, [u32; 2]);

    pub fn cycle(&mut self) {
        let pc = self.general_registers[PC] & !0x1;

        match self.cpsr.state() {
            CpuState::Arm => {
                let instruction = self.pipeline[0];
                self.pipeline[0] = self.pipeline[1];
                self.pipeline[1] = self.load_32(pc, self.next_memory_access);
                let lut_index = ((instruction >> 16) & 0x0FF0) | ((instruction >> 4) & 0x000F);
                let instruction = ArmInstruction::new(self.arm_lut[lut_index as usize], instruction, pc.saturating_sub(8));
                self.dissassembled_instruction = instruction.disassemble(self);

                println!("{}", instruction);
                println!("{}", self.dissassembled_instruction);

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
                let lut_index = (instruction) as u16 >> 6;
                let instruction =
                    ThumbInstruction::new(self.thumb_lut[lut_index as usize], instruction as u16, pc.saturating_sub(4));
                self.dissassembled_instruction = instruction.disassemble(self);

                println!("{}", instruction);
                println!("{}", self.dissassembled_instruction);

                match instruction.execute(self) {
                    CpuAction::Advance(memory_access) => {
                        self.advance_pc_thumb();
                        self.next_memory_access = memory_access;
                    }
                    CpuAction::PipelineFlush => {}
                };
            }
        }
    }

    pub fn is_condition_met(&self, condition: Condition) -> bool {
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

    pub fn set_negative(&mut self, status: bool) {
        self.cpsr.set_negative(status);
    }

    pub fn set_zero(&mut self, status: bool) {
        self.cpsr.set_zero(status);
    }

    pub fn set_carry(&mut self, status: bool) {
        self.cpsr.set_carry(status);
    }

    pub fn set_overflow(&mut self, status: bool) {
        self.cpsr.set_overflow(status);
    }

    pub fn set_flags(&mut self, value: u8) {
        self.cpsr.set_flags(value);
    }

    pub fn set_irq_disable(&mut self, status: bool) {
        self.cpsr.set_irq_disable(status);
    }

    pub fn set_state(&mut self, state: CpuState) {
        self.cpsr.set_state(state);
    }

    pub fn set_mode(&mut self, mode: CpuMode) {
        self.cpsr.set_mode(mode);
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

    pub fn pipeline_flush(&mut self) {
        match self.cpsr.state() {
            CpuState::Arm => {
                self.pipeline[0] = self.load_32(
                    self.general_registers[PC],
                    MemoryAccess::Instruction | MemoryAccess::NonSequential,
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

    pub fn register(&self, index: usize) -> u32 {
        match index {
            0..=7 | 15 => self.general_registers[index],
            8..=12 => match self.cpsr.mode() == CpuMode::Fiq {
                true => self.banked_registers_fiq[index - 8],
                false => self.general_registers[index],
            },
            13 | 14 => match self.cpsr.mode() {
                CpuMode::System | CpuMode::User => self.general_registers[index],
                CpuMode::Fiq => self.banked_registers_fiq[index - 8],
                CpuMode::Irq => self.banked_registers_irq[index - 13],
                CpuMode::Supervisor => self.banked_registers_svc[index - 13],
                CpuMode::Abort => self.banked_registers_abt[index - 13],
                CpuMode::Undefined => self.banked_registers_und[index - 13],
                CpuMode::Invalid => panic!("invalid mode"),
            },
            _ => panic!("Index out of range"),
        }
    }

    pub fn set_register(&mut self, index: usize, value: u32) {
        match index {
            0..=7 | 15 => self.general_registers[index] = value,
            8..=12 => match self.cpsr.mode() == CpuMode::Fiq {
                true => self.banked_registers_fiq[index - 8] = value,
                false => self.general_registers[index] = value,
            },
            13 | 14 => match self.cpsr.mode() {
                CpuMode::System | CpuMode::User => self.general_registers[index] = value,
                CpuMode::Fiq => self.banked_registers_fiq[index - 8] = value,
                CpuMode::Supervisor => self.banked_registers_svc[index - 13] = value,
                CpuMode::Abort => self.banked_registers_abt[index - 13] = value,
                CpuMode::Irq => self.banked_registers_irq[index - 13] = value,
                CpuMode::Undefined => self.banked_registers_und[index - 13] = value,
                CpuMode::Invalid => panic!("invalid mode"),
            },
            _ => panic!("Index out of range"),
        }
    }

    pub fn spsr(&self) -> ProgramStatusRegister {
        match self.cpsr.mode() {
            CpuMode::User | CpuMode::System => self.cpsr,
            CpuMode::Fiq => self.spsrs[0],
            CpuMode::Supervisor => self.spsrs[1],
            CpuMode::Abort => self.spsrs[2],
            CpuMode::Irq => self.spsrs[3],
            CpuMode::Undefined => self.spsrs[4],
            CpuMode::Invalid => panic!("invalid mode"),
        }
    }

    pub fn set_spsr(&mut self, spsr: ProgramStatusRegister) {
        match self.cpsr.mode() {
            CpuMode::User | CpuMode::System => self.cpsr = spsr,
            CpuMode::Fiq => self.spsrs[0] = spsr,
            CpuMode::Supervisor => self.spsrs[1] = spsr,
            CpuMode::Abort => self.spsrs[2] = spsr,
            CpuMode::Irq => self.spsrs[3] = spsr,
            CpuMode::Undefined => self.spsrs[4] = spsr,
            CpuMode::Invalid => panic!("invalid mode"),
        }
    }

    pub fn set_mode_spsr(&mut self, mode: CpuMode, spsr: ProgramStatusRegister) {
        match mode {
            CpuMode::User | CpuMode::System => self.cpsr = spsr,
            CpuMode::Fiq => self.spsrs[0] = spsr,
            CpuMode::Supervisor => self.spsrs[1] = spsr,
            CpuMode::Abort => self.spsrs[2] = spsr,
            CpuMode::Irq => self.spsrs[3] = spsr,
            CpuMode::Undefined => self.spsrs[4] = spsr,
            CpuMode::Invalid => panic!("invalid mode"),
        }
    }

    pub fn bus(&mut self) -> &mut I {
        &mut self.bus
    }

    pub fn set_bus(&mut self, bus: I) {
        self.bus = bus
    }

    pub fn exception(&mut self, exception: Exception) {
        let (mode, disable_irq, disable_fiq) = match exception {
            Exception::Reset => (CpuMode::Supervisor, true, true),
            Exception::Undefined => (CpuMode::Undefined, true, false),
            Exception::SoftwareInterrupt => (CpuMode::Supervisor, true, false),
            Exception::Irq => (CpuMode::Irq, true, false),
            Exception::Fiq => (CpuMode::Fiq, true, true),
        };

        self.set_mode_spsr(mode, self.cpsr());
        self.set_mode(mode);
        if disable_irq {
            self.cpsr.set_irq_disable(true);
        }
        if disable_fiq {
            self.cpsr.set_fiq_disable(true);
        }

        let return_pc = match self.cpsr.state() {
            CpuState::Arm => self.pc() - 4,
            CpuState::Thumb => self.pc() - 2,
        };
        self.set_register(LR, return_pc);
        self.set_state(CpuState::Arm);
        self.set_pc(exception as u32);
        self.pipeline_flush();
    }

    pub fn reset(&mut self) {
        self.exception(Exception::Reset);
    }

    pub fn irq(&mut self) {
        if !self.cpsr.irq_disable() {
            self.exception(Exception::Irq);
        }
    }
}
