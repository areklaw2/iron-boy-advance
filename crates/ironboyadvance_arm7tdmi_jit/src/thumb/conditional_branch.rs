use dynasmrt::{Assembler, DynasmApi, DynasmLabelApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::ConditionalBranch};
use ironboyadvance_common::{bits::SignExtend, memory::MemoryAccess};

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for ConditionalBranch {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let condition = self.cond() as u32;
        let offset = (self.offset().sign_extend(8) << 1) as u32;

        emit_prologue(assembler);

        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #condition
        }
        emit_call(assembler, guest::is_condition_met::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; tbnz w0, #0, >branch
        }
        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
        );

        dynasm! { assembler
            ; .arch aarch64
            ; b >end
            ; branch:
        }
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
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
        dynasm! { assembler
            ; .arch aarch64
            ; end:
        }
        emit_epilogue(assembler);
    }
}
