#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
    fn guest_load(cpu: *mut u8, address: u32, access_pattern: u32) -> u32;
    fn guest_store(cpu: *mut u8, address: u32, value: u32, access_pattern: u32);
    fn guest_idle_cycle(cpu: *mut u8);
    fn guest_pipeline_flush(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_1(cpu: *mut u8) -> u32 {
    unsafe {
        let address = guest_register(cpu, 1); //rn = 1
        let source = guest_register(cpu, 2) + 4; //rm = 2

        let value = guest_load(cpu, address, 0);
        guest_store(cpu, address, source, 8);

        guest_idle_cycle(cpu);
        guest_set_register(cpu, 3, value); //rd = 3
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
//    8  mov x20, x0
//    9  mov x0, x19
//   10  mov w1, #2
//   11  bl _guest_register
//   12  mov x21, x0
//   13  mov x0, x19
//   14  mov x1, x20
//   15  mov w2, #0
//   16  bl _guest_load
//   17  mov x22, x0
//   18  add w2, w21, #4
//   19  mov x0, x19
//   20  mov x1, x20
//   21  mov w3, #8
//   22  bl _guest_store
//   23  mov x0, x19
//   24  bl _guest_idle_cycle
//   25  mov x0, x19
//   26  mov w1, #3
//   27  mov x2, x22
//   28  bl _guest_set_register
//   29  mov x0, x19
//   30  bl _guest_pipeline_flush
//   31  mov w0, #-1
//   32  ldp x29, x30, [sp, #32]
//   33  ldp x20, x19, [sp, #16]
//   34  ldp x22, x21, [sp], #48
//   35  ret
