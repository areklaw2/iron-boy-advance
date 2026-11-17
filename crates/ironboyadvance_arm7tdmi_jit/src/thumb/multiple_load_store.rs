use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, memory::MemoryInterface, thumb::MultipleLoadStore};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, emit_epilogue, emit_prologue, guest};

impl Compile for MultipleLoadStore {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rb = self.rb() as u32;
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<u32> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();

        emit_prologue(assembler);
        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rb
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w2, #memory_access as u32
                    }
                    emit_call(assembler, guest::load_32::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov w1, w0
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::set_pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #rb
                        ; add w2, w20, #64
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                    emit_action(assembler, CpuAction::PipelineFlush);
                    emit_epilogue(assembler);
                    return;
                }

                for register in register_list.iter() {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w2, #memory_access as u32
                    }
                    emit_call(assembler, guest::load_32::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov w2, w0
                        ; mov x0, x19
                        ; mov w1, #*register
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; add w20, w20, #4
                    }
                    memory_access = MemoryAccess::Sequential;
                }

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());
                if !register_list.contains(&rb) {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #rb
                        ; mov w2, w20
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                }
            }
            false => {
                if register_list.is_empty() {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pc::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; add w2, w0, #2
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w3, #memory_access as u32
                    }
                    emit_call(assembler, guest::store_32::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #rb
                        ; add w2, w20, #64
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    emit_action(
                        assembler,
                        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
                    );
                    emit_epilogue(assembler);
                    return;
                }

                let transfer_bytes = register_list.len() as u32 * 4;
                for (i, register) in register_list.iter().enumerate() {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #*register
                    }
                    emit_call(assembler, guest::register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov w2, w0
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w3, #memory_access as u32
                    }
                    emit_call(assembler, guest::store_32::<I> as *const ());

                    if i == 0 {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov x0, x19
                            ; mov w1, #rb
                            ; add w2, w20, #transfer_bytes
                        }
                        emit_call(assembler, guest::set_register::<I> as *const ());
                    }

                    dynasm! { assembler
                        ; .arch aarch64
                        ; add w20, w20, #4
                    }
                    memory_access = MemoryAccess::Sequential;
                }
            }
        }

        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        );
        emit_epilogue(assembler);
    }
}
