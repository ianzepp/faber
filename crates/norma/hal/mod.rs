//! hal - Hardware Abstraction Layer
//!
//! Platform-specific implementations for system interaction.

pub mod arca;
pub mod consolum;
pub mod http;
pub mod processus;
pub mod solum;

#[cfg(test)]
mod http_test;
