//! User-facing Faber project and package orchestration.
//!
//! This crate owns the `faber` package-tool surface: project layout discovery,
//! source package loading, explain-command rendering, and the thin integration
//! points that hand validated source to the compiler library in `radix`.
//! Compiler feature work belongs in `radix`; this crate keeps CLI/package policy
//! close to user workflows.
//!
//! The public modules intentionally mirror user-facing capabilities: `package`
//! for project compilation and `explain` for language reference lookup. Private
//! helpers stay crate-local so lower-level compiler APIs do not inherit package
//! tool assumptions.

mod explain_render;
pub(crate) mod library;

pub mod explain;
pub mod package;

#[cfg(test)]
mod explain_test;
