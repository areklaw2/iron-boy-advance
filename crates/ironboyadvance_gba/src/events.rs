use ironboyadvance_common::scheduler::SystemEvent;

use crate::dma_control::RequestType;

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
pub enum ApuEvent {
    Sample,
    FrameSequence,
    FifoStep { timer_id: usize },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimerEvent {
    Overflow { timer_id: usize },
    ControlWrite { timer_id: usize, value: u8 },
    ReloadWrite { timer_id: usize, address: u32, value: u8 },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DmaEvent {
    Activate { channel_id: usize },
    Request(RequestType),
    StopVideo,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CartridgeEvent {
    EepromReady,
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum GbaEvent {
    Interrupt(InterruptEvent),
    Ppu(PpuEvent),
    Apu(ApuEvent),
    Timer(TimerEvent),
    Dma(DmaEvent),
    Cartridge(CartridgeEvent),
}

impl SystemEvent for GbaEvent {
    fn priority(&self) -> u8 {
        match self {
            GbaEvent::Interrupt(_) | GbaEvent::Ppu(_) | GbaEvent::Apu(_) | GbaEvent::Cartridge(_) => 0,
            GbaEvent::Dma(dma_event) => match dma_event {
                DmaEvent::Activate { .. } => 0,
                DmaEvent::Request(_) => 1,
                DmaEvent::StopVideo => 2,
            },
            GbaEvent::Timer(timer_event) => match timer_event {
                TimerEvent::Overflow { .. } => 0,
                TimerEvent::ReloadWrite { .. } => 1,
                TimerEvent::ControlWrite { .. } => 2,
            },
        }
    }
}

pub type FutureGbaEvent = (GbaEvent, usize);
