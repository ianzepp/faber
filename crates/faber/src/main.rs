//! User-facing Faber project and package tool (`faber` binary).
//!
//! Clap shapes live in [`cli`]; handlers live in [`commands`]. Package-aware
//! compilation routes through [`package`]; single-file compiler inspection
//! delegates to `radix::tool`.

mod cli;
mod commands;
mod library;
mod package;

#[cfg(test)]
#[path = "cli_test.rs"]
mod cli_test;

fn main() {
    commands::run();
}
