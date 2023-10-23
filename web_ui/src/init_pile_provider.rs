use handy_core::game::primitives::*;
use handy_core::solver::model::Matchup;
use handy_core::utils::pile_utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum PileProvider {
    Matchup(Matchup),
    Pile(Pile),
}

pub fn get_init_pile(pile_provider: &PileProvider) -> Pile {
    match pile_provider {
        PileProvider::Matchup(matchup) => {
            get_start_from_classes(matchup.0, matchup.1, &mut rand::thread_rng())
        }
        PileProvider::Pile(pile) => pile.clone(),
    }
}

pub trait InitPileProvider: InitPileProviderClone {
    fn get_init_pile(&self) -> Pile;

    fn is_pile_random(&self) -> bool;
}

pub trait InitPileProviderClone {
    fn clone_box(&self) -> Box<dyn InitPileProvider>;
}

impl<T> InitPileProviderClone for T
where
    T: 'static + InitPileProvider + Clone,
{
    fn clone_box(&self) -> Box<dyn InitPileProvider> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn InitPileProvider> {
    fn clone(&self) -> Box<dyn InitPileProvider> {
        self.clone_box()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchupPileProvider {
    pub matchup: Matchup,
}

impl InitPileProvider for MatchupPileProvider {
    fn get_init_pile(&self) -> Pile {
        get_start_from_classes(self.matchup.0, self.matchup.1, &mut rand::thread_rng())
    }

    fn is_pile_random(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExactPileProvider {
    pub pile: Pile,
}

impl InitPileProvider for ExactPileProvider {
    fn get_init_pile(&self) -> Pile {
        self.pile.clone()
    }

    fn is_pile_random(&self) -> bool {
        false
    }
}
