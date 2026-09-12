#![crate_type = "lib"]

#[repr(C)]
pub struct ShiftResult {
    pub value: u32,
    pub carry: u32,
}

unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
    fn guest_cpsr_carry(cpu: *mut u8) -> u32;
    fn guest_set_cpsr_to_spsr(cpu: *mut u8);
    fn guest_idle_cycle(cpu: *mut u8);
    fn guest_pipeline_flush(cpu: *mut u8);
    fn guest_and(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32, carry: u32) -> u32;
    fn guest_eor(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32, carry: u32) -> u32;
    fn guest_sub(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32) -> u32;
    fn guest_add(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32) -> u32;
    fn guest_cmp(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32) -> u32;
    fn guest_orr(cpu: *mut u8, set_flags: u32, operand1: u32, operand2: u32, carry: u32) -> u32;
    fn guest_mov(cpu: *mut u8, set_flags: u32, operand2: u32, carry: u32) -> u32;
    fn guest_lsl_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult;
}

// operand2 — immediate
//  1. ADD R0, R1, #0xFF — rotate=0, trivial constant fold
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_1(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let operand2 = 0xFF;
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

// operand2 — shift by immediate
//  2. ANDS R0, R1, R2, LSL #3 — normal LSL, carry present (logical+S)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let carry = (rm << 2) >> 31;
        let operand2 = rm << 3;
        let result = guest_and(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//  3. ADD R0, R1, R2, LSR #5 — normal LSR, no carry (arithmetic)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_3(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let operand2 = rm >> 5;
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//  4. ANDS R0, R1, R2, LSR #32 — LSR quirk (raw 0), carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_4(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let carry = rm & (1 << 31);
        let operand2 = 0;
        let result = guest_and(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result);
    }
    0b11 // Sequential Instruction Access
}

//  5. SUB R0, R1, R2, ASR #7 — normal ASR, no carry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_5(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let operand2 = ((rm as i32) >> 7) as u32;
        let result = guest_sub(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result);
    }
    0b11 // Sequential Instruction Access
}

//  6. ORRS R0, R1, R2, ASR #32 — ASR quirk, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_6(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let carry = rm & (1 << 31);
        let operand2 = match carry != 0 {
            true => u32::MAX,
            false => 0,
        };
        let result = guest_orr(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//  7. EORS R0, R1, R2, ROR #9 — normal ROR, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_7(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let operand2 = rm.rotate_right(9);
        let carry = operand2 >> 31;
        let result = guest_eor(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//  8. MOVS R0, R1, ROR #0 — RRX, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_8(cpu: *mut u8) -> u32 {
    unsafe {
        let rm = guest_register(cpu, 1); // rm = 1
        let carry = rm & 0b1;
        let operand2 = (rm >> 1) | (guest_cpsr_carry(cpu)) << 31;
        let result = guest_mov(cpu, 1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

// operand2 — shift by register
//  9. ADD R0, R1, R2, LSL R3 — baseline
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_9(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2

        guest_idle_cycle(cpu);
        let rs = guest_register(cpu, 3) & 0xFF; // rs = 3

        let ShiftResult {
            value: operand2,
            carry: _,
        } = guest_lsl_by_register(rm, rs, 0);
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b10 // Non-Sequential Instruction Access
}

// 10. ANDS R0, R1, R2, LSL R3, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_10(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2

        guest_idle_cycle(cpu);
        let rs = guest_register(cpu, 3) & 0xFF; // rs = 3
        let cpsr_carry = guest_cpsr_carry(cpu);

        let ShiftResult { value: operand2, carry } = guest_lsl_by_register(rm, rs, cpsr_carry);
        let result = guest_and(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b10 // Non-Sequential Instruction Access
}

//  11. ADD R0, PC, R2, LSL R3 — rn is PC, +4 adjust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_11(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 15) + 4; // rn = PC
        let rm = guest_register(cpu, 2); // rm = 2

        guest_idle_cycle(cpu);
        let rs = guest_register(cpu, 3) & 0xFF; // rs = 3

        let ShiftResult {
            value: operand2,
            carry: _,
        } = guest_lsl_by_register(rm, rs, 0);
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b10 // Non-Sequential Instruction Access
}

//  12. ADD R0, R1, PC, LSL R3 — rm is PC, +4 adjust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_12(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 15) + 4; // rm = PC

        guest_idle_cycle(cpu);
        let rs = guest_register(cpu, 3) & 0xFF; // rs = 3

        let ShiftResult {
            value: operand2,
            carry: _,
        } = guest_lsl_by_register(rm, rs, 0);
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b10 // Non-Sequential Instruction Access
}

// tail
//  13. MOVS PC, LR — rd is PC, S set: SPSR restore + pipeline flush
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_13(cpu: *mut u8) -> u32 {
    unsafe {
        let rm = guest_register(cpu, 14); // rm = LR
        let carry = guest_cpsr_carry(cpu);
        let operand2 = rm;
        let result = guest_mov(cpu, 1, operand2, carry);

        guest_set_cpsr_to_spsr(cpu);
        guest_set_register(cpu, 15, result); // rd = 15
        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF // Pipeline flush
}

//  14. CMP R0, R1 — test op, set_register never emitted
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_14(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 0); // rn = 0
        let operand2 = guest_register(cpu, 1); // rm = 1
        guest_cmp(cpu, 1, operand1, operand2);
    }
    0b11 // Sequential Instruction Access
}
