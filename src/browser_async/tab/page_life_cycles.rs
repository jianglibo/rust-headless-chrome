use super::super::task_describe::{
    page_events,
};

#[derive(Debug)]
pub struct PageLifeCycles {
    pub life_cycles: Vec<page_events::LifeCycle>,
}

impl PageLifeCycles {
    pub fn life_cycle_happened(&mut self, life_cycle_event: page_events::LifeCycle) {
        self.life_cycles.push(life_cycle_event);
    }

    pub fn last_life_cycle_event(&self) -> &page_events::LifeCycle {
        self.life_cycles
            .last()
            .expect("when last_life_cycle_event is called, it should already have events exists.")
    }

    pub fn life_cycle_event_count(&self, name: &str) -> usize {
        self.life_cycles
            .iter()
            .filter(|lc| lc.get_name() == name)
            .count()
    }
}