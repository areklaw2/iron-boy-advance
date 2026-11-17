use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::AddSubtract};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_immediate_32, guest};

impl Compile for AddSubtract {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rs = self.rs() as u32;
        let rn = self.rn() as u32;

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rs
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        match self.is_immediate() {
            true => emit_immediate_32(assembler, 21, self.offset() as u32),
            false => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #rn
                }
                emit_call(assembler, guest::register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w21, w0
                }
            }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #1
            ; mov w2, w20
            ; mov w3, w21
        }
        match self.opcode() {
            true => emit_call(assembler, guest::sub::<I> as *const ()),
            false => emit_call(assembler, guest::add::<I> as *const ()),
        }
        dynasm! { assembler
            ; .arch aarch64
            ; mov w2, w0
            ; mov x0, x19
            ; mov w1, #rd
        }
        emit_call(assembler, guest::set_register::<I> as *const ());
        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
        );
    }

    fn ends_block(&self) -> bool {
        false
    }
}
