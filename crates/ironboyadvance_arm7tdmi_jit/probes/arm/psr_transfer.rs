#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_cpsr_carry(cpu: *mut u8) -> u32;
    fn guest_cpsr_mode(cpu: *mut u8) -> u32;
    fn guest_psr(cpu: *mut u8) -> u32; // either cpsr or psr for simplicity
    fn guest_set_cpsr(cpu: *mut u8, value: u32);
    fn guest_set_spsr(cpu: *mut u8, value: u32);
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
}

// 1. MRS R0, CPSR — plain read, no mask logic
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_1(cpu: *mut u8) -> u32 {
    unsafe {
        let psr = guest_psr(cpu);
        guest_set_register(cpu, 0, psr); // rd = 0
    }
    0b11 // Sequential Instruction Access
}

//    _block_1:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  bl _guest_psr
//  6  mov x2, x0
//  7  mov x0, x19
//  8  mov w1, #0
//  9  bl _guest_set_register
// 10  mov w0, #3
// 11  ldp x29, x30, [sp, #16]
// 12  ldp x20, x19, [sp], #32
// 13  ret

// 2. MSR CPSR_flg, R1 — register operand, flags-only mask (0xFF000000):
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let mut operand = guest_register(cpu, 1); // rm = 1
        if guest_cpsr_mode(cpu) == 16 {
            let mask = 0xFF000000 & 0xFF000000;
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_cpsr(cpu, value)
        } else {
            let mask = 0xFF000000;
            if mask & 0xFF != 0 {
                operand |= 0x10;
            }
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_cpsr(cpu, value)
        }
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
//  9  bl _guest_cpsr_mode
// 10  mov x0, x19
// 11  bl _guest_psr
// 12  bfxil w20, w0, #0, #24
// 13  mov x0, x19
// 14  mov x1, x20
// 15  bl _guest_set_cpsr
// 16  mov w0, #3
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret

// 3. MSR CPSR_c, #0xD3 — immediate operand, control-only mask (0x000000FF)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_3(cpu: *mut u8) -> u32 {
    unsafe {
        let mut operand = 0xD3;
        if guest_cpsr_mode(cpu) == 16 {
            let mask = 0x000000FF & 0xFF000000;
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_cpsr(cpu, value)
        } else {
            let mask = 0x000000FF;
            if mask & 0xFF != 0 {
                operand |= 0x10;
            }
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_cpsr(cpu, value)
        }
    }
    0b11 // Sequential Instruction Access
}

//    _block_3:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  bl _guest_cpsr_mode
//  6  mov x20, x0
//  7  mov x0, x19
//  8  bl _guest_psr
//  9  mov w8, #211
// 10  mov x9, x0
// 11  bfxil w9, w8, #0, #8
// 12  cmp w20, #16
// 13  csel w1, w0, w9, eq
// 14  mov x0, x19
// 15  bl _guest_set_cpsr
// 16  mov w0, #3
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret
//    LBB3_3:
//  1  mov w0, #3
//  2  ldp x29, x30, [sp, #16]
//  3  ldp x20, x19, [sp], #32
//  4  ret
//    LBB4_3:
//  1  mov w0, #3
//  2  ldp x29, x30, [sp, #16]
//  3  ldp x20, x19, [sp], #32
//  4  ret

// 4. MSR SPSR_flg, R1 — register operand, flags-only mask, SPSR path never forces bit 4
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_4(cpu: *mut u8) -> u32 {
    unsafe {
        let operand = guest_register(cpu, 1); // rm = 1
        if guest_cpsr_mode(cpu) != 16 && guest_cpsr_mode(cpu) != 31 {
            let mask = 0xFF000000;
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_spsr(cpu, value)
        }
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
//  9  bl _guest_cpsr_mode
// 10  cmp w0, #16
// 11  b.eq LBB3_3
// 12  mov x0, x19
// 13  bl _guest_cpsr_mode
// 14  cmp w0, #31
// 15  b.eq LBB3_3
// 16  mov x0, x19
// 17  bl _guest_psr
// 18  bfxil w20, w0, #0, #24
// 19  mov x0, x19
// 20  mov x1, x20
// 21  bl _guest_set_spsr

// 5. MSR SPSR_c, #0xD3 — immediate operand, control-only mask, no bit 4 force
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_5(cpu: *mut u8) -> u32 {
    unsafe {
        let operand = 0xD3;
        if guest_cpsr_mode(cpu) != 16 && guest_cpsr_mode(cpu) != 31 {
            let mask = 0x000000FF;
            let value = (guest_psr(cpu) & !mask) | (operand & mask);
            guest_set_spsr(cpu, value)
        }
    }
    0b11 // Sequential Instruction Access
}

//    _block_5:
//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0                                                                                 5  bl _guest_cpsr_mode
//  6  cmp w0, #16
//  7  b.eq LBB4_3
//  8  mov x0, x19
//  9  bl _guest_cpsr_mode
// 10  cmp w0, #31
// 11  b.eq LBB4_3
// 12  mov x0, x19
// 13  bl _guest_psr
// 14  mov x1, x0
// 15  mov w8, #211
// 16  bfxil w1, w8, #0, #8
// 17  mov x0, x19
// 18  bl _guest_set_spsr
