use crate::game::card_ptr::CardPtrT;
use crate::game::primitives::{Allegiance, Class, Health, Pile, WinType, HEROS};
use enum_map::{enum_map, EnumMap};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameEndCheckType {
    Standard,
    PerHeroClass,
}

pub fn is_game_winner(pile: &Pile, game_winner_check_type: GameEndCheckType) -> WinType {
    match game_winner_check_type {
        GameEndCheckType::Standard => standard_check_is_game_winner(pile),
        GameEndCheckType::PerHeroClass => per_class_game_resolution(pile),
    }
}

pub fn standard_check_is_game_winner(pile: &Pile) -> WinType {
    let mut player_wins = true;
    let mut enemy_wins = true;

    for card in pile.iter() {
        let active_face = card.get_active_face();
        if active_face.health != Health::Empty {
            match active_face.allegiance {
                Allegiance::Hero => {
                    enemy_wins = false;
                    if !player_wins {
                        return WinType::Unresolved;
                    }
                }
                Allegiance::Monster => {
                    player_wins = false;
                    if !enemy_wins {
                        return WinType::Unresolved;
                    }
                }
                Allegiance::Werewolf | Allegiance::Rat => (),
            }
        }
    }

    if player_wins {
        WinType::Win
    } else if enemy_wins {
        WinType::Lose
    } else {
        WinType::Unresolved
    }
}

pub fn per_class_game_resolution(pile: &Pile) -> WinType {
    let mut player_wins = true;

    let mut seen_class: EnumMap<Class, bool> = enum_map!(_=>false);
    let mut alive_class: EnumMap<Class, bool> = enum_map!(_=>false);

    for card in pile.iter() {
        let active_face = card.get_active_face();
        let active_class = card.get_card_def().class;
        let is_hero_class = active_class.is_hero();
        let active_allegiance = active_face.allegiance;

        if is_hero_class {
            seen_class[active_class] = true;
            if active_allegiance == Allegiance::Hero {
                if active_face.health != Health::Empty {
                    alive_class[active_class] = true;
                }
            }
        } else if active_allegiance == Allegiance::Monster {
            if active_face.health != Health::Empty {
                player_wins = false;
            }
        }
    }

    if player_wins {
        return WinType::Win;
    }

    for class in HEROS {
        if seen_class[class] && !alive_class[class] {
            return WinType::Lose;
        }
    }

    WinType::Unresolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::string_to_pile;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_standard_check_is_game_winner() {
        {
            let pile = string_to_pile("6D 3C 2C 5D 8C 1C 4D 7C 9C");
            assert_eq!(standard_check_is_game_winner(&pile), WinType::Lose);
        }

        {
            let pile = string_to_pile("6C 3C 2C 5D 8C 1C 4D 7C 9C");
            assert_eq!(standard_check_is_game_winner(&pile), WinType::Win);
        }
    }

    #[test]
    fn test_per_class_game_resolution() {
        // Single classes
        {
            // All hero head, enemy alive
            let pile = string_to_pile("1C 2C 6A");
            assert_eq!(per_class_game_resolution(&pile), WinType::Lose);
        }

        {
            // All enemy dead, hero alive
            let pile = string_to_pile("1A 5C 6C");
            assert_eq!(per_class_game_resolution(&pile), WinType::Win);
        }

        {
            // Both all dead
            let pile = string_to_pile("1C 2C 6C 7C");
            assert_eq!(per_class_game_resolution(&pile), WinType::Win);
        }

        {
            // Both alive
            let pile = string_to_pile("1A 2C 6A 7C");
            assert_eq!(per_class_game_resolution(&pile), WinType::Unresolved);
        }

        {
            // Two heros, one of them dead
            let pile = string_to_pile("1C 10A 6A");
            assert_eq!(per_class_game_resolution(&pile), WinType::Lose);
        }
    }
}
