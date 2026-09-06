use ironboyadvance_arm7tdmi::testing::run_single_step_tests;
use ironboyadvance_arm7tdmi_jit::JitCompiler;

#[test]
fn single_step_tests() {
    run_single_step_tests::<JitCompiler>();
}
