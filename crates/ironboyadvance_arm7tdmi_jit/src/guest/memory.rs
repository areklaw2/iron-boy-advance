use ironboyadvance_arm7tdmi::{cpu::Arm7tdmiCpu, memory::MemoryInterface};

pub unsafe extern "C" fn load_8<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, address: u32, access_pattern: u32) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_8(address, access_pattern as u8)
}

pub unsafe extern "C" fn load_signed_8<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    access_pattern: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_signed_8(address, access_pattern as u8)
}

pub unsafe extern "C" fn load_signed_16<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    access_pattern: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_signed_16(address, access_pattern as u8)
}

pub unsafe extern "C" fn load_rotated_16<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    access_pattern: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_rotated_16(address, access_pattern as u8)
}

pub unsafe extern "C" fn load_32<I: MemoryInterface>(cpu: *mut Arm7tdmiCpu<I>, address: u32, access_pattern: u32) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_32(address, access_pattern as u8)
}

pub unsafe extern "C" fn load_rotated_32<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    access_pattern: u32,
) -> u32 {
    let cpu = unsafe { &mut *cpu };
    cpu.load_rotated_32(address, access_pattern as u8)
}

pub unsafe extern "C" fn store_8<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    source: u32,
    access_pattern: u32,
) {
    let cpu = unsafe { &mut *cpu };
    cpu.store_8(address, source as u8, access_pattern as u8);
}

pub unsafe extern "C" fn store_16<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    source: u32,
    access_pattern: u32,
) {
    let cpu = unsafe { &mut *cpu };
    cpu.store_16(address, source as u16, access_pattern as u8);
}

pub unsafe extern "C" fn store_32<I: MemoryInterface>(
    cpu: *mut Arm7tdmiCpu<I>,
    address: u32,
    source: u32,
    access_pattern: u32,
) {
    let cpu = unsafe { &mut *cpu };
    cpu.store_32(address, source, access_pattern as u8);
}
