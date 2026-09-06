use ironboyadvance_arm7tdmi::testing::run_single_step_tests;
use ironboyadvance_arm7tdmi_jit::Jit;

#[test]
fn single_step_tests() {
    let strategy = Jit::new();
    run_single_step_tests(&strategy);
}
