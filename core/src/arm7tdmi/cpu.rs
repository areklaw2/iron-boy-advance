use crate::memory::MemoryInterface;

use super::{
    disassembler::{CpuMode, CpuState},
    psr::ProgramStatusRegister,
};

const SP_INDEX: usize = 13;
const LR_INDEX: usize = 14;
const PC_INDEX: usize = 15;

pub trait Instruction {
    type Size;
    fn decode(value: Self::Size, address: u32) -> Self;
    fn disassable(&self) -> String;
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
        };

        match skip_bios {
            true => {
                cpu.general_registers_irq[0] = 0x3007FA0;
                cpu.general_registers_svc[0] = 0x3007FE0;

                cpu.cpsr.set_cpu_mode(CpuMode::System);
                cpu.cpsr.set_irq_disable(false);
                cpu.general_registers[SP_INDEX] = 0x03007F00;
                cpu.general_registers[LR_INDEX] = 0x08000000;
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

    pub fn cycle(&mut self) -> usize {
        match self.cpsr.cpu_state() {
            CpuState::Arm => {
                let pc = self.pc & !0b11;
                let executed_instruction = self.decoded_instruction;
                self.decoded_instruction = self.fetched_instruction;
                self.fetched_instruction = self.load_32(pc);

                //decode and execute instruction
                self.arm_decode_and_execute(executed_instruction, pc);
                self.pc = self.pc.wrapping_add(4);
            }
            CpuState::Thumb => {
                let pc = self.pc & !0b01;
                self.pc = self.pc.wrapping_add(2);
            }
        }

        0
    }
}
