//! Core Wasm module boundary for generated Rust artifacts.
//!
//! TARGET: Epic 5's generated Rust helper imports `capability-call` as a core
//! Wasm function. This runner proves the host side of that ABI without replacing
//! the Component Model path from Epic 4.

use std::fs;
use std::path::Path;

use wasmtime::{Config, Engine, Linker, Module, Store, TypedFunc};

use crate::syscall_import::{route_capability_code, CAPABILITY_CALL_IMPORT};
use crate::{Frame, HostKernel, Status};

pub type WasmResult<T> = Result<T, WasmHostError>;

#[derive(Clone, Debug, PartialEq)]
pub struct WasmCallOutput {
    pub module_status: u32,
    pub response: Frame,
}

#[derive(Debug)]
pub struct WasmHostError {
    message: String,
}

impl WasmHostError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WasmHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for WasmHostError {}

impl From<wasmtime::Error> for WasmHostError {
    fn from(error: wasmtime::Error) -> Self {
        Self::new(format!("{error:#}"))
    }
}

impl From<std::io::Error> for WasmHostError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

struct WasmState {
    kernel: HostKernel,
    last_response: Option<Frame>,
}

pub struct WasmHost {
    engine: Engine,
}

impl WasmHost {
    pub fn new() -> Self {
        let config = Config::new();

        Self {
            engine: Engine::new(&config).expect("wasm config should be valid"),
        }
    }

    pub fn call_export_from_file(
        &self,
        path: impl AsRef<Path>,
        export_name: &str,
        route_code: u32,
    ) -> WasmResult<WasmCallOutput> {
        let bytes = fs::read(path)?;
        self.call_export(&bytes, export_name, route_code)
    }

    pub fn call_export(
        &self,
        module_bytes: &[u8],
        export_name: &str,
        route_code: u32,
    ) -> WasmResult<WasmCallOutput> {
        let module = Module::new(&self.engine, module_bytes)?;
        let mut store = Store::new(
            &self.engine,
            WasmState {
                kernel: HostKernel::new(),
                last_response: None,
            },
        );
        let mut linker: Linker<WasmState> = Linker::new(&self.engine);

        linker.func_wrap(
            "",
            CAPABILITY_CALL_IMPORT,
            |mut caller: wasmtime::Caller<'_, WasmState>, route_code: i32| -> i32 {
                let response = route_capability_code(&caller.data().kernel, route_code);
                let module_status = if response.status == Status::Error {
                    1
                } else {
                    0
                };
                caller.data_mut().last_response = Some(response);
                module_status
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;
        let func: TypedFunc<i32, i32> =
            instance
                .get_typed_func(&mut store, export_name)
                .map_err(|error| {
                    WasmHostError::new(format!("module export not found: {export_name}: {error:#}"))
                })?;
        let module_status = func.call(&mut store, route_code as i32)? as u32;
        let response = store.data_mut().last_response.take().ok_or_else(|| {
            WasmHostError::new(format!(
                "module export did not call {CAPABILITY_CALL_IMPORT}"
            ))
        })?;

        Ok(WasmCallOutput {
            module_status,
            response,
        })
    }
}

impl Default for WasmHost {
    fn default() -> Self {
        Self::new()
    }
}
