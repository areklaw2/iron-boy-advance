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

        let result = operand1.wrapping_mul(operand2);

        guest_multiplier_array_cycles(cpu, operand2);

        guest_set_cpsr_negative(cpu, result);
        guest_set_cpsr_zero(cpu, result);

        guest_set_register(cpu, 4, result); //rd = 4
        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//      _block_1:
//    1  stp x20, x19, [sp, #-32]!
//    2  stp x29, x30, [sp, #16]
//    3  add x29, sp, #16
//    4  mov x19, x0
//    5  mov w1, #1
//    6  bl _guest_register
//    7  add w20, w0, #4
//    8  mov x0, x19
//    9  mov w1, #2
//   10  bl _guest_register
//   11  add w1, w0, #4
//   12  mul w20, w1, w20
//   13  mov x0, x19
//   14  bl _guest_multiplier_array_cycles
//   15  mov x0, x19
//   16  mov x1, x20
//   17  bl _guest_set_cpsr_negative
//   18  mov x0, x19
//   19  mov x1, x20
//   20  bl _guest_set_cpsr_zero
//   21  mov x0, x19
//   22  mov w1, #4
//   23  mov x2, x20
//   24  bl _guest_set_register
//   25  mov x0, x19
//   26  bl _guest_pipeline_flush
//   27  mov w0, #-1
//   28  ldp x29, x30, [sp, #16]
//   29  ldp x20, x19, [sp], #32
//   30  ret

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let operand1 = guest_register(cpu, 1) + 4; //rm = 1
        let operand2 = guest_register(cpu, 2) + 4; //rs = 2

        let product = operand1.wrapping_mul(operand2);

        guest_multiplier_array_cycles(cpu, operand2);

        //accumulate
        let accumulator = guest_register(cpu, 3) + 4; //rn = 3
        let result = product.wrapping_add(accumulator);
        guest_idle_cycle(cpu);

        guest_set_cpsr_negative(cpu, result);
        guest_set_cpsr_zero(cpu, result);

        guest_set_register(cpu, 4, result); //rd = 4
        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//    _block_2:
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
//   19  madd w20, w20, w21, w0
//   20  mov x0, x19
//   21  bl _guest_idle_cycle
//   22  add w1, w20, #4
//   23  mov x0, x19
//   24  bl _guest_set_cpsr_negative
//   25  add w1, w20, #4
//   26  mov x0, x19
//   27  bl _guest_set_cpsr_zero
//   28  add w2, w20, #4
//   29  mov x0, x19
//   30  mov w1, #4
//   31  bl _guest_set_register
//   32  mov x0, x19
//   33  bl _guest_pipeline_flush
//   34  mov w0, #-1
//   35  ldp x29, x30, [sp, #32]
//   36  ldp x20, x19, [sp, #16]
//   37  ldp x22, x21, [sp], #48
//   38  ret
