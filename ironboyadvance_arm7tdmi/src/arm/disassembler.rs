use bitvec::field::BitField;

use crate::{CpuMode, Register, alu::AluInstruction, barrel_shifter::ShiftBy, cpu::Arm7tdmiCpu, memory::MemoryInterface};

use super::ArmInstruction;

pub fn disassemble_branch_exchange(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let rn = instruction.rn();
    format!("BX{cond} {rn}")
}

pub fn disassemble_branch_and_branch_link(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let link = if instruction.link() { "L" } else { "" };
    let expression = instruction.offset();
    format!("B{link}{cond} 0x{expression:08X}")
}

pub fn disassemble_data_processing(instruction: &ArmInstruction) -> String {
    use AluInstruction::*;
    let cond = instruction.cond();
    let opcode = instruction.opcode();
    let s = if instruction.sets_flags() { "S" } else { "" };
    let rd = instruction.rd();
    let rn = instruction.rn();
    let operand_2 = match instruction.is_immediate() {
        true => {
            let rotate = 2 * instruction.rotate();
            let immediate = instruction.immediate();
            format!("0x{:08X}", immediate.rotate_right(rotate))
        }
        false => {
            let rm = instruction.rm();
            let shift_type = instruction.shift_type();
            match instruction.shift_by() {
                ShiftBy::Register => {
                    format!("{},{} {}", rm, shift_type, instruction.rs())
                }
                ShiftBy::Immediate => {
                    format!("{},{} #{}", rm, shift_type, instruction.shift_amount())
                }
            }
        }
    };

    match opcode {
        MOV | MVN => format!("{opcode}{cond}{s} {rd},{operand_2}"),
        CMP | CMN | TEQ | TST => format!("{opcode}{cond} {rn},{operand_2}"),
        _ => format!("{opcode}{cond}{s} {rd},{rn},{operand_2}"),
    }
}

pub fn disassemble_psr_transfer<I: MemoryInterface>(cpu: &mut Arm7tdmiCpu<I>, instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let is_spsr = instruction.is_spsr();
    let psr = match is_spsr {
        false => "CPSR",
        true => match cpu.cpsr().mode() {
            CpuMode::User | CpuMode::System => "CPSR",
            CpuMode::Fiq => "SPSR_fiq",
            CpuMode::Supervisor => "SPSR_svc",
            CpuMode::Abort => "SPSR_abt",
            CpuMode::Irq => "SPSR_irq",
            CpuMode::Undefined => "SPSR_und",
            CpuMode::Invalid => panic!("invalid mode"),
        },
    };

    match instruction.bits[16..=21].load::<u8>() == 0xF {
        true => {
            let rd = instruction.rd() as usize;
            return format!("MRS{} {},{}", cond, rd, psr);
        }
        false => {
            let operand = match instruction.is_immediate() {
                false => format!("{}", instruction.rm()),
                true => {
                    let rotate = 2 * instruction.rotate();
                    let immediate = instruction.immediate();
                    let expression = immediate.rotate_right(rotate);
                    format!("0x{:08X}", expression)
                }
            };

            match is_spsr {
                false => {
                    if cpu.cpsr().mode() == CpuMode::User {
                        return format!("MSR{} {}_flg,{}", cond, psr, operand);
                    }
                    format!("MSR{} {}_all,{}", cond, psr, operand)
                }
                true => format!("MSR{} {},{}", cond, psr, operand),
            }
        }
    }
}

pub fn disassemble_multiply(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let s = if instruction.sets_flags() { "S" } else { "" };
    let rd = instruction.rd();
    let rm = instruction.rm();
    let rs = instruction.rs();
    let rn = instruction.rn();
    match instruction.accumulate() {
        true => format!("MLA{}{} {},{},{},{}", cond, s, rd, rm, rs, rn),
        false => format!("MUL{}{} {},{},{}", cond, s, rd, rm, rs),
    }
}

