# Nova

Rewrite Classic in Rust. This project modernizes GamesmanClassic's infrastructure and style by taking advantage of Rust's Zero Cost abstractions, allowing for a centralized software architechture and greater flexibility, modularity, safety, and overall cleanliness. 

### Core objectives

* Make contributing to the Gamesman project safer, more fun, and more accessible.
* Provide access to the whole project's functionality in runtime by allowing the choice of which solver to use for which game through menus (in the case of user interfaces) and through communicative queries (in the case of networked calls). 
* Provide a set of interfaces (TUI, GUI, CLI, networked APIs, etc.) which offer the same functionality (solving, DB queries, availability queries, DB analysis, etc.).
* Integrate and automate our development process, shifting from local setups to remote deployments and builds to minimize the barrier of entry to contributions and to bolster our underlying physical infrastructure (and hence our storage and processing capabilities).
* Continuously harbor standardized best practices in Nova's codebase.

### Roadmap

##### Closed phase

It will be best to keep development highly involved and sequential for these tasks. 

- [x] Write abstract infrastructure and scaffolding through the use of Rust's powerful module, trait, type, and macro system.
- [x] Write a basic command line interface for observing general behavior.
- [ ] Write module tests for abstract solvers, databases, and analyzers.
- [x] Develop an abstract database interface system through traits.
- [x] Integrate database interfaces with solvers obscurely (users cannot choose which DB to use).
- [ ] Write database/solver integration tests. 
- [ ] Implement or port games from Classic.

##### Open phase

It will be possible to start taking care of these tasks concurrently, as most infrastructure will have taken shape by then, and it will be about conforming to it (as opposed to defining it).

- [ ] Develop new interfaces.
- [ ] Include or port more games.
- [ ] Write more solvers, databases, and analyzers as research permits.
- [ ] Shift to an off-premises deployment.
- [ ] Integrate and deploy a backend to GamesmanUni.
- [ ] Re-design abstraction infrastructure as needed.

### Why?

While I like C, it is minimal and unergonomic. Sometimes you have to realize that your system is not embedded, and that you can afford yourself certain luxuries. And sometimes you have to realize that even if your system were embedded, you should still use Rust.

Rust makes it easier to:

- Structure the project (modules, traits, etc.)
- Write safe and performant code (no garbage collector AND no mallocs)
- Literally grow a new arm to do a specialized task (metaprogramming with macros)
- Harbor and enforce good practices (`cargo doc`, `cargo fmt`, different types of docstrings and comments, etc.)
- Manage dependencies (Cargo, `config.toml` files, library crates, etc.)
- Build your project (and by a LOT -- `cargo build` vs. makefiles)
- Debug your code (the rust compiler gives incredibly helpful error messages)
- Write code in a functional paradigm (e.g. `for <item> in <iterator>` vs `for (int i = 0; i < LIMIT; i++) { typeptr[i] ..........`)
- Test your code (built in doc tests, unit tests, and integration tests, `cargo test`, etc.)
- Get and distribute packages (`cargo add`, [crates.io](https://crates.io/), standard manifests, etc.)
- Ask for help and get new features (incredibly active community working on growing the language)
- ...and more!

-- Max Fierro, Sun Apr 9, 2023
