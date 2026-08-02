mod common;

use common::Headless;

const STATUS_ADDRESS: u32 = 0xA000;
const STATUS_SIGNATURE: [u8; 3] = [0xDE, 0xB0, 0x61];
const STATUS_RUNNING: u8 = 0x80;
const COMPLETION_STATUSES: &[&str] = &["Passed", "Failed"];

enum Outcome {
    Status(u8),
    Serial(String),
    TimedOut,
}

fn status(headless: &Headless) -> Option<u8> {
    let signature: Vec<u8> = (1..4).map(|offset| headless.read_memory(STATUS_ADDRESS + offset)).collect();
    (signature == STATUS_SIGNATURE).then(|| headless.read_memory(STATUS_ADDRESS))
}

fn run_until_done(headless: &mut Headless, max_frames: usize) -> Outcome {
    let mut started = false;

    for _ in 0..max_frames {
        headless.run_frame();

        match status(headless) {
            Some(STATUS_RUNNING) => started = true,
            Some(code) if started => return Outcome::Status(code),
            _ => {}
        }

        let text = headless.serial_text();
        if COMPLETION_STATUSES.iter().any(|needle| text.contains(needle)) {
            return Outcome::Serial(text);
        }
    }

    Outcome::TimedOut
}

fn assert_passed(rom: &str, max_frames: usize) {
    let mut headless = Headless::load(rom);

    let failure = match run_until_done(&mut headless, max_frames) {
        Outcome::Status(0) => return,
        Outcome::Status(failures) => format!("{failures} test(s) failed"),
        Outcome::Serial(text) if text.contains("Passed") => return,
        Outcome::Serial(_) => "reported failure over serial".to_string(),
        Outcome::TimedOut => format!("no verdict within {max_frames} frames"),
    };

    panic!(
        "{rom}: {failure}\n--- serial ---\n{}\n--- screen ---\n{}",
        headless.serial_text(),
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
fn mem_timing_2() {
    assert_passed("external/gb-test-roms/mem_timing-2/mem_timing.gb", 2000);
}

#[test]
#[ignore = "writes no status, needs investigation"]
fn interrupt_time() {
    assert_passed("external/gb-test-roms/interrupt_time/interrupt_time.gb", 1000);
}

#[test]

fn dmg_sound() {
    assert_passed("external/gb-test-roms/dmg_sound/dmg_sound.gb", 4000);
}

#[test]

fn cgb_sound() {
    assert_passed("external/gb-test-roms/cgb_sound/cgb_sound.gb", 4000);
}

#[test]
#[ignore = "1 sub-test failing, oam bug not implemented"]
fn oam_bug() {
    assert_passed("external/gb-test-roms/oam_bug/oam_bug.gb", 4000);
}

#[test]
#[ignore = "halt bug not implemented, writes no status"]
fn halt_bug() {
    assert_passed("external/gb-test-roms/halt_bug.gb", 1000);
}
