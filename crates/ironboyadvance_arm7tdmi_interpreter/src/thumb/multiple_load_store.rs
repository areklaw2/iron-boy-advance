use crate::Execute;
use ironboyadvance_arm7tdmi::{CpuAction, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::MultipleLoadStore};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for MultipleLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rb = self.rb() as usize;
        let mut address = cpu.register(rb);
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<usize> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(rb, address.wrapping_add(64));
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                cpu.idle_cycle();
                if !register_list.contains(&rb) {
                    cpu.set_register(rb, address);
                }
            }
            false => {
                if register_list.is_empty() {
                    let value = cpu.pc().wrapping_add(2);
                    cpu.store_32(address, value, memory_access as u8);
                    cpu.set_register(rb, address.wrapping_add(64));
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                for (i, register) in register_list.iter().enumerate() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);

                    if i == 0 {
                        cpu.set_register(rb, address.wrapping_add(register_list.len() as u32 * 4));
                    }

                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}
