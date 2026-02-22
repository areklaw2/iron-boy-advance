use crate::{
    BitOps, CpuAction, LoRegister,
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct MultipleLoadStore {
    value: u16,
}

thumb_instruction!(MultipleLoadStore);

impl Instruction for MultipleLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rb = self.rb() as usize;
        let mut address = cpu.register(rb);
        let register_list = self.register_list();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(rb, address + 64);
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address += 4
                }

                cpu.idle_cycle();
                if !register_list.contains(&rb) {
                    cpu.set_register(rb, address);
                }
            }
            false => {
                if register_list.is_empty() {
                    let value = cpu.pc() + 2;
                    cpu.store_32(address, value, memory_access as u8);
                    cpu.set_register(rb, address + 64);
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                for (i, register) in register_list.iter().enumerate() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);

                    if i == 0 {
                        cpu.set_register(rb, address + register_list.len() as u32 * 4);
                    }

                    memory_access = MemoryAccess::Sequential;
                    address += 4
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rb = self.rb();
        let load = self.load();
        let register_list = self
            .register_list()
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match load {
            true => format!("LDMIA {}!,{{{}}}", rb, register_list),
            false => format!("STMIA {}!,{{{}}}", rb, register_list),
        }
    }
}

impl MultipleLoadStore {
    #[inline]
    pub fn register_list(&self) -> Vec<usize> {
        (0..=7).filter(|&i| self.value.bit(i)).collect()
    }

    #[inline]
    pub fn rb(&self) -> LoRegister {
        self.value.bits(8..=10).into()
    }

    #[inline]
    pub fn load(&self) -> bool {
        self.value.bit(11)
    }
}
