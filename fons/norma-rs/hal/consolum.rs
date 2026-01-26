//! consolum.rs - Console Device Implementation
//!
//! Native Rust implementation of the HAL console interface.
//! Uses std::io for sync, tokio::io for async.
//!
//! Verb conjugation encodes sync/async:
//!   - Imperative (-a, -e, -i): synchronous
//!   - Future indicative (-et, -ebit): asynchronous (returns impl Future)
//!
//! Aligns with language keywords: scribe (info), mone (warn), vide (debug)

use std::io::{self, BufRead, Read, Write};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

// =============================================================================
// STDIN - Bytes
// =============================================================================
// Verb: hauri/hauriet from "haurire" (to draw up)

/// Draw bytes from stdin (sync)
pub fn hauri(magnitudo: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; magnitudo];
    let bytes_read = io::stdin()
        .lock()
        .read(&mut buffer)
        .expect("failed to read from stdin");
    buffer.truncate(bytes_read);
    buffer
}

/// Draw bytes from stdin (async)
pub async fn hauriet(magnitudo: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; magnitudo];
    let bytes_read = tokio::io::stdin()
        .read(&mut buffer)
        .await
        .expect("failed to read from stdin");
    buffer.truncate(bytes_read);
    buffer
}

// =============================================================================
// STDIN - Text
// =============================================================================
// Verb: lege/leget from "legere" (to read)

/// Read line from stdin (sync, blocks until newline)
pub fn lege() -> String {
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .expect("failed to read line from stdin");
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

/// Read line from stdin (async)
pub async fn leget() -> String {
    let mut line = String::new();
    tokio::io::BufReader::new(tokio::io::stdin())
        .read_line(&mut line)
        .await
        .expect("failed to read line from stdin");
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

// =============================================================================
// STDOUT - Bytes
// =============================================================================
// Verb: funde/fundet from "fundere" (to pour)

/// Pour bytes to stdout (sync)
pub fn funde(data: &[u8]) {
    io::stdout()
        .lock()
        .write_all(data)
        .expect("failed to write to stdout");
    io::stdout().lock().flush().expect("failed to flush stdout");
}

/// Pour bytes to stdout (async)
pub async fn fundet(data: &[u8]) {
    let mut stdout = tokio::io::stdout();
    stdout
        .write_all(data)
        .await
        .expect("failed to write to stdout");
    stdout.flush().await.expect("failed to flush stdout");
}

// =============================================================================
// STDOUT - Text with Newline
// =============================================================================
// Verb: scribe/scribet from "scribere" (to write)

/// Write line to stdout with newline (sync)
pub fn scribe(msg: &str) {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", msg).expect("failed to write to stdout");
    stdout.flush().expect("failed to flush stdout");
}

/// Write line to stdout with newline (async)
pub async fn scribet(msg: &str) {
    let mut stdout = tokio::io::stdout();
    stdout
        .write_all(msg.as_bytes())
        .await
        .expect("failed to write to stdout");
    stdout
        .write_all(b"\n")
        .await
        .expect("failed to write newline");
    stdout.flush().await.expect("failed to flush stdout");
}

// =============================================================================
// STDOUT - Text without Newline
// =============================================================================
// Verb: dic/dicet from "dicere" (to say)

/// Say text to stdout without newline (sync)
pub fn dic(msg: &str) {
    let mut stdout = io::stdout().lock();
    write!(stdout, "{}", msg).expect("failed to write to stdout");
    stdout.flush().expect("failed to flush stdout");
}

/// Say text to stdout without newline (async)
pub async fn dicet(msg: &str) {
    let mut stdout = tokio::io::stdout();
    stdout
        .write_all(msg.as_bytes())
        .await
        .expect("failed to write to stdout");
    stdout.flush().await.expect("failed to flush stdout");
}

// =============================================================================
// STDERR - Warning/Error Output
// =============================================================================
// Verb: mone/monet from "monere" (to warn)

/// Warn line to stderr with newline (sync)
pub fn mone(msg: &str) {
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "{}", msg).expect("failed to write to stderr");
    stderr.flush().expect("failed to flush stderr");
}

/// Warn line to stderr with newline (async)
pub async fn monet(msg: &str) {
    let mut stderr = tokio::io::stderr();
    stderr
        .write_all(msg.as_bytes())
        .await
        .expect("failed to write to stderr");
    stderr
        .write_all(b"\n")
        .await
        .expect("failed to write newline");
    stderr.flush().await.expect("failed to flush stderr");
}

// =============================================================================
// DEBUG Output
// =============================================================================
// Verb: vide/videbit from "videre" (to see)

/// Debug line with newline (sync)
pub fn vide(msg: &str) {
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "{}", msg).expect("failed to write to stderr");
    stderr.flush().expect("failed to flush stderr");
}

/// Debug line with newline (async)
pub async fn videbit(msg: &str) {
    let mut stderr = tokio::io::stderr();
    stderr
        .write_all(msg.as_bytes())
        .await
        .expect("failed to write to stderr");
    stderr
        .write_all(b"\n")
        .await
        .expect("failed to write newline");
    stderr.flush().await.expect("failed to flush stderr");
}

// =============================================================================
// TTY Detection
// =============================================================================

/// Is stdin connected to a terminal?
pub fn est_terminale() -> bool {
    atty_stdin()
}

/// Is stdout connected to a terminal?
pub fn est_terminale_output() -> bool {
    atty_stdout()
}

// Internal TTY detection using libc
#[cfg(unix)]
fn atty_stdin() -> bool {
    unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
}

#[cfg(unix)]
fn atty_stdout() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}

#[cfg(not(unix))]
fn atty_stdin() -> bool {
    false // Conservative default on non-unix
}

#[cfg(not(unix))]
fn atty_stdout() -> bool {
    false // Conservative default on non-unix
}
