use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, arm::SingleDataSwap, cpu::PC, memory::MemoryInterface};

use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, guest};

impl Compile for SingleDataSwap {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rn = self.rn() as u32;
        let rm = self.rm() as u32;

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rn
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rm
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w21, w0
        }
        if rm as usize == PC {
            dynasm! { assembler
               ; .arch aarch64
               ; add w21, w21, #4
            }
        }

        match self.byte() {
            true => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, #0
                }
                emit_call(assembler, guest::load_8::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w22, w0
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, w21
                    ; mov w3, #8
                }
                emit_call(assembler, guest::store_8::<I> as *const ());
            }
            false => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, #0
                }
                emit_call(assembler, guest::load_rotated_32::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w22, w0
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, w21
                    ; mov w3, #8
                }
                emit_call(assembler, guest::store_32::<I> as *const ());
            }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::idle_cycle::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rd
            ; mov x2, x22
        }
        emit_call(assembler, guest::set_register::<I> as *const ());

        match rd as usize == PC {
            true => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                emit_action(assembler, CpuAction::PipelineFlush);
            }
            false => emit_action(
                assembler,
                CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
            ),
        }
    }

    fn ends_block(&self) -> bool {
        self.rd() as usize == PC
    }
}
