use handy_core::game::*;
use handy_core::solver::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn get_relevant_cards_for_class(class: Class) -> Vec<CardId> {
    CARDS
        .get_cards_for_class(class)
        .iter()
        .map(|c| c.id)
        .collect()
}

pub fn get_relevant_cards_for_matchup(matchup: Matchup) -> Vec<CardId> {
    let mut a = get_relevant_cards_for_class(matchup.0);
    a.extend(get_relevant_cards_for_class(matchup.1));
    a
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum StateEval {
    Win(usize),
    Loss,
}

pub fn state_eval_to_score(s: StateEval) -> usize {
    match s {
        StateEval::Win(v) => v,
        StateEval::Loss => 100,
    }
}

fn serialize_pile<S>(value: &Pile, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    (&value as &[_]).serialize(serializer)
}

fn deserialize_pile<'de, D>(deserializer: D) -> Result<Pile, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<CardPtr>::deserialize(deserializer)?;
    let pile: Pile = (&values as &[CardPtr]).try_into().unwrap();
    Ok(pile)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepthModeTrainingExample {
    #[serde(
        serialize_with = "serialize_pile",
        deserialize_with = "deserialize_pile"
    )]
    pub pile: Pile,
    pub eval: StateEval,
}
