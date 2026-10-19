use crate::{
    Dissasemble, MovCmpAddSubImmediateOpcode, cpu::Arm7tdmiCpu, memory::MemoryInterface,
    thumb::MoveCompareAddSubtractImmediate,
};

impl Dissasemble for MoveCompareAddSubtractImmediate {
    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let rd = self.rd();
        let offset = self.offset();
        let opcode = MovCmpAddSubImmediateOpcode::from(self.opcode());
        format!("{} {},#{}", opcode, rd, offset)
    }
}
