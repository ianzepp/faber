use crate::kernel::{Frame, HostEcho, Router, SyscallInfo};
use crate::manifest::{CapabilityManifest, RegisteredProvider};

/// Faber-owned host kernel for macOS route proofs.
///
/// This is the boundary future Wasm imports and server transports should call
/// into. Keeping it small and owned by the host crate avoids committing to a
/// daemon lifecycle or shared crate before the first syscall model is proven.
pub struct HostKernel {
    router: Router,
    providers: Vec<RegisteredProvider>,
}

impl HostKernel {
    pub fn new() -> Self {
        let mut router = Router::new();
        router.register(HostEcho);

        Self {
            router,
            providers: Vec::new(),
        }
    }

    pub fn route(&self, request: &Frame) -> Frame {
        self.router.route(request)
    }

    pub fn manifest(&self) -> CapabilityManifest {
        CapabilityManifest::from_parts(self.syscalls(), self.providers.clone())
    }

    pub fn syscalls(&self) -> Vec<SyscallInfo> {
        self.router.syscalls()
    }
}

impl Default for HostKernel {
    fn default() -> Self {
        Self::new()
    }
}
