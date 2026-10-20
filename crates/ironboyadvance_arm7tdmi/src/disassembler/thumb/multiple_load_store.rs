use crate::{Dissasemble, LoRegister, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::MultipleLoadStore};

impl Dissasemble for MultipleLoadStore {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rb = self.rb();
        let load = self.load();
        let register_list: Vec<usize> = (0..8).filter(|&i| (self.register_list_bits() >> i) & 1 == 1).collect();
        let register_list_str = register_list
            .iter()
            .map(|register| LoRegister::from(*register as u16).to_string())
            .collect::<Vec<String>>()
            .join(",");

        match load {
            true => format!("LDMIA {}!,{{{}}}", rb, register_list_str),
            false => format!("STMIA {}!,{{{}}}", rb, register_list_str),
        }
    }
}
