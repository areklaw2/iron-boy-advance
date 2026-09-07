use getset::CopyGetters;
use ironboyadvance_common::bits::BitOps;

use crate::{
    Condition, CpuMode, DataProcessingOpcode, Dissasemble, Register,
    barrel_shifter::{ShiftBy, ShiftType},
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
};

#[derive(Debug, Clone, Copy)]
pub enum ArmInstruction {
    DataProcessing(DataProcessing),
    PsrTransfer(PsrTransfer),
    Multiply(Multiply),
    MultiplyLong(MultiplyLong),
    SingleDataSwap(SingleDataSwap),
    BranchAndExchange(BranchAndExchange),
    HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer),
    SingleDataTransfer(SingleDataTransfer),
    Undefined(Undefined),
    BlockDataTransfer(BlockDataTransfer),
    BranchAndBranchWithLink(BranchAndBranchWithLink),
    SoftwareInterrupt(SoftwareInterrupt),
    CoprocessorDataOperation(Undefined),
    CoprocessorDataTransfer(Undefined),
    CoprocessorRegisterTransfer(Undefined),
}

impl ArmInstruction {
    pub fn cond(&self) -> Condition {
        match self {
            Self::DataProcessing(i) => i.cond(),
            Self::PsrTransfer(i) => i.cond(),
            Self::Multiply(i) => i.cond(),
            Self::MultiplyLong(i) => i.cond(),
            Self::SingleDataSwap(i) => i.cond(),
            Self::BranchAndExchange(i) => i.cond(),
            Self::HalfwordAndSignedDataTransfer(i) => i.cond(),
            Self::SingleDataTransfer(i) => i.cond(),
            Self::Undefined(i) => i.cond(),
            Self::BlockDataTransfer(i) => i.cond(),
            Self::BranchAndBranchWithLink(i) => i.cond(),
            Self::SoftwareInterrupt(i) => i.cond(),
            Self::CoprocessorDataOperation(i) => i.cond(),
            Self::CoprocessorDataTransfer(i) => i.cond(),
            Self::CoprocessorRegisterTransfer(i) => i.cond(),
        }
    }
}

impl Dissasemble for ArmInstruction {
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self {
            Self::DataProcessing(i) => i.disassemble(cpu),
            Self::PsrTransfer(i) => i.disassemble(cpu),
            Self::Multiply(i) => i.disassemble(cpu),
            Self::MultiplyLong(i) => i.disassemble(cpu),
            Self::SingleDataSwap(i) => i.disassemble(cpu),
            Self::BranchAndExchange(i) => i.disassemble(cpu),
            Self::HalfwordAndSignedDataTransfer(i) => i.disassemble(cpu),
            Self::SingleDataTransfer(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
            Self::BlockDataTransfer(i) => i.disassemble(cpu),
            Self::BranchAndBranchWithLink(i) => i.disassemble(cpu),
            Self::SoftwareInterrupt(i) => i.disassemble(cpu),
            Self::CoprocessorDataOperation(i) => i.disassemble(cpu),
            Self::CoprocessorDataTransfer(i) => i.disassemble(cpu),
            Self::CoprocessorRegisterTransfer(i) => i.disassemble(cpu),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BlockDataTransfer {
    cond: Condition,
    rn: Register,
    pre_index: bool,
    add: bool,
    write_back: bool,
    load: bool,
    load_psr_force_user: bool,
    register_list: u16,
}

impl BlockDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            pre_index: value.bit(24),
            add: value.bit(23),
            write_back: value.bit(21),
            load: value.bit(20),
            load_psr_force_user: value.bit(22),
            register_list: value as u16,
        }
    }
}

impl Dissasemble for BlockDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let pre_index = self.pre_index;
        let add = self.add;
        let load_psr_force_user = if self.load_psr_force_user { "^" } else { "" };
        let write_back = if self.write_back { "!" } else { "" };
        let load = self.load;
        let rn = self.rn;
        let register_list = (0..16u32)
            .filter(|&i| (self.register_list >> i) & 1 == 1)
            .map(|i| Register::from(i).to_string())
            .collect::<Vec<_>>()
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
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BranchAndBranchWithLink {
    cond: Condition,
    link: bool,
    offset: u32,
}

impl BranchAndBranchWithLink {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            link: value.bit(24),
            offset: value.bits(0..=23),
        }
    }
}

impl Dissasemble for BranchAndBranchWithLink {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let link = if self.link { "L" } else { "" };
        let expression = self.offset;
        format!("B{link}{cond} 0x{expression:08X}")
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BranchAndExchange {
    cond: Condition,
    rn: Register,
}

impl BranchAndExchange {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(0..=3).into(),
        }
    }
}

impl Dissasemble for BranchAndExchange {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let rn = self.rn;
        format!("BX{cond} {rn}")
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct DataProcessing {
    cond: Condition,
    rn: Register,
    rm: Register,
    rs: Register,
    rd: Register,
    is_immediate: bool,
    opcode: DataProcessingOpcode,
    sets_flags: bool,
    shift_by: ShiftBy,
    shift_amount: u32,
    shift_type: ShiftType,
    rotate: u32,
    immediate: u32,
}

impl DataProcessing {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            rd: value.bits(12..=15).into(),
            is_immediate: value.bit(25),
            opcode: value.bits(21..=24).into(),
            sets_flags: value.bit(20),
            shift_by: match value.bit(4) {
                true => ShiftBy::Register,
                false => ShiftBy::Immediate,
            },
            shift_amount: value.bits(7..=11),
            shift_type: value.bits(5..=6).into(),
            rotate: value.bits(8..=11),
            immediate: value.bits(0..=7),
        }
    }
}

impl Dissasemble for DataProcessing {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        use DataProcessingOpcode::*;

        let cond = self.cond;
        let opcode = self.opcode;
        let s = if self.sets_flags { "S" } else { "" };
        let rd = self.rd;
        let rn = self.rn;
        let operand_2 = match self.is_immediate {
            true => {
                let rotate = 2 * self.rotate;
                let immediate = self.immediate;
                format!("0x{:08X}", immediate.rotate_right(rotate))
            }
            false => {
                let rm = self.rm;
                let shift_type = self.shift_type;
                match self.shift_by {
                    ShiftBy::Register => {
                        format!("{},{} {}", rm, shift_type, self.rs)
                    }
                    ShiftBy::Immediate => {
                        format!("{},{} #{}", rm, shift_type, self.shift_amount)
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
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct HalfwordAndSignedDataTransfer {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    immediate_hi: u32,
    immediate_lo: u32,
    pre_index: bool,
    add: bool,
    write_back: bool,
    load: bool,
    signed: bool,
    halfword: bool,
}

impl HalfwordAndSignedDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: value.bit(22),
            immediate_hi: value.bits(8..=11),
            immediate_lo: value.bits(0..=3),
            pre_index: value.bit(24),
            add: value.bit(23),
            write_back: value.bit(21),
            load: value.bit(20),
            signed: value.bit(6),
            halfword: value.bit(5),
        }
    }
}

impl Dissasemble for HalfwordAndSignedDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let pre_index = self.pre_index;
        let add = if self.add { "+" } else { "-" };
        let rn = self.rn;
        let rd = self.rd;
        let immediate = self.immediate_hi << 4 | self.immediate_lo;
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let rm = self.rm;
                let offset = match self.is_immediate {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => format!(",{}{}", add, rm),
                };

                let write_back = if self.write_back && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        let s = self.signed;
        let h = self.halfword;
        let sh = match (s, h) {
            (false, false) => "",
            (false, true) => "H",
            (true, false) => "SB",
            (true, true) => "SH",
        };

        match self.load {
            true => format!("LDR{}{} {},{}", cond, sh, rd, address),
            false => format!("STR{}{} {},{}", cond, sh, rd, address),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MultiplyLong {
    cond: Condition,
    rd_hi: Register,
    rd_lo: Register,
    rm: Register,
    rs: Register,
    sets_flags: bool,
    accumulate: bool,
    unsigned: bool,
}

impl MultiplyLong {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rd_hi: value.bits(16..=19).into(),
            rd_lo: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            sets_flags: value.bit(20),
            accumulate: value.bit(21),
            unsigned: value.bit(22),
        }
    }
}

impl Dissasemble for MultiplyLong {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let s = if self.sets_flags { "S" } else { "" };
        let rd_hi = self.rd_hi;
        let rd_lo = self.rd_lo;
        let rm = self.rm;
        let rs = self.rs;
        let unsigned = self.unsigned;
        let accumulate = self.accumulate;
        match (unsigned, accumulate) {
            (true, false) => format!("UMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (true, true) => format!("UMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, false) => format!("SMULL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
            (false, true) => format!("SMLAL{}{} {},{},{},{}", cond, s, rd_lo, rd_hi, rm, rs),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Multiply {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    rs: Register,
    sets_flags: bool,
    accumulate: bool,
}

impl Multiply {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(12..=15).into(),
            rd: value.bits(16..=19).into(),
            rm: value.bits(0..=3).into(),
            rs: value.bits(8..=11).into(),
            sets_flags: value.bit(20),
            accumulate: value.bit(21),
        }
    }
}

impl Dissasemble for Multiply {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let s = if self.sets_flags { "S" } else { "" };
        let rd = self.rd;
        let rm = self.rm;
        let rs = self.rs;
        let rn = self.rn;
        match self.accumulate {
            true => format!("MLA{}{} {},{},{},{}", cond, s, rd, rm, rs, rn),
            false => format!("MUL{}{} {},{},{}", cond, s, rd, rm, rs),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PsrTransfer {
    cond: Condition,
    is_mrs: bool,
    psr_mask: u32,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    rotate: u32,
    immediate: u32,
    is_spsr: bool,
}

impl PsrTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        let field_bits = value.bits(16..=19);
        let mut psr_mask = 0u32;
        if field_bits & 0b1000 != 0 {
            psr_mask |= 0xFF000000;
        }
        if field_bits & 0b0100 != 0 {
            psr_mask |= 0xFF0000;
        }
        if field_bits & 0b0010 != 0 {
            psr_mask |= 0xFF00;
        }
        if field_bits & 0b0001 != 0 {
            psr_mask |= 0xFF;
        }

        Self {
            cond: value.bits(28..=31).into(),
            is_mrs: value.bits(16..=21) as u8 == 0xF,
            psr_mask,
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: value.bit(25),
            rotate: value.bits(8..=11),
            immediate: value.bits(0..=7),
            is_spsr: value.bit(22),
        }
    }
}

impl Dissasemble for PsrTransfer {
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let is_spsr = self.is_spsr;
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

