# Faber Manifest

Faber packages use `faber.toml` for project metadata and build configuration. Package configuration is intentionally outside `.fab` source files: Faber source defines programs, while `faber.toml` defines how the package tool finds and builds them.

## Minimal Manifest

```toml
[package]
name = "salve"
version = "0.1.0"
edition = "2026"

[paths]
source = "src"
entry = "main.fab"

[build]
target = "rust"
kind = "bin"
```

`faber init <dir>` creates this shape with `src/main.fab`.

## Sections

### `[package]`

```toml
[package]
name = "salve"
version = "0.1.0"
edition = "2026"
```

- `name` is required and must not be empty.
- `version` defaults to `"0.1.0"` when omitted.
- `edition` defaults to `"2026"` when omitted.

### `[paths]`

```toml
[paths]
source = "src"
entry = "main.fab"
```

- `source` is the package source root relative to the manifest directory.
- `entry` is the entry `.fab` file relative to `source`.
- Defaults are `source = "src"` and `entry = "main.fab"`.

### `[build]`

```toml
[build]
target = "rust"
kind = "bin"
```

- `target` defaults to `"rust"`.
- `kind` defaults to `"bin"`.
- Rust binary package compilation is the only supported package build mode today.

## Current Limits

- Dependency declarations are not implemented yet.
- Workspaces are not implemented yet.
- Package compilation is Rust-only.
- `faber.fab` is not a package manifest. Use `faber.toml`.

## Commands

```bash
faber init hello
faber check hello
faber build hello
faber emit -t rust --package hello/faber.toml
```

Directory package inputs use `faber.toml` when present. For compatibility with simple examples, a directory without `faber.toml` still falls back to `main.fab` at the directory root.
