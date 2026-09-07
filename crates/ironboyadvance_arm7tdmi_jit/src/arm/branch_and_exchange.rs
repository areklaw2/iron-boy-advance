use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::BranchAndExchange, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for BranchAndExchange {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rn = self.rn() as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; movz w1, #rn
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov x0, x19
            ; mov w1, w20
        }
        emit_call(assembler, guest::set_cpsr_state::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; and w1, w20, #0xfffffffe
            ; mov x0, x19
        }
        emit_call(assembler, guest::set_pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::pipeline_flush::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; movn w0, #0
        }
        emit_epilogue(assembler);
    }
}
