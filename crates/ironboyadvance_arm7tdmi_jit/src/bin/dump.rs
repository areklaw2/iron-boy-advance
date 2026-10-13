use dynasmrt::{DynasmApi, aarch64::Assembler};
use ironboyadvance_arm7tdmi::memory::{CpuContext, MemoryInterface};
use ironboyadvance_arm7tdmi_jit::{Compile, decode_arm, decode_thumb};

struct DummyBus;

impl MemoryInterface for DummyBus {
    fn load_8(&mut self, _address: u32, _access_pattern: u8) -> u32 {
        unimplemented!()
    }
    fn load_16(&mut self, _address: u32, _access_pattern: u8) -> u32 {
        unimplemented!()
    }
    fn load_32(&mut self, _address: u32, _access_pattern: u8) -> u32 {
        unimplemented!()
    }
    fn store_8(&mut self, _address: u32, _value: u8, _access_pattern: u8) {
        unimplemented!()
    }
    fn store_16(&mut self, _address: u32, _value: u16, _access_pattern: u8) {
        unimplemented!()
    }
    fn store_32(&mut self, _address: u32, _value: u32, _access_pattern: u8) {
        unimplemented!()
    }
    fn idle_cycle(&mut self) {
        unimplemented!()
    }
    fn cpu_context_mut(&mut self) -> &mut CpuContext {
        unimplemented!()
    }
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let mode = arguments.next().expect("missing mode: arm or thumb");
    let encoding = arguments.next().expect("missing encoding: hex, e.g. 0xE0234291");
    let encoding = u32::from_str_radix(encoding.trim_start_matches("0x"), 16).expect("encoding must be hex");

    let mut assembler = Assembler::new().unwrap();
    let start = assembler.offset();
    match mode.as_str() {
        "arm" => decode_arm(encoding).compile::<DummyBus>(&mut assembler),
        "thumb" => decode_thumb(encoding as u16).compile::<DummyBus>(&mut assembler),
        _ => panic!("mode must be arm or thumb"),
    }
    assembler.commit().unwrap();

    let reader = assembler.reader();
    let buffer = reader.lock();
    std::fs::write("/tmp/block.bin", &buffer[start.0..]).unwrap();
}
