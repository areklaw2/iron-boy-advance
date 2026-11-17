use dynasmrt::{Assembler, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::UnconditionalBranch};
use ironboyadvance_common::bits::SignExtend;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for UnconditionalBranch {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let offset = (self.offset().sign_extend(11) << 1) as u32;

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
        emit_action(assembler, CpuAction::PipelineFlush);

        emit_epilogue(assembler);
    }

    fn ends_block(&self) -> bool {
        true
    }
}
