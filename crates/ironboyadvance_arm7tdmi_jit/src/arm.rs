use crate::{Compile, MAX_BLOCK_INSTRUCTIONS, emit_action, emit_call, emit_epilogue, emit_prologue, guest};
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
    first: ArmInstruction,
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

        let Some(encoding) = cpu.peek_32(address.wrapping_add(4)) else {
            break;
        };
        let next = decode_arm(encoding);

        dynasm! { assembler
            ; .arch aarch64
            ; mov w1, w0
            ; mov x0, x19
        }
        emit_call(assembler, guest::fetch_next_arm::<I> as *const ());

        address = address.wrapping_add(4);
        instruction = next;
        count += 1;
    }

    emit_epilogue(assembler);
    start
}

pub fn decode_arm(value: u32) -> ArmInstruction {
    let pattern = value & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                ArmInstruction::BranchAndExchange(BranchAndExchange::new(value))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => ArmInstruction::MultiplyLong(MultiplyLong::new(value)),
                    false => ArmInstruction::Multiply(Multiply::new(value)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                ArmInstruction::SingleDataSwap(SingleDataSwap::new(value))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                ArmInstruction::HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(value))
            } else {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => ArmInstruction::Undefined(Undefined::new(value)),
            false => ArmInstruction::SingleDataTransfer(SingleDataTransfer::new(value)),
        },
        0b10 => match pattern.bit(25) {
            true => ArmInstruction::BranchAndBranchWithLink(BranchAndBranchWithLink::new(value)),
            false => ArmInstruction::BlockDataTransfer(BlockDataTransfer::new(value)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => ArmInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value)),
                false => match pattern.bit(4) {
                    true => ArmInstruction::CoprocessorRegisterTransfer(Undefined::new(value)),
                    false => ArmInstruction::CoprocessorDataOperation(Undefined::new(value)),
                },
            },
            false => ArmInstruction::CoprocessorDataTransfer(Undefined::new(value)),
        },
        _ => ArmInstruction::Undefined(Undefined::new(value)),
    }
}
