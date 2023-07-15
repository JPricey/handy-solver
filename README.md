# Handy Brawl Solver
A solver written in Rust for the very clever [Handy Brawl](https://boardgamegeek.com/boardgame/362692/handy-brawl) print & play card game by Igor Zuber.

## Usage
The game state solver lives in the `solver` module.
Input is either a class pairing (with optional randomization seed), or an exact ordering of cards.

The solver outputs incrementally better (faster) solutions until it has either exhausted the entire search space at the current depth, or is OOM killed.

### Class Input Mode
Use the `--classes` argument followed by a hero and monster class.
This will create a random starting state for the requested combat.
Optionally supply a `--seed` string to seed the random starting state.

Available hero classes:
- paladin
- huntress
- pyro
- werewolf
- beastmaster

Available monster classes:
- ogre
- vampire
- spider
- demon
- verdancy

### Pile Input Mode
An exact start state can be supplied with the `--pile` argument, followed by a pile state string.
A pile is a list of cards. A card is a card number, optionally followed by a card face (`A`, `B`, `C`, `D`). If no card face is given, `A` is defaulted to. Lowercase faces are accepted.

## Examples
Examples of how to start the program through cargo:
```
# Class input mode. Solve a random paladin vs ogre game
cargo run -p solver -- --classes paladin ogre

# Class input mode with seed. Solve a random paladin vs ogre game with a deterministic start state
cargo run -p solver -- --classes paladin ogre --seed someSeedString

# Pile input mode. Remember to quote the pile string if spaces are used as delimeters. Cards without a face default to face A.
cargo run -p solver -- --pile "1 2 3C 4C 5D 6b7c8d9B"

# Hint: use the cargo --release flag to have it run faster :)
cargo run -p solver --release -- --classes paladin ogre

# Run in CLI mode using the cli module. Pile inputs are the same as solver mode. See notes on CLI below.
cargo run -p cli -- --classes paladin ogre
```

## Cli Mode
Run the `cli` module to play handy brawl in interactive CLI mode. This is mostly for debugging, and you'll probably have more fun playing with cards instead :). In this mode, the program enumerates all options possible after the activation of an entire card. This can result in hundreds of options for the more complicated characters, and may be very difficult to make sense of.

## Notes / Future Work
The solver is currently implemented as basically an A* search, which branches after each full card activation. The evaluation function of a pile state is the sum of card face "values", which are hardcoded in `card_defs.rs`. Healthy / more powerful hero cards are more posively valuable, and healthy / more powerful monster cards are more negatively valuable. These values were picked somewhat roughly by hand, so some heros / enemies have more accurate evaluations than others. Once the solver detects a solution at depth N, all game states at depth >=N-1 are pruned to reduce the active search space.

Ideas for future work:
- More accurate game state evaluation by adding new features
- Automatic training of the evaluation model instead of hardcoding hand picked weights
- More sophisticated search algorithm
- Support for multithreading
- Web Assembly based GUI
- Puzzle generator
