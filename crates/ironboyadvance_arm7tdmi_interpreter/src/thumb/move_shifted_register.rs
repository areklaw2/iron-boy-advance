use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction,
    barrel_shifter::{ShiftType, asr, lsl, lsr},
    cpu::Arm7tdmiCpu,
    memory::MemoryInterface,
    thumb::MoveShiftedRegister,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for MoveShiftedRegister {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rs = self.rs() as usize;
        let offset5 = self.offset() as u32;

        let value = cpu.register(rs);
        let mut carry = cpu.cpsr().carry();
        let result = match ShiftType::from(self.opcode()) {
            ShiftType::LSL => lsl(value, offset5, &mut carry),
            ShiftType::LSR => lsr(value, offset5, &mut carry, true),
            ShiftType::ASR => asr(value, offset5, &mut carry, true),
            ShiftType::ROR => unimplemented!(),
        };

        cpu.cpsr_mut().set_negative(result >> 31 != 0);
        cpu.cpsr_mut().set_zero(result == 0);
        cpu.cpsr_mut().set_carry(carry);

        cpu.set_register(rd, result);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}
