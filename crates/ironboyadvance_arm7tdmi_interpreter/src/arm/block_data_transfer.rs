use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, CpuMode,
    arm::BlockDataTransfer,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for BlockDataTransfer {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rn = self.rn() as usize;
        let mut address = cpu.register(rn);

        let is_empty_list = self.register_list() == 0;
        let register_list: u16 = if is_empty_list { 1 << PC } else { self.register_list() };
        let transfer_pc = (register_list >> PC) & 1 == 1;
        let transfer_bytes = if is_empty_list { 64 } else { register_list.count_ones() * 4 };

        let load = self.load();
        let load_psr_force_user = self.load_psr_force_user();
        let mode = cpu.cpsr().mode();
        let switch_mode =
            load_psr_force_user && (!load || !transfer_pc) && ![CpuMode::User, CpuMode::System].contains(&mode);
        if switch_mode {
            cpu.cpsr_mut().set_mode(CpuMode::User);
        }

        let add = self.add();
        let mut pre_index = self.pre_index();
        let mut base_address = address;
        if !add {
            pre_index = !pre_index;
            address = address.wrapping_sub(transfer_bytes);
            base_address = base_address.wrapping_sub(transfer_bytes);
        } else {
            base_address = base_address.wrapping_add(transfer_bytes)
        }

        let write_back = self.write_back();
        let mut memory_access = MemoryAccess::NonSequential;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
        match load {
            true => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        address = address.wrapping_add(4)
                    }

                    let value = cpu.load_32(address, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address = base_address.wrapping_add(4);
                            if !transfer_pc {
                                cpu.pipeline_flush();
                            }
                        }
                        cpu.set_register(rn, base_address);
                    }
                    cpu.set_register(register, value);

                    if !pre_index {
                        address = address.wrapping_add(4)
                    }

                    memory_access = MemoryAccess::Sequential;
                }

                cpu.idle_cycle();
                if transfer_pc {
                    if load_psr_force_user {
                        cpu.set_cpsr(cpu.spsr());
                    }

                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            false => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        address = address.wrapping_add(4)
                    }

                    let mut value = cpu.register(register);
                    if register == PC {
                        match write_back && rn == PC {
                            true => value = value.wrapping_sub(4),
                            false => value = value.wrapping_add(4),
                        }
                    }

                    cpu.store_32(address, value, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address = base_address.wrapping_add(4);
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, base_address);
                    }

                    if !pre_index {
                        address = address.wrapping_add(4)
                    }

                    memory_access = MemoryAccess::Sequential;
                }
            }
        }

        if switch_mode {
            cpu.cpsr_mut().set_mode(mode);
        }

        action
    }
}
