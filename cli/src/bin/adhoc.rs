#![allow(dead_code)]
use cli::*;
use handy_core::game::*;
use handy_core::solver::model::Model;
use handy_core::utils::string_to_pile;
use serde;
use serde_jsonlines::json_lines;
use std::collections::HashSet;
use std::fmt::Debug;
// use csv::Writer;
// use inc_stats;
//
#[derive(serde::Serialize, Debug)]
struct PredictionRow {
    model1: f32,
    model2: f32,
}

fn compute_error(model: &Model, piles: &Vec<Pile>, depth: usize) -> f32 {
    let total_loss: f32 = piles
        .iter()
        .map(|pile| {
            let prediction = model.score_pile(pile);
            let diff = prediction - depth as f32;
            diff * diff
        })
        .sum();

    total_loss / piles.len() as f32
}

fn do_compare(hero: Class, enemy: Class) {
    println!("{:?} {:?}", hero, enemy);
    let matchup = (hero, enemy);

    let filename = format!("{:?}.{:?}.yaml", hero, enemy);
    let path_v1 = format!("data/models/{}", filename);
    let model_v1 = try_read_model_from_full_path(&path_v1).expect("Couldn't load model");

    let path_v2 = format!("data/models/old_models/2024-02-05-models/{}", filename);
    let model_v2 = try_read_model_from_full_path(&path_v2).expect("Couldn't load model");
    // println!("loaded models");

    let examples_path = training_path_for_matchup(matchup);
    // println!("Start Reading Examples");
    let all_examples = json_lines(examples_path)
        .unwrap()
        .collect::<Result<Vec<DepthModeTrainingExample>, _>>()
        .unwrap();
    // println!("Done Reading {} Examples", all_examples.len());

    let mut examples_by_depth: Vec<Vec<Pile>> = Vec::new();
    for example in all_examples {
        let StateEval::Win(depth) = example.eval else {
            continue;
        };

        while depth >= examples_by_depth.len() {
            examples_by_depth.push(Vec::new());
        }

        examples_by_depth[depth].push(example.pile);
    }

    let mut etot1 = 0.0;
    let mut etot2 = 0.0;
    let mut denom = 0.0;
    for (depth, piles) in examples_by_depth.iter().enumerate() {
        denom += 1.0;
        if piles.len() == 0 {
            continue;
        }

        let e1 = compute_error(&model_v1, piles, depth);
        let e2 = compute_error(&model_v2, piles, depth);

        etot1 += e1;
        etot2 += e2;

        println!("{:<2} ({:<8}): {:.4} | {:.4}", depth, piles.len(), e1, e2);
    }

    println!("Avg cmp: {:.4} | {:.4}", etot1 / denom, etot2 / denom);
}

fn script_compare() {
    let heros = vec![
        // others
        Class::Cursed,
        // others
    ];
    let enemies = vec![
        // others
        Class::Spider,
        // others
    ];

    for hero in &heros {
        for enemy in &enemies {
            // if hero == Class::Assassin || enemy == Class::Wall {
            //     continue;
            // }

            do_compare(*hero, *enemy);
        }
    }
}

fn state_amount_test() {
    let pile = string_to_pile("26A25C24A27B57A58A56A55A59A");
    // let state = GameStateWithEventLog::new(pile);
    {
        let state = GameStateNoEventLog::new(pile.clone());
        let outcomes = resolve_top_card(&state);
        println!("Total outcomes: {}", outcomes.len());
        let unique_outcomes = outcomes
            .into_iter()
            .map(|s| s.pile)
            .collect::<HashSet<_>>()
            .len();
        println!("Unique outcomes: {unique_outcomes}");
    }


    {
        let state = GameStateWithPileTrackedEventLog::new(pile.clone());
        let outcomes = resolve_top_card_starting_with_prefix_dedupe_excess(&state, &Vec::new());
        println!("Total outcomes: {}", outcomes.len());
        let unique_outcomes = outcomes
            .into_iter()
            .map(|s| s.pile)
            .collect::<HashSet<_>>()
            .len();
        println!("Unique outcomes: {unique_outcomes}");
    }
}

fn main() {
    state_amount_test()
}
