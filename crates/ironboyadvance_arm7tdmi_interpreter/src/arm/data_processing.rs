use crate::Execute;
use ironboyadvance_arm7tdmi::{
    CpuAction, DataProcessingOpcode,
    alu::*,
    arm::DataProcessing,
    barrel_shifter::*,
    cpu::{Arm7tdmiCpu, PC},
    memory::MemoryInterface,
};
use ironboyadvance_common::memory::MemoryAccess;

impl Execute for DataProcessing {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use DataProcessingOpcode::*;

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
            cpu.set_cpsr(cpu.spsr());
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
}
