use ironboyadvance_arm7tdmi::testing::run_single_step_tests;
use ironboyadvance_arm7tdmi_interpreter::Interpreter;

#[test]
fn single_step_tests() {
    let strategy = Interpreter::new();
    run_single_step_tests(&strategy);
}
