//! In-process Wasm instantiation probing for the exempla e2e harness.

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmInstantiationBucket {
    NoRuntime,
    MissingImport,
    InstantiationTrap,
    InstantiateValid,
}

impl Display for WasmInstantiationBucket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRuntime => write!(f, "no-runtime"),
            Self::MissingImport => write!(f, "missing-import"),
            Self::InstantiationTrap => write!(f, "instantiation-trap"),
            Self::InstantiateValid => write!(f, "instantiate-valid"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmImportSite {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmInstantiationProbe {
    pub bucket: WasmInstantiationBucket,
    pub reason: String,
    pub imports: Vec<WasmImportSite>,
}

pub fn parse_wat_import_sites(wat: &str) -> Vec<WasmImportSite> {
    let mut imports = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = wat[cursor..].find("(import ") {
        let absolute = cursor + start;
        let line_end = wat[absolute..]
            .find('\n')
            .map(|offset| absolute + offset)
            .unwrap_or(wat.len());
        let line = &wat[absolute..line_end];
        if let Some((module, name)) = parse_import_line(line) {
            imports.push(WasmImportSite { module, name });
        }
        cursor = line_end;
    }
    imports
}

fn parse_import_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<_> = line.split('"').collect();
    if parts.len() < 4 {
        return None;
    }
    Some((parts[1].to_owned(), parts[3].to_owned()))
}

pub fn probe_wat_instantiation(wat: &str) -> WasmInstantiationProbe {
    let imports = parse_wat_import_sites(wat);
    let engine = wasmtime::Engine::default();
    let module = match wasmtime::Module::new(&engine, wat) {
        Ok(module) => module,
        Err(err) => {
            return WasmInstantiationProbe {
                bucket: WasmInstantiationBucket::InstantiationTrap,
                reason: format!("Wasm module compile failed: {err}"),
                imports,
            };
        }
    };

    let mut store = wasmtime::Store::new(&engine, ());
    let linker = wasmtime::Linker::new(&engine);
    match linker.instantiate(&mut store, &module) {
        Ok(_) => WasmInstantiationProbe {
            bucket: WasmInstantiationBucket::InstantiateValid,
            reason: if imports.is_empty() {
                "instantiated with no imports".to_owned()
            } else {
                format!("instantiated without host imports ({})", summarize_imports(&imports))
            },
            imports,
        },
        Err(err) => {
            let message = err.to_string();
            let bucket = classify_instantiation_error(&message, !imports.is_empty());
            WasmInstantiationProbe {
                bucket,
                reason: format!(
                    "instantiation {:?}: {}{}",
                    bucket,
                    message,
                    if imports.is_empty() {
                        String::new()
                    } else {
                        format!("; unresolved imports: {}", summarize_imports(&imports))
                    }
                ),
                imports,
            }
        }
    }
}

fn classify_instantiation_error(message: &str, has_imports: bool) -> WasmInstantiationBucket {
    let lower = message.to_ascii_lowercase();
    if lower.contains("unknown import")
        || lower.contains("failed to find")
        || lower.contains("incompatible import")
        || lower.contains("no func export")
        || (has_imports && lower.contains("import"))
    {
        WasmInstantiationBucket::MissingImport
    } else if lower.contains("trap") {
        WasmInstantiationBucket::InstantiationTrap
    } else if has_imports {
        WasmInstantiationBucket::MissingImport
    } else {
        WasmInstantiationBucket::InstantiationTrap
    }
}

fn summarize_imports(imports: &[WasmImportSite]) -> String {
    imports
        .iter()
        .map(|import| format!("{}::{}", import.module, import.name))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "wasm_host_test.rs"]
mod tests;