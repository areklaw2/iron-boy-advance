use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::BranchAndBranchWithLink, cpu::LR, memory::MemoryInterface};
use ironboyadvance_common::bits::SignExtend;

use crate::{Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for BranchAndBranchWithLink {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let offset = (self.offset().sign_extend(24) << 2) as u32;

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
        }
        emit_call(assembler, guest::pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        if self.link() {
            let link_register = LR as u32;
            dynasm! { assembler
                ; .arch aarch64
                ; sub w2, w0, #4
                ; mov x0, x19
                ; movz w1, #link_register
            }
            emit_call(assembler, guest::set_register::<I> as *const ());
        }

        emit_immediate_32(assembler, 10, offset);
        dynasm! { assembler
            ; .arch aarch64
            ; add w1, w20, w10
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
