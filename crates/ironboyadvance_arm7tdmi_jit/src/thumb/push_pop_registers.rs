use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{LR, PC, SP},
    memory::MemoryInterface,
    thumb::PushPopRegisters,
};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_action, emit_call, guest};

impl Compile for PushPopRegisters {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<u32> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();
        let store_lr_load_pc = self.store_lr_load_pc();

        dynasm! { assembler
            ; .arch aarch64
            ; mov x0, x19
            ; mov w1, #SP as u32
        }
        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
        }

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() && !store_lr_load_pc {
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
                        ; mov w1, #SP as u32
                        ; add w2, w20, #64
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                    emit_action(assembler, CpuAction::PipelineFlush);
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

                if store_lr_load_pc {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w2, #memory_access as u32
                    }
                    emit_call(assembler, guest::load_32::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; and w2, w0, #0xfffffffe
                        ; mov x0, x19
                        ; mov w1, #PC as u32
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #SP as u32
                        ; add w2, w20, #4
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::idle_cycle::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                    emit_action(assembler, CpuAction::PipelineFlush);
                    return;
                }

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());
                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                    ; mov w1, #SP as u32
                    ; mov w2, w20
                }
                emit_call(assembler, guest::set_register::<I> as *const ());
            }
            false => {
                if register_list.is_empty() && !store_lr_load_pc {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; sub w20, w20, #64
                        ; mov x0, x19
                        ; mov w1, #SP as u32
                        ; mov w2, w20
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());
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
                    emit_action(
                        assembler,
                        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
                    );
                    return;
                }

                let transfer_bytes = match store_lr_load_pc {
                    true => register_list.len() as u32 * 4 + 4,
                    false => register_list.len() as u32 * 4,
                };
                dynasm! { assembler
                    ; .arch aarch64
                    ; sub w20, w20, #transfer_bytes
                    ; mov x0, x19
                    ; mov w1, #SP as u32
                    ; mov w2, w20
                }
                emit_call(assembler, guest::set_register::<I> as *const ());

                for register in register_list.iter() {
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
                    dynasm! { assembler
                        ; .arch aarch64
                        ; add w20, w20, #4
                    }
                    memory_access = MemoryAccess::Sequential;
                }

                if store_lr_load_pc {
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #LR as u32
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
                }
            }
        }

        emit_action(
            assembler,
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        );
    }

    fn ends_block(&self) -> bool {
        self.load() && (self.store_lr_load_pc() || self.register_list_bits() == 0)
    }
}
