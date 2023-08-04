# Handy Brawl Solver
A solver and web UI written in Rust for the very clever [Handy Brawl](https://boardgamegeek.com/boardgame/362692/handy-brawl) print & play card game by Igor Zuber.

## Usage
See v0 of the web UI [here](https://jpricey.github.io/handy-solver/).

The game state solver CLI lives in the `cli` module, `a_star` binary. See examples below for how to invoke it with either a random seed, or known state.

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

### Pile Input Mode
An exact start state can be supplied with the `--pile` argument, followed by a pile state string.
A pile is a list of cards. A card is a card number, optionally followed by a card face (`A`, `B`, `C`, `D`). If no card face is given, `A` is defaulted to. Lowercase faces are accepted.

## Examples
Examples of how to start the program through cargo:
```
# Class input mode. Solve a random paladin vs ogre game
cargo run -p cli --bin a_star -- --classes paladin ogre

# Class input mode with seed. Solve a random paladin vs ogre game with a deterministic start state
cargo run -p cli --bin a_star -- --classes paladin ogre --seed someSeedString

# Pile input mode. Remember to quote the pile string if spaces are used as delimeters. Cards without a face default to face A.
cargo run -p cli --bin a_star -- --pile "1 2 3C 4C 5D 6b7c8d9B"

# Hint: use the cargo --release flag to have it run faster :)
cargo run -p cli --bin a_star --release -- --classes paladin ogre
```

## Notes / Future Work
Ideas for future work:
- Use a neural network for modelling state instead of a feature based heuristic function
- More sophisticated search algorithm
- Support for multithreading
- Improvements to the web GUI
- Puzzle generator
