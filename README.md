# Handy Brawl Solver
A solver and web UI written in Rust for the very clever [Handy Brawl](https://boardgamegeek.com/boardgame/362692/handy-brawl) print & play card game by Igor Zuber.

## Usage
See v0 of the web UI [here](https://jpricey.github.io/handy-solver/).

See sections below for using the CLI

## A-Star Solver

The A-Star solver is built to solve standard game states of 9 cards with a single hero type vs a single enemy type.
This solver does not guarantee optimal solutions, but if a state can be solved it's pretty good at finding a solution, and will attempt to find a faster solution once found.
States can be input starting with an exact pile string, or generated randomly for a given matchup. See below for invocation examples.

### Pile Input Mode
An exact start state can be supplied with the `--pile` argument, followed by a pile state string.
A pile is a list of cards. A card is a card number, optionally followed by a card face (`A`, `B`, `C`, `D`). If no card face is given, `A` is defaulted to. Lowercase faces are accepted.
Pile strings are read left to right, with the leftmost card representing the top card.

### Class Input Mode
Use the `--classes` argument followed by a hero and monster class.
This will create a random starting state for the requested combat.
Optionally supply a `--seed` string to seed the random starting state.

Available hero classes:
- warrior
- huntress
- pyro
- cursed
- beastmaster

Available monster classes:
- ogre
- vampire
- spider
- demon
- flora

### Examples
Examples of how to start the program through cargo:
```
# Class input mode. Solve a random warrior vs ogre game
cargo run -p cli --bin a_star -- --classes warrior ogre

# Class input mode with seed. Solve a random warrior vs ogre game with a deterministic start state
cargo run -p cli --bin a_star -- --classes warrior ogre --seed someSeedString

# Pile input mode. Remember to quote the pile string if spaces are used as delimeters. Cards without a face default to face A.
cargo run -p cli --bin a_star -- --pile "1 2 3C 4C 5D 6b7c8d9B"

# Hint: use the cargo --release flag to have it run faster :)
cargo run -p cli --bin a_star --release -- --classes warrior ogre
```

## Exhaustive Search

The Exhaustive Search solver guarantees a full search to some depth, and can be configured to stop not just when the player wins/loses, but for other critera.
The arguments to the exhaustive search solver are:
```
--pile <a pile string, as described above>
--rev (Optional flag to reverse the order of the input file string. This can be useful for looking at puzzles from the Handy Brawl puzzle thread, which are read right to left)
--turns <number> (Optional number of turns. If specified, will only look for paths that find a victory condition after exactly this many turns. If not specified, will stop at the first turn number that solutions are found at)

Exactly one of these victory conditions must be specified:
--win (Ends when the player defeats all enemies)
--exhaust <card id> (Ends when the given card is exhausted)
--survive-until-top <card id> (Ends when the given card reaches the top of the pile without ever getting exhausted)
```

Example inputs for the [Handy Brawl Puzzles](https://boardgamegeek.com/thread/2971866/puzzles):
```
1:
cargo run -p cli --bin exhaustive_search --release -- --pile "5C > 6D > 2B > 9B" --survive-until-top 5 --rev

2:
cargo run -p cli --bin exhaustive_search --release -- --pile "15A > 18A > 5A > 2B > 4A" --exhaust 15 --turns 3 --rev

3:
cargo run -p cli --bin exhaustive_search --release -- --pile "8D > 13D > 14A > 6A > 11B > 12D" --win --turns 5 --rev

4:
cargo run -p cli --bin exhaustive_search --release -- --pile "8D > 13D > 14A > 6A > 11B > 12D" --win --turns 4 --rev

5:
cargo run -p cli --bin exhaustive_search --release -- --pile "12A > 9A > 21D > 14A > 10B > 11D" --win --turns 3 --rev

6:
cargo run -p cli --bin exhaustive_search --release -- --pile "9D > 11B > 10A > 8D > 20D" --win --turns 3 --rev
```

## Notes / Future Work
Ideas for future work:
- Use a neural network for modelling state instead of a feature based heuristic function
- More sophisticated search algorithm
- Support for multithreading
- Improvements to the web GUI
- Puzzle generator
