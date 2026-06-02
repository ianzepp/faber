use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct E2eResult {
    pub path: PathBuf,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug)]
pub(super) struct E2eFinding {
    pub path: PathBuf,
    pub reason: String,
}