#![crate_type = "lib"]
unsafe extern "C" {
    fn guest_pc(cpu: *mut u8) -> u32;
    fn guest_is_condition_met(cpu: *mut u8, value: u32) -> u32;
    fn guest_set_pc(cpu: *mut u8, value: u32);
    fn guest_pipeline_flush(cpu: *mut u8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn block(cpu: *mut u8) -> u32 {
    unsafe {
        if guest_is_condition_met(cpu, 1) != 0 {
            return 1;
        } else {
            let pc = guest_pc(cpu);
            guest_set_pc(cpu, pc.wrapping_add(0xFFFFFF00)); // offset = -256
            guest_pipeline_flush(cpu);
            return 0;
        }
    }
}
