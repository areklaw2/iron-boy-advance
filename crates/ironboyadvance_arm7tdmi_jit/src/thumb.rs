use crate::Compile;
use dynasmrt::{Assembler, aarch64::Aarch64Relocation};
use ironboyadvance_arm7tdmi::{
    memory::MemoryInterface,
    thumb::{
        AddOffsetToSp, AddSubtract, AluOperations, ConditionalBranch, HiRegisterOperationsBranchExchange, LoadAddress,
        LoadStoreHalfword, LoadStoreImmediateOffset, LoadStoreRegisterOffset, LoadStoreSignExtendedByteHalfword,
        LongBranchWithLink, MoveCompareAddSubtractImmediate, MoveShiftedRegister, MultipleLoadStore, PcRelativeLoad,
        PushPopRegisters, SoftwareInterrupt, SpRelativeLoadStore, ThumbInstruction, UnconditionalBranch, Undefined,
    },
};

mod add_offset_to_sp;
mod add_subtract;
mod alu_operations;
mod conditional_branch;
mod hi_register_operations_branch_exchange;
mod load_address;
mod load_store_halfword;
mod load_store_immediate_offset;
mod load_store_register_offset;
mod load_store_sign_extended_byte_halfword;
mod long_branch_with_link;
mod move_compare_add_subtract_immediate;
mod move_shifted_register;
mod multiple_load_store;
mod pc_relative_load;
mod push_pop_registers;
mod software_interrupt;
mod sp_relative_load_store;
mod unconditional_branch;
mod undefined;

impl Compile for ThumbInstruction {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        match self {
            Self::MoveShiftedRegister(i) => i.compile::<I>(assembler),
            Self::AddSubtract(i) => i.compile::<I>(assembler),
            Self::MoveCompareAddSubtractImmediate(i) => i.compile::<I>(assembler),
            Self::AluOperations(i) => i.compile::<I>(assembler),
            Self::HiRegisterOperationsBranchExchange(i) => i.compile::<I>(assembler),
            Self::PcRelativeLoad(i) => i.compile::<I>(assembler),
            Self::LoadStoreRegisterOffset(i) => i.compile::<I>(assembler),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.compile::<I>(assembler),
            Self::LoadStoreImmediateOffset(i) => i.compile::<I>(assembler),
            Self::LoadStoreHalfword(i) => i.compile::<I>(assembler),
            Self::SpRelativeLoadStore(i) => i.compile::<I>(assembler),
            Self::LoadAddress(i) => i.compile::<I>(assembler),
            Self::AddOffsetToSp(i) => i.compile::<I>(assembler),
            Self::PushPopRegisters(i) => i.compile::<I>(assembler),
            Self::MultipleLoadStore(i) => i.compile::<I>(assembler),
            Self::ConditionalBranch(i) => i.compile::<I>(assembler),
            Self::SoftwareInterrupt(i) => i.compile::<I>(assembler),
            Self::UnconditionalBranch(i) => i.compile::<I>(assembler),
            Self::LongBranchWithLink(i) => i.compile::<I>(assembler),
            Self::Undefined(i) => i.compile::<I>(assembler),
        }
    }
}

pub fn decode_thumb(value: u16) -> ThumbInstruction {
    if value & 0xF800 < 0x1800 {
        ThumbInstruction::MoveShiftedRegister(MoveShiftedRegister::new(value))
    } else if value & 0xF800 == 0x1800 {
        ThumbInstruction::AddSubtract(AddSubtract::new(value))
    } else if value & 0xE000 == 0x2000 {
        ThumbInstruction::MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate::new(value))
    } else if value & 0xFC00 == 0x4000 {
        ThumbInstruction::AluOperations(AluOperations::new(value))
    } else if value & 0xFC00 == 0x4400 {
        ThumbInstruction::HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange::new(value))
    } else if value & 0xF800 == 0x4800 {
        ThumbInstruction::PcRelativeLoad(PcRelativeLoad::new(value))
    } else if value & 0xF200 == 0x5000 {
        ThumbInstruction::LoadStoreRegisterOffset(LoadStoreRegisterOffset::new(value))
    } else if value & 0xF200 == 0x5200 {
        ThumbInstruction::LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword::new(value))
    } else if value & 0xE000 == 0x6000 {
        ThumbInstruction::LoadStoreImmediateOffset(LoadStoreImmediateOffset::new(value))
    } else if value & 0xF000 == 0x8000 {
        ThumbInstruction::LoadStoreHalfword(LoadStoreHalfword::new(value))
    } else if value & 0xF000 == 0x9000 {
        ThumbInstruction::SpRelativeLoadStore(SpRelativeLoadStore::new(value))
    } else if value & 0xF000 == 0xA000 {
        ThumbInstruction::LoadAddress(LoadAddress::new(value))
    } else if value & 0xFF00 == 0xB000 {
        ThumbInstruction::AddOffsetToSp(AddOffsetToSp::new(value))
    } else if value & 0xF600 == 0xB400 {
        ThumbInstruction::PushPopRegisters(PushPopRegisters::new(value))
    } else if value & 0xF000 == 0xC000 {
        ThumbInstruction::MultipleLoadStore(MultipleLoadStore::new(value))
    } else if value & 0xFF00 < 0xDF00 {
        ThumbInstruction::ConditionalBranch(ConditionalBranch::new(value))
    } else if value & 0xFF00 == 0xDF00 {
        ThumbInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value))
    } else if value & 0xF800 == 0xE000 {
        ThumbInstruction::UnconditionalBranch(UnconditionalBranch::new(value))
    } else if value & 0xF000 == 0xF000 {
        ThumbInstruction::LongBranchWithLink(LongBranchWithLink::new(value))
    } else {
        ThumbInstruction::Undefined(Undefined::new(value))
    }
}
