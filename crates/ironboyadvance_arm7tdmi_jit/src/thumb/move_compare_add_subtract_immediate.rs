use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    CpuAction, MovCmpAddSubImmediateOpcode, memory::MemoryInterface, thumb::MoveCompareAddSubtractImmediate,
};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for MoveCompareAddSubtractImmediate {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        use MovCmpAddSubImmediateOpcode::*;
        let rd = self.rd() as u32;
        let offset = self.offset() as u32;
        let opcode: MovCmpAddSubImmediateOpcode = self.opcode().into();

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rd
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        match opcode {
            MOV => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w3, w0
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, #offset
                }
                emit_call(assembler, guest::mov::<I> as *const ());
            }
            CMP => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, #offset
                }
                emit_call(assembler, guest::cmp::<I> as *const ());
            }
            ADD => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, #offset
                }
                emit_call(assembler, guest::add::<I> as *const ());
            }
            SUB => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, #offset
                }
                emit_call(assembler, guest::sub::<I> as *const ());
            }
        }

        if opcode != CMP {
            dynasm! { assembler
                ; .arch aarch64
                ; mov w2, w0
                ; mov x0, x19
                ; mov w1, #rd
            }
            emit_call(assembler, guest::set_register::<I> as *const ());
        }

        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
        );
        emit_epilogue(assembler);
    }

    fn ends_block(&self) -> bool {
        false
    }
}
