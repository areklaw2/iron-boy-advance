use dynasmrt::{Assembler, DynasmApi, DynasmLabelApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{arm::PsrTransfer, memory::MemoryInterface};

use crate::{Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for PsrTransfer {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rd = self.rd() as u32;
        let rm = self.rm() as u32;
        let is_spsr = self.is_spsr();
        let mask = self.psr_mask();
        let rotate = self.rotate() * 2;
        let immediate = self.immediate();

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
        }
        match self.is_mrs() {
            true => {
                match is_spsr {
                    false => emit_call(assembler, guest::cpsr::<I> as *const ()),
                    true => emit_call(assembler, guest::spsr::<I> as *const ()),
                };
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
            }
            false => {
                let forces_mode_bit = !is_spsr && mask & 0xFF != 0;
                match self.is_immediate() {
                    true => {
                        let mut operand = immediate.rotate_right(rotate);
                        if forces_mode_bit {
                            operand |= 0x10;
                        }
                        emit_immediate_32(assembler, 20, operand);
                    }
                    false => {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov w1, #rm
                        }
                        emit_call(assembler, guest::register::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov w20, w0
                        }
                        if forces_mode_bit {
                            dynasm! { assembler
                                ; .arch aarch64
                                ; orr w20, w20, #0x10
                            }
                        }
                    }
                }

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                match is_spsr {
                    false => {
                        emit_immediate_32(assembler, 21, mask);
                        emit_immediate_32(assembler, 22, mask & 0xFF000000);
                        emit_call(assembler, guest::cpsr_mode::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; cmp w0, #16 // User Mode
                            ; b.eq > user
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::cpsr::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; bic w0, w0, w21
                            ; and w1, w20, w21
                            ; orr w1, w0, w1
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::set_cpsr::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; b > done
                            ; user:
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::cpsr::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; bic w0, w0, w22
                            ; and w1, w20, w22
                            ; orr w1, w0, w1
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::set_cpsr::<I> as *const ());
                        dynasm! { assembler
                           ; .arch aarch64
                           ; done:
                        }
                    }
                    true => {
                        emit_immediate_32(assembler, 21, mask);
                        emit_call(assembler, guest::cpsr_mode::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; cmp w0, #16
                            ; b.eq >skip
                            ; cmp w0, #31
                            ; b.eq >skip
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::spsr::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; bic w0, w0, w21
                            ; and w1, w20, w21
                            ; orr w1, w0, w1
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::set_spsr::<I> as *const ());
                        dynasm! { assembler
                            ; .arch aarch64
                            ; skip:
                        }
                    }
                }
            }
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov w0, #3
        }
        emit_epilogue(assembler);
    }
}
