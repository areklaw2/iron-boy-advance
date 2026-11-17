use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, cpu::LR, memory::MemoryInterface, thumb::LongBranchWithLink};
use ironboyadvance_common::{bits::SignExtend, memory::MemoryAccess};

use crate::{Compile, emit_action, emit_call, emit_immediate_32, guest};

impl Compile for LongBranchWithLink {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::pc::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        match self.high() {
            true => {
                let offset = (self.offset() as u32) << 1;

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #LR as u32
                }
                emit_call(assembler, guest::register::<I> as *const ());

                dynasm! { assembler
                    ; .arch aarch64
                    ; and w1, w0, #0xfffffffe
                    ; add w1, w1, #offset
                    ; mov x0, x19
                }
                emit_call(assembler, guest::set_pc::<I> as *const ());

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #LR as u32
                    ; sub w20, w20, #2
                    ; orr w20, w20, #1
                    ; mov w2, w20
                }
                emit_call(assembler, guest::set_register::<I> as *const ());

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                emit_action(assembler, CpuAction::PipelineFlush);
            }
            false => {
                let offset = self.offset().sign_extend(11) << 12;
                emit_immediate_32(assembler, 10, offset as u32);
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #LR as u32
                    ; add w20, w20, w10
                    ; mov w2, w20
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
                emit_action(
                    assembler,
                    CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
                );
            }
        }
    }

    fn ends_block(&self) -> bool {
        self.high()
    }
}
