//! # Command Line and Terminal Module
//! 
//! This module defines a standard means of interaction with GamesmanNova 
//! through a UNIX-like CLI facilitated by [clap](https://docs.rs/clap/latest/clap/)
//! in addition to an interactive and modern TUI made possible by 
//! [tui-rs](https://github.com/fdehau/tui-rs).
//! 
//! #### Authorship
//! 
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/// Defines GamesmanNova's command line interface.
pub mod cli;

/// Defines GamesmanNova's terminal user interface.
pub mod tui;
