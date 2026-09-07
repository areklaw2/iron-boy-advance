use crate::{
    Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, trampoline_pc, trampoline_pipeline_flush,
    trampoline_register, trampoline_set_cpsr_state, trampoline_set_pc, trampoline_set_register,
};
use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    arm::{
        ArmInstruction, BlockDataTransfer, BranchAndBranchWithLink, BranchAndExchange, DataProcessing,
        HalfwordAndSignedDataTransfer, Multiply, MultiplyLong, PsrTransfer, SingleDataSwap, SingleDataTransfer,
        SoftwareInterrupt, Undefined,
    },
    cpu::LR,
    memory::MemoryInterface,
};
use ironboyadvance_common::bits::{BitOps, SignExtend};

impl Compile for ArmInstruction {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        match self {
            Self::DataProcessing(_i) => todo!(),
            Self::PsrTransfer(_i) => todo!(),
            Self::Multiply(_i) => todo!(),
            Self::MultiplyLong(_i) => todo!(),
            Self::SingleDataSwap(_i) => todo!(),
            Self::BranchAndExchange(i) => i.compile::<I>(assembler),
            Self::HalfwordAndSignedDataTransfer(_i) => todo!(),
            Self::SingleDataTransfer(_i) => todo!(),
            Self::Undefined(_i) => todo!(),
            Self::BlockDataTransfer(_i) => todo!(),
            Self::BranchAndBranchWithLink(i) => i.compile::<I>(assembler),
            Self::SoftwareInterrupt(_i) => todo!(),
            Self::CoprocessorDataOperation(_i) => todo!(),
            Self::CoprocessorDataTransfer(_i) => todo!(),
            Self::CoprocessorRegisterTransfer(_i) => todo!(),
        }
    }
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

impl Compile for BranchAndBranchWithLink {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let offset = (self.offset().sign_extend(24) << 2) as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
        }
        emit_call(assembler, trampoline_pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        if self.link() {
            let link_register = LR as u32;
            dynasm! { assembler
                ; .arch aarch64
                ; sub w2, w0, #4
                ; mov x0, x19
                ; movz w1, #link_register
            }
            emit_call(assembler, trampoline_set_register::<I> as *const ());
        }

        emit_immediate_32(assembler, 10, offset);
        dynasm! { assembler
            ; .arch aarch64
            ; add w1, w20, w10
            ; mov x0, x19
        }
        emit_call(assembler, trampoline_set_pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, trampoline_pipeline_flush::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; movn w0, #0
        }
        emit_epilogue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; ret
        }
    }
}

impl Compile for BranchAndExchange {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rn = self.rn() as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; movz w1, #rn
        }
        emit_call(assembler, trampoline_register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov x0, x19
            ; mov w1, w20
        }
        emit_call(assembler, trampoline_set_cpsr_state::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; and w1, w20, #0xfffffffe
            ; mov x0, x19
        }
        emit_call(assembler, trampoline_set_pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, trampoline_pipeline_flush::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; movn w0, #0
        }
        emit_epilogue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; ret
        }
    }
}

// impl Compile for SoftwareInterrupt {
//     fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
//         match !cpu.bios_loaded() && cpu.bios_call(self.comment() >> 16) {
//             true => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
//             false => {
//                 cpu.exception(Exception::SoftwareInterrupt);
//                 CpuAction::PipelineFlush
//             }
//         }
//     }
// }

// impl Compile for Undefined {
//     fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
//         cpu.exception(Exception::Undefined);
//         CpuAction::PipelineFlush
//     }
// }
