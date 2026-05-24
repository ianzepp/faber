use std::collections::BTreeMap;

use crate::kernel::{Frame, HostError, Syscall, SyscallInfo};

/// Prefix router for host syscalls.
///
/// The current router is deliberately synchronous and in-process. That is enough
/// to prove the host route contract while keeping daemon transport, cancellation
/// tokens, and streaming backpressure out of the first slice.
pub struct Router {
    routes: BTreeMap<&'static str, Box<dyn Syscall>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, syscall: impl Syscall + 'static) {
        self.routes.insert(syscall.prefix(), Box::new(syscall));
    }

    pub fn route(&self, request: &Frame) -> Frame {
        let Some(syscall) = self.routes.get(request.prefix()) else {
            let error = HostError::no_route(format!("no handler for call: {}", request.call));
            return request.error(&error);
        };

        match syscall.dispatch(request) {
            Ok(response) => response,
            Err(error) => request.error(&error),
        }
    }

    pub fn syscalls(&self) -> Vec<SyscallInfo> {
        self.routes
            .values()
            .flat_map(|syscall| syscall.syscalls().iter().cloned())
            .collect()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
