use crate::{Compile, MAX_BLOCK_INSTRUCTIONS, emit_call, emit_epilogue, emit_prologue, guest};
use dynasmrt::{Assembler, AssemblyOffset, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
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

    fn ends_block(&self) -> bool {
        match self {
            Self::MoveShiftedRegister(i) => i.ends_block(),
            Self::AddSubtract(i) => i.ends_block(),
            Self::MoveCompareAddSubtractImmediate(i) => i.ends_block(),
            Self::AluOperations(i) => i.ends_block(),
            Self::HiRegisterOperationsBranchExchange(i) => i.ends_block(),
            Self::PcRelativeLoad(i) => i.ends_block(),
            Self::LoadStoreRegisterOffset(i) => i.ends_block(),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.ends_block(),
            Self::LoadStoreImmediateOffset(i) => i.ends_block(),
            Self::LoadStoreHalfword(i) => i.ends_block(),
            Self::SpRelativeLoadStore(i) => i.ends_block(),
            Self::LoadAddress(i) => i.ends_block(),
            Self::AddOffsetToSp(i) => i.ends_block(),
            Self::PushPopRegisters(i) => i.ends_block(),
            Self::MultipleLoadStore(i) => i.ends_block(),
            Self::ConditionalBranch(i) => i.ends_block(),
            Self::SoftwareInterrupt(i) => i.ends_block(),
            Self::UnconditionalBranch(i) => i.ends_block(),
            Self::LongBranchWithLink(i) => i.ends_block(),
            Self::Undefined(i) => i.ends_block(),
        }
    }
}

pub(crate) fn compile_thumb_block<I: MemoryInterface>(
    assembler: &mut Assembler<Aarch64Relocation>,
    cpu: &mut Arm7tdmiCpu<I>,
    address: u32,
    first: ThumbInstruction,
) -> AssemblyOffset {
    let start = assembler.offset();
    emit_prologue(assembler);

    let mut address = address;
    let mut instruction = first;
    let mut count = 1;
    let max_instructions = match *cpu.show_logs() {
        true => 1,
        false => MAX_BLOCK_INSTRUCTIONS,
    };

    loop {
        instruction.compile::<I>(assembler);
        if instruction.ends_block() || count == max_instructions {
            break;
        }

        let Some(encoding) = cpu.peek_16(address.wrapping_add(2)) else {
            break;
        };
        let next = decode_thumb(encoding as u16);

        dynasm! { assembler
            ; .arch aarch64
            ; mov w1, w0
            ; mov x0, x19
        }
        emit_call(assembler, guest::fetch_next_thumb::<I> as *const ());

        address = address.wrapping_add(2);
        instruction = next;
        count += 1;
    }

    emit_epilogue(assembler);
    start
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
