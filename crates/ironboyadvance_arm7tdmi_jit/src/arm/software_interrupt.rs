use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::SoftwareInterrupt, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue_link_only, emit_prologue_link_only, guest};

impl Compile for SoftwareInterrupt {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let function = self.comment() >> 16;
        emit_prologue_link_only(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; movz w1, #function
        }
        emit_call(assembler, guest::software_interrupt::<I> as *const ());
        emit_epilogue_link_only(assembler);
    }
}
