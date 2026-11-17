use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{AluOperationsOpcode, CpuAction, memory::MemoryInterface, thumb::AluOperations};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for AluOperations {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        use AluOperationsOpcode::*;
        let rd = self.rd() as u32;
        let rs = self.rs() as u32;
        let opcode: AluOperationsOpcode = self.opcode().into();
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rd
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov x0, x19
            ; mov w1, #rs
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w21, w0
            ; mov x0, x19
        }
        emit_call(assembler, guest::cpsr_carry::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w22, w0
        }

        match opcode {
            AND | EOR | TST | ORR | BIC => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, w21
                    ; mov w4, w22
                }
                match opcode {
                    AND => emit_call(assembler, guest::and::<I> as *const ()),
                    EOR => emit_call(assembler, guest::eor::<I> as *const ()),
                    TST => emit_call(assembler, guest::tst::<I> as *const ()),
                    ORR => emit_call(assembler, guest::orr::<I> as *const ()),
                    _ => emit_call(assembler, guest::bic::<I> as *const ()),
                }
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                }
            }
            ADC | SBC | CMP | CMN => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, w21
                }
                match opcode {
                    ADC => emit_call(assembler, guest::adc::<I> as *const ()),
                    SBC => emit_call(assembler, guest::sbc::<I> as *const ()),
                    CMP => emit_call(assembler, guest::cmp::<I> as *const ()),
                    _ => emit_call(assembler, guest::cmn::<I> as *const ()),
                }
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                }
            }
            NEG => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, #0
                    ; mov w3, w21
                }
                emit_call(assembler, guest::sub::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                }
            }
            MVN => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w21
                    ; mov w3, w22
                }
                emit_call(assembler, guest::mvn::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                }
            }
            LSL | LSR | ASR | ROR => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w0, w20
                    ; and w1, w21, #0xff
                    ; mov w2, w22
                    ; mov w3, #0
                }
                match opcode {
                    LSL => emit_call(assembler, guest::lsl as *const ()),
                    LSR => emit_call(assembler, guest::lsr as *const ()),
                    ASR => emit_call(assembler, guest::asr as *const ()),
                    _ => emit_call(assembler, guest::ror as *const ()),
                }
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w20, w0
                    ; lsr x22, x0, #32
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());
                action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

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
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w22
                }
                emit_call(assembler, guest::set_cpsr_carry::<I> as *const ());
            }
            MUL => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                }
                emit_call(assembler, guest::multiplier_array_cycles::<I> as *const ());
                action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                dynasm! { assembler
                    ; .arch aarch64
                    ; mul w20, w20, w21
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
        }

        if !matches!(opcode, TST | CMP | CMN) {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, #rd
                ; mov w2, w20
            }
            emit_call(assembler, guest::set_register::<I> as *const ());
        }

        emit_action(assembler, action);
        emit_epilogue(assembler);
    }

    fn ends_block(&self) -> bool {
        false
    }
}
