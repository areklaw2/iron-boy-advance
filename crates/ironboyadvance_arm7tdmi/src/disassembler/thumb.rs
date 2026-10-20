use crate::{Dissasemble, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::*};

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

impl Dissasemble for ThumbInstruction {
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
