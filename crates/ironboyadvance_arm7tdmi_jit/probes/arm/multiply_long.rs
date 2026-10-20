#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
    fn guest_multiplier_array_cycles(cpu: *mut u8, operand: u32);
    fn guest_idle_cycle(cpu: *mut u8);
    fn guest_set_cpsr_negative(cpu: *mut u8, value: u32);
    fn guest_set_cpsr_zero(cpu: *mut u8, value: u32);
    fn guest_pipeline_flush(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_1(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1) + 4; //rm = 1
        let operand2 = guest_register(cpu, 2) + 4; //rs = 2

        // signed
        let result = (operand1 as i32 as i64).wrapping_mul(operand2 as i32 as i64) as u64;

        guest_multiplier_array_cycles(cpu, operand2);

        let result_lo = (result & 0xFFFFFFFF) as u32;
        let result_hi = (result >> 32) as u32;

        guest_set_cpsr_negative(cpu, result_hi);
        guest_set_cpsr_zero(cpu, result_lo | result_hi);

        guest_set_register(cpu, 3, result_lo); //rd_lo = 3
        guest_set_register(cpu, 4, result_hi); //rd_hi = 4

        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//      _block_1:
//    1  stp x22, x21, [sp, #-48]!
//    2  stp x20, x19, [sp, #16]
//    3  stp x29, x30, [sp, #32]
//    4  add x29, sp, #32
//    5  mov x19, x0
//    6  mov w1, #1
//    7  bl _guest_register
//    8  add w20, w0, #4
//    9  mov x0, x19
//   10  mov w1, #2
//   11  bl _guest_register
//   12  add w1, w0, #4
//   13  smull x20, w1, w20
//   14  mov x0, x19
//   15  bl _guest_multiplier_array_cycles
//   16  lsr x21, x20, #32
//   17  mov x0, x19
//   18  mov x1, x21
//   19  bl _guest_set_cpsr_negative
//   20  orr w1, w21, w20
//   21  mov x0, x19
//   22  bl _guest_set_cpsr_zero
//   23  mov x0, x19
//   24  mov w1, #3
//   25  mov x2, x20
//   26  bl _guest_set_register
//   27  mov x0, x19
//   28  mov w1, #4
//   29  mov x2, x21
//   30  bl _guest_set_register
//   31  mov x0, x19
//   32  bl _guest_pipeline_flush
//   33  mov w0, #-1
//   34  ldp x29, x30, [sp, #32]
//   35  ldp x20, x19, [sp, #16]
//   36  ldp x22, x21, [sp], #48
//   37  ret

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1) + 4; //rm = 1
        let operand2 = guest_register(cpu, 2) + 4; //rs = 2

        // unsigned
        let result = (operand1 as u64).wrapping_mul(operand2 as u64);

        guest_multiplier_array_cycles(cpu, operand2);

        let result_lo = (result & 0xFFFFFFFF) as u32;
        let result_hi = (result >> 32) as u32;

        guest_set_cpsr_negative(cpu, result_hi);
        guest_set_cpsr_zero(cpu, result_lo | result_hi);

        guest_set_register(cpu, 3, result_lo); //rd_lo = 3
        guest_set_register(cpu, 4, result_hi); //rd_hi = 4

        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//      _block_2:
//    1  stp x22, x21, [sp, #-48]!
//    2  stp x20, x19, [sp, #16]
//    3  stp x29, x30, [sp, #32]
//    4  add x29, sp, #32
//    5  mov x19, x0
//    6  mov w1, #1
//    7  bl _guest_register
//    8  add w20, w0, #4
//    9  mov x0, x19
//   10  mov w1, #2
//   11  bl _guest_register
//   12  add w1, w0, #4
//   13  umull x20, w1, w20
//   14  mov x0, x19
//   15  bl _guest_multiplier_array_cycles
//   16  lsr x21, x20, #32
//   17  mov x0, x19
//   18  mov x1, x21
//   19  bl _guest_set_cpsr_negative
//   20  orr w1, w21, w20
//   21  mov x0, x19
//   22  bl _guest_set_cpsr_zero
//   23  mov x0, x19
//   24  mov w1, #3
//   25  mov x2, x20
//   26  bl _guest_set_register
//   27  mov x0, x19
//   28  mov w1, #4
//   29  mov x2, x21
//   30  bl _guest_set_register
//   31  mov x0, x19
//   32  bl _guest_pipeline_flush
//   33  mov w0, #-1
//   34  ldp x29, x30, [sp, #32]
//   35  ldp x20, x19, [sp, #16]
//   36  ldp x22, x21, [sp], #48
//   37  ret

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_3(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1) + 4; //rm = 1
        let operand2 = guest_register(cpu, 2) + 4; //rs = 2

        // unsigned
        let product = (operand1 as u64).wrapping_mul(operand2 as u64);

        guest_multiplier_array_cycles(cpu, operand2);

        //accumulate
        let accumulator_lo = guest_register(cpu, 3) as u64 + 4; //rd_lo = 3
        let accumulator_hi = guest_register(cpu, 4) as u64 + 4; //rd_hi = 4
        let result = product.wrapping_add(accumulator_hi << 32 | accumulator_lo);
        guest_idle_cycle(cpu);

        let result_lo = (result & 0xFFFFFFFF) as u32;
        let result_hi = (result >> 32) as u32;

        guest_set_cpsr_negative(cpu, result_hi);
        guest_set_cpsr_zero(cpu, result_lo | result_hi);

        guest_set_register(cpu, 3, result_lo); //rd_lo = 3
        guest_set_register(cpu, 4, result_hi); //rd_hi = 4

        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//    1  stp x22, x21, [sp, #-48]!
//    2  stp x20, x19, [sp, #16]
//    3  stp x29, x30, [sp, #32]
//    4  add x29, sp, #32
//    5  mov x19, x0
//    6  mov w1, #1
//    7  bl _guest_register
//    8  add w21, w0, #4
//    9  mov x0, x19
//   10  mov w1, #2
//   11  bl _guest_register
//   12  add w20, w0, #4
//   13  mov x0, x19
//   14  mov x1, x20
//   15  bl _guest_multiplier_array_cycles
//   16  mov x0, x19
//   17  mov w1, #3
//   18  bl _guest_register
//   19  mov w8, w0
//   20  add x22, x8, #4
//   21  mov x0, x19
//   22  mov w1, #4
//   23  bl _guest_register
//   24  mov x8, #17179869184
//   25  add x8, x8, x0, lsl #32
//   26  orr x8, x8, x22
//   27  umaddl x20, w20, w21, x8
//   28  mov x0, x19
//   29  bl _guest_idle_cycle
//   30  lsr x21, x20, #32
//   31  mov x0, x19
//   32  mov x1, x21
//   33  bl _guest_set_cpsr_negative
//   34  orr w1, w21, w20
//   35  mov x0, x19
//   36  bl _guest_set_cpsr_zero
//   37  mov x0, x19
//   38  mov w1, #3
//   39  mov x2, x20
//   40  bl _guest_set_register
//   41  mov x0, x19
//   42  mov w1, #4
//   43  mov x2, x21
//   44  bl _guest_set_register
//   45  mov x0, x19
//   46  bl _guest_pipeline_flush
//   47  mov w0, #-1
//   48  ldp x29, x30, [sp, #32]
//   49  ldp x20, x19, [sp, #16]
//   50  ldp x22, x21, [sp], #48
//   51  ret
