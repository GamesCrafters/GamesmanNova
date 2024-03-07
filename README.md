# GamesmanNova

Nova is a research project primarily built for efficiently searching extensive-form games, such as Tic-Tac-Toe, Connect4, and Chess (if it weren't so darn big). In particular, the prupose of this project is to take learnings and ambitions from [Prof. Dan Garcia's](https://people.eecs.berkeley.edu/~ddgarcia/) multi-decade project [GamesmanClassic](https://github.com/GamesCrafters/GamesmanClassic) and provide a software system that is more general, performant, and ergonomic.

## Installation

Before doing anything, you will want to install [the Rust compiler and toolchain](https://www.rust-lang.org/tools/install). GamesmanNova is on [crates.io](https://crates.io/crates/gamesman-nova), so to get the `nova` executable, you can then run:

```
$ cargo install gamesman-nova
```

Otherwise, if you would like to build Nova from source, you can also:

1. Clone this repository to your preferred `location`.

```
$ git clone https://github.com/GamesCrafters/GamesmanNova.git location
```

2. Go to your installation (`cd location`), and install the executable:

```
$ cargo install --path .
```

This will add the `nova` executable to your list of cargo binaries.

## Development

### Important Fronts

The current big objectives for this project are:

* Supporting transferrable utility coalitional games (i.e., games where you can re-distribute winnings at the end among those who helped you win).
* Creating a memory-efficient and thread-safe implementation of GamesmanClassic's database engine.
* Designing a database file serialization protocol to make GamesmanClassic's databases compatible with Nova.
* Building a wonderful terminal user interface and expanding the current command-line interface.
* Implementing and wrapping a distributed MPI controller for distribution.

### Subprojects

Smaller modules currently being built:

* An implementation of the [Crossteaser](https://www.jaapsch.net/puzzles/crosstsr.htm) puzzle.

### Contributing

This project was created in affiliation with GamesCrafters, a computational game theory applied research group at UC Berkeley. If you would like to contribute as someone outside the GamesCrafters org, feel free to fork the repository and open a PR. If you are part of the org, create a branch anytime and code away (but do let others know, so we can help or thank you). Some light CI is set up to ensure some loose measure of quality on the main branch (namely compilation, testing, and formatting checks).

-- Cheers, [Max Fierro](https://www.maxfierro.me/)
