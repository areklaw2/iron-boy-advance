use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, cpu::SP, memory::MemoryInterface, thumb::AddOffsetToSp};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for AddOffsetToSp {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let offset = (self.offset() * 4) as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #SP as u32
        }
        emit_call(assembler, guest::register::<I> as *const ());

        match self.signed() {
            true => dynasm! { assembler
                ; .arch aarch64
                ; sub w2, w0, #offset
            },
            false => dynasm! { assembler
                ; .arch aarch64
                ; add w2, w0, #offset
            },
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #SP as u32
        }
        emit_call(assembler, guest::set_register::<I> as *const ());

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
