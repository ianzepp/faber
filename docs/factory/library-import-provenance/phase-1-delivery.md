# Phase 1 Delivery: Source Syntax Hard Cut

## Objective

Make provider-qualified `norma:...` the only accepted built-in Norma import
syntax in package compilation.

## Implementation

- Updated `crates/faber/src/library.rs` to parse provider-qualified
  `provider:module/path` specifiers.
- Preserved `./` and `../` imports as local relative imports.
- Rejected old built-in `norma/...` specifiers with a targeted replacement
  diagnostic.
- Claimed unknown provider-qualified specifiers as provider imports so they no
  longer fall through to local file-path resolution.
- Updated active Faber package tests and HTTP fixture setup to use
  `norma:json`, `norma:toml`, `norma:hal/consolum`, and `norma:hal/http`.
- Added focused tests for old slash-form rejection, relative local import
  preservation, and unknown provider diagnostics.

## Checkpoint Evidence

- `importa ex "norma:json" privata json` resolves in package compilation.
- `importa ex "norma:hal/http" privata http` resolves in the HTTP package
  fixture.
- `importa ex "norma/hal/http" privata http` fails with:
  `built-in Norma imports use provider syntax; write "norma:hal/http"`.
- `importa ex "./norma/json" privata local` remains a local relative import.
- `importa ex "sqlite:client" privata client` reports an unknown provider and
  does not fall back to the local import diagnostic.

## Validation

Run during the phase:

```bash
cargo test -p faber library
cargo test -p faber compile_package_reports_unknown_provider_without_local_fallback
cargo test -p faber compile_package_keeps_relative_norma_paths_as_local_imports
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
```

The first three passed. The HTTP fixture passed for both duplicated Faber test
targets after building its generated crate.

One attempted command with multiple Cargo test filters failed because Cargo
accepts only one positional test filter; it was rerun as valid individual
commands and is not a product failure.
