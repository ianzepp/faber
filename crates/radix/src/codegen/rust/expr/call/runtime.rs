//! Runtime module bridge helpers for built-in norma calls.

use crate::hir::{DefId, LibraryProvider, LibraryRegistry};

pub(super) fn norma_runtime_module_path(receiver_def_id: DefId, libraries: &LibraryRegistry) -> Option<&'static str> {
    let binding = libraries.bindings.get(&receiver_def_id)?;
    if binding.identity.provider != LibraryProvider::BuiltinNorma {
        return None;
    }

    match binding
        .identity
        .module_path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["json"] => Some("norma::json"),
        ["toml"] => Some("norma::toml"),
        ["hal", "consolum"] => Some("norma::hal::consolum"),
        ["hal", "http"] => Some("norma::hal::http"),
        _ => None,
    }
}

pub(super) fn norma_runtime_method_name(method_name: &str) -> String {
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
