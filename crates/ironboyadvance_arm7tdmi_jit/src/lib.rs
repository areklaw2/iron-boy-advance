use ironboyadvance_arm7tdmi::{CpuAction, ExecutionStrategy, cpu::Arm7tdmiCpu, memory::MemoryInterface};

pub trait Execute {
    fn execute<I: MemoryInterface>(&self, cpu: &mut Arm7tdmiCpu<I>) -> CpuAction;
}

pub struct Jit {}

impl Jit {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Jit {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionStrategy for Jit {
    fn cycle<I: MemoryInterface>(&self, _cpu: &mut Arm7tdmiCpu<I>) {
        todo!()
    }
}
