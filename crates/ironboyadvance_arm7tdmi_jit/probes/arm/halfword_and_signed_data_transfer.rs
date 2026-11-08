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
        let mut address = guest_register(cpu, 1); //rn = 1
        // Is immediate
        let mut offset = 0x1234;

        // subtract not add
        // negate
        offset = (-(offset as i64)) as u32;

        //pre index
        address = address.wrapping_add(offset);

        //load
        let value = guest_load(cpu, address, 0);
        // write back || not pre index
        // rn != rd && rd == PC
        guest_pipeline_flush(cpu);
        let write_back = guest_register(cpu, 1).wrapping_add(offset);
        guest_set_register(cpu, 1, write_back);

        guest_idle_cycle(cpu);
        guest_set_register(cpu, 2, value); //rd = 2

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
//    8  mov w21, #-4660
//    9  add w1, w0, w21
//   10  mov x0, x19
//   11  mov w2, #0
//   12  bl _guest_load
//   13  mov x20, x0
//   14  mov x0, x19
//   15  bl _guest_pipeline_flush
//   16  mov x0, x19
//   17  mov w1, #1
//   18  bl _guest_register
//   19  add w2, w0, w21
//   20  mov x0, x19
//   21  mov w1, #1
//   22  bl _guest_set_register
//   23  mov x0, x19
//   24  bl _guest_idle_cycle
//   25  mov x0, x19
//   26  mov w1, #2
//   27  mov x2, x20
//   28  bl _guest_set_register
//   29  mov x0, x19
//   30  bl _guest_pipeline_flush
//   31  mov w0, #-1
//   32  ldp x29, x30, [sp, #32]
//   33  ldp x20, x19, [sp, #16]
//   34  ldp x22, x21, [sp], #48
//   35  ret
