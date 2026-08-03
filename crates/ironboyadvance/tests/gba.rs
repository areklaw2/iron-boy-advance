mod common;

use common::Headless;

const ALL_TESTS_PASSED: u64 = 0xFD05_07D8_C680_90A5;

fn assert_frame_hash(rom: &str, frames: usize, expected: u64) {
    let mut headless = Headless::load(rom);
    headless.run_frames(frames);
    let hash = headless.frame_hash();

    assert_eq!(
        hash,
        expected,
        "{rom} frame hash mismatch after {frames} frames\nexpected {expected:#018X}\n  actual {hash:#018X}\n{}",
        headless.frame_ascii()
    );
}

#[test]
fn arm() {
    assert_frame_hash("external/gba-tests/arm/arm.gba", 600, ALL_TESTS_PASSED);
}

#[test]
fn thumb() {
    assert_frame_hash("external/gba-tests/thumb/thumb.gba", 600, ALL_TESTS_PASSED);
}

#[test]
fn memory() {
    assert_frame_hash("external/gba-tests/memory/memory.gba", 600, ALL_TESTS_PASSED);
}

#[test]
fn nes() {
    assert_frame_hash("external/gba-tests/nes/nes.gba", 600, ALL_TESTS_PASSED);
}

#[test]
fn save_none() {
    assert_frame_hash("external/gba-tests/save/none.gba", 3000, ALL_TESTS_PASSED);
}

#[test]
fn save_sram() {
    assert_frame_hash("external/gba-tests/save/sram.gba", 3000, ALL_TESTS_PASSED);
}

#[test]
fn save_flash64() {
    assert_frame_hash("external/gba-tests/save/flash64.gba", 3000, ALL_TESTS_PASSED);
}

#[test]
fn save_flash128() {
    assert_frame_hash("external/gba-tests/save/flash128.gba", 3000, ALL_TESTS_PASSED);
}

#[test]
fn unsafe_access() {
    assert_frame_hash("external/gba-tests/unsafe/unsafe.gba", 600, ALL_TESTS_PASSED);
}
