mod common;

use common::Headless;
use ironboyadvance::ExecutionMode;
use std::{
    env, fs, process,
    sync::atomic::{AtomicUsize, Ordering},
};

const ENTRY_BRANCH_TO_PROGRAM: u32 = 0xEA00002E;
const HEADER_SIZE: usize = 0xC0;
const HEADER_CHECKSUM_START: usize = 0xA0;
const HEADER_CHECKSUM_END: usize = 0xBC;
const HEADER_CHECKSUM_OFFSET: usize = 0xBD;
const HEADER_CHECKSUM_BIAS: u8 = 0x19;
const MINIMUM_ROM_SIZE: usize = 0x1000;

const PROGRAM: &[u8] = include_bytes!("self_modifying.bin");

const RESULT_BASE: u32 = 0x0300_0100;
const FUNCTION_ONE_RESULT: u32 = 1;
const OVERWRITTEN_WITH_FUNCTION_TWO_RESULT: u32 = 2;
const SELF_REWRITING_FUNCTION_RESULT: u32 = 2;
const EXPECTED_RESULTS: [u32; 3] = [
    FUNCTION_ONE_RESULT,
    OVERWRITTEN_WITH_FUNCTION_TWO_RESULT,
    SELF_REWRITING_FUNCTION_RESULT,
];

static ROM_COUNT: AtomicUsize = AtomicUsize::new(0);

fn build_rom(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; HEADER_SIZE];
    rom[0..4].copy_from_slice(&ENTRY_BRANCH_TO_PROGRAM.to_le_bytes());

    let mut checksum = 0u8;
    for byte in &rom[HEADER_CHECKSUM_START..=HEADER_CHECKSUM_END] {
        checksum = checksum.wrapping_sub(*byte);
    }
    rom[HEADER_CHECKSUM_OFFSET] = checksum.wrapping_sub(HEADER_CHECKSUM_BIAS);

    rom.extend_from_slice(program);
    rom.resize(rom.len().max(MINIMUM_ROM_SIZE), 0);
    rom
}

fn load_program(program: &[u8], mode: ExecutionMode) -> Headless {
    let count = ROM_COUNT.fetch_add(1, Ordering::Relaxed);
    let directory = env::temp_dir()
        .join("ironboyadvance-tests")
        .join(format!("rom-{}-{}", process::id(), count));
    fs::create_dir_all(&directory).unwrap_or_else(|error| panic!("failed to create {directory:?}: {error}"));

    let path = directory.join("program.gba");
    fs::write(&path, build_rom(program)).unwrap_or_else(|error| panic!("failed to write {path:?}: {error}"));

    let headless = Headless::load_with(&path.to_string_lossy(), mode);
    fs::remove_dir_all(&directory).unwrap_or_else(|error| panic!("failed to remove {directory:?}: {error}"));
    headless
}

fn read_word(headless: &Headless, address: u32) -> u32 {
    let bytes = [0, 1, 2, 3].map(|offset| headless.read_memory(address + offset));
    u32::from_le_bytes(bytes)
}

fn run_results(mode: ExecutionMode) -> [u32; 3] {
    let mut headless = load_program(PROGRAM, mode);
    headless.run_frames(2);
    [0, 4, 8].map(|offset| read_word(&headless, RESULT_BASE + offset))
}

#[test]
fn interpreter_sees_rewritten_code() {
    assert_eq!(run_results(ExecutionMode::Interpreter), EXPECTED_RESULTS);
}

#[cfg(target_arch = "aarch64")]
#[test]
fn jit_sees_rewritten_code() {
    assert_eq!(run_results(ExecutionMode::Jit), EXPECTED_RESULTS);
}
