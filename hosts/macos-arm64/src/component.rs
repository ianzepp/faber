//! Wasm Component Model boundary for the macOS host.
//!
//! TARGET: This module proves that a component import can be smaller than a
//! full frame ABI while still entering the host as a frame before routing.
//! The final Faber WIT world is intentionally not locked here; this is the
//! narrow bridge needed to validate the Epic 4 host/kernel shape.

use std::fs;
use std::path::Path;

use wasmtime::component::{Component, Linker, Val};
use wasmtime::{Config, Engine, Store};

use crate::syscall_import::route_capability_code;
pub use crate::syscall_import::{
    CAPABILITY_CALL_IMPORT, COMPONENT_CODE_HOST_ECHO, COMPONENT_CODE_PG_QUERY,
};
use crate::{Frame, HostKernel, Status};

pub type ComponentResult<T> = Result<T, ComponentHostError>;

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentCallOutput {
    pub component_status: u32,
    pub response: Frame,
}

#[derive(Debug)]
pub struct ComponentHostError {
    message: String,
}

impl ComponentHostError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ComponentHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ComponentHostError {}

impl From<wasmtime::Error> for ComponentHostError {
    fn from(error: wasmtime::Error) -> Self {
        Self::new(format!("{error:#}"))
    }
}

impl From<std::io::Error> for ComponentHostError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

struct ComponentState {
    kernel: HostKernel,
    last_response: Option<Frame>,
}

pub struct ComponentHost {
    engine: Engine,
}

impl ComponentHost {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);

        Self {
            engine: Engine::new(&config).expect("component model config should be valid"),
        }
    }

    pub fn call_export_from_file(
        &self,
        path: impl AsRef<Path>,
        export_name: &str,
        route_code: u32,
    ) -> ComponentResult<ComponentCallOutput> {
        let bytes = fs::read(path)?;
        self.call_export(&bytes, export_name, route_code)
    }

    pub fn call_export(
        &self,
        component_bytes: &[u8],
        export_name: &str,
        route_code: u32,
    ) -> ComponentResult<ComponentCallOutput> {
        let component = Component::new(&self.engine, component_bytes)?;
        let mut store = Store::new(
            &self.engine,
            ComponentState {
                kernel: HostKernel::new(),
                last_response: None,
            },
        );
        let mut linker: Linker<ComponentState> = Linker::new(&self.engine);

        linker.root().func_wrap(
            CAPABILITY_CALL_IMPORT,
            |mut store, (route_code,): (i32,)| {
                let response = route_capability_code(&store.data().kernel, route_code);
                let component_status = if response.status == Status::Error {
                    1
                } else {
                    0
                };
                store.data_mut().last_response = Some(response);
                Ok((component_status,))
            },
        )?;

        let instance = linker.instantiate(&mut store, &component)?;
        let func = instance.get_func(&mut store, export_name).ok_or_else(|| {
            ComponentHostError::new(format!("component export not found: {export_name}"))
        })?;

        let params = [Val::S32(route_code as i32)];
        let mut results = [Val::S32(-1)];
        func.call(&mut store, &params, &mut results)?;

        let component_status = match results.into_iter().next() {
            Some(Val::S32(value)) => value as u32,
            Some(other) => {
                return Err(ComponentHostError::new(format!(
                    "component export returned non-s32 value: {other:?}"
                )));
            }
            None => {
                return Err(ComponentHostError::new(
                    "component export returned no value",
                ))
            }
        };
        let response = store.data_mut().last_response.take().ok_or_else(|| {
            ComponentHostError::new(format!(
                "component export did not call {CAPABILITY_CALL_IMPORT}"
            ))
        })?;

        Ok(ComponentCallOutput {
            component_status,
            response,
        })
    }
}

impl Default for ComponentHost {
    fn default() -> Self {
        Self::new()
    }
}
