use handy_core::game::*;
use handy_core::utils::*;
use rand::Rng;

pub fn get_random_pile_matching_stats<R: Rng>(hero: Class, monster: Class, randomize_sides_pct: usize, randomize_hero_sides_pct: usize, rng: &mut R) -> Pile {
    let score: usize = rng.gen_range(0..100);
    if score < randomize_sides_pct {
        return get_random_pile_with_no_winner(hero, monster, rng);
    } else if score < randomize_sides_pct + randomize_hero_sides_pct {
        let mut pile = get_start_from_classes(hero, monster, rng);
        randomize_hero_sides(&mut pile, rng);
        return pile;
    } else {
        return get_start_from_classes(hero, monster, rng);
    }
}

pub fn get_fully_random_pile<R: Rng>(hero: Class, monster: Class, rng: &mut R) -> Pile {
    let mut pile = get_start_from_classes(hero, monster, rng);
    randomize_sides(&mut pile, rng);
    return pile;
}

pub fn get_random_pile_with_no_winner<R: Rng>(hero: Class, monster: Class, rng: &mut R) -> Pile {
    let mut pile = get_start_from_classes(hero, monster, rng);
    randomize_sides(&mut pile, rng);
    while is_game_winner(&pile).is_some() {
        randomize_sides(&mut pile, rng);
    }
    return pile;
}

pub fn get_random_won_pile<R: Rng>(hero: Class, monster: Class, rng: &mut R) -> Pile {
    let mut pile = get_start_from_classes(hero, monster, rng);
    randomize_sides(&mut pile, rng);
    for card in &mut pile {
        if card.get_active_face().allegiance == Allegiance::Baddie {
            card.key = get_random_exhausted_face(rng, card.get_card_def());
        }
    }
    return pile;
}

pub fn randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        card_ptr.key = get_random_face(rng);
    }
}

pub fn randomize_hero_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        if card_ptr.get_active_face().allegiance != Allegiance::Baddie {
            card_ptr.key = get_random_face(rng);
        }
    }

    if is_game_winner(pile).is_some() {
        randomize_hero_sides(pile, rng);
    }
}
