use cli::paths::*;
use handy_core::game::*;
use handy_core::solver::a_star::*;
use handy_core::solver::*;

fn solve_pile(start_pile: Pile, model: Model) {
    let mut a_star_solver = AStarSolver::new(&vec![start_pile.clone()], model.clone());

    loop {
        let iter_result = a_star_solver.single_iter();
        match iter_result {
            AStarIterResult::Done(_) => {
                panic!("Could not solve pile: {:?}", start_pile);
            }
            AStarIterResult::NewBest(_) => {
                println!(": done {}", a_star_solver.total_iters);
                return;
            }
            AStarIterResult::Continue(_) => {}
        }
    }
}

// http://antoinecomeau.blogspot.com/2014/07/mapping-between-permutations-and.html
fn number_to_ordering(number: usize, n: usize) -> Vec<usize> {
    let mut result = vec![0; n];
    let mut elems = vec![0; n];
    for i in 1..n {
        elems[i] = i;
    }

    let mut ind;
    let mut m = number as usize;
    for i in 0..n {
        ind = m % (n - i);
        m = m / (n - i);
        result[i] = elems[ind];
        elems[ind] = elems[n - i - 1];
    }
    result
}

fn main() {
    let warrior_cards = vec![
        CardPtr::new_from_id(1, FaceKey::A),
        CardPtr::new_from_id(2, FaceKey::A),
        CardPtr::new_from_id(3, FaceKey::A),
        CardPtr::new_from_id(4, FaceKey::A),
        CardPtr::new_from_id(5, FaceKey::A),
    ];
    let demon_cards = vec![
        CardPtr::new_from_id(33, FaceKey::A),
        CardPtr::new_from_id(34, FaceKey::A),
        CardPtr::new_from_id(35, FaceKey::A),
        CardPtr::new_from_id(36, FaceKey::A),
    ];

    let num_warrior_perms = 5 * 4 * 3 * 2;
    let num_demon_perms = 4 * 3 * 2;

    let model = try_read_model_for_matchup((Class::Warrior, Class::Demon)).unwrap();
    for warrior_i in 0..num_warrior_perms {
        let warrior_perm = number_to_ordering(warrior_i, 5);

        for demon_i in 0..num_demon_perms {
            let demon_perm = number_to_ordering(demon_i, 4);
            let pile: Pile = Pile::from_iter(
                vec![
                    demon_cards[demon_perm[0]],
                    demon_cards[demon_perm[1]],
                    demon_cards[demon_perm[2]],
                    demon_cards[demon_perm[3]],
                    warrior_cards[warrior_perm[0]],
                    warrior_cards[warrior_perm[1]],
                    warrior_cards[warrior_perm[2]],
                    warrior_cards[warrior_perm[3]],
                    warrior_cards[warrior_perm[4]],
                ]
                .into_iter(),
            );

            print!("w: {}, d: {}, pile: {:?}", warrior_i, demon_i, pile);
            solve_pile(pile, model.clone());
        }
    }
}
