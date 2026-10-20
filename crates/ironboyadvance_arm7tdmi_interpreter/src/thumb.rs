use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::Arm7tdmiCpu,
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

impl Execute for ThumbInstruction {
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
}

pub type ThumbInstructionFactory = fn(u16) -> ThumbInstruction;

pub(crate) fn generate_thumb_lut() -> [ThumbInstructionFactory; 1024] {
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
