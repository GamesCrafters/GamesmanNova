# GamesmanNova

Nova is a UC Berkeley research project primarily built for efficiently searching extensive-form games, such as Tic-Tac-Toe, Connect4, and Chess (if it weren't so darn big). In particular, the prupose of this project is to take learnings and ambitions from [Prof. Dan Garcia's](https://people.eecs.berkeley.edu/~ddgarcia/) multi-decade project [GamesmanClassic](https://github.com/GamesCrafters/GamesmanClassic) and provide a software system that is more general, performant, and ergonomic.

## Installation

Before doing anything, you will want to install [the Rust compiler and toolchain](https://www.rust-lang.org/tools/install). GamesmanNova is on [crates.io](https://crates.io/crates/gamesman-nova), so to get the `nova` executable, you can then run:

```
cargo install gamesman-nova
```

Otherwise, if you would like to build Nova from source, you can also:

1. Clone this repository to your preferred `location`.

```
git clone https://github.com/GamesCrafters/GamesmanNova.git location
```

2. Go to your installation (`cd location`), and install the executable:

```
cargo install --path .
```

This will add the `nova` executable to your list of cargo binaries.

## Usage

Once you have the `nova` executable in a directory that is part of your `PATH`, you can run:

```
nova --help
```

This will display a list of sub-commands and their descriptions. Nova uses [`clap`](https://docs.rs/clap/latest/clap/) for Unix-like command-line argument parsing.

## Development

As a research project, the primary users of Nova will be people who intend to build on it as a platform. In the long-term, we wish to make this evident by splitting Nova into multiple modular crates that better express this, in a way that makes the project more accessible.

For now, we make our best attempt at providing a good experience when building from this repository via rustdocs and a reasonable architecture. The project adheres to semantic versioning per Rust's standards, and is currently unstable.

-- Cheers, [Max Fierro](https://www.maxfierro.me/)
