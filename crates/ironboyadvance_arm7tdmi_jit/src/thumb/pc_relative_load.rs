use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, cpu::PC, memory::MemoryInterface, thumb::PcRelativeLoad};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, guest};

impl Compile for PcRelativeLoad {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let offset = (self.offset() << 2) as u32;
        let rd = self.rd() as u32;
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #PC as u32
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; and w0, w0, #0xfffffffd
            ; add w0, w0, #offset
            ; mov w1, w0
            ; mov x0, x19
            ; mov w2, #0
        }
        emit_call(assembler, guest::load_32::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w2, w0
            ; mov x0, x19
            ; mov w1, #rd
        }
        emit_call(assembler, guest::set_register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::idle_cycle::<I> as *const ());
        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        );
    }

    fn ends_block(&self) -> bool {
        false
    }
}
