#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_software_interrupt(cpu: *mut u8, value: u32) -> u32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block(cpu: *mut u8) -> u32 {
    unsafe { guest_software_interrupt(cpu, 0x06) }
}

// 1  mov w1, #6
// 2  b _guest_software_interrupt
