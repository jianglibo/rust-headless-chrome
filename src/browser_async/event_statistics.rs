use std::time::{Instant};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EventName {
    SetChildNodes,
    ExecutionContextCreated,
    RequestIntercepted,
    LoadEventFired,
    FrameNavigated,
}

#[derive(Debug)]
pub struct OneEventStatistics {
    pub count: u64,
    pub last_happened_at: Instant,
}

impl OneEventStatistics {
    pub fn new() -> Self {
        Self {
            count: 1,
            last_happened_at: Instant::now(),
        }
    }
}

impl std::default::Default for OneEventStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct EventStatistics {
    events: HashMap<EventName, OneEventStatistics>,
}

impl EventStatistics {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn event_happened(&mut self, event_name: EventName) {
        self.events.entry(event_name).and_modify(|ent|{
                ent.count += 1;
                ent.last_happened_at = Instant::now();
            }).or_default();
    }

    pub fn happened_count(&self, event_name: EventName) -> u64 {
        self.events.get(&event_name).map_or(0, |oes| oes.count)
    }

    pub fn happened_before_secs(&self, event_name: EventName, seconds: u64) -> bool {
        let last_happened_at = self.events.get(&event_name).map_or(Instant::now(), |ent|ent.last_happened_at);
        last_happened_at.elapsed().as_secs() > seconds
    }
    pub fn happened_within_secs(&self, event_name: EventName, seconds: u64) -> bool {
        !self.happened_before_secs(event_name, seconds)
    }

    pub fn load_event_fired_count(&self) -> u64 {
        self.happened_count(EventName::LoadEventFired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;


    #[test]
    fn test_enum_partial_eq() {
        assert!(EventName::RequestIntercepted == EventName::RequestIntercepted);
        assert!(EventName::RequestIntercepted != EventName::SetChildNodes);

        let mut h = HashMap::<EventName, u8>::new();
        h.insert(EventName::RequestIntercepted, 8);
        h.insert(EventName::SetChildNodes, 6);
        assert_eq!(h.get(&EventName::RequestIntercepted), Some(&8));
        assert_eq!(h.get(&EventName::SetChildNodes), Some(&6));

        h.insert(EventName::RequestIntercepted, 0);
        assert_eq!(h.get(&EventName::RequestIntercepted), Some(&0));
    }
}
