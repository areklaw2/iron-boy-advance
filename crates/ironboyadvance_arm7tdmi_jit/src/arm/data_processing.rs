use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::DataProcessing, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for DataProcessing {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        emit_prologue(assembler);

        emit_epilogue(assembler);
    }
}
