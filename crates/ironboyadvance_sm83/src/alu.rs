use crate::cpu::Sm83;
use crate::memory::MemoryInterface;

pub(crate) fn add<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let accumulator = cpu.registers().a();
    let result = accumulator.wrapping_add(operand);
    cpu.registers_mut().set_a(result);

    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut()
        .f_mut()
        .set_half_carry((accumulator & 0x0F) + (operand & 0x0F) > 0x0F);
    cpu.registers_mut()
        .f_mut()
        .set_carry(accumulator as u16 + operand as u16 > 0xFF);
}

pub(crate) fn adc<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let accumulator = cpu.registers().a();
    let carry = if cpu.registers().f().carry() { 1 } else { 0 };
    let result = accumulator.wrapping_add(operand).wrapping_add(carry);
    cpu.registers_mut().set_a(result);

    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut()
        .f_mut()
        .set_half_carry((accumulator & 0x0F) + (operand & 0x0F) + carry > 0x0F);
    cpu.registers_mut()
        .f_mut()
        .set_carry(accumulator as u16 + operand as u16 + carry as u16 > 0xFF);
}

pub(crate) fn sub<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let accumulator = cpu.registers().a();
    let result = accumulator.wrapping_sub(operand);
    cpu.registers_mut().set_a(result);

    set_subtraction_flags(cpu, accumulator, operand, result);
}

pub(crate) fn sbc<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let accumulator = cpu.registers().a();
    let carry = if cpu.registers().f().carry() { 1 } else { 0 };
    let result = accumulator.wrapping_sub(operand).wrapping_sub(carry);
    cpu.registers_mut().set_a(result);

    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(true);
    cpu.registers_mut()
        .f_mut()
        .set_half_carry((accumulator & 0x0F) < (operand & 0x0F) + carry);
    cpu.registers_mut()
        .f_mut()
        .set_carry((accumulator as u16) < (operand as u16) + carry as u16);
}

pub(crate) fn and<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let result = cpu.registers().a() & operand;
    cpu.registers_mut().set_a(result);

    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut().f_mut().set_half_carry(true);
    cpu.registers_mut().f_mut().set_carry(false);
}

pub(crate) fn xor<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let result = cpu.registers().a() ^ operand;
    cpu.registers_mut().set_a(result);

    set_logical_flags(cpu, result);
}

pub(crate) fn or<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let result = cpu.registers().a() | operand;
    cpu.registers_mut().set_a(result);

    set_logical_flags(cpu, result);
}

pub(crate) fn cp<I: MemoryInterface>(cpu: &mut Sm83<I>, operand: u8) {
    let accumulator = cpu.registers().a();
    let result = accumulator.wrapping_sub(operand);

    set_subtraction_flags(cpu, accumulator, operand, result);
}

pub(crate) fn add_offset_to_stack_pointer<I: MemoryInterface>(cpu: &mut Sm83<I>, offset: u16) -> u16 {
    let sp = cpu.registers().sp();

    cpu.registers_mut().f_mut().set_zero(false);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut()
        .f_mut()
        .set_half_carry((sp & 0x000F) + (offset & 0x000F) > 0x000F);
    cpu.registers_mut()
        .f_mut()
        .set_carry((sp & 0x00FF) + (offset & 0x00FF) > 0x00FF);

    sp.wrapping_add(offset)
}

fn set_subtraction_flags<I: MemoryInterface>(cpu: &mut Sm83<I>, accumulator: u8, operand: u8, result: u8) {
    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(true);
    cpu.registers_mut()
        .f_mut()
        .set_half_carry((accumulator & 0x0F) < (operand & 0x0F));
    cpu.registers_mut().f_mut().set_carry((accumulator as u16) < (operand as u16));
}

fn set_logical_flags<I: MemoryInterface>(cpu: &mut Sm83<I>, result: u8) {
    cpu.registers_mut().f_mut().set_zero(result == 0);
    cpu.registers_mut().f_mut().set_subtraction(false);
    cpu.registers_mut().f_mut().set_half_carry(false);
    cpu.registers_mut().f_mut().set_carry(false);
}
