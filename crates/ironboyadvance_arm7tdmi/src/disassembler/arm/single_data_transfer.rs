use crate::{Dissasemble, arm::SingleDataTransfer, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for SingleDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let pre_index = self.pre_index();
        let t = if pre_index { "" } else { "T" };
        let add = if self.add() { "+" } else { "-" };
        let byte = if self.byte() { "B" } else { "" };
        let rn = self.rn();
        let rd = self.rd();
        let immediate = self.immediate();
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let offset = match self.is_immediate() {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => {
                        let rm = self.rm();
                        let shift_type = self.shift_type();
                        format!(",{}{},{} #{}", add, rm, shift_type, self.shift_amount())
                    }
                };

                let write_back = if self.write_back() && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        match self.load() {
            true => format!("LDR{}{}{} {},{}", cond, byte, t, rd, address),
            false => format!("STR{}{}{} {},{}", cond, byte, t, rd, address),
        }
    }
}
