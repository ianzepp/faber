+++
term = "manifest"
kind = "concept"
category = "package"
canonical = true
summary = "The faber.toml package metadata file used by faber build, run, and test."
syntax = "faber.toml"
aliases = ["faber.toml", "package manifest"]
related = ["cli", "incipit", "proba"]
+++

`faber.toml` declares the package name, version, source root, entry file, and build target. Package compilation currently supports Rust binary output.

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

```fab
incipit {
    scribe "salve"
}
```
