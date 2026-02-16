use crate::{
    CpuAction,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::MemoryInterface,
    thumb::{
        add_offset_to_sp::AddOffsetToSp, add_subtract::AddSubtract, alu_operations::AluOperations,
        conditional_branch::ConditionalBranch, hi_register_operations_branch_exchange::HiRegisterOperationsBranchExchange,
        load_address::LoadAddress, load_store_halfword::LoadStoreHalfword,
        load_store_immediate_offset::LoadStoreImmediateOffset, load_store_register_offset::LoadStoreRegisterOffset,
        load_store_sign_extended_byte_halfword::LoadStoreSignExtendedByteHalfword,
        long_branch_with_link::LongBranchWithLink, move_compare_add_subtract_immediate::MoveCompareAddSubtractImmediate,
        move_shifted_register::MoveShiftedRegister, multiple_load_store::MultipleLoadStore,
        pc_relative_load::PcRelativeLoad, push_pop_registers::PushPopRegisters, software_interrupt::SoftwareInterrupt,
        sp_relative_load_store::SpRelativeLoadStore, unconditional_branch::UnconditionalBranch, undefined::Undefined,
    },
};

pub mod add_offset_to_sp;
pub mod add_subtract;
pub mod alu_operations;
pub mod conditional_branch;
pub mod hi_register_operations_branch_exchange;
pub mod load_address;
pub mod load_store_halfword;
pub mod load_store_immediate_offset;
pub mod load_store_register_offset;
pub mod load_store_sign_extended_byte_halfword;
pub mod long_branch_with_link;
pub mod move_compare_add_subtract_immediate;
pub mod move_shifted_register;
pub mod multiple_load_store;
pub mod pc_relative_load;
pub mod push_pop_registers;
pub mod software_interrupt;
pub mod sp_relative_load_store;
pub mod unconditional_branch;
pub mod undefined;

macro_rules! thumb_instruction {
    ($name:ident) => {
        impl $name {
            pub fn new(value: u16) -> Self {
                Self { value }
            }
        }
    };
}

pub(crate) use thumb_instruction;

pub type ThumbInstructionFactory = fn(u16) -> ThumbInstruction;

#[derive(Debug, Clone, Copy)]
pub enum ThumbInstruction {
    MoveShiftedRegister(MoveShiftedRegister),
    AddSubtract(AddSubtract),
    MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate),
    AluOperations(AluOperations),
    HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange),
    PcRelativeLoad(PcRelativeLoad),
    LoadStoreRegisterOffset(LoadStoreRegisterOffset),
    LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword),
    LoadStoreImmediateOffset(LoadStoreImmediateOffset),
    LoadStoreHalfword(LoadStoreHalfword),
    SpRelativeLoadStore(SpRelativeLoadStore),
    LoadAddress(LoadAddress),
    AddOffsetToSp(AddOffsetToSp),
    PushPopRegisters(PushPopRegisters),
    MultipleLoadStore(MultipleLoadStore),
    ConditionalBranch(ConditionalBranch),
    SoftwareInterrupt(SoftwareInterrupt),
    UnconditionalBranch(UnconditionalBranch),
    LongBranchWithLink(LongBranchWithLink),
    Undefined(Undefined),
}

impl Instruction for ThumbInstruction {
    #[inline]
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self {
            Self::MoveShiftedRegister(i) => i.execute(cpu),
            Self::AddSubtract(i) => i.execute(cpu),
            Self::MoveCompareAddSubtractImmediate(i) => i.execute(cpu),
            Self::AluOperations(i) => i.execute(cpu),
            Self::HiRegisterOperationsBranchExchange(i) => i.execute(cpu),
            Self::PcRelativeLoad(i) => i.execute(cpu),
            Self::LoadStoreRegisterOffset(i) => i.execute(cpu),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.execute(cpu),
            Self::LoadStoreImmediateOffset(i) => i.execute(cpu),
            Self::LoadStoreHalfword(i) => i.execute(cpu),
            Self::SpRelativeLoadStore(i) => i.execute(cpu),
            Self::LoadAddress(i) => i.execute(cpu),
            Self::AddOffsetToSp(i) => i.execute(cpu),
            Self::PushPopRegisters(i) => i.execute(cpu),
            Self::MultipleLoadStore(i) => i.execute(cpu),
            Self::ConditionalBranch(i) => i.execute(cpu),
            Self::SoftwareInterrupt(i) => i.execute(cpu),
            Self::UnconditionalBranch(i) => i.execute(cpu),
            Self::LongBranchWithLink(i) => i.execute(cpu),
            Self::Undefined(i) => i.execute(cpu),
        }
    }

    #[inline]
    fn disassemble<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> String {
        match self {
            Self::MoveShiftedRegister(i) => i.disassemble(cpu),
            Self::AddSubtract(i) => i.disassemble(cpu),
            Self::MoveCompareAddSubtractImmediate(i) => i.disassemble(cpu),
            Self::AluOperations(i) => i.disassemble(cpu),
            Self::HiRegisterOperationsBranchExchange(i) => i.disassemble(cpu),
            Self::PcRelativeLoad(i) => i.disassemble(cpu),
            Self::LoadStoreRegisterOffset(i) => i.disassemble(cpu),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.disassemble(cpu),
            Self::LoadStoreImmediateOffset(i) => i.disassemble(cpu),
            Self::LoadStoreHalfword(i) => i.disassemble(cpu),
            Self::SpRelativeLoadStore(i) => i.disassemble(cpu),
            Self::LoadAddress(i) => i.disassemble(cpu),
            Self::AddOffsetToSp(i) => i.disassemble(cpu),
            Self::PushPopRegisters(i) => i.disassemble(cpu),
            Self::MultipleLoadStore(i) => i.disassemble(cpu),
            Self::ConditionalBranch(i) => i.disassemble(cpu),
            Self::SoftwareInterrupt(i) => i.disassemble(cpu),
            Self::UnconditionalBranch(i) => i.disassemble(cpu),
            Self::LongBranchWithLink(i) => i.disassemble(cpu),
            Self::Undefined(i) => i.disassemble(cpu),
        }
    }
}

