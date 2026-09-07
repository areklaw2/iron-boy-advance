#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_undefined_exception(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block(cpu: *mut u8) -> u32 {
    unsafe {
        guest_undefined_exception(cpu);
    }
    0xFFFF_FFFF
}

//  1  stp x29, x30, [sp, #-16]!
//  2  mov x29, sp
//  3  bl _guest_undefined_exception
//  4  mov w0, #-1
//  5  ldp x29, x30, [sp], #16
//  6  ret
