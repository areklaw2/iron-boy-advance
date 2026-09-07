#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_cpsr_state(cpu: *mut u8, value: u32);
    fn guest_set_pc(cpu: *mut u8, value: u32);
    fn guest_pipeline_flush(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block(cpu: *mut u8) -> u32 {
    unsafe {
        let value = guest_register(cpu, 4); // register 4
        guest_set_cpsr_state(cpu, value);
        guest_set_pc(cpu, value & !0x1);
        guest_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  mov w1, #4
//  6  bl _guest_register
//  7  mov x20, x0
//  8  mov x0, x19
//  9  mov x1, x20
// 10  bl _guest_set_cpsr_state
// 11  and w1, w20, #0xfffffffe
// 12  mov x0, x19
// 13  bl _guest_set_pc
// 14  mov x0, x19
// 15  bl _guest_pipeline_flush
// 16  mov w0, #-1
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret
