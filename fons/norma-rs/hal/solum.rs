//! solum.rs - File System Implementation
//!
//! Native Rust implementation of the HAL solum (filesystem) interface.
//! Uses std::fs for sync, tokio::fs for async.
//!
//! Verb conjugation encodes sync/async:
//!   - Imperative (-a, -e, -i): synchronous
//!   - Future indicative (-et, -ebit): asynchronous (returns impl Future)

use std::fs;
use std::io::Write;
use std::path::Path;

/// Full file status returned by describe/describet
#[derive(Debug, Clone)]
pub struct SolumStatus {
    pub modus: u32,           // permission bits (e.g., 0o755)
    pub nexus: u64,           // hard link count
    pub possessor: u32,       // owner uid
    pub grex: u32,            // group gid
    pub magnitudo: u64,       // size in bytes
    pub modificatum: u64,     // mtime (ms since epoch)
    pub est_directorii: bool,
    pub est_vinculum: bool,   // is symlink
}

// =============================================================================
// READING - Text
// =============================================================================
// Verb: lege/leget from "legere" (to read, gather)

/// Read entire file as text (sync)
pub fn lege(via: &str) -> String {
    fs::read_to_string(via).expect("failed to read file")
}

/// Read entire file as text (async)
pub async fn leget(via: &str) -> String {
    tokio::fs::read_to_string(via)
        .await
        .expect("failed to read file")
}

// =============================================================================
// READING - Bytes
// =============================================================================
// Verb: hauri/hauriet from "haurire" (to draw up, draw water)

/// Draw entire file as bytes (sync)
pub fn hauri(via: &str) -> Vec<u8> {
    fs::read(via).expect("failed to read file")
}

/// Draw entire file as bytes (async)
pub async fn hauriet(via: &str) -> Vec<u8> {
    tokio::fs::read(via).await.expect("failed to read file")
}

// =============================================================================
// READING - Lines
// =============================================================================
// Verb: carpe/carpiet from "carpere" (to pluck, pick, harvest)

/// Pluck lines from file (sync)
pub fn carpe(via: &str) -> Vec<String> {
    let content = fs::read_to_string(via).expect("failed to read file");
    content.lines().map(|s| s.to_string()).collect()
}

/// Pluck lines from file (async)
pub async fn carpiet(via: &str) -> Vec<String> {
    let content = tokio::fs::read_to_string(via)
        .await
        .expect("failed to read file");
    content.lines().map(|s| s.to_string()).collect()
}

// =============================================================================
// WRITING - Text
// =============================================================================
// Verb: scribe/scribet from "scribere" (to write)

/// Write text to file, overwrites existing (sync)
pub fn scribe(via: &str, data: &str) {
    fs::write(via, data).expect("failed to write file");
}

/// Write text to file, overwrites existing (async)
pub async fn scribet(via: &str, data: &str) {
    tokio::fs::write(via, data)
        .await
        .expect("failed to write file");
}

// =============================================================================
// WRITING - Bytes
// =============================================================================
// Verb: funde/fundet from "fundere" (to pour, pour out)

/// Pour bytes to file, overwrites existing (sync)
pub fn funde(via: &str, data: &[u8]) {
    fs::write(via, data).expect("failed to write file");
}

/// Pour bytes to file, overwrites existing (async)
pub async fn fundet(via: &str, data: &[u8]) {
    tokio::fs::write(via, data)
        .await
        .expect("failed to write file");
}

// =============================================================================
// WRITING - Append
// =============================================================================
// Verb: appone/apponet from "apponere" (to place near, add to)

/// Append text to file (sync)
pub fn appone(via: &str, data: &str) {
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(via)
        .expect("failed to open file for append");
    file.write_all(data.as_bytes())
        .expect("failed to append to file");
}

/// Append text to file (async)
pub async fn apponet(via: &str, data: &str) {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(via)
        .await
        .expect("failed to open file for append");
    file.write_all(data.as_bytes())
        .await
        .expect("failed to append to file");
}

