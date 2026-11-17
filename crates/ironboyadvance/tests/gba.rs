mod common;

use common::Headless;
use ironboyadvance::ExecutionMode;

const ALL_TESTS_PASSED: u64 = 0xFD05_07D8_C680_90A5;

fn assert_frame_hash(rom: &str, frames: usize, expected: u64) {
    assert_frame_hash_with(rom, frames, expected, ExecutionMode::Interpreter);
}

fn assert_frame_hash_with(rom: &str, frames: usize, expected: u64, mode: ExecutionMode) {
    let mut headless = Headless::load_with(rom, mode);
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

#[cfg(target_arch = "aarch64")]
mod jit {
    use super::*;

    fn assert_jit_frame_hash(rom: &str, frames: usize) {
        assert_frame_hash_with(rom, frames, ALL_TESTS_PASSED, ExecutionMode::Jit);
    }

    #[test]
    fn arm() {
        assert_jit_frame_hash("external/gba-tests/arm/arm.gba", 600);
    }

    #[test]
    fn thumb() {
        assert_jit_frame_hash("external/gba-tests/thumb/thumb.gba", 600);
    }

    #[test]
    fn memory() {
        assert_jit_frame_hash("external/gba-tests/memory/memory.gba", 600);
    }

    #[test]
    fn nes() {
        assert_jit_frame_hash("external/gba-tests/nes/nes.gba", 600);
    }

    #[test]
    fn save_none() {
        assert_jit_frame_hash("external/gba-tests/save/none.gba", 3000);
    }

    #[test]
    fn save_sram() {
        assert_jit_frame_hash("external/gba-tests/save/sram.gba", 3000);
    }

    #[test]
    fn save_flash64() {
        assert_jit_frame_hash("external/gba-tests/save/flash64.gba", 3000);
    }

    #[test]
    fn save_flash128() {
        assert_jit_frame_hash("external/gba-tests/save/flash128.gba", 3000);
    }

    #[test]
    fn unsafe_access() {
        assert_jit_frame_hash("external/gba-tests/unsafe/unsafe.gba", 600);
    }
}
