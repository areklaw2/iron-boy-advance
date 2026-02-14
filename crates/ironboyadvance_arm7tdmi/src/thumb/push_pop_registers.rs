use ironboyadvance_utils::bit::BitOps;

use crate::{
    CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, LR, PC, SP},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct PushPopRegisters {
    value: u16,
}

thumb_instruction!(PushPopRegisters);

impl PushPopRegisters {
    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut address = cpu.register(SP);
        let register_list = self.register_list();
        let store_lr_load_pc = self.store_lr_load_pc();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() && !store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(SP, address + 64);
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address += 4
                }

                if store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(PC, value & !0b1);
                    cpu.set_register(SP, address + 4);
                    cpu.idle_cycle();
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                cpu.idle_cycle();
                cpu.set_register(SP, address);
            }
            false => {
                if register_list.is_empty() && !store_lr_load_pc {
                    address -= 64;
                    cpu.set_register(SP, address);
                    let value = cpu.pc() + 2;
                    cpu.store_32(address, value, memory_access as u8);
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                address -= register_list.len() as u32 * 4;
                if store_lr_load_pc {
                    address -= 4
                }
                cpu.set_register(SP, address);

                for register in register_list.iter() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);
                    memory_access = MemoryAccess::Sequential;
                    address += 4
                }

                if store_lr_load_pc {
                    let value = cpu.register(LR);
                    cpu.store_32(address, value, memory_access as u8);
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    pub fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let load = self.load();
        let store_lr_load_pc = self.store_lr_load_pc();
        let register_list = self
            .register_list()
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match (load, store_lr_load_pc) {
            (false, false) => format!("PUSH {{{}}}", register_list),
            (false, true) => format!("PUSH {{{},lr}}", register_list),
            (true, false) => format!("POP {{{}}}", register_list),
            (true, true) => format!("POP {{{},pc}}", register_list),
        }
    }

    #[inline]
    pub fn register_list(&self) -> Vec<usize> {
        (0..=7).filter(|&i| self.value.bit(i)).collect()
    }

    #[inline]
    pub fn store_lr_load_pc(&self) -> bool {
        self.value.bit(8)
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(11)
    }
}
