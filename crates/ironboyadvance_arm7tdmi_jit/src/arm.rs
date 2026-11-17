use crate::{
    Compile, MAX_BLOCK_INSTRUCTIONS, emit_action, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest,
};
use dynasmrt::{Assembler, AssemblyOffset, DynasmApi, DynasmLabelApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    Condition, CpuAction,
    arm::{
        ArmInstruction, BlockDataTransfer, BranchAndBranchWithLink, BranchAndExchange, DataProcessing,
        HalfwordAndSignedDataTransfer, Multiply, MultiplyLong, PsrTransfer, SingleDataSwap, SingleDataTransfer,
        SoftwareInterrupt, Undefined,
    },
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
};
use ironboyadvance_common::{bits::BitOps, memory::MemoryAccess};

mod block_data_transfer;
mod branch_and_branch_with_link;
mod branch_and_exchange;
mod data_processing;
mod halfword_and_signed_data_transfer;
mod multiply;
mod multiply_long;
mod psr_transfer;
mod single_data_swap;
mod single_data_transfer;
mod software_interrupt;
mod undefined;

impl Compile for ArmInstruction {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let condition = self.cond();

        if condition != Condition::AL {
            let condition = condition as u32;
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, #condition
            }
            emit_call(assembler, guest::is_condition_met::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; tbz w0, #0, >condition_skipped
            }
        }

        match self {
            Self::DataProcessing(i) => i.compile::<I>(assembler),
            Self::PsrTransfer(i) => i.compile::<I>(assembler),
            Self::Multiply(i) => i.compile::<I>(assembler),
            Self::MultiplyLong(i) => i.compile::<I>(assembler),
            Self::SingleDataSwap(i) => i.compile::<I>(assembler),
            Self::BranchAndExchange(i) => i.compile::<I>(assembler),
            Self::HalfwordAndSignedDataTransfer(i) => i.compile::<I>(assembler),
            Self::SingleDataTransfer(i) => i.compile::<I>(assembler),
            Self::Undefined(i) => i.compile::<I>(assembler),
            Self::BlockDataTransfer(i) => i.compile::<I>(assembler),
            Self::BranchAndBranchWithLink(i) => i.compile::<I>(assembler),
            Self::SoftwareInterrupt(i) => i.compile::<I>(assembler),
            Self::CoprocessorDataOperation(i) => i.compile::<I>(assembler),
            Self::CoprocessorDataTransfer(i) => i.compile::<I>(assembler),
            Self::CoprocessorRegisterTransfer(i) => i.compile::<I>(assembler),
        }

        if condition != Condition::AL {
            dynasm! { assembler
                ; .arch aarch64
                ; b >condition_done
                ; condition_skipped:
            }
            emit_action(
                assembler,
                CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
            );
            dynasm! { assembler
                ; .arch aarch64
                ; condition_done:
            }
        }
    }

    fn ends_block(&self) -> bool {
        match self {
            Self::DataProcessing(i) => i.ends_block(),
            Self::PsrTransfer(i) => i.ends_block(),
            Self::Multiply(i) => i.ends_block(),
            Self::MultiplyLong(i) => i.ends_block(),
            Self::SingleDataSwap(i) => i.ends_block(),
            Self::BranchAndExchange(i) => i.ends_block(),
            Self::HalfwordAndSignedDataTransfer(i) => i.ends_block(),
            Self::SingleDataTransfer(i) => i.ends_block(),
            Self::Undefined(i) => i.ends_block(),
            Self::BlockDataTransfer(i) => i.ends_block(),
            Self::BranchAndBranchWithLink(i) => i.ends_block(),
            Self::SoftwareInterrupt(i) => i.ends_block(),
            Self::CoprocessorDataOperation(i) => i.ends_block(),
            Self::CoprocessorDataTransfer(i) => i.ends_block(),
            Self::CoprocessorRegisterTransfer(i) => i.ends_block(),
        }
    }
}

pub(crate) fn compile_arm_block<I: MemoryInterface>(
    assembler: &mut Assembler<Aarch64Relocation>,
    cpu: &mut Arm7tdmiCpu<I>,
    address: u32,
    first_encoding: u32,
) -> AssemblyOffset {
    let start = assembler.offset();
    emit_prologue(assembler);

    let mut address = address;
    let mut encoding = first_encoding;
    let mut instruction = decode_arm(encoding);
    let mut count = 1;
    let show_logs = *cpu.show_logs();

    loop {
        if show_logs {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w2, #1
            }
            emit_immediate_32(assembler, 1, encoding);
            emit_call(assembler, guest::log_instruction::<I> as *const ());
        }

        instruction.compile::<I>(assembler);
        if instruction.ends_block() || count == MAX_BLOCK_INSTRUCTIONS {
            break;
        }

        let Some(next_encoding) = cpu.peek_32(address.wrapping_add(4)) else {
            break;
        };

        dynasm! { assembler
            ; .arch aarch64
            ; mov w1, w0
            ; mov x0, x19
        }
        emit_call(assembler, guest::fetch_next_arm::<I> as *const ());

        address = address.wrapping_add(4);
        encoding = next_encoding;
        instruction = decode_arm(next_encoding);
        count += 1;
    }

    emit_epilogue(assembler);
    start
}

pub fn decode_arm(encoding: u32) -> ArmInstruction {
    let pattern = encoding & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(encoding)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(encoding)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                ArmInstruction::BranchAndExchange(BranchAndExchange::new(encoding))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => ArmInstruction::MultiplyLong(MultiplyLong::new(encoding)),
                    false => ArmInstruction::Multiply(Multiply::new(encoding)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                ArmInstruction::SingleDataSwap(SingleDataSwap::new(encoding))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                ArmInstruction::HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(encoding))
            } else {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(encoding)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(encoding)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => ArmInstruction::Undefined(Undefined::new(encoding)),
            false => ArmInstruction::SingleDataTransfer(SingleDataTransfer::new(encoding)),
        },
        0b10 => match pattern.bit(25) {
            true => ArmInstruction::BranchAndBranchWithLink(BranchAndBranchWithLink::new(encoding)),
            false => ArmInstruction::BlockDataTransfer(BlockDataTransfer::new(encoding)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => ArmInstruction::SoftwareInterrupt(SoftwareInterrupt::new(encoding)),
                false => match pattern.bit(4) {
                    true => ArmInstruction::CoprocessorRegisterTransfer(Undefined::new(encoding)),
                    false => ArmInstruction::CoprocessorDataOperation(Undefined::new(encoding)),
                },
            },
            false => ArmInstruction::CoprocessorDataTransfer(Undefined::new(encoding)),
        },
        _ => ArmInstruction::Undefined(Undefined::new(encoding)),
    }
}
