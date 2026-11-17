use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, cpu::SP, memory::MemoryInterface, thumb::LoadAddress};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, guest};

impl Compile for LoadAddress {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let offset = (self.offset() * 4) as u32;

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }

        match self.sp() {
            true => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w1, #SP as u32
                }
                emit_call(assembler, guest::register::<I> as *const ());
            }
            false => {
                emit_call(assembler, guest::pc::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; and w0, w0, #0xfffffffd
                }
            }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; add w2, w0, #offset
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