// =============================================================================
// FILE INFO - Existence
// =============================================================================
// Verb: exstat/exstabit from "exstare" (to stand out, exist)

/// Check if path exists (sync)
pub fn exstat(via: &str) -> bool {
    Path::new(via).exists()
}

/// Check if path exists (async)
pub async fn exstabit(via: &str) -> bool {
    tokio::fs::try_exists(via).await.unwrap_or(false)
}

// =============================================================================
// FILE INFO - Details
// =============================================================================
// Verb: describe/describet from "describere" (to describe, delineate)

/// Get file details (sync)
pub fn describe(via: &str) -> SolumStatus {
    use std::os::unix::fs::MetadataExt;

    let meta = fs::symlink_metadata(via).expect("failed to get file metadata");
    SolumStatus {
        modus: (meta.mode() & 0o7777) as u32,
        nexus: meta.nlink(),
        possessor: meta.uid(),
        grex: meta.gid(),
        magnitudo: meta.size(),
        modificatum: meta.mtime() as u64 * 1000,
        est_directorii: meta.is_dir(),
        est_vinculum: meta.is_symlink(),
    }
}

/// Get file details (async)
pub async fn describet(via: &str) -> SolumStatus {
    use std::os::unix::fs::MetadataExt;

    let meta = tokio::fs::symlink_metadata(via)
        .await
        .expect("failed to get file metadata");
    SolumStatus {
        modus: (meta.mode() & 0o7777) as u32,
        nexus: meta.nlink(),
        possessor: meta.uid(),
        grex: meta.gid(),
        magnitudo: meta.size(),
        modificatum: meta.mtime() as u64 * 1000,
        est_directorii: meta.is_dir(),
        est_vinculum: meta.is_symlink(),
    }
}

// =============================================================================
// FILE INFO - Symlinks
// =============================================================================
// Verb: sequere/sequetur from "sequi" (to follow)

/// Follow symlink to get target path (sync)
pub fn sequere(via: &str) -> String {
    fs::read_link(via)
        .expect("failed to read symlink")
        .to_string_lossy()
        .into_owned()
}

/// Follow symlink to get target path (async)
pub async fn sequetur(via: &str) -> String {
    tokio::fs::read_link(via)
        .await
        .expect("failed to read symlink")
        .to_string_lossy()
        .into_owned()
}

// =============================================================================
// FILE OPERATIONS - Delete
// =============================================================================
// Verb: dele/delet from "delere" (to destroy, delete)

/// Delete file (sync)
pub fn dele(via: &str) {
    fs::remove_file(via).expect("failed to delete file");
}

/// Delete file (async)
pub async fn delet(via: &str) {
    tokio::fs::remove_file(via)
        .await
        .expect("failed to delete file");
}

// =============================================================================
// FILE OPERATIONS - Copy
// =============================================================================
// Verb: exscribe/exscribet from "exscribere" (to copy out, transcribe)

/// Copy file (sync)
pub fn exscribe(fons: &str, destinatio: &str) {
    fs::copy(fons, destinatio).expect("failed to copy file");
}

/// Copy file (async)
pub async fn exscribet(fons: &str, destinatio: &str) {
    tokio::fs::copy(fons, destinatio)
        .await
        .expect("failed to copy file");
}

// =============================================================================
// FILE OPERATIONS - Rename/Move
// =============================================================================
// Verb: renomina/renominabit from "renominare" (to rename)

/// Rename or move file (sync)
pub fn renomina(fons: &str, destinatio: &str) {
    fs::rename(fons, destinatio).expect("failed to rename file");
}

/// Rename or move file (async)
pub async fn renominabit(fons: &str, destinatio: &str) {
    tokio::fs::rename(fons, destinatio)
        .await
        .expect("failed to rename file");
}

