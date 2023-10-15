use crate::game::*;
use crate::utils::*;

pub fn print_steps_between_piles(parent: &Pile, child: &Pile, log: &dyn Fn(&str) -> ()) {
    let possible_paths = resolve_top_card(&GameStateWithEventLog::new(parent.clone()));
    for path in &possible_paths {
        if &path.pile == child {
            for e in &path.events {
                let line = format!("\t{}", format_event_for_cli(e));
                log(&line);
            }
            return;
        }
    }
    log(&format!("Could not find path from {parent:?} to {child:?}"));
}
