use getset::{CopyGetters, Setters};

const INTERRUPT_MASK: u8 = 0x1F;

#[derive(Debug, Default, Copy, Clone, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct InterruptContext {
    interrupt_enabled: u8,
    interrupt_flags: u8,
}

impl InterruptContext {
    pub fn pending_interrupt(&self) -> u8 {
        self.interrupt_enabled & self.interrupt_flags & INTERRUPT_MASK
    }

    pub fn raise_interrupt(&mut self, bit: u8) {
        self.interrupt_flags |= 1 << bit;
    }

    pub fn clear_interrupt(&mut self, bit: u8) {
        self.interrupt_flags &= !(1 << bit);
    }
}

pub trait MemoryInterface {
    fn load_8(&mut self, address: u16) -> u8;

    fn load_16(&mut self, address: u16) -> u16;

    fn store_8(&mut self, address: u16, value: u8);

    fn store_16(&mut self, address: u16, value: u16);

    fn idle_cycle(&mut self);

    fn change_speed(&mut self) -> bool;

    fn interrupt_context(&self) -> &InterruptContext;

    fn interrupt_context_mut(&mut self) -> &mut InterruptContext;
}
