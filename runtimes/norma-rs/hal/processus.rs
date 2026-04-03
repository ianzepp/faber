//! processus.rs - Process Device Implementation
//!
//! Native Rust implementation of the HAL process interface.
//! Uses std::process for sync, tokio::process for async.
//!
//! Spawn semantics encoded via different verbs:
//!   - genera: spawn attached, caller manages lifecycle
//!   - dimitte: spawn detached, dismiss to run independently

use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Spawned subprocess handle for attached processes
pub struct Subprocessus {
    pub pid: u32,
    child: std::process::Child,
}

impl Subprocessus {
    /// Wait for process to exit and return exit code (sync)
    pub fn expiravit(&mut self) -> i32 {
        self.child
            .wait()
            .map(|status| status.code().unwrap_or(-1))
            .unwrap_or(-1)
    }

    /// Wait for process to exit and return exit code (async)
    pub async fn expirabit(&mut self) -> i32 {
        // Convert to tokio child for async wait
        // Note: this is a limitation - ideally we'd store tokio::process::Child
        // but that requires different spawn path. For now, wrap sync.
        self.expiravit()
    }
}

/// Async spawned subprocess handle
pub struct SubprocessusAsync {
    pub pid: u32,
    child: tokio::process::Child,
}

impl SubprocessusAsync {
    /// Wait for process to exit and return exit code (async)
    pub async fn expirabit(&mut self) -> i32 {
        self.child
            .wait()
            .await
            .map(|status| status.code().unwrap_or(-1))
            .unwrap_or(-1)
    }
}

// =============================================================================
// SPAWN - Attached
// =============================================================================
// Verb: genera from "generare" (to generate, beget)

/// Spawn attached process - caller can wait for exit via handle.expiravit()
pub fn genera(
    argumenta: &[&str],
    directorium: Option<&str>,
    ambitus: Option<&HashMap<String, String>>,
) -> Subprocessus {
    if argumenta.is_empty() {
        panic!("genera: argumenta cannot be empty");
    }

    let mut cmd = Command::new(argumenta[0]);
    if argumenta.len() > 1 {
        cmd.args(&argumenta[1..]);
    }

    if let Some(dir) = directorium {
        cmd.current_dir(dir);
    }

    if let Some(env) = ambitus {
        cmd.envs(env.iter());
    }

    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let child = cmd.spawn().expect("failed to spawn process");
    let pid = child.id();

    Subprocessus { pid, child }
}

/// Spawn attached process (async) - returns async handle
pub async fn generabit(
    argumenta: &[&str],
    directorium: Option<&str>,
    ambitus: Option<&HashMap<String, String>>,
) -> SubprocessusAsync {
    if argumenta.is_empty() {
        panic!("generabit: argumenta cannot be empty");
    }

    let mut cmd = tokio::process::Command::new(argumenta[0]);
    if argumenta.len() > 1 {
        cmd.args(&argumenta[1..]);
    }

    if let Some(dir) = directorium {
        cmd.current_dir(dir);
    }

    if let Some(env) = ambitus {
        cmd.envs(env.iter());
    }

    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let child = cmd.spawn().expect("failed to spawn process");
    let pid = child.id().unwrap_or(0);

    SubprocessusAsync { pid, child }
}

// =============================================================================
// SPAWN - Detached
// =============================================================================
// Verb: dimitte from "dimittere" (to send away, dismiss)

/// Dismiss process to run independently - returns PID
pub fn dimitte(
    argumenta: &[&str],
    directorium: Option<&str>,
    ambitus: Option<&HashMap<String, String>>,
) -> u32 {
    if argumenta.is_empty() {
        panic!("dimitte: argumenta cannot be empty");
    }

    let mut cmd = Command::new(argumenta[0]);
    if argumenta.len() > 1 {
        cmd.args(&argumenta[1..]);
    }

    if let Some(dir) = directorium {
        cmd.current_dir(dir);
    }

    if let Some(env) = ambitus {
        cmd.envs(env.iter());
    }

    // Detached: ignore all stdio
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    let child = cmd.spawn().expect("failed to spawn detached process");
    child.id()
}

// =============================================================================
// SHELL EXECUTION
// =============================================================================
// Verb: exsequi/exsequetur from "exsequi" (to execute, accomplish)

/// Execute shell command, block until complete, return stdout (sync)
pub fn exsequi(imperium: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(imperium)
        .output()
        .expect("failed to execute shell command");

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Execute shell command, return stdout when complete (async)
pub async fn exsequetur(imperium: &str) -> String {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(imperium)
        .output()
        .await
        .expect("failed to execute shell command");

    String::from_utf8_lossy(&output.stdout).into_owned()
}

// =============================================================================
// ENVIRONMENT - Read
// =============================================================================
// Verb: lege from "legere" (to read)

/// Read environment variable (returns None if not set)
pub fn lege(nomen: &str) -> Option<String> {
    std::env::var(nomen).ok()
}

// =============================================================================
// ENVIRONMENT - Write
// =============================================================================
// Verb: scribe from "scribere" (to write)

/// Write environment variable
pub fn scribe(nomen: &str, valor: &str) {
    std::env::set_var(nomen, valor);
}

// =============================================================================
// PROCESS INFO - Working Directory
// =============================================================================

/// Get current working directory (where the process dwells)
pub fn sedes() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".to_string())
}

// =============================================================================
// PROCESS INFO - Change Directory
// =============================================================================
// Verb: muta from "mutare" (to change)

/// Change current working directory
pub fn muta(via: &str) {
    std::env::set_current_dir(via).expect("failed to change directory");
}

// =============================================================================
// PROCESS INFO - Identity
// =============================================================================

/// Get process ID
pub fn identitas() -> u32 {
    std::process::id()
}

// =============================================================================
// PROCESS INFO - Arguments
// =============================================================================

/// Get command line arguments (excludes runtime and script path)
pub fn argumenta() -> Vec<String> {
    std::env::args().skip(1).collect()
}

// =============================================================================
// EXIT
// =============================================================================
// Verb: exi from "exire" (to exit, depart)

/// Exit process with code (never returns)
pub fn exi(code: i32) -> ! {
    std::process::exit(code)
}
