use std::collections::VecDeque;

use ironboyadvance_common::memory::SystemMemoryAccess;

const FIFO_CAPACITY: usize = 32;
const FIFO_REFILL_THRESHOLD: usize = 16;

#[derive(Debug)]
pub struct FifoChannel {
    queue: VecDeque<i8>,
    current_sample: i8,
}

impl SystemMemoryAccess for FifoChannel {
    type Address = u32;

    fn read_8(&self, _address: u32) -> u8 {
        0
    }

    fn write_8(&mut self, _address: u32, value: u8) {
        if self.queue.len() < FIFO_CAPACITY {
            self.queue.push_back(value as i8);
        }
    }
}

impl FifoChannel {
    pub fn new() -> Self {
        FifoChannel {
            queue: VecDeque::with_capacity(FIFO_CAPACITY),
            current_sample: 0,
        }
    }

    pub fn step(&mut self) -> bool {
        if let Some(sample) = self.queue.pop_front() {
            self.current_sample = sample;
        }
        self.queue.len() <= FIFO_REFILL_THRESHOLD
    }

    pub fn reset(&mut self) {
        self.queue.clear();
        self.current_sample = 0;
    }

    pub fn output(&self) -> i8 {
        self.current_sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(fifo: &mut FifoChannel, value: u8) {
        fifo.write_8(0x040000A0, value);
    }

    #[test]
    fn pushes_and_pops_in_order() {
        let mut fifo = FifoChannel::new();
        push(&mut fifo, 10);
        push(&mut fifo, 20);
        fifo.step();
        assert_eq!(fifo.output(), 10);
        fifo.step();
        assert_eq!(fifo.output(), 20);
    }

    #[test]
    fn holds_last_sample_when_empty() {
        let mut fifo = FifoChannel::new();
        push(&mut fifo, 42);
        fifo.step();
        assert_eq!(fifo.output(), 42);
        // Empty now; another step holds the last latched value.
        fifo.step();
        assert_eq!(fifo.output(), 42);
    }

    #[test]
    fn signed_samples_round_trip() {
        let mut fifo = FifoChannel::new();
        push(&mut fifo, 0x80); // -128
        fifo.step();
        assert_eq!(fifo.output(), -128);
    }

    #[test]
    fn refill_requested_at_half_empty() {
        let mut fifo = FifoChannel::new();
        for _ in 0..FIFO_CAPACITY {
            push(&mut fifo, 1);
        }
        // Popping stays quiet until the queue crosses the half-empty threshold.
        for remaining in (0..FIFO_CAPACITY).rev() {
            assert_eq!(fifo.step(), remaining <= FIFO_REFILL_THRESHOLD);
        }
    }

    #[test]
    fn drops_writes_when_full() {
        let mut fifo = FifoChannel::new();
        for sample in 0..FIFO_CAPACITY {
            push(&mut fifo, sample as u8);
        }
        push(&mut fifo, 99); // dropped
        // Head is still the original first sample, not the dropped write.
        fifo.step();
        assert_eq!(fifo.output(), 0);
    }

    #[test]
    fn reset_clears_queue_and_output() {
        let mut fifo = FifoChannel::new();
        push(&mut fifo, 50);
        fifo.step();
        fifo.reset();
        assert_eq!(fifo.output(), 0);
        // Nothing queued: step holds the reset value.
        fifo.step();
        assert_eq!(fifo.output(), 0);
    }
}
