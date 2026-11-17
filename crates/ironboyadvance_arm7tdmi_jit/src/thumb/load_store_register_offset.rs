use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::LoadStoreRegisterOffset};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for LoadStoreRegisterOffset {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let ro = self.ro() as u32;
        let rb = self.rb() as u32;
        let rd = self.rd() as u32;
        let byte = self.byte();
        let load = self.load();

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #ro
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov x0, x19
            ; mov w1, #rb
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; add w20, w20, w0
        }

        match (load, byte) {
            (false, false) => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w3, #0
                }
                emit_call(assembler, guest::store_32::<I> as *const ());
            }
            (false, true) => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w3, #0
                }
                emit_call(assembler, guest::store_8::<I> as *const ());
            }
            (true, false) => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, #0
                }
                emit_call(assembler, guest::load_rotated_32::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());
            }
            (true, true) => {
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, w20
                    ; mov w2, #0
                }
                emit_call(assembler, guest::load_8::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov w2, w0
                    ; mov x0, x19
                    ; mov w1, #rd
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());
            }
        }

        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        );
        emit_epilogue(assembler);
    }
}
