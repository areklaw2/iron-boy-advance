use crate::Compile;
use ironboyadvance_arm7tdmi::{
    CpuAction, CpuMode, CpuState, DataProcessingOpcode, Exception,
    alu::*,
    arm::{
        ArmInstruction, BlockDataTransfer, BranchAndBranchWithLink, BranchAndExchange, DataProcessing,
        HalfwordAndSignedDataTransfer, Multiply, MultiplyLong, PsrTransfer, SingleDataSwap, SingleDataTransfer,
        SoftwareInterrupt, Undefined,
    },
    barrel_shifter::*,
    cpu::{Arm7tdmiCpu, LR, PC},
    memory::MemoryInterface,
    psr::ProgramStatusRegister,
};
use ironboyadvance_common::{
    bits::{BitOps, SignExtend},
    memory::MemoryAccess,
};

impl Compile for ArmInstruction {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match self {
            Self::DataProcessing(i) => i.compile(cpu),
            Self::PsrTransfer(i) => i.compile(cpu),
            Self::Multiply(i) => i.compile(cpu),
            Self::MultiplyLong(i) => i.compile(cpu),
            Self::SingleDataSwap(i) => i.compile(cpu),
            Self::BranchAndExchange(i) => i.compile(cpu),
            Self::HalfwordAndSignedDataTransfer(i) => i.compile(cpu),
            Self::SingleDataTransfer(i) => i.compile(cpu),
            Self::Undefined(i) => i.compile(cpu),
            Self::BlockDataTransfer(i) => i.compile(cpu),
            Self::BranchAndBranchWithLink(i) => i.compile(cpu),
            Self::SoftwareInterrupt(i) => i.compile(cpu),
            Self::CoprocessorDataOperation(i) => i.compile(cpu),
            Self::CoprocessorDataTransfer(i) => i.compile(cpu),
            Self::CoprocessorRegisterTransfer(i) => i.compile(cpu),
        }
    }
}

pub fn decode_arm(value: u32) -> ArmInstruction {
    let pattern = value & 0x0FFFFFFF;
    let set_flags = pattern.bit(20);
    let opcode = pattern.bits(21..=24);
    let test_opcode = (0b1000..=0b1011).contains(&opcode);
    match pattern.bits(26..=27) {
        0b00 => {
            if pattern.bit(25) {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            } else if pattern & 0x0FF000F0 == 0x01200010 {
                ArmInstruction::BranchAndExchange(BranchAndExchange::new(value))
            } else if pattern & 0x010000F0 == 0x00000090 {
                match pattern.bit(23) {
                    true => ArmInstruction::MultiplyLong(MultiplyLong::new(value)),
                    false => ArmInstruction::Multiply(Multiply::new(value)),
                }
            } else if pattern & 0x010000F0 == 0x01000090 {
                ArmInstruction::SingleDataSwap(SingleDataSwap::new(value))
            } else if pattern & 0x000000F0 == 0x000000B0 || pattern & 0x000000D0 == 0x000000D0 {
                ArmInstruction::HalfwordAndSignedDataTransfer(HalfwordAndSignedDataTransfer::new(value))
            } else {
                match !set_flags && test_opcode {
                    true => ArmInstruction::PsrTransfer(PsrTransfer::new(value)),
                    false => ArmInstruction::DataProcessing(DataProcessing::new(value)),
                }
            }
        }
        0b01 => match pattern & 0x02000010 == 0x02000010 {
            true => ArmInstruction::Undefined(Undefined::new(value)),
            false => ArmInstruction::SingleDataTransfer(SingleDataTransfer::new(value)),
        },
        0b10 => match pattern.bit(25) {
            true => ArmInstruction::BranchAndBranchWithLink(BranchAndBranchWithLink::new(value)),
            false => ArmInstruction::BlockDataTransfer(BlockDataTransfer::new(value)),
        },
        0b11 => match pattern.bit(25) {
            true => match pattern.bit(24) {
                true => ArmInstruction::SoftwareInterrupt(SoftwareInterrupt::new(value)),
                false => match pattern.bit(4) {
                    true => ArmInstruction::CoprocessorRegisterTransfer(Undefined::new(value)),
                    false => ArmInstruction::CoprocessorDataOperation(Undefined::new(value)),
                },
            },
            false => ArmInstruction::CoprocessorDataTransfer(Undefined::new(value)),
        },
        _ => ArmInstruction::Undefined(Undefined::new(value)),
    }
}

