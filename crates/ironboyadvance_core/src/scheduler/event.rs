use std::cmp::Ordering;

use getset::CopyGetters;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum InterruptEvent {
    LcdVBlank,
    LcdHBlank,
    LcdVCounterMatch,
    Timer0Overflow,
    Timer1Overflow,
    Timer2Overflow,
    Timer3Overflow,
    SerialCommunication,
    Dma0Overflow,
    Dma1Overflow,
    Dma2Overflow,
    Dma3Overflow,
    Keypad,
    GamePak,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PpuEvent {
    HDraw,
    HBlank,
    VBlankHDraw,
    VBlankHBlank,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ApuEvent {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimerEvent {
    Overflow { timer_id: usize },
    ControlWrite { timer_id: usize, value: u8 },
    ReloadWrite { timer_id: usize, address: u32, value: u8 },
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EventType {
    FrameComplete,
    Interrupt(InterruptEvent),
    Ppu(PpuEvent),
    Apu(ApuEvent),
    Timer(TimerEvent),
}

impl EventType {
    pub fn priority(&self) -> u8 {
        match self {
            EventType::FrameComplete | EventType::Interrupt(_) | EventType::Ppu(_) | EventType::Apu(_) => 0,
            EventType::Timer(timer_event) => match timer_event {
                TimerEvent::Overflow { .. } => 0,
                TimerEvent::ReloadWrite { .. } => 1,
                TimerEvent::ControlWrite { .. } => 2,
            },
        }
    }
}

pub type FutureEvent = (EventType, usize);

#[derive(Debug, Clone, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Event {
    event_type: EventType,
    time: usize,
}

impl Event {
    pub fn new(event_type: EventType, time: usize) -> Event {
        Event { event_type, time }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.time, other.event_type.priority()).cmp(&(self.time, self.event_type.priority()))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
