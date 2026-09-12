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

//    _block_1:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x2, x0
//  8  mov x0, x19
//  9  mov w1, #0
// 10  mov w3, #255
// 11  bl _guest_add
// 12  mov x2, x0
// 13  mov x0, x19
// 14  mov w1, #0
// 15  bl _guest_set_register
// 16  mov w0, #3
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret

//  2. ANDS R0, R1, #0xFF — rotate=0, carry unaffected (fetch + pass through)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let carry = guest_cpsr_carry(cpu);
        let operand2 = 0xFF;
        let result = guest_and(cpu, 1, operand1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//    _block_2:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  bl _guest_cpsr_carry
// 10  mov x4, x0
// 11  mov x0, x19
// 12  mov w1, #1
// 13  mov x2, x20
// 14  mov w3, #255
// 15  bl _guest_and
// 16  mov x2, x0
// 17  mov x0, x19
// 18  mov w1, #0
// 19  bl _guest_set_register
// 20  mov w0, #3
// 21  ldp x29, x30, [sp, #16]
// 22  ldp x20, x19, [sp], #32
// 23  ret

//  3. MOVS R0, #0xF0000000 — rotate=4 (0x0F ror 4), carry fully constant-folded
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_3(cpu: *mut u8) -> u32 {
    unsafe {
        let operand2 = 0x0Fu32.rotate_right(4);
        let carry = operand2 >> 31;
        let result = guest_mov(cpu, 1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//    _block_3:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  mov w2, #-268435456
//  7  mov w3, #1
//  8  bl _guest_mov
//  9  mov x2, x0
// 10  mov x0, x19
// 11  mov w1, #0
// 12  bl _guest_set_register
// 13  mov w0, #3
// 14  ldp x29, x30, [sp, #16]
// 15  ldp x20, x19, [sp], #32
// 16  ret

// operand2 — shift by immediate
//  4. ANDS R0, R1, R2, LSL #3 — normal LSL, carry present (logical+S)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_4(cpu: *mut u8) -> u32 {
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

//    _block_4:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  ubfx w4, w0, #29, #1
// 12  lsl w3, w0, #3
// 13  mov x0, x19
// 14  mov w1, #1
// 15  mov x2, x20
// 16  bl _guest_and
// 17  mov x2, x0
// 18  mov x0, x19
// 19  mov w1, #0
// 20  bl _guest_set_register
// 21  mov w0, #3
// 22  ldp x29, x30, [sp, #16]
// 23  ldp x20, x19, [sp], #32
// 24  ret

//  5. ADD R0, R1, R2, LSR #5 — normal LSR, no carry (arithmetic)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_5(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let operand2 = rm >> 5;
        let result = guest_add(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//    _block_5:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  lsr w3, w0, #5
// 12  mov x0, x19
// 13  mov w1, #0
// 14  mov x2, x20
// 15  bl _guest_add
// 16  mov x2, x0
// 17  mov x0, x19
// 18  mov w1, #0
// 19  bl _guest_set_register
// 20  mov w0, #3
// 21  ldp x29, x30, [sp, #16]
// 22  ldp x20, x19, [sp], #32
// 23  ret

//  6. ANDS R0, R1, R2, LSR #32 — LSR quirk (raw 0), carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_6(cpu: *mut u8) -> u32 {
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

//    _block_6:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  and w4, w0, #0x80000000
// 12  mov x0, x19
// 13  mov w1, #1
// 14  mov x2, x20
// 15  mov w3, #0
// 16  bl _guest_and
// 17  mov x2, x0
// 18  mov x0, x19
// 19  mov w1, #0
// 20  bl _guest_set_register
// 21  mov w0, #3
// 22  ldp x29, x30, [sp, #16]
// 23  ldp x20, x19, [sp], #32
// 24  ret

//  7. SUB R0, R1, R2, ASR #7 — normal ASR, no carry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_7(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1); // rn = 1
        let rm = guest_register(cpu, 2); // rm = 2
        let operand2 = ((rm as i32) >> 7) as u32;
        let result = guest_sub(cpu, 0, operand1, operand2);
        guest_set_register(cpu, 0, result);
    }
    0b11 // Sequential Instruction Access
}

//    _block_7:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  asr w3, w0, #7
// 12  mov x0, x19
// 13  mov w1, #0
// 14  mov x2, x20
// 15  bl _guest_sub
// 16  mov x2, x0
// 17  mov x0, x19
// 18  mov w1, #0
// 19  bl _guest_set_register
// 20  mov w0, #3
// 21  ldp x29, x30, [sp, #16]
// 22  ldp x20, x19, [sp], #32
// 23  ret

//  8. ORRS R0, R1, R2, ASR #32 — ASR quirk, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_8(cpu: *mut u8) -> u32 {
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

//    _block_8:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  asr w3, w0, #31
// 12  and w4, w0, #0x80000000
// 13  mov x0, x19
// 14  mov w1, #1
// 15  mov x2, x20
// 16  bl _guest_orr
// 17  mov x2, x0
// 18  mov x0, x19
// 19  mov w1, #0
// 20  bl _guest_set_register
// 21  mov w0, #3
// 22  ldp x29, x30, [sp, #16]
// 23  ldp x20, x19, [sp], #32
// 24  ret

//  9. EORS R0, R1, R2, ROR #9 — normal ROR, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_9(cpu: *mut u8) -> u32 {
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

//    _block_9:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #2
// 10  bl _guest_register
// 11  ror w3, w0, #9
// 12  lsr w4, w3, #31
// 13  mov x0, x19
// 14  mov w1, #1
// 15  mov x2, x20
// 16  bl _guest_eor
// 17  mov x2, x0
// 18  mov x0, x19
// 19  mov w1, #0
// 20  bl _guest_set_register
// 21  mov w0, #3
// 22  ldp x29, x30, [sp, #16]
// 23  ldp x20, x19, [sp], #32
// 24  ret

// 10. MOVS R0, R1, ROR #0 — RRX, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_10(cpu: *mut u8) -> u32 {
    unsafe {
        let rm = guest_register(cpu, 1); // rm = 1
        let carry = rm & 0b1;
        let operand2 = (rm >> 1) | (guest_cpsr_carry(cpu)) << 31;
        let result = guest_mov(cpu, 1, operand2, carry);
        guest_set_register(cpu, 0, result); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//    _block_10:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #1
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  bl _guest_cpsr_carry
// 10  extr w2, w0, w20, #1
// 11  and w3, w20, #0x1
// 12  mov x0, x19
// 13  mov w1, #1
// 14  bl _guest_mov
// 15  mov x2, x0
// 16  mov x0, x19
// 17  mov w1, #0
// 18  bl _guest_set_register
// 19  mov w0, #3
// 20  ldp x29, x30, [sp, #16]
// 21  ldp x20, x19, [sp], #32
// 22  ret

// operand2 — shift by register
// 11. ADD R0, R1, R2, LSL R3 — baseline
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_11(cpu: *mut u8) -> u32 {
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

//    _block_11:
//  1  stp x22, x21, [sp, #-48]!
//  2  stp x20, x19, [sp, #16]
//  3  stp x29, x30, [sp, #32]
//  4  add x29, sp, #32
//  5  mov x19, x0
//  6  mov w1, #1
//  7  bl _guest_register
//  8  mov x20, x0
//  9  mov x0, x19
// 10  mov w1, #2
// 11  bl _guest_register
// 12  mov x21, x0
// 13  mov x0, x19
// 14  bl _guest_idle_cycle
// 15  mov x0, x19
// 16  mov w1, #3
// 17  bl _guest_register
// 18  and w1, w0, #0xff
// 19  mov x0, x21
// 20  mov w2, #0
// 21  bl _guest_lsl_by_register
// 22  mov x3, x0
// 23  mov x0, x19
// 24  mov w1, #0
// 25  mov x2, x20
// 26  bl _guest_add
// 27  mov x2, x0
// 28  mov x0, x19
// 29  mov w1, #0
// 30  bl _guest_set_register
// 31  mov w0, #2
// 32  ldp x29, x30, [sp, #32]
// 33  ldp x20, x19, [sp, #16]
// 34  ldp x22, x21, [sp], #48
// 35  ret

// 12. ANDS R0, R1, R2, LSL R3, carry present
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_12(cpu: *mut u8) -> u32 {
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

//    _block_12:
//  1  stp x22, x21, [sp, #-48]!
//  2  stp x20, x19, [sp, #16]
//  3  stp x29, x30, [sp, #32]
//  4  add x29, sp, #32
//  5  mov x19, x0
//  6  mov w1, #1
//  7  bl _guest_register
//  8  mov x20, x0
//  9  mov x0, x19
// 10  mov w1, #2
// 11  bl _guest_register
// 12  mov x21, x0
// 13  mov x0, x19
// 14  bl _guest_idle_cycle
// 15  mov x0, x19
// 16  mov w1, #3
// 17  bl _guest_register
// 18  mov x22, x0
// 19  mov x0, x19
// 20  bl _guest_cpsr_carry
// 21  mov x2, x0
// 22  and w1, w22, #0xff
// 23  mov x0, x21
// 24  bl _guest_lsl_by_register
// 25  mov x3, x0
// 26  lsr x4, x0, #32
// 27  mov x0, x19
// 28  mov w1, #1
// 29  mov x2, x20
// 30  bl _guest_and
// 31  mov x2, x0
// 32  mov x0, x19
// 33  mov w1, #0
// 34  bl _guest_set_register
// 35  mov w0, #2
// 36  ldp x29, x30, [sp, #32]
// 37  ldp x20, x19, [sp, #16]
// 38  ldp x22, x21, [sp], #48
// 39  ret

// 13. ADD R0, PC, R2, LSL R3 — rn is PC, +4 adjust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_13(cpu: *mut u8) -> u32 {
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

//    _block_13:
//  1  stp x22, x21, [sp, #-48]!
//  2  stp x20, x19, [sp, #16]
//  3  stp x29, x30, [sp, #32]
//  4  add x29, sp, #32
//  5  mov x19, x0
//  6  mov w1, #15
//  7  bl _guest_register
//  8  mov x20, x0
//  9  mov x0, x19
// 10  mov w1, #2
// 11  bl _guest_register
// 12  mov x21, x0
// 13  mov x0, x19
// 14  bl _guest_idle_cycle
// 15  mov x0, x19
// 16  mov w1, #3
// 17  bl _guest_register
// 18  and w1, w0, #0xff
// 19  mov x0, x21
// 20  mov w2, #0
// 21  bl _guest_lsl_by_register
// 22  mov x3, x0
// 23  add w2, w20, #4
// 24  mov x0, x19
// 25  mov w1, #0
// 26  bl _guest_add
// 27  mov x2, x0
// 28  mov x0, x19
// 29  mov w1, #0
// 30  bl _guest_set_register
// 31  mov w0, #2
// 32  ldp x29, x30, [sp, #32]
// 33  ldp x20, x19, [sp, #16]
// 34  ldp x22, x21, [sp], #48
// 35  ret

// 14. ADD R0, R1, PC, LSL R3 — rm is PC, +4 adjust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_14(cpu: *mut u8) -> u32 {
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

//    _block_14:
//  1  stp x22, x21, [sp, #-48]!
//  2  stp x20, x19, [sp, #16]
//  3  stp x29, x30, [sp, #32]
//  4  add x29, sp, #32
//  5  mov x19, x0
//  6  mov w1, #1
//  7  bl _guest_register
//  8  mov x20, x0
//  9  mov x0, x19
// 10  mov w1, #15
// 11  bl _guest_register
// 12  mov x21, x0
// 13  mov x0, x19
// 14  bl _guest_idle_cycle
// 15  mov x0, x19
// 16  mov w1, #3
// 17  bl _guest_register
// 18  mov x8, x0
// 19  add w0, w21, #4
// 20  and w1, w8, #0xff
// 21  mov w2, #0
// 22  bl _guest_lsl_by_register
// 23  mov x3, x0
// 24  mov x0, x19
// 25  mov w1, #0
// 26  mov x2, x20
// 27  bl _guest_add
// 28  mov x2, x0
// 29  mov x0, x19
// 30  mov w1, #0
// 31  bl _guest_set_register
// 32  mov w0, #2
// 33  ldp x29, x30, [sp, #32]
// 34  ldp x20, x19, [sp, #16]
// 35  ldp x22, x21, [sp], #48
// 36  ret

// tail
// 15. MOVS PC, LR — rd is PC, S set: SPSR restore + pipeline flush
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_15(cpu: *mut u8) -> u32 {
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

//    _block_15:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #14
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  bl _guest_cpsr_carry
// 10  mov x3, x0
// 11  mov x0, x19
// 12  mov w1, #1
// 13  mov x2, x20
// 14  bl _guest_mov
// 15  mov x20, x0
// 16  mov x0, x19
// 17  bl _guest_set_cpsr_to_spsr
// 18  mov x0, x19
// 19  mov w1, #15
// 20  mov x2, x20
// 21  bl _guest_set_register
// 22  mov x0, x19
// 23  bl _guest_pipeline_flush
// 24  mov w0, #-1
// 25  ldp x29, x30, [sp, #16]
// 26  ldp x20, x19, [sp], #32
// 27  ret

// 16. CMP R0, R1 — test op, set_register never emitted
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_16(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 0); // rn = 0
        let operand2 = guest_register(cpu, 1); // rm = 1
        guest_cmp(cpu, 1, operand1, operand2);
    }
    0b11 // Sequential Instruction Access
}

//    _block_16:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #0
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov w1, #1
// 10  bl _guest_register
// 11  mov x3, x0
// 12  mov x0, x19
// 13  mov w1, #1
// 14  mov x2, x20
// 15  bl _guest_cmp
// 16  mov w0, #3
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret
