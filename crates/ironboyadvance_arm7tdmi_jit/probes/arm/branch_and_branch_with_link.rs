#![crate_type = "lib"]
unsafe extern "C" {
    fn trampoline_pc(cpu: *mut u8) -> u32;
    fn trampoline_set_register(cpu: *mut u8, register: u32, value: u32);
    fn trampoline_set_pc(cpu: *mut u8, value: u32);
    fn trampoline_pipeline_flush(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block(cpu: *mut u8) -> u32 {
    unsafe {
        let pc = trampoline_pc(cpu);
        trampoline_set_register(cpu, 14, pc.wrapping_sub(4)); // link = true
        trampoline_set_pc(cpu, pc.wrapping_add(0xFFFFFF00)); // offset = -256
        trampoline_pipeline_flush(cpu);
    }
    0xFFFF_FFFF
}

//  1  stp x20, x19, [sp, #-32]!
//  2  stp x29, x30, [sp, #16]
//  3  add x29, sp, #16
//  4  mov x19, x0
//  5  bl _trampoline_pc
//  6  mov x20, x0
//  7  sub w2, w0, #4
//  8  mov x0, x19
//  9  mov w1, #14
// 10  bl _trampoline_set_register
// 11  sub w1, w20, #256
// 12  mov x0, x19
// 13  bl _trampoline_set_pc
// 14  mov x0, x19
// 15  bl _trampoline_pipeline_flush
// 16  mov w0, #-1
// 17  ldp x29, x30, [sp, #16]
// 18  ldp x20, x19, [sp], #32
// 19  ret