        match self.is_mrs {
            true => {
                let rd = self.rd as usize;
                format!("MRS{} {},{}", cond, rd, psr)
            }
            false => {
                let operand = match self.is_immediate {
                    false => format!("{}", self.rm),
                    true => {
                        let rotate = 2 * self.rotate;
                        let immediate = self.immediate;
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
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SingleDataSwap {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    byte: bool,
}

impl SingleDataSwap {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            byte: value.bit(22),
        }
    }
}

impl Dissasemble for SingleDataSwap {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let byte = if self.byte { "B" } else { "" };
        let rd = self.rd;
        let rm = self.rm;
        let rn = self.rn;
        format!("SWP{}{} {},{},[{}]", cond, byte, rd, rm, rn)
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SingleDataTransfer {
    cond: Condition,
    rn: Register,
    rd: Register,
    rm: Register,
    is_immediate: bool,
    shift_amount: u32,
    shift_type: ShiftType,
    immediate: u32,
    pre_index: bool,
    add: bool,
    byte: bool,
    write_back: bool,
    load: bool,
}

impl SingleDataTransfer {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            rn: value.bits(16..=19).into(),
            rd: value.bits(12..=15).into(),
            rm: value.bits(0..=3).into(),
            is_immediate: !value.bit(25),
            shift_amount: value.bits(7..=11),
            shift_type: value.bits(5..=6).into(),
            immediate: value.bits(0..=11),
            pre_index: value.bit(24),
            add: value.bit(23),
            byte: value.bit(22),
            write_back: value.bit(21),
            load: value.bit(20),
        }
    }
}

impl Dissasemble for SingleDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let pre_index = self.pre_index;
        let t = if pre_index { "" } else { "T" };
        let add = if self.add { "+" } else { "-" };
        let byte = if self.byte { "B" } else { "" };
        let rn = self.rn;
        let rd = self.rd;
        let immediate = self.immediate;
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let offset = match self.is_immediate {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => {
                        let rm = self.rm;
                        let shift_type = self.shift_type;
                        format!(",{}{},{} #{}", add, rm, shift_type, self.shift_amount)
                    }
                };

                let write_back = if self.write_back && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        match self.load {
            true => format!("LDR{}{}{} {},{}", cond, byte, t, rd, address),
            false => format!("STR{}{}{} {},{}", cond, byte, t, rd, address),
        }
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SoftwareInterrupt {
    cond: Condition,
    comment: u32,
}

impl SoftwareInterrupt {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
            comment: value.bits(0..=23),
        }
    }
}

impl Dissasemble for SoftwareInterrupt {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond;
        let comment = self.comment;
        format!("SWI{} 0x{:08X}", cond, comment)
    }
}

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Undefined {
    cond: Condition,
}

impl Undefined {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self {
            cond: value.bits(28..=31).into(),
        }
    }
}

impl Dissasemble for Undefined {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        "Undefined".into()
    }
}
