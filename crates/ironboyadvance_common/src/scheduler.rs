use getset::CopyGetters;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug};

pub trait SystemEvent: Copy + Eq + Ord + Debug {
    fn priority(&self) -> u8;
}

#[derive(Debug, Clone, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Event<E: SystemEvent> {
    event_type: E,
    time: usize,
}

impl<E: SystemEvent> Event<E> {
    pub fn new(event_type: E, time: usize) -> Event<E> {
        Event { event_type, time }
    }
}

impl<E: SystemEvent> Ord for Event<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.time, other.event_type.priority()).cmp(&(self.time, self.event_type.priority()))
    }
}

impl<E: SystemEvent> PartialOrd for Event<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: SystemEvent> PartialEq for Event<E> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

pub struct Scheduler<E: SystemEvent> {
    time: usize,
    events: BinaryHeap<Event<E>>,
}

impl<E: SystemEvent> Scheduler<E> {
    pub fn new() -> Scheduler<E> {
        Scheduler {
            time: 0,
            events: BinaryHeap::new(),
        }
    }

    pub fn peek(&self) -> Option<E> {
        self.events.peek().map(|e| e.event_type())
    }

    pub fn pop(&mut self) -> Option<(E, usize)> {
        match self.events.peek() {
            Some(event) => {
                if self.time >= event.time() {
                    let event = self.events.pop().unwrap_or_else(|| unreachable!());
                    Some((event.event_type(), event.time()))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn cancel_events(&mut self, event_type: E) {
        let mut new_events = BinaryHeap::new();
        self.events
            .iter()
            .filter(|e| e.event_type() != event_type)
            .for_each(|e| new_events.push(e.clone()));
        self.events = new_events
    }

    pub fn schedule(&mut self, event: (E, usize)) {
        let (event_type, delta_time) = event;
        let event = Event::new(event_type, self.time + delta_time);
        self.events.push(event);
    }

    pub fn schedule_at_timestamp(&mut self, event_type: E, timestamp: usize) {
        self.events.push(Event::new(event_type, timestamp));
    }

    pub fn cycles_until_next_event(&self) -> usize {
        if let Some(event) = self.events.peek() {
            event.time().saturating_sub(self.time)
        } else {
            0
        }
    }

    pub fn step(&mut self, cycles: usize) {
        self.time += cycles;
    }

    pub fn step_to_next_event(&mut self) {
        self.time += self.cycles_until_next_event();
    }

    pub fn timestamp_of_next_event(&self) -> usize {
        if let Some(event) = self.events.peek() {
            event.time()
        } else {
            panic!("No events")
        }
    }

    pub fn timestamp(&self) -> usize {
        self.time
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl<E: SystemEvent> Default for Scheduler<E> {
    fn default() -> Self {
        Self::new()
    }
}
