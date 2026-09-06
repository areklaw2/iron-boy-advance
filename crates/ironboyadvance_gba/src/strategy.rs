use ironboyadvance_arm7tdmi::{ExecutionStrategy, cpu::Arm7tdmiCpu, memory::MemoryInterface};
use ironboyadvance_arm7tdmi_interpreter::Interpreter;
use ironboyadvance_arm7tdmi_jit::JitCompiler;

pub enum Strategy {
    Interpreter(Box<Interpreter>),
    Jit(JitCompiler),
}

impl Strategy {
    pub fn interpreter() -> Self {
        Strategy::Interpreter(Box::new(Interpreter::new()))
    }

    pub fn jit() -> Self {
        Strategy::Jit(JitCompiler::new())
    }
}

impl ExecutionStrategy for Strategy {
    fn cycle<I: MemoryInterface>(&mut self, cpu: &mut Arm7tdmiCpu<I>) {
        match self {
            Strategy::Interpreter(s) => s.cycle(cpu),
            Strategy::Jit(s) => s.cycle(cpu),
        }
    }
}
