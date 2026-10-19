use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    cpu::{Arm7tdmiCpu, LR, PC, SP},
    memory::MemoryInterface,
    thumb::PushPopRegisters,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for PushPopRegisters {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut address = cpu.register(SP);
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<usize> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();
        let store_lr_load_pc = self.store_lr_load_pc();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() && !store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(SP, address.wrapping_add(64));
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                if store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(PC, value & !0b1);
                    cpu.set_register(SP, address.wrapping_add(4));
                    cpu.idle_cycle();
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                cpu.idle_cycle();
                cpu.set_register(SP, address);
            }
            false => {
                if register_list.is_empty() && !store_lr_load_pc {
                    address = address.wrapping_sub(64);
                    cpu.set_register(SP, address);
                    let value = cpu.pc().wrapping_add(2);
                    cpu.store_32(address, value, memory_access as u8);
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                address = address.wrapping_sub(register_list.len() as u32 * 4);
                if store_lr_load_pc {
                    address = address.wrapping_sub(4)
                }
                cpu.set_register(SP, address);

                for register in register_list.iter() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                if store_lr_load_pc {
                    let value = cpu.register(LR);
                    cpu.store_32(address, value, memory_access as u8);
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
