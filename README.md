# Nova

This is a re-focused spinoff of Dr. Dan Garcia's [GamesmanClassic](https://github.com/GamesCrafters/GamesmanClassic), a system for strongly solving (_takes a deep breadth_) complete-information turn-based deterministic abstract strategy games, such as Tic-Tac-Toe, Connect4, and Chess (if it weren't so darn big). In particular, the purpose of Nova is to take learnings and ambitions from GamesmanClassic and provide a software system with an architecture that can accommodate them through meaningful, equally performant, and safe abstractions. 

## Important Fronts

Current big objectives for this project:

* Supporting _N_-player non-transferrable utility games (e.g. tractable multiplayer variants of Chinese Checkers).
* Supporting transferrable utility coalitional games (i.e. games where you can re-distribute winnings at the end among those who helped you win).
* Creating a memory-efficient and thread-safe implementation of GamesmanClassic's database engine.
* Designing a database file serialization protocol to make GamesmanClassic's databases compatible with Nova.
* Building a wonderful terminal user interface, and expanding the current command-line interface.
* Implementing and wrapping a distributed MPI controller for solves on the Savio cluster.
* Setting up some light CI to allow for contributions, both internal and external.

## Subprojects

Smaller modules currently being built:

* An implementation of the [Crossteaser](https://www.jaapsch.net/puzzles/crosstsr.htm) puzzle.

## Work

This project was created in affiliation with GamesCrafters, a computational game theory applied research group at UC Berkeley. CI is currently under development, so if you would like to contribute and are not part of GamesCrafters, feel free to reach out!

-- Max Fierro, maxfierro@berkeley.edu
