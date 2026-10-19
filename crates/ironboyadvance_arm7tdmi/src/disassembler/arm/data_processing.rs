use crate::{
    DataProcessingOpcode, Dissasemble, arm::DataProcessing, barrel_shifter::ShiftBy, cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
};

impl Dissasemble for DataProcessing {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        use DataProcessingOpcode::*;

        let cond = self.cond();
        let opcode = self.opcode();
        let s = if self.sets_flags() { "S" } else { "" };
        let rd = self.rd();
        let rn = self.rn();
        let operand_2 = match self.is_immediate() {
            true => {
                let rotate = 2 * self.rotate();
                let immediate = self.immediate();
                format!("0x{:08X}", immediate.rotate_right(rotate))
            }
            false => {
                let rm = self.rm();
                let shift_type = self.shift_type();
                match self.shift_by() {
                    ShiftBy::Register => {
                        format!("{},{} {}", rm, shift_type, self.rs())
                    }
                    ShiftBy::Immediate => {
                        format!("{},{} #{}", rm, shift_type, self.shift_amount())
                    }
                }
            }
        };

        match opcode {
            MOV | MVN => format!("{opcode}{cond}{s} {rd},{operand_2}"),
            CMP | CMN | TEQ | TST => format!("{opcode}{cond} {rn},{operand_2}"),
            _ => format!("{opcode}{cond}{s} {rd},{rn},{operand_2}"),
        }
    }
}
