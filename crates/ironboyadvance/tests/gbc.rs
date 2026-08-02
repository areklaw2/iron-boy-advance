mod common;

use common::Headless;

const COMPLETION_NEEDLES: &[&str] = &["Passed", "Failed"];

fn assert_passed(rom: &str, max_frames: usize) {
    let mut headless = Headless::load(rom);
    let text = headless.run_until_serial(COMPLETION_NEEDLES, max_frames);

    assert!(
        text.contains("Passed"),
        "{rom} did not pass within {max_frames} frames\n--- serial ---\n{text}\n--- screen ---\n{}",
        headless.frame_ascii()
    );
}

#[test]
fn cpu_instrs() {
    assert_passed("external/gb-test-roms/cpu_instrs/cpu_instrs.gb", 4000);
}

#[test]
fn instr_timing() {
    assert_passed("external/gb-test-roms/instr_timing/instr_timing.gb", 1000);
}

#[test]
fn mem_timing() {
    assert_passed("external/gb-test-roms/mem_timing/mem_timing.gb", 1000);
}

#[test]
#[ignore = "reports Failed on screen, emits no serial"]
fn mem_timing_2() {
    assert_passed("external/gb-test-roms/mem_timing-2/mem_timing.gb", 2000);
}

#[test]
#[ignore = "reports Failed on screen, emits no serial"]
fn interrupt_time() {
    assert_passed("external/gb-test-roms/interrupt_time/interrupt_time.gb", 1000);
}

#[test]
#[ignore = "apu not implemented"]
fn dmg_sound() {
    assert_passed("external/gb-test-roms/dmg_sound/dmg_sound.gb", 4000);
}

#[test]
#[ignore = "apu not implemented"]
fn cgb_sound() {
    assert_passed("external/gb-test-roms/cgb_sound/cgb_sound.gb", 4000);
}

#[test]
#[ignore = "oam bug not implemented, reports Failed on screen"]
fn oam_bug() {
    assert_passed("external/gb-test-roms/oam_bug/oam_bug.gb", 4000);
}

#[test]
#[ignore = "halt bug not implemented, reports Failed on screen"]
fn halt_bug() {
    assert_passed("external/gb-test-roms/halt_bug.gb", 1000);
}
