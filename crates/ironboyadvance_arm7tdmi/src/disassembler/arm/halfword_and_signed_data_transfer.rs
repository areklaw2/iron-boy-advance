use crate::{Dissasemble, arm::HalfwordAndSignedDataTransfer, cpu::Arm7tdmiCpu, memory::MemoryInterface};

impl Dissasemble for HalfwordAndSignedDataTransfer {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let cond = self.cond();
        let pre_index = self.pre_index();
        let add = if self.add() { "+" } else { "-" };
        let rn = self.rn();
        let rd = self.rd();
        let immediate = self.immediate_hi() << 4 | self.immediate_lo();
        let address = match rd as usize == 15 {
            true => format!("#{:08X}", immediate),
            false => {
                let rm = self.rm();
                let offset = match self.is_immediate() {
                    true => match immediate {
                        0 => "".into(),
                        _ => format!(",#{}{}", add, immediate),
                    },
                    false => format!(",{}{}", add, rm),
                };

                let write_back = if self.write_back() && !offset.is_empty() { "!" } else { "" };
                match pre_index {
                    true => format!("[{}{}]{}", rn, offset, write_back),
                    false => format!("[{}]{}", rn, offset),
                }
            }
        };

        let s = self.signed();
        let h = self.halfword();
        let sh = match (s, h) {
            (false, false) => "",
            (false, true) => "H",
            (true, false) => "SB",
            (true, true) => "SH",
        };

        match self.load() {
            true => format!("LDR{}{} {},{}", cond, sh, rd, address),
            false => format!("STR{}{} {},{}", cond, sh, rd, address),
        }
    }
}
