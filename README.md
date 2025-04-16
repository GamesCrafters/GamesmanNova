# GamesmanNova

Nova is a research project for generating datasets of sequential games with finite state representations, which includes games Tic-Tac-Toe and Connect4. As a system, it is the first step of data pipelines that provide analyses of these games' state spaces.

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

As a research project, the primary users of Nova will be people who intend to build on it as a platform. 

For now, we make our best attempt at providing a good experience when building from this repository via rustdocs and a reasonable architecture. The project adheres to semantic versioning per Rust's standards, and will remain unstable for the foreseeable future.

-- Cheers, [Max Fierro](https://www.maxfierro.me/)
