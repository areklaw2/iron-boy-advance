use dynasmrt::{Assembler, DynasmApi, aarch64::Aarch64Relocation, dynasm};
use ironboyadvance_arm7tdmi::{CpuAction, arm::BlockDataTransfer, cpu::PC, memory::MemoryInterface};
use ironboyadvance_common::memory::MemoryAccess;

use crate::{Compile, emit_call, emit_epilogue_extended, emit_prologue_extended, guest};

impl Compile for BlockDataTransfer {
    fn compile<I: MemoryInterface>(&self, assembler: &mut Assembler<Aarch64Relocation>) {
        let rn = self.rn() as u32;

        let is_empty_list = self.register_list() == 0;
        let register_list: u16 = if is_empty_list { 1 << PC } else { self.register_list() };
        let transfer_pc = (register_list >> PC) & 1 == 1;
        let transfer_bytes = if is_empty_list { 64 } else { register_list.count_ones() * 4 };

        let load = self.load();
        let load_psr_force_user = self.load_psr_force_user();
        let switch_mode = load_psr_force_user && (!load || !transfer_pc);

        let add = self.add();
        let mut pre_index = self.pre_index();
        let write_back = self.write_back();

        emit_prologue_extended(assembler);

        dynasm! { assembler
            ; .arch aarch64
            ; mov x19, x0
            ; mov w1, #rn
        }

        emit_call(assembler, guest::register::<I> as *const ());
        dynasm! { assembler
            ; .arch aarch64
            ; mov w20, w0
            ; mov w21, w0
        }

        if switch_mode {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
            }
            emit_call(assembler, guest::enter_user_mode::<I> as *const ());
            dynasm! { assembler
                ; .arch aarch64
                ; mov w22, w0
            }
        }

        if !add {
            pre_index = !pre_index;
            dynasm! { assembler
                ; .arch aarch64
                ; sub w20, w20, #transfer_bytes
                ; sub w21, w21, #transfer_bytes
            }
        } else {
            dynasm! { assembler
                ; .arch aarch64
                ; add w21, w21, #transfer_bytes
            }
        }

        let mut memory_access = MemoryAccess::NonSequential as u32;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
        match load {
            true => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; add w20, w20, #4
                        }
                    }

                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w2, #memory_access
                    }
                    emit_call(assembler, guest::load_32::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov w23, w0
                    }

                    if write_back && i == 0 {
                        if rn as usize == PC {
                            dynasm! { assembler
                                ; .arch aarch64
                                ; add w21, w21, #4
                            }
                            if !transfer_pc {
                                dynasm! { assembler
                                    ; .arch aarch64
                                    ; mov x0, x19
                                }
                                emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                            }
                        }

                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov x0, x19
                            ; mov w1, #rn
                            ; mov w2, w21
                        }
                        emit_call(assembler, guest::set_register::<I> as *const ());
                    }
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #register
                        ; mov w2, w23
                    }
                    emit_call(assembler, guest::set_register::<I> as *const ());

                    if !pre_index {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; add w20, w20, #4
                        }
                    }

                    memory_access = MemoryAccess::Sequential as u32;
                }

                dynasm! { assembler
                    ; .arch aarch64
                    ; mov x0, x19
                }
                emit_call(assembler, guest::idle_cycle::<I> as *const ());

                if transfer_pc {
                    if load_psr_force_user {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; mov x0, x19
                        }
                        emit_call(assembler, guest::set_cpsr_to_spsr::<I> as *const ());
                    }

                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                    }
                    emit_call(assembler, guest::pipeline_flush::<I> as *const ());
                    action = CpuAction::PipelineFlush;
                }
            }
            false => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; add w20, w20, #4
                        }
                    }

                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, #register
                    }
                    emit_call(assembler, guest::register::<I> as *const ());
                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov w23, w0
                    }
                    if register as usize == PC {
                        match write_back && rn as usize == PC {
                            true => dynasm! { assembler
                                ; .arch aarch64
                                ; sub w23, w23, #4
                            },
                            false => dynasm! { assembler
                                ; .arch aarch64
                                ; add w23, w23, #4
                            },
                        }
                    }

                    dynasm! { assembler
                        ; .arch aarch64
                        ; mov x0, x19
                        ; mov w1, w20
                        ; mov w2, w23
                        ; mov w3, #memory_access
                    }
                    emit_call(assembler, guest::store_32::<I> as *const ());
                    if write_back && i == 0 {
                        if rn as usize == PC {
                            dynasm! { assembler
                                ; .arch aarch64
                                ; add w21, w21, #4
                            }
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
                            ; mov w2, w21
                        }
                        emit_call(assembler, guest::set_register::<I> as *const ());
                    }

                    if !pre_index {
                        dynasm! { assembler
                            ; .arch aarch64
                            ; add w20, w20, #4
                        }
                    }

                    memory_access = MemoryAccess::Sequential as u32;
                }
            }
        }

        if switch_mode {
            dynasm! { assembler
                ; .arch aarch64
                ; mov x0, x19
                ; mov w1, w22
            }
            emit_call(assembler, guest::set_cpsr_mode::<I> as *const ());
        }

        match action {
            CpuAction::PipelineFlush => dynasm! { assembler
                ; .arch aarch64
                ; movn w0, #0
            },
            CpuAction::Advance(access) => dynasm! { assembler
                ; .arch aarch64
                ; mov w0, #access as u32
            },
        }

        emit_epilogue_extended(assembler);
    }
}
