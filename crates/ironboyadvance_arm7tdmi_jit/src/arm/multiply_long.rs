use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::MultiplyLong, cpu::PC, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for MultiplyLong {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd_lo = self.rd_lo() as u32;
        let rd_hi = self.rd_hi() as u32;
        let rm = self.rm() as u32;
        let rs = self.rs() as u32;

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

        match self.unsigned() {
            true => dynasm! { assembler
                ; .arch aarch64
                ; smull x20, w20, w21
            },
            false => dynasm! { assembler
                ; .arch aarch64
                ; umull x20, w20, w21
            },
        };

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, w21
        };
        emit_call(assembler, guest::multiplier_array_cycles::<I> as *const ());

        if self.accumulate() {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, #rd_lo
            }
            emit_call(assembler, guest::register::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; mov w22, w0
            }
            if rd_lo as usize == PC {
                dynasm! { assembler ; .arch aarch64 ; add w22, w22, #4 }
            }

            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, #rd_hi
            }
            emit_call(assembler, guest::register::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; mov w21, w0
            }
            if rd_hi as usize == PC {
                dynasm! { assembler ; .arch aarch64 ; add w21, w21, #4 }
            }

            dynasm! { assembler
                ; .arch aarch64
                ; lsl x21, x21, #32
                ; orr x21, x21, x22
                ; add x20, x20, x21 // add the accumulator
                ; mov x0, x19
            }
            emit_call(assembler, guest::idle_cycle::<I> as *const ());
        }

        dynasm! { assembler
            ; .arch aarch64
            ; lsr x21, x20, #32
        }
        if self.sets_flags() {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, w21
            }
            emit_call(assembler, guest::set_cpsr_negative::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; orr w1, w20, w21
                ; mov x0, x19
            }
            emit_call(assembler, guest::set_cpsr_zero::<I> as *const ());
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rd_lo
            ; mov w2, w20
        }
        emit_call(assembler, guest::set_register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #rd_hi
            ; mov w2, w21
        }
        emit_call(assembler, guest::set_register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }

        match rd_hi as usize == PC || rd_lo as usize == PC {
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
