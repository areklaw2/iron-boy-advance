use crate::Execute;
use ironboyadvance_arm7tdmi::{
    AluOperationsOpcode, CpuAction, CpuState, Exception, HiRegOpsBxOpcode, MovCmpAddSubImmediateOpcode,
    alu::*,
    barrel_shifter::{ShiftType, asr, lsl, lsr, ror},
    cpu::{Arm7tdmiCpu, LR, PC, SP},
    memory::MemoryInterface,
    thumb::{
        AddOffsetToSp, AddSubtract, AluOperations, ConditionalBranch, HiRegisterOperationsBranchExchange, LoadAddress,
        LoadStoreHalfword, LoadStoreImmediateOffset, LoadStoreRegisterOffset, LoadStoreSignExtendedByteHalfword,
        LongBranchWithLink, MoveCompareAddSubtractImmediate, MoveShiftedRegister, MultipleLoadStore, PcRelativeLoad,
        PushPopRegisters, SoftwareInterrupt, SpRelativeLoadStore, ThumbInstruction, UnconditionalBranch, Undefined,
    },
};
use ironboyadvance_common::{bits::SignExtend, memory::MemoryAccess};

impl Execute for ThumbInstruction {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self {
            Self::MoveShiftedRegister(i) => i.execute(cpu),
            Self::AddSubtract(i) => i.execute(cpu),
            Self::MoveCompareAddSubtractImmediate(i) => i.execute(cpu),
            Self::AluOperations(i) => i.execute(cpu),
            Self::HiRegisterOperationsBranchExchange(i) => i.execute(cpu),
            Self::PcRelativeLoad(i) => i.execute(cpu),
            Self::LoadStoreRegisterOffset(i) => i.execute(cpu),
            Self::LoadStoreSignExtendedByteHalfword(i) => i.execute(cpu),
            Self::LoadStoreImmediateOffset(i) => i.execute(cpu),
            Self::LoadStoreHalfword(i) => i.execute(cpu),
            Self::SpRelativeLoadStore(i) => i.execute(cpu),
            Self::LoadAddress(i) => i.execute(cpu),
            Self::AddOffsetToSp(i) => i.execute(cpu),
            Self::PushPopRegisters(i) => i.execute(cpu),
            Self::MultipleLoadStore(i) => i.execute(cpu),
            Self::ConditionalBranch(i) => i.execute(cpu),
            Self::SoftwareInterrupt(i) => i.execute(cpu),
            Self::UnconditionalBranch(i) => i.execute(cpu),
            Self::LongBranchWithLink(i) => i.execute(cpu),
            Self::Undefined(i) => i.execute(cpu),
        }
    }
}

pub type ThumbInstructionFactory = fn(u16) -> ThumbInstruction;

pub(crate) fn generate_thumb_lut() -> [ThumbInstructionFactory; 1024] {
    let mut thumb_lut: [ThumbInstructionFactory; 1024] = [|value| ThumbInstruction::Undefined(Undefined::new(value)); 1024];
    for (i, factory) in thumb_lut.iter_mut().enumerate() {
        *factory = decode_thumb((i as u16) << 6);
    }
    thumb_lut
}

