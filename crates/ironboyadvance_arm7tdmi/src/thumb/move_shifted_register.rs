use crate::{
    BitOps, CpuAction, LoRegister,
    barrel_shifter::{ShiftType, asr, lsl, lsr},
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
    thumb::thumb_instruction,
};

#[derive(Debug, Clone, Copy)]
pub struct MoveShiftedRegister {
    value: u16,
}

thumb_instruction!(MoveShiftedRegister);

impl Instruction for MoveShiftedRegister {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rs = self.rs() as usize;
        let offset5 = self.offset() as u32;

        let value = cpu.register(rs);
        let mut carry = cpu.cpsr().carry();
        let result = match self.opcode().into() {
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

    fn disassemble<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) -> String {
        let shift_type = self.opcode();
        let offset5 = self.offset();
        let rs = self.rs();
        let rd = self.rd();
        format!("{} {},{},#{}", shift_type, rd, rs, offset5)
    }
}

impl MoveShiftedRegister {
    #[inline]
    pub fn rd(&self) -> LoRegister {
        self.value.bits(0..=2).into()
    }

    #[inline]
    pub fn rs(&self) -> LoRegister {
        self.value.bits(3..=5).into()
    }

    #[inline]
    pub fn offset(&self) -> u16 {
        self.value.bits(6..=10)
    }

    #[inline]
    pub fn opcode(&self) -> u16 {
        self.value.bits(11..=12)
    }
}