pub fn generate_thumb_lut() -> [ThumbInstructionFactory; 1024] {
    let mut thumb_lut: [ThumbInstructionFactory; 1024] = [|value| ThumbInstruction::Undefined(Undefined::new(value)); 1024];
    for (i, factory) in thumb_lut.iter_mut().enumerate() {
        *factory = decode_thumb((i as u16) << 6);
    }
    thumb_lut
}

fn decode_thumb(instruction: u16) -> ThumbInstructionFactory {
    if instruction & 0xF800 < 0x1800 {
        |value| ThumbInstruction::MoveShiftedRegister(MoveShiftedRegister::new(value))
    } else if instruction & 0xF800 == 0x1800 {
        |value| ThumbInstruction::AddSubtract(AddSubtract::new(value))
    } else if instruction & 0xE000 == 0x2000 {
        |value| ThumbInstruction::MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate::new(value))
    } else if instruction & 0xFC00 == 0x4000 {
        |value| ThumbInstruction::AluOperations(AluOperations::new(value))
    } else if instruction & 0xFC00 == 0x4400 {
        |value| ThumbInstruction::HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange::new(value))
    } else if instruction & 0xF800 == 0x4800 {
        |value| ThumbInstruction::PcRelativeLoad(PcRelativeLoad::new(value))
    } else if instruction & 0xF200 == 0x5000 {
        |value| ThumbInstruction::LoadStoreRegisterOffset(LoadStoreRegisterOffset::new(value))
    } else if instruction & 0xF200 == 0x5200 {
        |value| ThumbInstruction::LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword::new(value))
    } else if instruction & 0xE000 == 0x6000 {
        |value| ThumbInstruction::LoadStoreImmediateOffset(LoadStoreImmediateOffset::new(value))
    } else if instruction & 0xF000 == 0x8000 {
        |value| ThumbInstruction::LoadStoreHalfword(LoadStoreHalfword::new(value))
    } else if instruction & 0xF000 == 0x9000 {
        |value| ThumbInstruction::SpRelativeLoadStore(SpRelativeLoadStore::new(value))
    } else if instruction & 0xF000 == 0xA000 {
        |value| ThumbInstruction::LoadAddress(LoadAddress::new(value))
    } else if instruction & 0xFF00 == 0xB000 {
        |value| ThumbInstruction::AddOffsetToSp(AddOffsetToSp::new(value))
    } else if instruction & 0xF600 == 0xB400 {
        |value| ThumbInstruction::PushPopRegisters(PushPopRegisters::new(value))
    } else if instruction & 0xF000 == 0xC000 {
        |value| ThumbInstruction::MultipleLoadStore(MultipleLoadStore::new(value))
    } else if instruction & 0xFF00 < 0xDF00 {
        |value| ThumbInstruction::ConditionalBranch(ConditionalBranch::new(value))
    } else if instruction & 0xFF00 == 0xDF00 {
        |value| ThumbInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value))
    } else if instruction & 0xF800 == 0xE000 {
        |value| ThumbInstruction::UnconditionalBranch(UnconditionalBranch::new(value))
    } else if instruction & 0xF000 == 0xF000 {
        |value| ThumbInstruction::LongBranchWithLink(LongBranchWithLink::new(value))
    } else {
        |value| ThumbInstruction::Undefined(Undefined::new(value))
    }
}
