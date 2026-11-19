use ironboyadvance_arm7tdmi::testing::TestBus;

use crate::JitInterface;

impl JitInterface for TestBus {
    fn peek_16(&mut self, _address: u32) -> Option<u32> {
        None
    }

    fn peek_32(&mut self, _address: u32) -> Option<u32> {
        None
    }

    fn watch_memory_range(&mut self, _address: u32, _length: u32) {}

    fn consume_watched_write(&mut self) -> bool {
        false
    }

    fn block_exit_pending(&mut self, _irq_enabled: bool) -> bool {
        false
    }
}
