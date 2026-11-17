use dynasmrt::{Assembler, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, arm::Undefined, memory::MemoryInterface};

use crate::{Compile, emit_action, emit_call, guest};

impl Compile for Undefined {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::undefined_exception::<I> as *const ());
        emit_action(assembler, CpuAction::PipelineFlush);
    }

    fn ends_block(&self) -> bool {
        true
    }
}
