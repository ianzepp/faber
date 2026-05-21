use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum LibraryProviderKind {
    Builtin,
    PackageDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedLibraryModule {
    pub package: String,
    pub module_path: Vec<String>,
    pub interface_path: PathBuf,
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

    pub(crate) fn module_name(&self) -> Option<&str> {
        self.module_path.last().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LibraryResolveError {
    UnknownBuiltinModule {
        specifier: String,
        package: String,
        known_modules: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct LibraryResolver {
    stdlib_root: PathBuf,
}

impl LibraryResolver {
    pub(crate) fn new(stdlib_root: impl Into<PathBuf>) -> Self {
        Self {
            stdlib_root: stdlib_root.into(),
        }
    }

    pub(crate) fn default() -> Self {
        Self::new(default_stdlib_root())
    }

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
