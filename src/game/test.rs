//! # Game Test Utilities Module
//!
//! This module provides integration and unit testing utilities for the `game`
//! module.

use anyhow::Context;
use anyhow::Result;
use petgraph::dot::{Config, Dot};

use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::game::mock;
use crate::game::Bounded;
use crate::test::*;

/* IMPLEMENTATIONS */

impl mock::Session<'_> {
    /// Creates an SVG visualization of the game graph in the visuals directory
    /// under the development data directory at the project root.
    pub fn visualize(&self, module: &str) -> Result<()> {
        match test_setting()? {
            TestSetting::Correctness => return Ok(()),
            TestSetting::Development => (),
        }

        let subdir = PathBuf::from(module);
        let mut dir = get_directory(DevelopmentData::Visuals, subdir)?;
        let name = format!("{}.svg", self.name()).replace(' ', "-");

        dir.push(name);
        let file = File::create(dir)?;
        let mut dot = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(file)
            .spawn()
            .context("Failed to execute 'dot' command.")?;

        if let Some(mut stdin) = dot.stdin.take() {
            let graph = format!("{}", self);
            stdin.write_all(graph.as_bytes())?;
        }

        dot.wait()?;
        Ok(())
    }
}

impl Display for mock::Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.graph(),
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, n| {
                    let (_, node) = n;
                    let mut attrs = String::new();
                    match node {
                        mock::Node::Medial(turn) => {
                            attrs += &format!("label=P{turn} ");
                            attrs += "style=filled  ";
                            if self.start() == self.state(node).unwrap() {
                                attrs += "shape=doublecircle ";
                                attrs += "fillcolor=navajowhite3 ";
                            } else {
                                attrs += "shape=circle ";
                                attrs += "fillcolor=lightsteelblue ";
                            }
                        },
                        mock::Node::Terminal(util) => {
                            attrs += &format!("label=\"{:?}\" ", util);
                            attrs += "shape=plain ";
                        },
                    }
                    attrs
                }
            )
        )
    }
}
