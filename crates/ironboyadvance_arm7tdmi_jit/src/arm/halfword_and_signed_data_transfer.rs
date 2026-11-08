use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::HalfwordAndSignedDataTransfer, cpu::PC, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for HalfwordAndSignedDataTransfer {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rn = self.rn() as u32;
        let rm = self.rm() as u32;
        let pre_index = self.pre_index();
        let load = self.load();
        let updates_base = self.write_back() || !pre_index;
        let s = self.signed();
        let h = self.halfword();

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
            true => emit_immediate_32(assembler, 21, self.immediate_hi() << 4 | self.immediate_lo()),
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
                let load_function = match (s, h) {
                    (false, false) => None,
                    (false, true) => Some(guest::load_rotated_16::<I> as *const ()),
                    (true, false) => Some(guest::load_signed_8::<I> as *const ()),
                    (true, true) => Some(guest::load_signed_16::<I> as *const ()),
                };

                if let Some(load_function) = load_function {
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

                match (s, h) {
                    (false, false) => {}
                    (false, true) => {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov x0, x19
                            ; mov w1, w20
                            ; mov w2, w22
                            ; mov w3, #0
                        }
                        emit_call(assembler, guest::store_16::<I> as *const ());

                        if updates_base {
                            emit_write_back::<I>(assembler, rn, rn as usize == PC);
                        }
                    }
                    (true, _) => {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::idle_cycle::<I> as *const ());

                        if updates_base {
                            emit_write_back::<I>(assembler, rn, rn as usize == PC);
                        }
                    }
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
