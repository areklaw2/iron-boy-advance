use dynasmrt::{Assembler, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::Undefined, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue_link_only, emit_prologue_link_only, guest};

impl Compile for Undefined {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        emit_prologue_link_only(assembler);
        emit_call(assembler, guest::undefined_exception::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; movn w0, #0
        }
        emit_epilogue_link_only(assembler);
    }
}
