#![crate_type = "lib"]

#[repr(C)]
pub struct ShiftResult {
    pub value: u32,
    pub carry: u32,
}

unsafe extern "C" {
    fn guest_register(cpu: *mut u8, register: u32) -> u32;
    fn guest_set_register(cpu: *mut u8, register: u32, value: u32);
    fn guest_load(cpu: *mut u8, address: u32, access_pattern: u32) -> u32;
    fn guest_store(cpu: *mut u8, address: u32, value: u32, access_pattern: u32);
    fn guest_idle_cycle(cpu: *mut u8);
    fn guest_pipeline_flush(cpu: *mut u8);
    fn guest_cpsr_carry(cpu: *mut u8) -> u32;
    fn guest_lsl_by_register(value: u32, amount: u32, carry: u32) -> ShiftResult;
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_2(cpu: *mut u8) -> u32 {
    unsafe {
        let mut address = guest_register(cpu, 1); //rn = 1
        // Is immediate

        let value = guest_register(cpu, 3); //rm = 3
        let shift_amount = 0x12;
        let carry = guest_cpsr_carry(cpu);
        let ShiftResult {
            value: mut offset,
            carry: _,
        } = guest_lsl_by_register(value, shift_amount, carry);

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

//      _block_2:
//    1  stp x22, x21, [sp, #-48]!
//    2  stp x20, x19, [sp, #16]
//    3  stp x29, x30, [sp, #32]
//    4  add x29, sp, #32
//    5  mov x19, x0
//    6  mov w1, #1
//    7  bl _guest_register
//    8  mov x20, x0
//    9  mov x0, x19
//   10  mov w1, #3
//   11  bl _guest_register
//   12  mov x21, x0
//   13  mov x0, x19
//   14  bl _guest_cpsr_carry
//   15  mov x2, x0
//   16  mov x0, x21
//   17  mov w1, #18
//   18  bl _guest_lsl_by_register
//   19  mov x21, x0
//   20  sub w1, w20, w21
//   21  mov x0, x19
//   22  mov w2, #0
//   23  bl _guest_load
//   24  mov x20, x0
//   25  mov x0, x19
//   26  bl _guest_pipeline_flush
//   27  mov x0, x19
//   28  mov w1, #1
//   29  bl _guest_register
//   30  sub w2, w0, w21
//   31  mov x0, x19
//   32  mov w1, #1
//   33  bl _guest_set_register
//   34  mov x0, x19
//   35  bl _guest_idle_cycle
//   36  mov x0, x19
//   37  mov w1, #2
//   38  mov x2, x20
//   39  bl _guest_set_register
//   40  mov x0, x19
//   41  bl _guest_pipeline_flush
//   42  mov w0, #-1
//   43  ldp x29, x30, [sp, #32]
//   44  ldp x20, x19, [sp, #16]
//   45  ldp x22, x21, [sp], #48
//   46  ret
