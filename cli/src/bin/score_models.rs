use cli::state_eval_to_score;
use cli::training_path_for_matchup;
use cli::try_read_model_for_matchup;
use cli::DepthModeTrainingExample;
use handy_core::solver::Matchup;
use handy_core::solver::BADDIES;
use handy_core::solver::HEROS;
// use handy_core::game::Class;
use serde_jsonlines::json_lines;

fn main() {
    let mut all_results: Vec<(Matchup, f32)> = Vec::new();
    for hero in HEROS {
        for baddie in BADDIES {
            let matchup = (hero, baddie);
            let Ok(struct_model) = try_read_model_for_matchup(matchup) else {
                println!("Could not load model for: {matchup:?}");
                all_results.push((matchup, 1000000.0));
                continue;
            };
            let examples_path = training_path_for_matchup(matchup);
            let all_examples_base = json_lines(examples_path);

            let Ok(all_examples_base) = all_examples_base else {
                continue;
            };
            let all_examples_base =
                all_examples_base.collect::<Result<Vec<DepthModeTrainingExample>, _>>();

            let Ok(all_examples_base) = all_examples_base else {
                continue;
            };

            let total_loss: f32 = all_examples_base
                .iter()
                .map(|ex| {
                    let model_score = struct_model.score_pile(&ex.pile);
                    let expected = state_eval_to_score(ex.eval) as f32;
                    let diff = model_score - expected;
                    return (diff * diff) / all_examples_base.len() as f32;
                })
                .sum();

            println!("{matchup:?} {total_loss}");

            all_results.push((matchup, total_loss));
        }
    }

    all_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (matchup, score) in all_results {
        println!("{: <40} {score}", format!("{:?}", matchup));
    }
}
