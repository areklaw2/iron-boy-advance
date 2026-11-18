use crate::{Compile, MAX_BLOCK_INSTRUCTIONS, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};
use dynasmrt::{Assembler, AssemblyOffset, DynasmApi, DynasmLabelApi, aarch64::Aarch64Relocation, dynasm};
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
    first_encoding: u32,
) -> AssemblyOffset {
    let start = assembler.offset();
    emit_prologue(assembler);

    let start_address = address;
    let mut address = address;
    let mut encoding = first_encoding;
    let mut instruction = decode_thumb(encoding as u16);
    let mut count = 1;
    let show_logs = *cpu.show_logs();

    loop {
        if show_logs {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w2, #0
            }
            emit_immediate_32(assembler, 1, encoding);
            emit_call(assembler, guest::log_instruction::<I> as *const ());
        }

        instruction.compile::<I>(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        if instruction.ends_block() || count == MAX_BLOCK_INSTRUCTIONS {
            break;
        }

        let Some(next_encoding) = cpu.peek_16(address.wrapping_add(2)) else {
            break;
        };

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::should_exit_block::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; tbnz w0, #0, >block_exit
            ; mov x0, x19
            ; mov w1, w20
        }
        emit_call(assembler, guest::fetch_next_thumb::<I> as *const ());

        address = address.wrapping_add(2);
        encoding = next_encoding;
        instruction = decode_thumb(next_encoding as u16);
        count += 1;
    }

    cpu.watch_memory_range(start_address, count * 2);

    dynasm! { assembler
        ; .arch aarch64
        ; block_exit:
        ; mov w0, w20
    }
    emit_epilogue(assembler);
    start
}

pub fn decode_thumb(encoding: u16) -> ThumbInstruction {
    if encoding & 0xF800 < 0x1800 {
        ThumbInstruction::MoveShiftedRegister(MoveShiftedRegister::new(encoding))
    } else if encoding & 0xF800 == 0x1800 {
        ThumbInstruction::AddSubtract(AddSubtract::new(encoding))
    } else if encoding & 0xE000 == 0x2000 {
        ThumbInstruction::MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate::new(encoding))
    } else if encoding & 0xFC00 == 0x4000 {
        ThumbInstruction::AluOperations(AluOperations::new(encoding))
    } else if encoding & 0xFC00 == 0x4400 {
        ThumbInstruction::HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange::new(encoding))
    } else if encoding & 0xF800 == 0x4800 {
        ThumbInstruction::PcRelativeLoad(PcRelativeLoad::new(encoding))
    } else if encoding & 0xF200 == 0x5000 {
        ThumbInstruction::LoadStoreRegisterOffset(LoadStoreRegisterOffset::new(encoding))
    } else if encoding & 0xF200 == 0x5200 {
        ThumbInstruction::LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword::new(encoding))
    } else if encoding & 0xE000 == 0x6000 {
        ThumbInstruction::LoadStoreImmediateOffset(LoadStoreImmediateOffset::new(encoding))
    } else if encoding & 0xF000 == 0x8000 {
        ThumbInstruction::LoadStoreHalfword(LoadStoreHalfword::new(encoding))
    } else if encoding & 0xF000 == 0x9000 {
        ThumbInstruction::SpRelativeLoadStore(SpRelativeLoadStore::new(encoding))
    } else if encoding & 0xF000 == 0xA000 {
        ThumbInstruction::LoadAddress(LoadAddress::new(encoding))
    } else if encoding & 0xFF00 == 0xB000 {
        ThumbInstruction::AddOffsetToSp(AddOffsetToSp::new(encoding))
    } else if encoding & 0xF600 == 0xB400 {
        ThumbInstruction::PushPopRegisters(PushPopRegisters::new(encoding))
    } else if encoding & 0xF000 == 0xC000 {
        ThumbInstruction::MultipleLoadStore(MultipleLoadStore::new(encoding))
    } else if encoding & 0xFF00 < 0xDF00 {
        ThumbInstruction::ConditionalBranch(ConditionalBranch::new(encoding))
    } else if encoding & 0xFF00 == 0xDF00 {
        ThumbInstruction::SoftwareInterrupt(SoftwareInterrupt::new(encoding))
    } else if encoding & 0xF800 == 0xE000 {
        ThumbInstruction::UnconditionalBranch(UnconditionalBranch::new(encoding))
    } else if encoding & 0xF000 == 0xF000 {
        ThumbInstruction::LongBranchWithLink(LongBranchWithLink::new(encoding))
    } else {
        ThumbInstruction::Undefined(Undefined::new(encoding))
    }
}
