use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    CpuAction, HiRegOpsBxOpcode, cpu::PC, memory::MemoryInterface, thumb::HiRegisterOperationsBranchExchange,
};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for HiRegisterOperationsBranchExchange {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        use HiRegOpsBxOpcode::*;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);

        emit_prologue(assembler);

        let destination = match self.h1() {
            true => self.hd() as u32 + 8,
            false => self.rd() as u32,
        };
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #destination
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        let source = match self.h2() {
            true => self.hs() as u32 + 8,
            false => self.rs() as u32,
        };
        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #source
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w21, w0
        }
        if source as usize == PC {
            dynasm! { assembler
                ; .arch aarch64
                ; and w21, w21, #0xfffffffe
            }
        }

        match self.opcode().into() {
            CMP => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #1
                    ; mov w2, w20
                    ; mov w3, w21
                }
                emit_call(assembler, guest::cmp::<I> as *const ());
            }
            ADD => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #0
                    ; mov w2, w20
                    ; mov w3, w21
                }
                emit_call(assembler, guest::add::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, #destination
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
                if destination as usize == PC {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; and w1, w0, #0xfffffffe
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::set_pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());

                    action = CpuAction::PipelineFlush;
                }
            }
            MOV => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::cpsr_carry::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w3, w0
                    ; mov x0, x19
                    ; mov w1, #0
                    ; mov w2, w21
                }
                emit_call(assembler, guest::mov::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, #destination
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
                if destination as usize == PC {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; and w1, w0, #0xfffffffe
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::set_pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());

                    action = CpuAction::PipelineFlush;
                }
            }
            BX => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w21
                }
                emit_call(assembler, guest::set_cpsr_state::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; and w1, w21, #0xfffffffe
                }
                emit_call(assembler, guest::set_pc::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                action = CpuAction::PipelineFlush;
            }
        };

        emit_action(assembler, action);
        emit_epilogue(assembler);
    }

    fn ends_block(&self) -> bool {
        let opcode: HiRegOpsBxOpcode = self.opcode().into();
        if opcode == HiRegOpsBxOpcode::BX {
            return true;
        }

        let destination = match self.h1() {
            true => self.hd() as usize + 8,
            false => self.rd() as usize,
        };
        destination == PC
    }
}
