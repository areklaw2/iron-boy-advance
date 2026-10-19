use crate::{Dissasemble, barrel_shifter::ShiftType, cpu::Arm7tdmiCpu, memory::MemoryInterface, thumb::MoveShiftedRegister};

impl Dissasemble for MoveShiftedRegister {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let shift_type = ShiftType::from(self.opcode());
        let offset5 = self.offset();
        let rs = self.rs();
        let rd = self.rd();
        format!("{} {},{},#{}", shift_type, rd, rs, offset5)
    }
}