pub fn disassemble_multiply_long(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let s = if instruction.sets_flags() { "S" } else { "" };
    let rd_hi = instruction.rd_hi();
    let rd_lo = instruction.rd_lo();
    let rm = instruction.rm();
    let rs = instruction.rs();
    let unsigned = instruction.unsigned();
    let accumulate = instruction.accumulate();
    match (unsigned, accumulate) {
        (true, false) => format!("UMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        (true, true) => format!("UMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        (false, false) => format!("SMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        (false, true) => format!("SMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
    }
}

pub fn disassemble_single_data_transfer(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let pre_index = instruction.pre_index();
    let t = if pre_index { "" } else { "T" };
    let add = if instruction.add() { "+" } else { "-" };
    let byte = if instruction.byte() { "B" } else { "" };
    let rn = instruction.rn();
    let rd = instruction.rd();
    let immediate = instruction.immediate();
    let address = match rd as usize == 15 {
        true => format!("#{:08X}", immediate),
        false => {
            let offset = match instruction.is_immediate() {
                true => match immediate {
                    0 => "".into(),
                    _ => format!(",#{}{}", add, immediate),
                },
                false => {
                    let rm = instruction.rm();
                    let shift_type = instruction.shift_type();
                    format!(",{}{},{} #{}", add, rm, shift_type, instruction.shift_amount())
                }
            };

            let write_back = if instruction.write_back() && offset != "" { "!" } else { "" };
            match pre_index {
                true => format!("[{}{}]{}", rn, offset, write_back),
                false => format!("[{}]{}", rn, offset),
            }
        }
    };

    match instruction.load() {
        true => format!("LDR{}{}{} {},{}", cond, byte, t, rd, address),
        false => format!("STR{}{}{} {},{}", cond, byte, t, rd, address),
    }
}

pub fn disassemble_halfword_and_signed_data_transfer(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let pre_index = instruction.pre_index();
    let add = if instruction.add() { "+" } else { "-" };
    let rn = instruction.rn();
    let rd = instruction.rd();
    let immediate = instruction.immediate_hi() << 4 | instruction.immediate_lo();
    let address = match rd as usize == 15 {
        true => format!("#{:08X}", immediate),
        false => {
            let rm = instruction.rm();
            let offset = match instruction.is_immediate() {
                true => match immediate {
                    0 => "".into(),
                    _ => format!(",#{}{}", add, immediate),
                },
                false => format!(",{}{}", add, rm),
            };

            let write_back = if instruction.write_back() && offset != "" { "!" } else { "" };
            match pre_index {
                true => format!("[{}{}]{}", rn, offset, write_back),
                false => format!("[{}]{}", rn, offset),
            }
        }
    };

    let s = instruction.signed();
    let h = instruction.halfword();
    let sh = match (s, h) {
        (false, false) => "",
        (false, true) => "H",
        (true, false) => "SB",
        (true, true) => "SH",
    };

    match instruction.load() {
        true => format!("LDR{}{} {},{}", cond, sh, rd, address),
        false => format!("STR{}{} {},{}", cond, sh, rd, address),
    }
}

pub fn disassemble_block_data_transfer(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let pre_index = instruction.pre_index();
    let add = instruction.add();
    let load_psr_force_user = if instruction.load_psr_force_user() { "^" } else { "" };
    let write_back = if instruction.write_back() { "!" } else { "" };
    let load = instruction.load();
    let rn = instruction.rn();
    let register_list = instruction
        .register_list()
        .iter()
        .map(|register| Register::from(*register as u32).to_string())
        .collect::<Vec<String>>()
        .join(",");

    let mnemonic = match (load, pre_index, add) {
        (true, true, true) => match rn == Register::R13 {
            true => format!("LDM{}ED", cond),
            false => format!("LDM{}IB", cond),
        },
        (true, false, true) => match rn == Register::R13 {
            true => format!("LDM{}FD", cond),
            false => format!("LDM{}IA", cond),
        },
        (true, true, false) => match rn == Register::R13 {
            true => format!("LDM{}EA", cond),
            false => format!("LDM{}DB", cond),
        },
        (true, false, false) => match rn == Register::R13 {
            true => format!("LDM{}FA", cond),
            false => format!("LDM{}DA", cond),
        },
        (false, true, true) => match rn == Register::R13 {
            true => format!("STM{}FA", cond),
            false => format!("STM{}IB", cond),
        },
        (false, false, true) => match rn == Register::R13 {
            true => format!("STM{}EA", cond),
            false => format!("STM{}IA", cond),
        },
        (false, true, false) => match rn == Register::R13 {
            true => format!("STM{}FD", cond),
            false => format!("STM{}DB", cond),
        },
        (false, false, false) => match rn == Register::R13 {
            true => format!("STM{}ED", cond),
            false => format!("STM{}DA", cond),
        },
    };

    format!("{} {}{},({}){}", mnemonic, rn, write_back, register_list, load_psr_force_user)
}

pub fn disassemble_single_data_swap(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let byte = if instruction.byte() { "B" } else { "" };
    let rd = instruction.rd();
    let rm = instruction.rm();
    let rn = instruction.rn();
    format!("SWP{}{} {},{},[{}]", cond, byte, rd, rm, rn)
}

pub fn disassemble_software_interrupt(instruction: &ArmInstruction) -> String {
    let cond = instruction.cond();
    let comment = instruction.comment();
    format!("SWI{} 0x{:08X}", cond, comment)
}

pub fn disassemble_undefined(_instruction: &ArmInstruction) -> String {
    format!("Undefined")
}
