use ironboyadvance_utils::bit::BitOps;

use crate::{
    Condition, CpuAction, DataProcessingOpcode, Register,
    alu::*,
    barrel_shifter::*,
    cpu::{Arm7tdmiCpu, PC},
    memory::{MemoryAccess, MemoryInterface},
};

use DataProcessingOpcode::*;

#[derive(Debug, Clone)]
pub struct DataProcessing {
    value: u32,
    executed_pc: u32,
}

impl DataProcessing {
    #[inline]
    pub fn cond(&self) -> Condition {
        self.value.bits(28..=31).into()
    }

    pub fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
        let rn = self.rn() as usize;
        let mut operand1 = cpu.register(rn);
        let mut carry = cpu.cpsr().carry();

        let operand2 = match self.is_immediate() {
            true => {
                let rotate = 2 * self.rotate();
                let immediate = self.immediate();
                ror(immediate, rotate, &mut carry, false)
            }
            false => {
                let rm = self.rm() as usize;
                let mut rm_value = cpu.register(rm);
                let shift_by = self.shift_by();
                let shift_amount = match shift_by {
                    ShiftBy::Immediate => self.shift_amount(),
                    ShiftBy::Register => {
                        if rn == PC {
                            operand1 += 4;
                        }
                        if rm == PC {
                            rm_value += 4;
                        }
                        cpu_action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                        cpu.idle_cycle();
                        cpu.register(self.rs() as usize) & 0xFF
                    }
                };
                match self.shift_type() {
                    ShiftType::LSL => lsl(rm_value, shift_amount, &mut carry),
                    ShiftType::LSR => lsr(rm_value, shift_amount, &mut carry, shift_by.into()),
                    ShiftType::ASR => asr(rm_value, shift_amount, &mut carry, shift_by.into()),
                    ShiftType::ROR => ror(rm_value, shift_amount, &mut carry, shift_by.into()),
                }
            }
        };

        let set_flags = self.sets_flags();
        let opcode = self.opcode();
        let result = match opcode {
            AND => and(cpu, set_flags, operand1, operand2, carry),
            EOR => eor(cpu, set_flags, operand1, operand2, carry),
            SUB => sub(cpu, set_flags, operand1, operand2),
            RSB => rsb(cpu, set_flags, operand2, operand1),
            ADD => add(cpu, set_flags, operand1, operand2),
            ADC => adc(cpu, set_flags, operand1, operand2),
            SBC => sbc(cpu, set_flags, operand1, operand2),
            RSC => rsc(cpu, set_flags, operand2, operand1),
            TST => tst(cpu, set_flags, operand1, operand2, carry),
            TEQ => teq(cpu, set_flags, operand1, operand2, carry),
            CMP => cmp(cpu, set_flags, operand1, operand2),
            CMN => cmn(cpu, set_flags, operand1, operand2),
            ORR => orr(cpu, set_flags, operand1, operand2, carry),
            MOV => mov(cpu, set_flags, operand2, carry),
            BIC => bic(cpu, set_flags, operand1, operand2, carry),
            MVN => mvn(cpu, set_flags, operand2, carry),
        };

        let rd = self.rd() as usize;
        if set_flags && rd == PC {
            let spsr = cpu.spsr();
            cpu.set_cpsr(spsr);
        }

        if !matches!(opcode, TST | TEQ | CMP | CMN) {
            cpu.set_register(rd, result);
            if rd == PC {
                cpu.pipeline_flush();
                cpu_action = CpuAction::PipelineFlush
            }
        }

        cpu_action
    }

    pub fn disassemble(&self) -> String {
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

    #[inline]
    pub fn rn(&self) -> Register {
        self.value.bits(16..=19).into()
    }

    #[inline]
    pub fn rm(&self) -> Register {
        self.value.bits(0..=3).into()
    }

    #[inline]
    pub fn rs(&self) -> Register {
        self.value.bits(8..=11).into()
    }

    #[inline]
    pub fn rd(&self) -> Register {
        self.value.bits(12..=15).into()
    }

    #[inline]
    pub fn is_immediate(&self) -> bool {
        self.value.bit(25)
    }

    #[inline]
    pub fn opcode(&self) -> DataProcessingOpcode {
        self.value.bits(21..=24).into()
    }

    #[inline]
    pub fn sets_flags(&self) -> bool {
        self.value.bit(20)
    }

    #[inline]
    pub fn shift_by(&self) -> ShiftBy {
        match self.value.bit(4) {
            true => ShiftBy::Register,
            false => ShiftBy::Immediate,
        }
    }

    #[inline]
    pub fn shift_amount(&self) -> u32 {
        self.value.bits(7..=11)
    }

    #[inline]
    pub fn shift_type(&self) -> ShiftType {
        self.value.bits(5..=6).into()
    }

    #[inline]
    pub fn rotate(&self) -> u32 {
        self.value.bits(8..=11)
    }

    #[inline]
    pub fn immediate(&self) -> u32 {
        self.value.bits(0..=7)
    }
}
