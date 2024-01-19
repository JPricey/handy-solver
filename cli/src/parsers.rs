use handy_core::game::*;
use handy_core::solver::*;

pub fn parse_dot_separated_matchup(s: &str) -> Result<Matchup, String> {
    let pos = s
        .find('.')
        .ok_or_else(|| format!("invalid KEY=value: no `.` found in `{}`", s))?;
    let hero: Class = s[..pos].parse().map_err(|err| format!("{}", err))?;
    let enemy: Class = s[pos + 1..].parse().map_err(|err| format!("{}", err))?;

    if !is_hero_class(hero) {
        return Err("First class must be a hero".into());
    }

    if is_hero_class(enemy) {
        return Err("Second class must be an enemy".into());
    }

    Ok((hero, enemy))
}
