//! Library import resolution for package compilation.
//!
//! `faber` treats built-in libraries as source-level interfaces under
//! `stdlib/`, not as hard-coded compiler magic. The resolver translates an
//! import specifier such as `norma:hal/solum` into the `.fab` interface that
//! package loading can parse, typecheck, and wire into generated output.
//!
//! The current provider surface is intentionally small. `norma` is resolved
//! from the repository stdlib today, while the data model already has enough
//! shape for future package-backed providers without baking Rust runtime
//! metadata into import resolution.
//!
//! INVARIANTS
//! ==========
//! - A resolver returns `Ok(None)` when an import is not provider-shaped; local
//!   package import resolution owns those paths.
//! - Once a known provider is selected, malformed or missing modules are
//!   diagnostics with known-module hints.
//! - Resolved modules always point at `.fab` interface files, keeping stdlib
//!   APIs on the normal parse/typecheck path.
//! - Old built-in slash forms such as `norma/json` are rejected instead of
//!   silently reinterpreted.

use std::path::{Path, PathBuf};

/// Origin class for a resolved library module.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum LibraryProviderKind {
    /// Built into this Faber distribution and backed by `stdlib/`.
    Builtin,

    /// Reserved for package-managed libraries resolved outside the built-in stdlib.
    PackageDependency,
}

/// Resolved import target for a source-level library module.
///
/// The path points at the Faber interface source, not at generated Rust or a
/// runtime artifact. Downstream package loading relies on that distinction so
/// imported APIs go through the same parser and typechecker as local sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedLibraryModule {
    /// Provider package name, such as `norma`.
    pub package: String,

    /// Module path inside the provider, split on `/`.
    pub module_path: Vec<String>,

    /// Source interface file consumed by package loading.
    pub interface_path: PathBuf,

    /// Provider class used by diagnostics and future resolver routing.
    pub provider: LibraryProviderKind,
}

impl ResolvedLibraryModule {
    pub(crate) fn new(
        package: impl Into<String>,
        module_path: Vec<String>,
        interface_path: impl Into<PathBuf>,
        provider: LibraryProviderKind,
    ) -> Self {
        Self {
            package: package.into(),
            module_path,
            interface_path: interface_path.into(),
            provider,
        }
    }

    /// Return the terminal module segment expected to match named imports.
    pub(crate) fn module_name(&self) -> Option<&str> {
        self.module_path.last().map(String::as_str)
    }
}

/// Errors that mean a specifier selected a library provider but not a module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LibraryResolveError {
    /// The old built-in slash form was used for a Norma module.
    OldBuiltinNormaSpecifier {
        /// Original import specifier from source.
        specifier: String,

        /// Provider-qualified replacement.
        replacement: String,
    },

    /// The specifier is provider-shaped but malformed.
    InvalidProviderSpecifier {
        /// Original import specifier from source.
        specifier: String,

        /// Targeted reason for the invalid shape.
        reason: String,
    },

    /// The provider separator selected a provider that is not implemented.
    UnknownProvider {
        /// Original import specifier from source.
        specifier: String,

        /// Provider segment from the source specifier.
        provider: String,
    },

    /// A built-in package was named, but no matching interface exists.
    UnknownBuiltinModule {
        /// Original import specifier from source.
        specifier: String,

        /// Built-in provider package selected by the specifier.
        package: String,

        /// Known modules for corrective diagnostics.
        known_modules: Vec<String>,
    },
}

/// Resolver for built-in and package-backed Faber library imports.
///
/// `resolve` returns `Ok(None)` when the specifier is not provider-shaped. That
/// lets package loading fall through to local import resolution without
/// treating every plain package path as a library error.
#[derive(Debug, Clone)]
pub(crate) struct LibraryResolver {
    stdlib_root: PathBuf,
}

impl LibraryResolver {
    /// Build a resolver rooted at an explicit `stdlib` directory.
    pub(crate) fn new(stdlib_root: impl Into<PathBuf>) -> Self {
        Self {
            stdlib_root: stdlib_root.into(),
        }
    }

    /// Build a resolver for the workspace stdlib bundled with this crate.
    pub(crate) fn default() -> Self {
        Self::new(default_stdlib_root())
    }

    /// Resolve a Faber import specifier to a library interface, if applicable.
    ///
    /// The resolver claims provider-qualified specifiers and old built-in Norma
    /// slash specifiers. For `norma`, malformed paths and missing interface
    /// files are reported as library diagnostics because the user clearly
    /// selected the built-in provider and should see available module names.
    pub(crate) fn resolve(
        &self,
        specifier: &str,
    ) -> Result<Option<ResolvedLibraryModule>, LibraryResolveError> {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            return Ok(None);
        }

        if let Some(module_path) = specifier.strip_prefix("norma/") {
            return Err(LibraryResolveError::OldBuiltinNormaSpecifier {
                specifier: specifier.to_owned(),
                replacement: format!("norma:{module_path}"),
            });
        }

        let Some((provider, module_path)) = specifier.split_once(':') else {
            return Ok(None);
        };

        if provider.is_empty() {
            return Err(LibraryResolveError::InvalidProviderSpecifier {
                specifier: specifier.to_owned(),
                reason: "provider segment must not be empty".to_owned(),
            });
        }

        if module_path.is_empty() {
            return Err(LibraryResolveError::InvalidProviderSpecifier {
                specifier: specifier.to_owned(),
                reason: "module path segment must not be empty".to_owned(),
            });
        }

        if module_path.contains(':') {
            return Err(LibraryResolveError::InvalidProviderSpecifier {
                specifier: specifier.to_owned(),
                reason: "library specifier must contain exactly one provider separator".to_owned(),
            });
        }

        if provider != "norma" {
            return Err(LibraryResolveError::UnknownProvider {
                specifier: specifier.to_owned(),
                provider: provider.to_owned(),
            });
        }

        let segments = module_path.split('/').collect::<Vec<_>>();
        if !segments
            .iter()
            .all(|segment| is_valid_module_segment(segment))
        {
            return Err(LibraryResolveError::InvalidProviderSpecifier {
                specifier: specifier.to_owned(),
                reason: "module path must not contain empty, dot, or dot-dot segments".to_owned(),
            });
        }

        let interface_path = self
            .stdlib_root
            .join("norma")
            .join(format!("{module_path}.fab"));
        if !interface_path.exists() {
            return Err(LibraryResolveError::UnknownBuiltinModule {
                specifier: specifier.to_owned(),
                package: "norma".to_owned(),
                known_modules: self.known_norma_modules(),
            });
        }

        Ok(Some(ResolvedLibraryModule::new(
            "norma",
            segments
                .iter()
                .map(|segment| (*segment).to_owned())
                .collect(),
            interface_path,
            LibraryProviderKind::Builtin,
        )))
    }

    fn known_norma_modules(&self) -> Vec<String> {
        let mut modules = Vec::new();
        let norma_root = self.stdlib_root.join("norma");
        collect_fab_modules(&norma_root, &norma_root, &mut modules);
        modules.sort();
        modules
    }
}

fn is_valid_module_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn collect_fab_modules(root: &Path, dir: &Path, modules: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_fab_modules(root, &path, modules);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("fab") {
            continue;
        }

        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let mut module = relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        if module.starts_with('/') {
            module.remove(0);
        }
        modules.push(module);
    }
}

fn default_stdlib_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."))
        .join("stdlib")
}
