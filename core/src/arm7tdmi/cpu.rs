use crate::memory::MemoryInterface;

use super::{disassembler::CpuState, psr::ProgramStatusRegister};

pub trait Instruction {
    type Size;
    fn decode(value: Self::Size, address: u32) -> Self;
    fn disassable(&self) -> String;
    fn value(&self) -> Self::Size;
}

pub struct Arm7tdmiCpu<I: MemoryInterface> {
    general_registers: Vec<Vec<u32>>,
    pc: u32,
    cpsr: ProgramStatusRegister,
    spsrs: Vec<ProgramStatusRegister>,
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
    pub fn new(bus: I) -> Self {
        Arm7tdmiCpu {
            general_registers: build_general_registers(),
            pc: 0,
            cpsr: ProgramStatusRegister::new(0x0000_00D3), // TODO: figure out why this is the initial value
            spsrs: vec![ProgramStatusRegister::new(0x0000_00D3); 5],
            fetched_instruction: 0,
            decoded_instruction: 0,
            bus,
        }
    }

    pub fn cycle(&mut self) -> usize {
        let state = self.cpsr.cpu_state();
        // BX execution switches state
        // When executing an execution if cpu is in thumb it will switch to
        // arm execute arm and then switch back to thumb

        match state {
            CpuState::Arm => {
                let pc = self.pc & !0b11;
                let executed_instruction = self.decoded_instruction;
                self.decoded_instruction = self.fetched_instruction;
                self.fetched_instruction = self.bus.load_32(pc);

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

fn build_general_registers() -> Vec<Vec<u32>> {
    let mut general_registers = Vec::new();
    for i in 0..15 {
        match i {
            0..=7 => general_registers.push(vec![0; 1]),
            8..=12 => general_registers.push(vec![0; 2]),
            _ => general_registers.push(vec![0; 6]),
        }
    }
    general_registers
}
