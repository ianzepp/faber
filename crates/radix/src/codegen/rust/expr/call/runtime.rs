//! Runtime module bridge helpers for library-backed calls.

use crate::hir::{DefId, LibraryRegistry};

pub(super) fn library_runtime_module_path(receiver_def_id: DefId, libraries: &LibraryRegistry) -> Option<&str> {
    let binding = libraries.bindings.get(&receiver_def_id)?;
    binding.rust_runtime_module.as_deref()
}

pub(super) fn library_runtime_method_name(method_name: &str) -> String {
    let mut lowered = String::with_capacity(method_name.len());
    for (i, ch) in method_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                lowered.push('_');
            }
            lowered.push(ch.to_ascii_lowercase());
        } else {
            lowered.push(ch);
        }
    }
    lowered
}
