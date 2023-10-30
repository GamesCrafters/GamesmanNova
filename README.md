# Nova

This is a re-focused spinoff of Dr. Dan Garcia's [GamesmanClassic](https://github.com/GamesCrafters/GamesmanClassic), a system for strongly solving (_takes a deep breath_) complete-information turn-based deterministic abstract strategy games, such as Tic-Tac-Toe, Connect4, and Chess (if it weren't so darn big). In particular, the purpose of Nova is to take learnings and ambitions from GamesmanClassic and provide a software system with an architecture that can accommodate them through meaningful, equally performant, and safe abstractions. 

## Installation

Eventually Nova will be published as a binary crate to [crates.io](crates.io), which will reduce the following to `cargo install nova` (or something like that). 

For now though, do the following to set things up:

1. [Install the Rust compiler and toolchain](https://www.rust-lang.org/tools/install).
  
3. Clone this repository to your preferred `location`.

```
$ git clone https://github.com/GamesCrafters/GamesmanNova.git location
```

3. Go to your installation (`cd location`), and install the executable:

```
$ cargo install --path nova
```

> NOTE: The `nova` argument to `--path` here refers to the inner directory within the project, which is the Nova binary crate. It is not at the top level because this project also contains [procedural macros](https://doc.rust-lang.org/beta/reference/procedural-macros.html) as other crates of their own, meaning that the overall repository is a [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

This will add the `nova` executable to your list of cargo binaries. If you add this list of binaries to your `$PATH`, you will be able to call `nova` just like any other command.

## Development

### Important Fronts

The current big objectives for this project are:

* Supporting _N_-player non-transferrable utility games (e.g., tractable multiplayer variants of Chinese Checkers).
* Supporting transferrable utility coalitional games (i.e., games where you can re-distribute winnings at the end among those who helped you win).
* Creating a memory-efficient and thread-safe implementation of GamesmanClassic's database engine.
* Designing a database file serialization protocol to make GamesmanClassic's databases compatible with Nova.
* Building a wonderful terminal user interface and expanding the current command-line interface.
* Implementing and wrapping a distributed MPI controller for solves on the Savio cluster.
* Setting up some light CI to allow for contributions, both internal and external.

### Subprojects

Smaller modules currently being built:

* An implementation of the [Crossteaser](https://www.jaapsch.net/puzzles/crosstsr.htm) puzzle.

### Contributing

This project was created in affiliation with GamesCrafters, a computational game theory applied research group at UC Berkeley. If you would like to contribute as someone outside the GamesCrafters org, feel free to fork the repository and open a PR. If you are part of the org, create a branch anytime and code away (but do let others know, so we can help or thank you). Some light CI is set up to ensure some loose measure of quality on the main branch (namely compilation, testing, and formatting checks).

-- Max Fierro, maxfierro@berkeley.edu