impl Compile for BlockDataTransfer {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rn = self.rn() as usize;
        let mut address = cpu.register(rn);

        let is_empty_list = self.register_list() == 0;
        let register_list: u16 = if is_empty_list { 1 << PC } else { self.register_list() };
        let transfer_pc = (register_list >> PC) & 1 == 1;
        let transfer_bytes = if is_empty_list { 64 } else { register_list.count_ones() * 4 };

        let load = self.load();
        let load_psr_force_user = self.load_psr_force_user();
        let mode = cpu.cpsr().mode();
        let switch_mode =
            load_psr_force_user && (!load || !transfer_pc) && ![CpuMode::User, CpuMode::System].contains(&mode);
        if switch_mode {
            cpu.cpsr_mut().set_mode(CpuMode::User);
        }

        let add = self.add();
        let mut pre_index = self.pre_index();
        let mut base_address = address;
        if !add {
            pre_index = !pre_index;
            address = address.wrapping_sub(transfer_bytes);
            base_address = base_address.wrapping_sub(transfer_bytes);
        } else {
            base_address = base_address.wrapping_add(transfer_bytes)
        }

        let write_back = self.write_back();
        let mut memory_access = MemoryAccess::NonSequential;
        let mut action = CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential);
        match load {
            true => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        address = address.wrapping_add(4)
                    }

                    let value = cpu.load_32(address, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address = base_address.wrapping_add(4);
                            if !transfer_pc {
                                cpu.pipeline_flush();
                            }
                        }
                        cpu.set_register(rn, base_address);
                    }
                    cpu.set_register(register, value);

                    if !pre_index {
                        address = address.wrapping_add(4)
                    }

                    memory_access = MemoryAccess::Sequential;
                }

                cpu.idle_cycle();
                if transfer_pc {
                    if load_psr_force_user {
                        cpu.set_cpsr(cpu.spsr());
                    }

                    cpu.pipeline_flush();
                    action = CpuAction::PipelineFlush;
                }
            }
            false => {
                for (i, register) in (0..16).filter(|&r| (register_list >> r) & 1 == 1).enumerate() {
                    if pre_index {
                        address = address.wrapping_add(4)
                    }

                    let mut value = cpu.register(register);
                    if register == PC {
                        match write_back && rn == PC {
                            true => value = value.wrapping_sub(4),
                            false => value = value.wrapping_add(4),
                        }
                    }

                    cpu.store_32(address, value, memory_access as u8);
                    if write_back && i == 0 {
                        if rn == PC {
                            base_address = base_address.wrapping_add(4);
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, base_address);
                    }

                    if !pre_index {
                        address = address.wrapping_add(4)
                    }

                    memory_access = MemoryAccess::Sequential;
                }
            }
        }

        if switch_mode {
            cpu.cpsr_mut().set_mode(mode);
        }

        action
    }
}

impl Compile for BranchAndBranchWithLink {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        if self.link() {
            cpu.set_register(LR, cpu.pc() - 4)
        }

        let offset = self.offset().sign_extend(24) << 2;
        cpu.set_pc((cpu.pc() as i32).wrapping_add(offset) as u32);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}

impl Compile for BranchAndExchange {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let value = cpu.register(self.rn() as usize);
        cpu.cpsr_mut().set_state(CpuState::from_bits((value & 0x1) as u8));
        cpu.set_pc(value & !0x1);
        cpu.pipeline_flush();
        CpuAction::PipelineFlush
    }
}

impl Compile for DataProcessing {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
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
}

impl Compile for HalfwordAndSignedDataTransfer {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;

        let mut address = cpu.register(rn);
        let mut offset = match self.is_immediate() {
            true => self.immediate_hi() << 4 | self.immediate_lo(),
            false => cpu.register(self.rm() as usize),
        };

        if !self.add() {
            offset = (-(offset as i64)) as u32
        }

        let pre_index = self.pre_index();
        if pre_index {
            address = address.wrapping_add(offset)
        }

