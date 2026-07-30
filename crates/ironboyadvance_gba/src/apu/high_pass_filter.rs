/// High-pass charge factor at 32768 Hz. Closer to 1.0 = gentler (more bass).
/// DMG hardware value: `0.999958^(4194304/32768) ≈ 0.9946`.
const HIGH_PASS_CHARGE_FACTOR: f32 = 0.999;

/// One-pole high-pass that removes the DC offset from the mixed output, so a
/// channel enabling/disabling doesn't click. Models the hardware output capacitor.
#[derive(Debug)]
pub struct HighPassFilter {
    charge_factor: f32,
    previous_left_input: f32,
    previous_left_output: f32,
    previous_right_input: f32,
    previous_right_output: f32,
}

impl HighPassFilter {
    pub fn new() -> Self {
        HighPassFilter {
            charge_factor: HIGH_PASS_CHARGE_FACTOR,
            previous_left_input: 0.0,
            previous_left_output: 0.0,
            previous_right_input: 0.0,
            previous_right_output: 0.0,
        }
    }

    pub fn process(&mut self, left_input: f32, right_input: f32) -> (f32, f32) {
        let left_output = left_input - self.previous_left_input + self.charge_factor * self.previous_left_output;
        self.previous_left_input = left_input;
        self.previous_left_output = left_output;

        let right_output = right_input - self.previous_right_input + self.charge_factor * self.previous_right_output;
        self.previous_right_input = right_input;
        self.previous_right_output = right_output;

        (left_output, right_output)
    }
}
