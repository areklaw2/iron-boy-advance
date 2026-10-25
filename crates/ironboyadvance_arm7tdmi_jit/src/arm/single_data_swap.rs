use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::SingleDataSwap, cpu::PC, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for SingleDataSwap {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rn = self.rn() as u32;
        let rm = self.rm() as u32;

        emit_prologue(assembler);

        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
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
            dynasm! { assembler ; .arch aarch64 ; add w21, w21, #4 }
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
                emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; movn w0, #0
                }
            }
            false => dynasm! { assembler
                ; .arch aarch64
                ; mov w0, #2
            },
        }
        emit_epilogue(assembler);
    }
}
