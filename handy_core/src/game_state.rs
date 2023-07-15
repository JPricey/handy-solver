use super::types::*;

use std::fmt::Debug;

pub const NO_ENERGY_USED: EnergyIds = vec![];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStateWithEventLog {
    pub pile: Pile,
    pub events: Vec<Event>,
    pub event_level: i32,
}

pub trait EngineGameState: Clone + Debug {
    fn new(pile: Pile) -> Self;
    fn append_event(self, event: Event) -> Self;
    fn mut_append_event(&mut self, event: Event);
    fn mut_push_event_level(&mut self);
    fn get_pile(&self) -> &Pile;
    fn get_pile_mut(&mut self) -> &mut Pile;
    fn combine(first: Self, second: Self) -> Self;
}

impl EngineGameState for GameStateWithEventLog {
    fn new(pile: Pile) -> GameStateWithEventLog {
        GameStateWithEventLog {
            pile,
            events: vec![],
            event_level: 0,
        }
    }

    fn append_event(mut self, event: Event) -> Self {
        self.mut_append_event(event);
        self
    }

    fn mut_append_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn mut_push_event_level(&mut self) {
        self.event_level += 1;
        assert!(self.event_level >= 0);
    }

    fn get_pile(&self) -> &Pile {
        &self.pile
    }

    fn get_pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    fn combine(mut first: GameStateWithEventLog, second: GameStateWithEventLog) -> GameStateWithEventLog {
        first.events.extend(second.events);
        GameStateWithEventLog {
            pile: second.pile,
            events: first.events,
            event_level: second.event_level,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStateNoEventLog {
    pub pile: Pile,
}

impl EngineGameState for GameStateNoEventLog {
    fn new(pile: Pile) -> GameStateNoEventLog {
        GameStateNoEventLog { pile }
    }

    fn append_event(self, _event: Event) -> Self {
        self
    }

    fn mut_append_event(&mut self, _event: Event) {}

    fn mut_push_event_level(&mut self) {}

    fn get_pile(&self) -> &Pile {
        &self.pile
    }

    fn get_pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    fn combine(_first: GameStateNoEventLog, second: GameStateNoEventLog) -> GameStateNoEventLog {
        second
    }
}
