use dynasmrt::{Assembler, aarch64::Aarch64Relocation};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::Undefined};

use crate::{Compile, emit_action, emit_call, emit_epilogue_link_only, emit_prologue_link_only, guest};

impl Compile for Undefined {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        emit_prologue_link_only(assembler);
        emit_call(assembler, guest::undefined_exception::<I> as *const ());
        emit_action(assembler, CpuAction::PipelineFlush);
        emit_epilogue_link_only(assembler);
    }
}
