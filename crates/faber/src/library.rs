//! Library import resolution for package compilation.
//!
//! `faber` treats built-in libraries as source-level interfaces under
//! `stdlib/`, not as hard-coded compiler magic. The resolver translates an
//! import specifier such as `norma/hal/solum` into the `.fab` interface that
//! package loading can parse, typecheck, and wire into generated output.
//!
//! The current provider surface is intentionally small. `norma` is resolved
//! from the repository stdlib today, while the data model already has enough
//! shape for future package-backed providers without baking Rust runtime
//! metadata into import resolution.

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
/// `resolve` returns `Ok(None)` when the specifier does not belong to a known
/// library provider. That lets package loading fall through to local import
/// resolution without treating every unknown string as a library error.
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
    /// The resolver only claims specifiers whose first path segment is a known
    /// provider. For `norma`, malformed paths and missing interface files are
    /// reported as library diagnostics because the user clearly selected the
    /// built-in provider and should see available module names.
    pub(crate) fn resolve(
        &self,
        specifier: &str,
    ) -> Result<Option<ResolvedLibraryModule>, LibraryResolveError> {
        let segments = specifier
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        if segments.first() != Some(&"norma") {
            return Ok(None);
        }

        if segments.len() < 2
            || !segments[1..]
                .iter()
                .all(|segment| is_valid_module_segment(segment))
        {
            return Err(LibraryResolveError::UnknownBuiltinModule {
                specifier: specifier.to_owned(),
                package: "norma".to_owned(),
                known_modules: self.known_norma_modules(),
            });
        }

        let module_path = segments[1..].join("/");
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
            segments[1..]
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