fn decode_thumb(instruction: u16) -> ThumbInstructionFactory {
    if instruction & 0xF800 < 0x1800 {
        |value| ThumbInstruction::MoveShiftedRegister(MoveShiftedRegister::new(value))
    } else if instruction & 0xF800 == 0x1800 {
        |value| ThumbInstruction::AddSubtract(AddSubtract::new(value))
    } else if instruction & 0xE000 == 0x2000 {
        |value| ThumbInstruction::MoveCompareAddSubtractImmediate(MoveCompareAddSubtractImmediate::new(value))
    } else if instruction & 0xFC00 == 0x4000 {
        |value| ThumbInstruction::AluOperations(AluOperations::new(value))
    } else if instruction & 0xFC00 == 0x4400 {
        |value| ThumbInstruction::HiRegisterOperationsBranchExchange(HiRegisterOperationsBranchExchange::new(value))
    } else if instruction & 0xF800 == 0x4800 {
        |value| ThumbInstruction::PcRelativeLoad(PcRelativeLoad::new(value))
    } else if instruction & 0xF200 == 0x5000 {
        |value| ThumbInstruction::LoadStoreRegisterOffset(LoadStoreRegisterOffset::new(value))
    } else if instruction & 0xF200 == 0x5200 {
        |value| ThumbInstruction::LoadStoreSignExtendedByteHalfword(LoadStoreSignExtendedByteHalfword::new(value))
    } else if instruction & 0xE000 == 0x6000 {
        |value| ThumbInstruction::LoadStoreImmediateOffset(LoadStoreImmediateOffset::new(value))
    } else if instruction & 0xF000 == 0x8000 {
        |value| ThumbInstruction::LoadStoreHalfword(LoadStoreHalfword::new(value))
    } else if instruction & 0xF000 == 0x9000 {
        |value| ThumbInstruction::SpRelativeLoadStore(SpRelativeLoadStore::new(value))
    } else if instruction & 0xF000 == 0xA000 {
        |value| ThumbInstruction::LoadAddress(LoadAddress::new(value))
    } else if instruction & 0xFF00 == 0xB000 {
        |value| ThumbInstruction::AddOffsetToSp(AddOffsetToSp::new(value))
    } else if instruction & 0xF600 == 0xB400 {
        |value| ThumbInstruction::PushPopRegisters(PushPopRegisters::new(value))
    } else if instruction & 0xF000 == 0xC000 {
        |value| ThumbInstruction::MultipleLoadStore(MultipleLoadStore::new(value))
    } else if instruction & 0xFF00 < 0xDF00 {
        |value| ThumbInstruction::ConditionalBranch(ConditionalBranch::new(value))
    } else if instruction & 0xFF00 == 0xDF00 {
        |value| ThumbInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value))
    } else if instruction & 0xF800 == 0xE000 {
        |value| ThumbInstruction::UnconditionalBranch(UnconditionalBranch::new(value))
    } else if instruction & 0xF000 == 0xF000 {
        |value| ThumbInstruction::LongBranchWithLink(LongBranchWithLink::new(value))
    } else {
        |value| ThumbInstruction::Undefined(Undefined::new(value))
    }
}

impl Execute for AddOffsetToSp {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset() * 4;
        let sp_value = cpu.register(SP);
        let value = match self.signed() {
            true => sp_value.wrapping_sub(offset as u32),
            false => sp_value.wrapping_add(offset as u32),
        };
        cpu.set_register(SP, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}

impl Execute for AddSubtract {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let operand1 = cpu.register(self.rs() as usize);
        let operand2 = match self.is_immediate() {
            true => self.offset() as u32,
            false => cpu.register(self.rn() as usize),
        };

        let result = match self.opcode() != 0 {
            true => sub(cpu, true, operand1, operand2),
            false => add(cpu, true, operand1, operand2),
        };

        cpu.set_register(rd, result);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}

impl Execute for AluOperations {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use AluOperationsOpcode::*;
        let rd = self.rd() as usize;
        let operand1 = cpu.register(rd);
        let mut operand2 = cpu.register(self.rs() as usize);
        let mut carry = cpu.cpsr().carry();
        let opcode: AluOperationsOpcode = self.opcode().into();
        let mut access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);

