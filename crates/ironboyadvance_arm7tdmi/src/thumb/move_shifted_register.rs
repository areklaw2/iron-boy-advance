use crate::{
    BitOps, CpuAction, LoRegister,
    barrel_shifter::{ShiftType, asr, lsl, lsr},
    cpu::{Arm7tdmiCpu, Instruction},
    memory::{MemoryAccess, MemoryInterface},
};

#[derive(Debug, Clone, Copy)]
pub struct MoveShiftedRegister {
    rd: LoRegister,
    rs: LoRegister,
    offset: u16,
    opcode: u16,
}

impl MoveShiftedRegister {
    pub fn new(value: u16) -> Self {
        Self {
            rd: value.bits(0..=2).into(),
            rs: value.bits(3..=5).into(),
            offset: value.bits(6..=10),
            opcode: value.bits(11..=12),
        }
    }
}

impl Instruction for MoveShiftedRegister {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd as usize;
        let rs = self.rs as usize;
        let offset5 = self.offset as u32;

        let value = cpu.register(rs);
        let mut carry = cpu.cpsr().carry();
        let result = match ShiftType::from(self.opcode) {
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
        let shift_type = ShiftType::from(self.opcode);
        let offset5 = self.offset;
        let rs = self.rs;
        let rd = self.rd;
        format!("{} {},{},#{}", shift_type, rd, rs, offset5)
    }
}
