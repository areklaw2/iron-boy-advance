use ironboyadvance_common::scheduler::SystemEvent;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum InterruptEvent {
    VBlank,
    Lcd,
    Timer,
    Serial,
    Joypad,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PpuEvent {
    OamScan,
    DrawingPixels,
    HBlank,
    VBlank,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ApuEvent {
    Sample,
    FrameSequence,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimerEvent {
    Overflow,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DmaEvent {
    OamTransfer,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SerialEvent {
    TransferBit,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum GbcEvent {
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
            GbcEvent::Ppu(_) | GbcEvent::Apu(_) | GbcEvent::Dma(_) | GbcEvent::Serial(_) | GbcEvent::Timer(_) => 0,
            GbcEvent::Interrupt(_) => 1,
        }
    }
}

pub type FutureGbcEvent = (GbcEvent, usize);
