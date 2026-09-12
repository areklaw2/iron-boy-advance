use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    DataProcessingOpcode,
    arm::DataProcessing,
    barrel_shifter::{ShiftBy, ShiftType},
    cpu::PC,
    memory::MemoryInterface,
};

use crate::{Compile, emit_call, emit_epilogue, emit_immediate_32, emit_prologue, guest};

impl Compile for DataProcessing {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        use DataProcessingOpcode::*;
        let rn = self.rn() as u32;
        let rm = self.rm() as u32;
        let rd = self.rd() as u32;
        let opcode = self.opcode();
        let is_immediate = self.is_immediate();
        let immediate = self.immediate();
        let rotate = 2 * self.rotate();
        let shift_by = self.shift_by();
        let shift_type = self.shift_type();
        let shift_amount = self.shift_amount();
        let set_flags = self.sets_flags();

        let needs_carry = set_flags && matches!(opcode, AND | EOR | TST | TEQ | ORR | MOV | BIC | MVN);

        let no_operand1 = matches!(opcode, MOV | MVN);
        let operand2_arg_register: u8 = if no_operand1 { 2 } else { 3 };
        let carry_arg_register: u8 = if no_operand1 { 3 } else { 4 };

        let swap_operands = matches!(opcode, RSB | RSC);
        let operand1_arg_register: u8 = if swap_operands { 3 } else { 2 };
        let computed_value_arg_register: u8 = if swap_operands { 2 } else { operand2_arg_register };

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; movz w1, #rn
        }
        emit_call(assembler, guest::register::<I> as *const ());

        match (is_immediate, shift_by) {
            (true, _) => match needs_carry {
                false => {
                    if !no_operand1 {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov W(operand1_arg_register), w0
                        }
                    }
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #set_flags as u32
                    }
                    emit_immediate_32(assembler, computed_value_arg_register, immediate.rotate_right(rotate));
                }
                true if rotate == 0 => {
                    if !no_operand1 {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov w20, w0
                        }
                    }
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov W(carry_arg_register), w0
                        ; mov x0, x19
                        ; mov w1, #set_flags as u32
                    }
                    if !no_operand1 {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov w2, w20
                        }
                    }
                    emit_immediate_32(assembler, operand2_arg_register, immediate);
                }
                true => {
                    let value = immediate.rotate_right(rotate);
                    let carry = value >> 31;
                    if !no_operand1 {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov w2, w0
                        }
                    }
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #set_flags as u32
                    }
                    emit_immediate_32(assembler, operand2_arg_register, value);
                    emit_immediate_32(assembler, carry_arg_register, carry);
                }
            },
            (false, ShiftBy::Immediate) => {
                use ShiftType::*;
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                    ; mov x0, x19
                    ; movz w1, #rm
                }
                emit_call(assembler, guest::register::<I> as *const ());
                match shift_type {
                    LSL => match shift_amount {
                        0 => {
                            if needs_carry {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; str w0, [sp, #24]
                                }
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; mov x0, x19
                                }
                                emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; mov W(carry_arg_register), w0
                                    ; ldr w0, [sp, #24]
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; mov W(computed_value_arg_register), w0
                            }
                        }
                        _ => {
                            if needs_carry {
                                let lsb = 32 - shift_amount;
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w0, #lsb, #1
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; lsl W(computed_value_arg_register), w0, #shift_amount
                            }
                        }
                    },
                    LSR => match shift_amount {
                        0 => {
                            if needs_carry {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w0, #31, #1
                                }
                            }
                            emit_immediate_32(assembler, computed_value_arg_register, 0);
                        }
                        _ => {
                            if needs_carry {
                                let lsb = shift_amount - 1;
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w0, #lsb, #1
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; lsr W(computed_value_arg_register), w0, #shift_amount
                            }
                        }
                    },
                    ASR => match shift_amount {
                        0 => {
                            if needs_carry {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w0, #31, #1
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; asr W(computed_value_arg_register), w0, #31
                            }
                        }
                        _ => {
                            if needs_carry {
                                let lsb = shift_amount - 1;
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w0, #lsb, #1
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; asr W(computed_value_arg_register), w0, #shift_amount
                            }
                        }
                    },
                    ROR => match shift_amount {
                        0 => {
                            // RRX: operand2 = (rm >> 1) | (cpsr_carry << 31), carry = rm & 1
                            dynasm! { assembler
                                ; .arch aarch64
                                ; str w0, [sp, #24]
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; mov x0, x19
                            }
                            emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                            dynasm! { assembler
                                ; .arch aarch64
                                ; ldr w9, [sp, #24]
                            }
                            if needs_carry {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; ubfx W(carry_arg_register), w9, #0, #1
                                }
                            }
                            dynasm! { assembler
                                ; .arch aarch64
                                ; extr W(computed_value_arg_register), w0, w9, #1
                            }
                        }
                        _ => {
                            dynasm! { assembler
                                ; .arch aarch64
                                ; ror W(computed_value_arg_register), w0, #shift_amount
                            }
                            if needs_carry {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; lsr W(carry_arg_register), W(computed_value_arg_register), #31
                                }
                            }
                        }
                    },
                }
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #set_flags as u32
                }
                if !no_operand1 {
                    dynasm! { assembler
                       ; .arch aarch64
                       ; mov W(operand1_arg_register), w20
                    }
                }
            }
            (false, ShiftBy::Register) => todo!(),
        }

        match opcode {
            AND => emit_call(assembler, guest::and::<I> as *const ()),
            EOR => emit_call(assembler, guest::eor::<I> as *const ()),
            SUB => emit_call(assembler, guest::sub::<I> as *const ()),
            RSB => emit_call(assembler, guest::rsb::<I> as *const ()),
            ADD => emit_call(assembler, guest::add::<I> as *const ()),
            ADC => emit_call(assembler, guest::adc::<I> as *const ()),
            SBC => emit_call(assembler, guest::sbc::<I> as *const ()),
            RSC => emit_call(assembler, guest::rsc::<I> as *const ()),
            TST => emit_call(assembler, guest::tst::<I> as *const ()),
            TEQ => emit_call(assembler, guest::teq::<I> as *const ()),
            CMP => emit_call(assembler, guest::cmp::<I> as *const ()),
            CMN => emit_call(assembler, guest::cmn::<I> as *const ()),
            ORR => emit_call(assembler, guest::orr::<I> as *const ()),
            MOV => emit_call(assembler, guest::mov::<I> as *const ()),
            BIC => emit_call(assembler, guest::bic::<I> as *const ()),
            MVN => emit_call(assembler, guest::mvn::<I> as *const ()),
        }

        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        if set_flags && rd as usize == PC {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
            }
            emit_call(assembler, guest::set_cpsr_to_spsr::<I> as *const ())
        }

        if !matches!(opcode, TST | TEQ | CMP | CMN) {
            dynasm! { assembler
                ; .arch aarch64
                ; mov w2, w20
                ; mov x0, x19
                ; mov w1, #rd
            }
            emit_call(assembler, guest::set_register::<I> as *const ())
        }

        match !matches!(opcode, TST | TEQ | CMP | CMN) && rd as usize == PC {
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
            false => match shift_by {
                ShiftBy::Immediate => dynasm! { assembler
                    ; .arch aarch64
                    ; mov w0, #3
                },
                ShiftBy::Register => dynasm! { assembler
                    ; .arch aarch64
                    ; mov w0, #2
                },
            },
        }

        emit_epilogue(assembler);
    }
}
