use std::path::{Path, PathBuf};

const BUILTIN_NORMA_MODULES: &[&str] = &["json", "toml", "hal/consolum"];

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
    MissingInterface {
        specifier: String,
        interface_path: PathBuf,
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

        let module_path = segments[1..].join("/");
        if segments.len() < 2 || !BUILTIN_NORMA_MODULES.contains(&module_path.as_str()) {
            return Err(LibraryResolveError::UnknownBuiltinModule {
                specifier: specifier.to_owned(),
                package: "norma".to_owned(),
                known_modules: BUILTIN_NORMA_MODULES
                    .iter()
                    .map(|module| (*module).to_owned())
                    .collect(),
            });
        }

        let interface_path = self
            .stdlib_root
            .join("norma")
            .join(format!("{module_path}.fab"));
        if !interface_path.exists() {
            return Err(LibraryResolveError::MissingInterface {
                specifier: specifier.to_owned(),
                interface_path,
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
}

fn default_stdlib_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."))
        .join("stdlib")
}
