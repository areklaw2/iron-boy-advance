use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::Multiply, cpu::PC, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for Multiply {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rm = self.rm() as u32;
        let rd = self.rd() as u32;
        let rs = self.rs() as u32;
        let rn = self.rn() as u32;

        emit_prologue(assembler);

        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rm
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }
        if rm as usize == PC {
            dynasm! { assembler ; .arch aarch64 ; add w20, w20, #4 }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rs
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w21, w0
        }
        if rs as usize == PC {
            dynasm! { assembler ; .arch aarch64 ; add w21, w21, #4 }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mul w20, w20, w21 // actual multiply
            ; mov x0, x19
            ; mov w1, w21
        }
        emit_call(assembler, guest::multiplier_array_cycles::<I> as *const ());

        if self.accumulate() {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, #rn
            }
            emit_call(assembler, guest::register::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; mov w22, w0
            }
            if rn as usize == PC {
                dynasm! { assembler ; .arch aarch64 ; add w22, w22, #4 }
            }

            dynasm! { assembler
                ; .arch aarch64
                ; add w20, w20, w22 // add the accumulator
                ; mov x0, x19
            }
            emit_call(assembler, guest::idle_cycle::<I> as *const ());
        }

        if self.sets_flags() {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, w20
            }
            emit_call(assembler, guest::set_cpsr_negative::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, w20
            }
            emit_call(assembler, guest::set_cpsr_zero::<I> as *const ());
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rd
            ; mov w2, w20
        }
        emit_call(assembler, guest::set_register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }

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