        let result = match opcode {
            AND => and(cpu, true, operand1, operand2, carry),
            EOR => eor(cpu, true, operand1, operand2, carry),
            LSL => {
                operand2 &= 0xFF;
                let result = lsl(operand1, operand2, &mut carry);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            LSR => {
                operand2 &= 0xFF;
                let result = lsr(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            ASR => {
                operand2 &= 0xFF;
                let result = asr(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            ADC => adc(cpu, true, operand1, operand2),
            SBC => sbc(cpu, true, operand1, operand2),
            ROR => {
                operand2 &= 0xFF;
                let result = ror(operand1, operand2, &mut carry, false);
                cpu.idle_cycle();
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                cpu.cpsr_mut().set_carry(carry);
                result
            }
            TST => tst(cpu, true, operand1, operand2, carry),
            NEG => sub(cpu, true, 0, operand2),
            CMP => cmp(cpu, true, operand1, operand2),
            CMN => cmn(cpu, true, operand1, operand2),
            ORR => orr(cpu, true, operand1, operand2, carry),
            MUL => {
                let multiplier_cycles = multiplier_array_cycles(operand1);
                for _ in 0..multiplier_cycles {
                    cpu.idle_cycle();
                }
                access = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);

                let result = operand1.wrapping_mul(operand2);
                cpu.cpsr_mut().set_negative(result >> 31 != 0);
                cpu.cpsr_mut().set_zero(result == 0);
                result
            }
            BIC => bic(cpu, true, operand1, operand2, carry),
            MVN => mvn(cpu, true, operand2, carry),
        };

        if ![TST, CMP, CMN].contains(&opcode) {
            cpu.set_register(rd, result);
        }

        access
    }
}

impl Execute for ConditionalBranch {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let condition = self.cond();
        if !cpu.is_condition_met(condition) {
            CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
        } else {
            let offset = self.offset().sign_extend(8) << 1;
            cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
            cpu.pipeline_flush();
            CpuAction::PipelineFlush
        }
    }
}

impl Execute for HiRegisterOperationsBranchExchange {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use HiRegOpsBxOpcode::*;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential);
        let destination = match self.h1() {
            true => self.hd() as usize + 8,
            false => self.rd() as usize,
        };
        let operand1 = cpu.register(destination);

        let source = match self.h2() {
            true => self.hs() as usize + 8,
            false => self.rs() as usize,
        };
        let mut operand2 = cpu.register(source);
        if source == PC {
            operand2 &= !0x1
        }

        match self.opcode().into() {
            CMP => {
                cmp(cpu, true, operand1, operand2);
            }
            ADD => {
                let result = add(cpu, false, operand1, operand2);
                cpu.set_register(destination, result);
                if destination == PC {
                    cpu.set_pc(cpu.pc() & !0x1);
                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            MOV => {
                let result = mov(cpu, false, operand2, cpu.cpsr().carry());
                cpu.set_register(destination, result);
                if destination == PC {
                    cpu.set_pc(cpu.pc() & !0x1);
                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            BX => {
                cpu.cpsr_mut().set_state(CpuState::from_bits((operand2 & 0x1) as u8));
                cpu.set_pc(operand2 & !0x1);
                cpu.pipeline_flush();
                action = CpuAction::PipelineFlush;
            }
        };

        action
    }
}

impl Execute for LoadAddress {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let offset = self.offset() * 4;
        let value = match self.sp() {
            true => cpu.register(SP).wrapping_add(offset as u32),
            false => (cpu.pc() & !0b10).wrapping_add(offset as u32),
        };
        cpu.set_register(rd, value);
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}

impl Execute for LoadStoreHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 2;
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for LoadStoreImmediateOffset {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = if self.byte() { self.offset() } else { self.offset() * 4 };
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(immediate as u32);

        let rd = self.rd() as usize;
        let byte = self.byte();
        let load = self.load();
        match (load, byte) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.register(rd);
                cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
            }
            (true, false) => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for LoadStoreRegisterOffset {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro() as usize);
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd() as usize;
        let byte = self.byte();
        let load = self.load();
        match (load, byte) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.register(rd);
                cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8);
            }
            (true, false) => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for LoadStoreSignExtendedByteHalfword {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let ro_value = cpu.register(self.ro() as usize);
        let rb_value = cpu.register(self.rb() as usize);
        let address = rb_value.wrapping_add(ro_value);

        let rd = self.rd() as usize;
        let signed = self.signed();
        let halfword = self.halfword();
        match (signed, halfword) {
            (false, false) => {
                let value = cpu.register(rd);
                cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
            }
            (false, true) => {
                let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, false) => {
                let value = cpu.load_signed_8(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            (true, true) => {
                let value = cpu.load_signed_16(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for LongBranchWithLink {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self.high() {
            true => {
                let offset = (self.offset() as u32) << 1;
                let temp = (cpu.pc() - 2) | 0b1;
                cpu.set_pc((cpu.register(LR) & !0b1).wrapping_add(offset));
                cpu.set_register(LR, temp);
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => {
                let offset = self.offset().sign_extend(11) << 12;
                cpu.set_register(LR, cpu.pc().wrapping_add(offset as u32));
                CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
            }
        }
    }
}

impl Execute for MoveCompareAddSubtractImmediate {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        use MovCmpAddSubImmediateOpcode::*;
        let rd = self.rd() as usize;
        let operand1 = cpu.register(rd);
        let offset = self.offset();
        let opcode: MovCmpAddSubImmediateOpcode = self.opcode().into();
        let result = match opcode {
            MOV => mov(cpu, true, offset as u32, cpu.cpsr().carry()),
            CMP => cmp(cpu, true, operand1, offset as u32),
            ADD => add(cpu, true, operand1, offset as u32),
            SUB => sub(cpu, true, operand1, offset as u32),
        };

        if opcode != CMP {
            cpu.set_register(rd, result);
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}

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

impl Execute for MultipleLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rb = self.rb() as usize;
        let mut address = cpu.register(rb);
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<usize> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(rb, address.wrapping_add(64));
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                cpu.idle_cycle();
                if !register_list.contains(&rb) {
                    cpu.set_register(rb, address);
                }
            }
            false => {
                if register_list.is_empty() {
                    let value = cpu.pc().wrapping_add(2);
                    cpu.store_32(address, value, memory_access as u8);
                    cpu.set_register(rb, address.wrapping_add(64));
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                for (i, register) in register_list.iter().enumerate() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);

                    if i == 0 {
                        cpu.set_register(rb, address.wrapping_add(register_list.len() as u32 * 4));
                    }

                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for PcRelativeLoad {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset();
        let address = (cpu.register(PC) & !0x2).wrapping_add((offset << 2) as u32);
        let value = cpu.load_32(address, MemoryAccess::NonSequential as u8);
        cpu.set_register(self.rd() as usize, value);
        cpu.idle_cycle();
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for PushPopRegisters {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let mut address = cpu.register(SP);
        let register_list_bits = self.register_list_bits();
        let register_list: Vec<usize> = (0..8).filter(|&i| (register_list_bits >> i) & 1 == 1).collect();
        let store_lr_load_pc = self.store_lr_load_pc();

        let mut memory_access = MemoryAccess::NonSequential;
        match self.load() {
            true => {
                if register_list.is_empty() && !store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_pc(value);
                    cpu.set_register(SP, address.wrapping_add(64));
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                for register in register_list.iter() {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(*register, value);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                if store_lr_load_pc {
                    let value = cpu.load_32(address, memory_access as u8);
                    cpu.set_register(PC, value & !0b1);
                    cpu.set_register(SP, address.wrapping_add(4));
                    cpu.idle_cycle();
                    cpu.pipeline_flush();
                    return CpuAction::PipelineFlush;
                }

                cpu.idle_cycle();
                cpu.set_register(SP, address);
            }
            false => {
                if register_list.is_empty() && !store_lr_load_pc {
                    address = address.wrapping_sub(64);
                    cpu.set_register(SP, address);
                    let value = cpu.pc().wrapping_add(2);
                    cpu.store_32(address, value, memory_access as u8);
                    return CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
                }

                address = address.wrapping_sub(register_list.len() as u32 * 4);
                if store_lr_load_pc {
                    address = address.wrapping_sub(4)
                }
                cpu.set_register(SP, address);

                for register in register_list.iter() {
                    let value = cpu.register(*register);
                    cpu.store_32(address, value, memory_access as u8);
                    memory_access = MemoryAccess::Sequential;
                    address = address.wrapping_add(4)
                }

                if store_lr_load_pc {
                    let value = cpu.register(LR);
                    cpu.store_32(address, value, memory_access as u8);
                }
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for SoftwareInterrupt {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match !cpu.bios_loaded() && cpu.bios_call(self.offset() as u32) {
            true => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
            false => {
                cpu.exception(Exception::SoftwareInterrupt);
                CpuAction::PipelineFlush
            }
        }
    }
}

impl Execute for SpRelativeLoadStore {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let immediate = self.offset() * 4;
        let sp_value = cpu.register(SP);
        let address = sp_value.wrapping_add(immediate as u32);
        let rd = self.rd() as usize;
        match self.load() {
            true => {
                let value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.set_register(rd, value);
                cpu.idle_cycle();
            }
            false => {
                let value = cpu.register(rd);
                cpu.store_32(address, value, MemoryAccess::NonSequential as u8);
            }
        }

        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential)
    }
}

impl Execute for UnconditionalBranch {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let offset = self.offset().sign_extend(11) << 1;
        cpu.set_pc(cpu.pc().wrapping_add(offset as u32));
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}

impl Execute for Undefined {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }
}
