use ironboyadvance_common::scheduler::SystemEvent;

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum InterruptEvent {
    VBlank,
    Lcd,
    Timer,
    Serial,
    Joypad,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PpuEvent {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[allow(unused)]
pub enum ApuEvent {
    Sample,
    FrameSequence,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimerEvent {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DmaEvent {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SerialEvent {
    TransferBit,
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum GbcEvent {
    FrameComplete,
    Interrupt(InterruptEvent),
    Ppu(PpuEvent),
    Apu(ApuEvent),
    Timer(TimerEvent),
    Dma(DmaEvent),
    Serial(SerialEvent),
}

impl SystemEvent for GbcEvent {
    fn priority(&self) -> u8 {
        match self {
            GbcEvent::FrameComplete
            | GbcEvent::Interrupt(_)
            | GbcEvent::Ppu(_)
            | GbcEvent::Apu(_)
            | GbcEvent::Dma(_)
            | GbcEvent::Serial(_)
            | GbcEvent::Timer(_) => 0,
        }
    }
}

pub type FutureGbcEvent = (GbcEvent, usize);
