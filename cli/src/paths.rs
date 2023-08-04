use handy_core::solver::model::{Matchup, Model};
use lazy_static::lazy_static;
use serde_yaml;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

lazy_static! {
    pub static ref DATA_DIR: PathBuf = Path::new("./data").into();
    pub static ref TRAINING_DATA_DIR: PathBuf = DATA_DIR.join("training_data");
    pub static ref MODELS_DIR: PathBuf = DATA_DIR.join("models");
}

pub fn matchup_to_str(matchup: Matchup) -> String {
    format!("{:?}.{:?}", matchup.0, matchup.1)
}

fn matchup_with_ext(matchup: Matchup, ext: &str) -> String {
    format!("{}.{ext}", matchup_to_str(matchup))
}

pub fn matchup_to_yaml_str(matchup: Matchup) -> String {
    matchup_with_ext(matchup, "yaml")
}

pub fn matchup_to_jsonl_str(matchup: Matchup) -> String {
    matchup_with_ext(matchup, "jsonl")
}

pub fn model_path_for_matchup(matchup: Matchup) -> String {
    MODELS_DIR
        .join(matchup_to_yaml_str(matchup))
        .to_str()
        .unwrap()
        .to_owned()
}

pub fn swap_model_path_for_matchup(matchup: Matchup) -> String {
    MODELS_DIR
        .join(format!("swap.{}", matchup_to_yaml_str(matchup)))
        .to_str()
        .unwrap()
        .to_owned()
}

pub fn training_path_for_matchup(matchup: Matchup) -> String {
    TRAINING_DATA_DIR
        .join(matchup_to_jsonl_str(matchup))
        .to_str()
        .unwrap()
        .to_owned()
}

fn try_read_model_from_full_path(full_path: &str) -> Result<Model, String> {
    let file = File::open(full_path).map_err(|err| format!("{err}"))?;
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).map_err(|err| format!("{err}"))
}

pub fn try_read_model_from_custom_name(name: &str) -> Result<Model, String> {
    let full_path = MODELS_DIR.join(name);
    try_read_model_from_full_path(full_path.to_str().unwrap())
}

pub fn try_read_model_for_matchup(matchup: Matchup) -> Result<Model, String> {
    let full_path = model_path_for_matchup(matchup);
    try_read_model_from_full_path(&full_path)
}

pub fn write_model_for_matchup_with_custom_suffix(model: &Model, matchup: Matchup, suffix: &str) {
    let full_path = match suffix {
        "" => model_path_for_matchup(matchup),
        _ => format!("{}.{suffix}", model_path_for_matchup(matchup)),
    };
    let swap_path = swap_model_path_for_matchup(matchup);
    if let Err(err) = fs::rename(&full_path, &swap_path) {
        println!("Failed to write swap file: {err}");
    }

    let file = File::create(full_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_yaml::to_writer(&mut writer, model).unwrap();
    writer.flush().unwrap();
}

pub fn write_model_for_matchup(model: &Model, matchup: Matchup) {
    write_model_for_matchup_with_custom_suffix(model, matchup, "")
}
