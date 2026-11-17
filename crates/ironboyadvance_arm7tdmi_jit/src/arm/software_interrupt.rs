use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::SoftwareInterrupt, memory::MemoryInterface};

use crate::{Compile, emit_call, guest};

impl Compile for SoftwareInterrupt {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let function = self.comment() >> 16;
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; movz w1, #function
        }
        emit_call(assembler, guest::software_interrupt::<I> as *const ());
    }

    fn ends_block(&self) -> bool {
        true
    }
}
