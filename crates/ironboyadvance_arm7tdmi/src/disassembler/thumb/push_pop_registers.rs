use crate::{Dissasemble, LoRegister, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::PushPopRegisters};

impl Dissasemble for PushPopRegisters {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let load = self.load();
        let store_lr_load_pc = self.store_lr_load_pc();
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits() >> i) & 1 == 1).collect();
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
