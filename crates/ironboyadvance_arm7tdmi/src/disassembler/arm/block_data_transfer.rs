use crate::{Dissasemble, Register, arm::BlockDataTransfer, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for BlockDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let pre_index = self.pre_index();
        let add = self.add();
        let load_psr_force_user = if self.load_psr_force_user() { "^" } else { "" };
        let write_back = if self.write_back() { "!" } else { "" };
        let load = self.load();
        let rn = self.rn();
        let register_list = (0..16u32)
            .filter(|&i| (self.register_list() >> i) & 1 == 1)
            .map(|i| Register::from(i).to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mnemonic = match (load, pre_index, add) {
            (true, true, true) => match rn == Register::R13 {
                true => format!("LDM{}ED", cond),
                false => format!("LDM{}IB", cond),
            },
            (true, false, true) => match rn == Register::R13 {
                true => format!("LDM{}FD", cond),
                false => format!("LDM{}IA", cond),
            },
            (true, true, false) => match rn == Register::R13 {
                true => format!("LDM{}EA", cond),
                false => format!("LDM{}DB", cond),
            },
            (true, false, false) => match rn == Register::R13 {
                true => format!("LDM{}FA", cond),
                false => format!("LDM{}DA", cond),
            },
            (false, true, true) => match rn == Register::R13 {
                true => format!("STM{}FA", cond),
                false => format!("STM{}IB", cond),
            },
            (false, false, true) => match rn == Register::R13 {
                true => format!("STM{}EA", cond),
                false => format!("STM{}IA", cond),
            },
            (false, true, false) => match rn == Register::R13 {
                true => format!("STM{}FD", cond),
                false => format!("STM{}DB", cond),
            },
            (false, false, false) => match rn == Register::R13 {
                true => format!("STM{}ED", cond),
                false => format!("STM{}DA", cond),
            },
        };

        format!("{} {}{},({}){}", mnemonic, rn, write_back, register_list, load_psr_force_user)
    }
}
