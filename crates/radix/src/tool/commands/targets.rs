//! Target capability reporting for `radix targets` and `faber targets`.

/// Capability row returned by [`target_capabilities`].
pub struct TargetCapabilities {
    /// Whether `check` is supported for this target.
    pub check: bool,
    /// Whether file-level `build` emission is supported.
    pub build: bool,
    /// Whether `faber run` can execute this target.
    pub run: bool,
    /// Whether `faber` package workflows are supported.
    pub package: bool,
    /// Human-readable capability note for `faber targets` / `radix targets`.
    pub note: &'static str,
}

/// Print backend capability rows for terminal discovery.
pub fn cmd_targets() {
    for target in [
        crate::codegen::Target::Rust,
        crate::codegen::Target::Go,
        crate::codegen::Target::WasmText,
        crate::codegen::Target::LlvmText,
        crate::codegen::Target::TypeScript,
        crate::codegen::Target::Faber,
    ] {
        let capabilities = target_capabilities(target);
        println!(
            "{} check={} build={} run={} package={} note={}",
            target_name(target),
            yes_no(capabilities.check),
            yes_no(capabilities.build),
            yes_no(capabilities.run),
            yes_no(capabilities.package),
            capabilities.note
        );
    }
}

pub(crate) fn target_extension(target: crate::codegen::Target) -> &'static str {
    match target {
        crate::codegen::Target::Rust => "rs",
        crate::codegen::Target::Faber => "fab",
        crate::codegen::Target::TypeScript => "ts",
        crate::codegen::Target::Go => "go",
        crate::codegen::Target::WasmText => "wat",
        crate::codegen::Target::LlvmText => "ll",
    }
}

pub(crate) fn target_name(target: crate::codegen::Target) -> &'static str {
    match target {
        crate::codegen::Target::Rust => "rust",
        crate::codegen::Target::Faber => "faber",
        crate::codegen::Target::TypeScript => "ts",
        crate::codegen::Target::Go => "go",
        crate::codegen::Target::WasmText => "wasm-text",
        crate::codegen::Target::LlvmText => "llvm-text",
    }
}

/// Return current command-surface support for a backend target.
///
/// These rows describe CLI capability, not necessarily backend maturity. For
/// example, a target can support file emission while package compilation still
/// remains unavailable from `radix`.
pub fn target_capabilities(target: crate::codegen::Target) -> TargetCapabilities {
    match target {
        crate::codegen::Target::Rust => TargetCapabilities {
            check: true,
            build: true,
            run: true,
            package: true,
            note: "primary backend; full package build + run via `faber`",
        },
        crate::codegen::Target::Go => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        crate::codegen::Target::TypeScript => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        crate::codegen::Target::Faber => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "canonical pretty-print target; package compilation not yet supported",
        },
        crate::codegen::Target::WasmText => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "experimental MIR-backed WAT probe; not a binary WASM backend yet",
        },
        crate::codegen::Target::LlvmText => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "experimental MIR-backed LLVM text probe; not native codegen yet",
        },
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}