//! In-process Wasm instantiation and entry-run probing for the exempla e2e harness.

use std::fmt::{self, Display, Formatter};
use wasmtime::{ExternType, Linker, Module, Store, Val};

pub const WASM_ENTRY_EXPORT: &str = "incipit";

#[derive(Debug, Default)]
pub struct FaberStubHostState {
    pub diag_events: Vec<String>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmRunBucket {
    NoEntryExport,
    EntryTrap,
    Runnable,
}

impl Display for WasmRunBucket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEntryExport => write!(f, "no-entry-export"),
            Self::EntryTrap => write!(f, "entry-trap"),
            Self::Runnable => write!(f, "runnable"),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmRunProbe {
    pub bucket: WasmRunBucket,
    pub reason: String,
    pub diag_events: Vec<String>,
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

pub fn probe_wat_instantiation_with_stub_host(wat: &str) -> WasmInstantiationProbe {
    let imports = parse_wat_import_sites(wat);
    match compile_module(wat) {
        Ok((engine, module)) => {
            let mut store = Store::new(&engine, FaberStubHostState::default());
            let mut linker = Linker::new(&engine);
            if let Err(reason) = link_faber_stub_host(&mut linker, &mut store, &module) {
                return WasmInstantiationProbe { bucket: WasmInstantiationBucket::InstantiationTrap, reason, imports };
            }
            match linker.instantiate(&mut store, &module) {
                Ok(_) => WasmInstantiationProbe {
                    bucket: WasmInstantiationBucket::InstantiateValid,
                    reason: if imports.is_empty() {
                        "stub host instantiated module with no imports".to_owned()
                    } else {
                        format!("stub host instantiated module ({} imports satisfied)", imports.len())
                    },
                    imports,
                },
                Err(err) => WasmInstantiationProbe {
                    bucket: WasmInstantiationBucket::InstantiationTrap,
                    reason: format!("stub host instantiation failed: {err}"),
                    imports,
                },
            }
        }
        Err(reason) => WasmInstantiationProbe { bucket: WasmInstantiationBucket::InstantiationTrap, reason, imports },
    }
}

pub fn run_wat_entry_with_stub_host(wat: &str) -> WasmRunProbe {
    match compile_module(wat) {
        Ok((engine, module)) => {
            let mut store = Store::new(&engine, FaberStubHostState::default());
            let mut linker = Linker::new(&engine);
            if let Err(reason) = link_faber_stub_host(&mut linker, &mut store, &module) {
                return WasmRunProbe { bucket: WasmRunBucket::EntryTrap, reason, diag_events: Vec::new() };
            }
            let instance = match linker.instantiate(&mut store, &module) {
                Ok(instance) => instance,
                Err(err) => {
                    return WasmRunProbe {
                        bucket: WasmRunBucket::EntryTrap,
                        reason: format!("stub host instantiation failed: {err}"),
                        diag_events: Vec::new(),
                    };
                }
            };
            let Some(func) = instance.get_func(&mut store, WASM_ENTRY_EXPORT) else {
                return WasmRunProbe {
                    bucket: WasmRunBucket::NoEntryExport,
                    reason: format!("no `{WASM_ENTRY_EXPORT}` export in module"),
                    diag_events: store.data().diag_events.clone(),
                };
            };
            match func.call(&mut store, &[], &mut []) {
                Ok(()) => WasmRunProbe {
                    bucket: WasmRunBucket::Runnable,
                    reason: format!("invoked export `{WASM_ENTRY_EXPORT}`"),
                    diag_events: store.data().diag_events.clone(),
                },
                Err(err) => WasmRunProbe {
                    bucket: WasmRunBucket::EntryTrap,
                    reason: format!("export `{WASM_ENTRY_EXPORT}` trapped: {err}"),
                    diag_events: store.data().diag_events.clone(),
                },
            }
        }
        Err(reason) => WasmRunProbe { bucket: WasmRunBucket::EntryTrap, reason, diag_events: Vec::new() },
    }
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

    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
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
                    "{}{}",
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

fn compile_module(wat: &str) -> Result<(wasmtime::Engine, Module), String> {
    let engine = wasmtime::Engine::default();
    let module = Module::new(&engine, wat).map_err(|err| format!("Wasm module compile failed: {err}"))?;
    Ok((engine, module))
}

fn link_faber_stub_host(
    linker: &mut Linker<FaberStubHostState>,
    store: &mut Store<FaberStubHostState>,
    module: &Module,
) -> Result<(), String> {
    for import in module.imports() {
        if import.module() != "faber_diag" {
            continue;
        }
        let ExternType::Func(func_ty) = import.ty() else {
            continue;
        };
        let import_name = import.name().to_string();
        let event_prefix = import_name.clone();
        linker
            .func_new(import.module(), import.name(), func_ty, move |mut caller, params, _results| {
                let formatted = params
                    .iter()
                    .map(|val| format_val(*val))
                    .collect::<Vec<_>>()
                    .join(",");
                caller
                    .data_mut()
                    .diag_events
                    .push(format!("{event_prefix}:{formatted}"));
                Ok(())
            })
            .map_err(|err| format!("stub host diag import `{import_name}` failed: {err}"))?;
    }
    linker
        .define_unknown_imports_as_default_values(store, module)
        .map_err(|err| format!("stub host linking failed: {err}"))
}

fn format_val(val: Val) -> String {
    match val {
        Val::I32(bits) => bits.to_string(),
        Val::I64(bits) => bits.to_string(),
        Val::F32(bits) => bits.to_string(),
        Val::F64(bits) => bits.to_string(),
        _ => "?".to_owned(),
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