// =============================================================================
// FILE OPERATIONS - Touch
// =============================================================================
// Verb: tange/tanget from "tangere" (to touch)

/// Touch file - create or update mtime (sync)
pub fn tange(via: &str) {
    use std::fs::OpenOptions;

    if Path::new(via).exists() {
        // Update mtime by opening and syncing
        let file = OpenOptions::new()
            .write(true)
            .open(via)
            .expect("failed to open file for touch");
        file.sync_all().expect("failed to sync file");
    } else {
        // Create empty file
        fs::write(via, "").expect("failed to create file");
    }
}

/// Touch file - create or update mtime (async)
pub async fn tanget(via: &str) {
    if tokio::fs::try_exists(via).await.unwrap_or(false) {
        // Update mtime by opening and syncing
        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(via)
            .await
            .expect("failed to open file for touch");
        file.sync_all().await.expect("failed to sync file");
    } else {
        // Create empty file
        tokio::fs::write(via, "")
            .await
            .expect("failed to create file");
    }
}

// =============================================================================
// DIRECTORY OPERATIONS - Create
// =============================================================================
// Verb: crea/creabit from "creare" (to create, bring forth)

/// Create directory, recursive (sync)
pub fn crea(via: &str) {
    fs::create_dir_all(via).expect("failed to create directory");
}

/// Create directory, recursive (async)
pub async fn creabit(via: &str) {
    tokio::fs::create_dir_all(via)
        .await
        .expect("failed to create directory");
}

// =============================================================================
// DIRECTORY OPERATIONS - List
// =============================================================================
// Verb: enumera/enumerabit from "enumerare" (to count out, enumerate)

/// List directory contents (sync)
pub fn enumera(via: &str) -> Vec<String> {
    fs::read_dir(via)
        .expect("failed to read directory")
        .map(|entry| {
            entry
                .expect("failed to read directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// List directory contents (async)
pub async fn enumerabit(via: &str) -> Vec<String> {
    let mut entries = tokio::fs::read_dir(via)
        .await
        .expect("failed to read directory");
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await.expect("failed to read entry") {
        result.push(entry.file_name().to_string_lossy().into_owned());
    }
    result
}

// =============================================================================
// DIRECTORY OPERATIONS - Prune/Remove
// =============================================================================
// Verb: amputa/amputabit from "amputare" (to cut off, prune)

/// Prune directory tree, recursive (sync)
pub fn amputa(via: &str) {
    let _ = fs::remove_dir_all(via); // force: true equivalent
}

/// Prune directory tree, recursive (async)
pub async fn amputabit(via: &str) {
    let _ = tokio::fs::remove_dir_all(via).await; // force: true equivalent
}

// =============================================================================
// PATH UTILITIES
// =============================================================================
// Pure functions on path strings, not filesystem I/O. Sync only.

/// Join path segments
pub fn iunge(partes: &[&str]) -> String {
    let mut path = std::path::PathBuf::new();
    for part in partes {
        path.push(part);
    }
    path.to_string_lossy().into_owned()
}

/// Get directory part of path
pub fn directorium(via: &str) -> String {
    Path::new(via)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Get filename part of path
pub fn basis(via: &str) -> String {
    Path::new(via)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Get file extension (includes dot)
pub fn extensio(via: &str) -> String {
    Path::new(via)
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default()
}

/// Resolve to absolute path
pub fn absolve(via: &str) -> String {
    std::fs::canonicalize(via)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| {
            // If file doesn't exist, do best-effort resolution
            let path = Path::new(via);
            if path.is_absolute() {
                via.to_string()
            } else {
                let cwd = std::env::current_dir().unwrap_or_default();
                cwd.join(via).to_string_lossy().into_owned()
            }
        })
}

/// Get user's home directory
pub fn domus() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}

/// Get system temp directory
pub fn temporarium() -> String {
    std::env::temp_dir().to_string_lossy().into_owned()
}
