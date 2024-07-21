use core::bus::{Bus, MemoryAccess};

use arm::ArmInstruction;
use dissassembler::{CpuMode, CpuState};
use psr::ProgramStatusRegister;

mod arm;
mod dissassembler;
mod psr;
mod thumb;

const SP: usize = 13;
const LR: usize = 14;

pub trait Instruction {
    type Size;
    fn decode(value: Self::Size, address: u32) -> Self;
    fn disassable(&self) -> String;
    fn value(&self) -> Self::Size;
}

pub struct Cpu {
    general_registers: Vec<Vec<u32>>,
    pc: u32,
    cpsr: ProgramStatusRegister,
    spsrs: Vec<ProgramStatusRegister>,
    fetched_instruction: u32,
    decoded_instruction: u32,
    bus: Bus,
}

impl MemoryAccess for Cpu {
    fn read_byte(&self, address: u32) -> u8 {
        self.bus.read_byte(address)
    }

    fn read_half_word(&self, address: u32) -> u16 {
        self.bus.read_half_word(address)
    }

    fn read_word(&self, address: u32) -> u32 {
        self.bus.read_word(address)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        self.bus.write_byte(address, value);
    }

    fn write_half_word(&mut self, address: u32, value: u16) {
        self.bus.write_half_word(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        self.bus.write_word(address, value);
    }
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu {
            general_registers: build_general_registers(),
            pc: 0,
            cpsr: ProgramStatusRegister::new(0),
            spsrs: vec![ProgramStatusRegister::new(0); 5],
            fetched_instruction: 0,
            decoded_instruction: 0,
            bus,
        }
    }

    pub fn cycle(&mut self) {
        let state = self.cpsr.state();
        match state {
            CpuState::Arm => {
                let pc = self.pc & !0b11;
                let executed_instruction = self.decoded_instruction;
                self.decoded_instruction = self.fetched_instruction;
                self.fetched_instruction = self.bus.read_word(pc);

                //decode and execute instruction
                self.arm_decode_and_execute(executed_instruction, pc);
                self.pc = self.pc.wrapping_add(4);
            }
            CpuState::Thumb => {
                let pc = self.pc & !0b01;
                self.pc = self.pc.wrapping_add(2);
            }
        }
    }
}

fn build_general_registers() -> Vec<Vec<u32>> {
    let mut general_registers = Vec::new();
    for i in 0..16 {
        match i {
            0..=7 => general_registers.push(vec![0; 1]),
            8..=12 => general_registers.push(vec![0; 2]),
            13 | 14 => general_registers.push(vec![0; 6]),
            _ => general_registers.push(vec![0; 1]),
        }
    }
    general_registers
}
