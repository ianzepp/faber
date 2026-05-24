use serde::{Deserialize, Serialize};

use crate::kernel::SyscallInfo;

/// Machine-readable surface exported by this host.
///
/// Strict compilation will eventually consume a richer version of this manifest.
/// The first slice records only built-in syscalls and registered providers so
/// host capability discovery has a concrete artifact before policy exists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub host: String,
    pub manifest_version: u32,
    pub builtins: Vec<SyscallManifest>,
    pub providers: Vec<RegisteredProvider>,
}

impl CapabilityManifest {
    pub fn from_parts(syscalls: Vec<SyscallInfo>, providers: Vec<RegisteredProvider>) -> Self {
        Self {
            host: "macos-arm64".into(),
            manifest_version: 1,
            builtins: syscalls.into_iter().map(SyscallManifest::from).collect(),
            providers,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyscallManifest {
    pub name: String,
    pub prefix: String,
    pub summary: String,
}

impl From<SyscallInfo> for SyscallManifest {
    fn from(info: SyscallInfo) -> Self {
        Self {
            name: info.name.into(),
            prefix: info.prefix.into(),
            summary: info.summary.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredProvider {
    pub name: String,
    pub owner: String,
    pub prefix: Option<String>,
    pub calls: Vec<String>,
}
