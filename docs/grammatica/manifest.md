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

## Package Build Layout

When you run `faber build` (or `faber run`) on a package, Faber produces two kinds of output under the package root:

```
<package>/
├── faber.toml
├── src/
│   └── main.fab
└── target/
    ├── faber/                 # generated Rust crate (source of truth for the backend)
    │   ├── Cargo.toml
    │   └── src/
    │       └── main.rs
    ├── debug/                 # Rust debug artifacts (the actual executable)
    │   └── <package-name>
    └── release/               # Rust release artifacts
        └── <package-name>
```

- `target/faber/` contains the Rust crate that the compiler emits. It is **generated source**.
- `target/debug/` and `target/release/` are the real build artifacts produced by invoking Cargo against the generated crate (using `--target-dir target`).
- These three directories are **siblings**. You will never see `target/faber/target/...`.
- All `target/` contents are build artifacts and should be ignored by version control (the default `.gitignore` covers them).

The exact Cargo invocation used is equivalent to:

```bash
cargo build --manifest-path target/faber/Cargo.toml --target-dir target
# or with --release for release builds
```

`faber emit -t rust --package` is the inspection path if you only want to see the generated Rust on stdout without writing the crate tree.

## Testing Packages

`faber test` uses the same generated-crate layout as `faber build`, but it executes Cargo's test harness instead of the binary build.

The implementation first writes the package to `target/faber/`, then runs:

```bash
cargo test --manifest-path target/faber/Cargo.toml --target-dir target
```

That keeps Cargo artifacts in the package `target/` sibling directories and avoids `target/faber/target/`.

Selection is implemented by the generated Rust harness:

- `--name` matches the original Faber test name.
- `--suite` matches the full suite path joined with `/`.
- `--tag` matches the `tag` modifier.
- `solum` cases are treated as focused tests and win the default selection set when any are present.
- `omitte` and `futurum` still compile as ignored Rust tests with reasons.
- `--ignored` and `--include-ignored` are forwarded to Cargo's Rust harness and therefore also affect selection-ignored tests.

The current implementation is Rust-only and keeps all tests generated so the compiler still checks deselected test bodies.

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
faber run hello
faber build --release hello
faber emit -t rust --package hello/faber.toml
```

Directory package inputs use `faber.toml` when present. For compatibility with simple examples, a directory without `faber.toml` still falls back to `main.fab` at the directory root.
