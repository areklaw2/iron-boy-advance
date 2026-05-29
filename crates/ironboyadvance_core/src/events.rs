use ironboyadvance_common::scheduler::SystemEvent;

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
pub enum GbaEvent {
    FrameComplete,
    Interrupt(InterruptEvent),
    Ppu(PpuEvent),
    Apu(ApuEvent),
    Timer(TimerEvent),
    Dma(usize),
}

impl SystemEvent for GbaEvent {
    fn priority(&self) -> u8 {
        match self {
            GbaEvent::FrameComplete | GbaEvent::Interrupt(_) | GbaEvent::Ppu(_) | GbaEvent::Apu(_) | GbaEvent::Dma(_) => 0,
            GbaEvent::Timer(timer_event) => match timer_event {
                TimerEvent::Overflow { .. } => 0,
                TimerEvent::ReloadWrite { .. } => 1,
                TimerEvent::ControlWrite { .. } => 2,
            },
        }
    }
}

pub type FutureGbaEvent = (GbaEvent, usize);
