use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::SingleDataTransfer, barrel_shifter::ShiftType, cpu::PC, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for SingleDataTransfer {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rn = self.rn() as u32;
        let rm = self.rm() as u32;
        let shift_amount = self.shift_amount();
        let pre_index = self.pre_index();
        let load = self.load();
        let byte = self.byte();
        let updates_base = self.write_back() || !pre_index;

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

        match self.is_immediate() {
            true => emit_immediate_32(assembler, 21, self.immediate()),
            false => {
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

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov w1, #shift_amount
                    ; mov w0, w21
                    ; mov w3, #1
                }
                match self.shift_type() {
                    ShiftType::LSL => emit_call(assembler, guest::lsl_shift as *const ()),
                    ShiftType::LSR => emit_call(assembler, guest::lsr_shift as *const ()),
                    ShiftType::ASR => emit_call(assembler, guest::asr_shift as *const ()),
                    ShiftType::ROR => emit_call(assembler, guest::ror_shift as *const ()),
                }
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w21, w0
                }
            }
        }

        if !self.add() {
            dynasm! { assembler
                ; .arch aarch64
                ; neg w21, w21
            }
        }

        if pre_index {
            dynasm! { assembler
                ; .arch aarch64
                ; add w20, w20, w21
            }
        }

        match load {
            true => {
                let load_function = match byte {
                    true => guest::load_8::<I> as *const (),
                    false => guest::load_rotated_32::<I> as *const (),
                };

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, #0
                }
                emit_call(assembler, load_function);
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w22, w0
                }

                if updates_base {
                    emit_write_back::<I>(assembler, rn, rn != rd && rn as usize == PC);
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
                    ; mov w2, w22
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
            }
            false => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w22, w0
                }
                if rd as usize == PC {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; add w22, w22, #4
                    }
                }

                let store_function = match byte {
                    true => guest::store_8::<I> as *const (),
                    false => guest::store_32::<I> as *const (),
                };

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, w22
                    ; mov w3, #0
                }
                emit_call(assembler, store_function);

                if updates_base {
                    emit_write_back::<I>(assembler, rn, rn as usize == PC);
                }
            }
        }

        match load && rd as usize == PC {
            true => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
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

fn emit_write_back<I: MemoryInterface>(assembler: &mut Assembler<Aarch64Relocation>, rn: u32, flush_first: bool) {
    if flush_first {
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
        }
        emit_call(assembler, guest::pipeline_flush::<I> as *const ());
    }

    dynasm! { assembler
        ; .arch aarch64
        ; mov x0, x19
        ; mov w1, #rn
    }
    emit_call(assembler, guest::register::<I> as *const ());
    dynasm! { assembler
        ; .arch aarch64
        ; mov w20, w0
        ; add w20, w20, w21
    }

    dynasm! { assembler
        ; .arch aarch64
        ; mov x0, x19
        ; mov w1, #rn
        ; mov w2, w20
    }
    emit_call(assembler, guest::set_register::<I> as *const ());
}