        let load = self.load();
        let write_back = self.write_back();
        let s = self.signed();
        let h = self.halfword();
        match load {
            true => match (s, h) {
                (false, false) => {}
                (false, true) => {
                    let value = cpu.load_rotated_16(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
                (true, false) => {
                    let value = cpu.load_signed_8(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
                (true, true) => {
                    let value = cpu.load_signed_16(address, MemoryAccess::NonSequential as u8);
                    if write_back || !pre_index {
                        if rn != rd && rn == PC {
                            cpu.pipeline_flush();
                        }
                        cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                    }
                    cpu.idle_cycle();
                    cpu.set_register(rd, value);
                }
            },
            false => {
                let mut value = cpu.register(rd);
                if rd == PC {
                    value += 4;
                }
                match (s, h) {
                    (false, false) => {}
                    (false, true) => {
                        cpu.store_16(address, value as u16, MemoryAccess::NonSequential as u8);
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                    (true, false) => {
                        cpu.idle_cycle();
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                    (true, true) => {
                        cpu.idle_cycle();
                        if write_back || !pre_index {
                            if rn == PC {
                                cpu.pipeline_flush();
                            }
                            cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                        }
                    }
                };
            }
        }

        match load && rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}

impl Compile for MultiplyLong {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd_lo = self.rd_lo() as usize;
        let rd_hi = self.rd_hi() as usize;
        let rm = self.rm() as usize;
        let rs = self.rs() as usize;

        let mut operand1 = cpu.register(rm);
        if rm == PC {
            operand1 += 4
        }
        let mut operand2 = cpu.register(rs);
        if rs == PC {
            operand2 += 4
        }

        let mut result = match self.unsigned() {
            true => (operand1 as i32 as i64).wrapping_mul(operand2 as i32 as i64) as u64,
            false => (operand1 as u64).wrapping_mul(operand2 as u64),
        };

        let multiplier_cycles = multiplier_array_cycles(operand2);
        for _ in 0..multiplier_cycles {
            cpu.idle_cycle();
        }

        if self.accumulate() {
            let mut accumulator_lo = cpu.register(rd_lo) as u64;
            if rd_lo == PC {
                accumulator_lo += 4
            }
            let mut accumulator_hi = cpu.register(rd_hi) as u64;
            if rd_hi == PC {
                accumulator_hi += 4
            }
            result = result.wrapping_add(accumulator_hi << 32 | accumulator_lo);
            cpu.idle_cycle();
        };

        let result_lo = (result & 0xFFFFFFFF) as u32;
        let result_hi = (result >> 32) as u32;
        if self.sets_flags() {
            cpu.cpsr_mut().set_negative(result_hi >> 31 != 0);
            cpu.cpsr_mut().set_zero(result == 0);
        }

        cpu.set_register(rd_lo, result_lo);
        cpu.set_register(rd_hi, result_hi);
        match rd_hi == PC || rd_lo == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}

impl Compile for Multiply {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rm = self.rm() as usize;
        let rs = self.rs() as usize;
        let rn = self.rn() as usize;

        let mut operand1 = cpu.register(rm);
        if rm == PC {
            operand1 += 4
        }
        let mut operand2 = cpu.register(rs);
        if rs == PC {
            operand2 += 4
        }

        let mut result = operand1.wrapping_mul(operand2);
        let multiplier_cycles = multiplier_array_cycles(operand2);
        for _ in 0..multiplier_cycles {
            cpu.idle_cycle();
        }

        if self.accumulate() {
            let mut accumulator = cpu.register(rn);
            if rn == PC {
                accumulator += 4
            }
            result = result.wrapping_add(accumulator);
            cpu.idle_cycle();
        };

        if self.sets_flags() {
            cpu.cpsr_mut().set_negative(result >> 31 != 0);
            cpu.cpsr_mut().set_zero(result == 0);
        }

        cpu.set_register(rd, result);
        match rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}

impl Compile for PsrTransfer {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let is_spsr = self.is_spsr();
        match self.is_mrs() {
            true => {
                let rd = self.rd() as usize;
                let psr = match is_spsr {
                    false => *cpu.cpsr(),
                    true => cpu.spsr(),
                };
                cpu.set_register(rd, psr.into_bits());
            }
            false => {
                let mask = self.psr_mask();

                let mut operand = match self.is_immediate() {
                    false => cpu.register(self.rm() as usize),
                    true => {
                        let mut carry = cpu.cpsr().carry();
                        let rotate = 2 * self.rotate();
                        let immediate = self.immediate();
                        ror(immediate, rotate, &mut carry, false)
                    }
                };

                match is_spsr {
                    false => {
                        if cpu.cpsr().mode() == CpuMode::User {
                            let mask = mask & 0xFF000000;
                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        } else {
                            if mask & 0xFF != 0 {
                                operand |= 0x10;
                            }

                            let bits = (cpu.cpsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_cpsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        }
                    }
                    true => {
                        if cpu.cpsr().mode() != CpuMode::User && cpu.cpsr().mode() != CpuMode::System {
                            let bits = (cpu.spsr().into_bits() & !mask) | (operand & mask);
                            cpu.set_spsr(ProgramStatusRegister::from_bits_with_defaults(bits));
                        }
                    }
                }
            }
        }
        CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential)
    }
}

impl Compile for SingleDataSwap {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;
        let rm = self.rm() as usize;

        let address = cpu.register(rn);
        let mut source = cpu.register(rm);
        if rm == PC {
            source += 4;
        }

        let value: u32;
        match self.byte() {
            true => {
                value = cpu.load_8(address, MemoryAccess::NonSequential as u8);
                cpu.store_8(address, source as u8, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
            false => {
                value = cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8);
                cpu.store_32(address, source, MemoryAccess::NonSequential | MemoryAccess::Lock);
            }
        };

        cpu.idle_cycle();
        cpu.set_register(rd, value);
        match rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}

impl Compile for SingleDataTransfer {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        let rd = self.rd() as usize;
        let rn = self.rn() as usize;

        let mut address = cpu.register(rn);
        let mut offset = match self.is_immediate() {
            true => self.immediate(),
            false => {
                let rm_value = cpu.register(self.rm() as usize);
                let shift_amount = self.shift_amount();
                let mut carry = cpu.cpsr().carry();
                match self.shift_type() {
                    ShiftType::LSL => lsl(rm_value, shift_amount, &mut carry),
                    ShiftType::LSR => lsr(rm_value, shift_amount, &mut carry, true),
                    ShiftType::ASR => asr(rm_value, shift_amount, &mut carry, true),
                    ShiftType::ROR => ror(rm_value, shift_amount, &mut carry, true),
                }
            }
        };

        if !self.add() {
            offset = (-(offset as i64)) as u32
        }

        let pre_index = self.pre_index();
        if pre_index {
            address = address.wrapping_add(offset)
        }

        let load = self.load();
        let byte = self.byte();
        let write_back = self.write_back();
        match load {
            true => {
                let value = match byte {
                    true => cpu.load_8(address, MemoryAccess::NonSequential as u8),
                    false => cpu.load_rotated_32(address, MemoryAccess::NonSequential as u8),
                };
                if write_back || !pre_index {
                    if rn != rd && rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
                cpu.idle_cycle();
                cpu.set_register(rd, value);
            }
            false => {
                let mut value = cpu.register(rd);
                if rd == PC {
                    value += 4;
                }
                match byte {
                    true => cpu.store_8(address, value as u8, MemoryAccess::NonSequential as u8),
                    false => cpu.store_32(address, value, MemoryAccess::NonSequential as u8),
                };
                if write_back || !pre_index {
                    if rn == PC {
                        cpu.pipeline_flush();
                    }
                    cpu.set_register(rn, cpu.register(rn).wrapping_add(offset));
                }
            }
        }

        match load && rd == PC {
            true => {
                cpu.pipeline_flush();
                CpuAction::PipelineFlush
            }
            false => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::NonSequential),
        }
    }
}

impl Compile for SoftwareInterrupt {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        match !cpu.bios_loaded() && cpu.bios_call(self.comment() >> 16) {
            true => CpuAction::Advance(MemoryAccess::Instruction | MemoryAccess::Sequential),
            false => {
                cpu.exception(Exception::SoftwareInterrupt);
                CpuAction::PipelineFlush
            }
        }
    }
}

impl Compile for Undefined {
    fn compile<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction {
        cpu.exception(Exception::Undefined);
        CpuAction::PipelineFlush
    }
}
