use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction, LR, PC, SP},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct PushPopRegisters {
    register_list_bits: u8,
    store_lr_load_pc: bool,
    load: bool,
}

impl PushPopRegisters {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            register_list_bits: value as u8,
            store_lr_load_pc: value.bit(8),
            load: value.bit(11),
        }
    }
}

impl Instruction for PushPopRegisters {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut address = cpu.register(SP);
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits >> i) & 1 == 1).collect();
        let store_lr_load_pc = self.store_lr_load_pc;

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load {
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let load = self.load;
        let store_lr_load_pc = self.store_lr_load_pc;
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits >> i) & 1 == 1).collect();
        let register_list_str = register_list
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match (load, store_lr_load_pc) {
            (false, false) => format!("PUSH {{{}}}", register_list_str),
            (false, true) => format!("PUSH {{{},lr}}", register_list_str),
            (true, false) => format!("POP {{{}}}", register_list_str),
            (true, true) => format!("POP {{{},pc}}", register_list_str),
        }
    }
}
