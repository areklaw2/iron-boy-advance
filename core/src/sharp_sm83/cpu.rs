use crate::memory::MemoryInterface;

pub struct SharpSm83Cpu<I: MemoryInterface> {
    bus: I,
}
