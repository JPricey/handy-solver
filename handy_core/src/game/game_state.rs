use std::fmt::Debug;
use crate::game::primitives::*;

pub trait EngineGameState: Clone + Debug {
    fn new(pile: Pile) -> Self;
    fn append_event(self, event: Event) -> Self;
    fn mut_append_event(&mut self, event: Event);
    fn get_pile(&self) -> &Pile;
    fn get_pile_mut(&mut self) -> &mut Pile;
    fn combine(first: Self, second: Self) -> Self;
}

// GameStateNoEventLog
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStateNoEventLog {
    pub pile: Pile,
}

impl EngineGameState for GameStateNoEventLog {
    fn new(pile: Pile) -> Self {
        Self { pile }
    }

    fn append_event(self, _event: Event) -> Self {
        self
    }

    fn mut_append_event(&mut self, _event: Event) {}

    fn get_pile(&self) -> &Pile {
        &self.pile
    }

    fn get_pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    fn combine(_first: Self, second: Self) -> Self {
        second
    }
}

// GameStateWithEventLog
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStateWithEventLog {
    pub pile: Pile,
    pub events: Vec<Event>,
}

impl EngineGameState for GameStateWithEventLog {
    fn new(pile: Pile) -> Self {
        Self {
            pile,
            events: vec![],
        }
    }

    fn append_event(mut self, event: Event) -> Self {
        self.mut_append_event(event);
        self
    }

    fn mut_append_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn get_pile(&self) -> &Pile {
        &self.pile
    }

    fn get_pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    fn combine(mut first: Self, second: Self) -> Self {
        first.events.extend(second.events);
        Self {
            pile: second.pile,
            events: first.events,
        }
    }
}


// GameStateWithPileTrackedEventLog
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStateWithPileTrackedEventLog {
    pub pile: Pile,
    pub events: Vec<(Pile, Event)>,
}

impl EngineGameState for GameStateWithPileTrackedEventLog {
    fn new(pile: Pile) -> Self {
        Self {
            pile,
            events: vec![],
        }
    }

    fn append_event(mut self, event: Event) -> Self {
        self.mut_append_event(event);
        self
    }

    fn mut_append_event(&mut self, event: Event) {
        self.events.push((self.pile.clone(), event));
    }

    fn get_pile(&self) -> &Pile {
        &self.pile
    }

    fn get_pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    fn combine(mut first: Self, second: Self) -> Self {
        first.events.extend(second.events);
        Self {
            pile: second.pile,
            events: first.events,
        }
    }
}

