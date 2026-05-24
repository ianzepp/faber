# Epic 6.1 Norma Classification

## Labels

- `core-language`: pure language/library semantics the compiler or target
  runtime can own directly.
- `host-effect`: outside-world effects and IO that route semantically through
  Faber host syscall capability surfaces.
- `rust-bridge`: native Rust support kept for current HIR-to-Rust output while
  the host path matures.
- `needs-decision`: surfaces that require later review before classification is
  final enough for migration.

## Architectural Decisions

- `@ externa` inside `stdlib/norma/hal/*.fab` is treated as a host capability
  contract marker by convention for Epic 6. No new syscall annotation syntax is
  added in this slice.
- Existing `@ subsidia rs ...` links identify temporary native Rust bridge
  implementations. They do not decide final ownership.
- Pure `stdlib/norma/innatum/*.fab`, `mathesis.fab`, `codex.fab`, and the data
  format pacta are classified as `core-language` contracts because they describe
  deterministic in-memory behavior.
- All IO and outside-world interaction is `host-effect`, including console IO.
- `crates/norma/**` remains available as `rust-bridge` support for current
  native Rust output. This slice does not delete or move Rust implementations.
- Native Rust direct output for `nota`, `vide`, `mone`, and related printing is
  an implementation policy of the current backend. Semantically, those effects
  are host IO and should converge with `consolum`.
- Host syscall failures stay on the generic frame/host error path proven by the
  earlier host epics. Epic 6.1 does not introduce per-`consolum` error
  taxonomies.

## Stdlib File Classification

| File | Classification | Evidence / Notes |
| --- | --- | --- |
| `stdlib/norma/codex.fab` | `core-language` | Pure base64, hex, and URL encoding/decoding transforms over bytes/text. |
| `stdlib/norma/json.fab` | `core-language` | Pure JSON parse/serialize and value inspection contract; current Rust implementation is bridge support. |
| `stdlib/norma/mathesis.fab` | `core-language` | Explicitly documented as pure math with target translations. |
| `stdlib/norma/toml.fab` | `core-language` | Pure TOML parse/serialize and value inspection contract; current Rust implementation is bridge support. |
| `stdlib/norma/yaml.fab` | `core-language` | Pure YAML parse/serialize and value inspection contract; current Rust implementation is bridge support. |
| `stdlib/norma/innatum/copia.fab` | `core-language` | Built-in set API and pure collection operations. |
| `stdlib/norma/innatum/fractus.fab` | `core-language` | Built-in floating-point API. |
| `stdlib/norma/innatum/lista.fab` | `core-language` | Built-in list API and collection operations. Sampling/shuffle names may need later target policy, but this file remains core-language in 6.1. |
| `stdlib/norma/innatum/numerus.fab` | `core-language` | Built-in integer API. |
| `stdlib/norma/innatum/tabula.fab` | `core-language` | Built-in map API and collection operations. |
| `stdlib/norma/innatum/textus.fab` | `core-language` | Built-in string API. |
| `stdlib/norma/hal/aleator.fab` | `host-effect` | Randomness and entropy are host-owned capability behavior; seeded deterministic test policy remains future work. |
| `stdlib/norma/hal/arca.fab` | `host-effect` | Database connections, queries, and transactions are outside-world provider effects. |
| `stdlib/norma/hal/caelum.fab` | `host-effect` | TCP/TLS listener and connection operations are network effects. |
| `stdlib/norma/hal/consolum.fab` | `host-effect` | Console stdin/stdout/stderr and TTY detection are outside-world IO; this is the first proof contract. |
| `stdlib/norma/hal/crypta.fab` | `host-effect` | Cryptographic keys, signing, secure generation, and algorithm policy should be host/runtime capability owned. |
| `stdlib/norma/hal/http.fab` | `host-effect` | HTTP client/server operations are network effects; response/request object helpers remain part of the host contract. |
| `stdlib/norma/hal/nuncius.fab` | `host-effect` | Shared memory, ports, mutexes, semaphores, and cross-worker messaging are runtime/host effects. |
| `stdlib/norma/hal/pressura.fab` | `core-language` | Compression/decompression are pure byte transforms in the current contract; streaming compressor handles may become target-runtime support later. |
| `stdlib/norma/hal/processus.fab` | `host-effect` | Process spawn, shell execution, env vars, cwd, args, and exit are host effects. |
| `stdlib/norma/hal/solum.fab` | `host-effect` | File IO, filesystem metadata, symlinks, directory mutation, home/temp dirs, and path resolution touch host state. The pure path-string helpers may be split into `core-language` later. |
| `stdlib/norma/hal/tempus.fab` | `host-effect` | Clock, monotonic time, sleep, timers, and cancellation handles are host/runtime effects; unit constants are pure but stay with the contract for now. |
| `stdlib/norma/hal/thesaurus.fab` | `host-effect` | Cache/key-value/pubsub behavior is an external provider capability. |
| `stdlib/norma/ems/README.md` | `needs-decision` | Planned entity management surface; likely host-effect where it reaches storage, but no executable contract exists yet. |
| `stdlib/norma/llm/README.md` | `needs-decision` | Planned provider bindings; likely host-effect, but no executable contract exists yet. |
| `stdlib/norma/vfs/README.md` | `needs-decision` | Planned virtual filesystem surface; likely host-effect over `solum`, but no executable contract exists yet. |

## Crate File Classification

| File | Classification | Evidence / Notes |
| --- | --- | --- |
| `crates/norma/Cargo.toml` | `rust-bridge` | Native Rust support package manifest and dependencies. |
| `crates/norma/.gitignore` | `rust-bridge` | Crate-local generated artifact hygiene. |
| `crates/norma/lib.rs` | `rust-bridge` | Native Rust module surface for generated Rust support. |
| `crates/norma/datum.rs` | `rust-bridge` | Native Rust `Valor` bridge for current data-format implementations; semantically supports core-language data contracts. |
| `crates/norma/datum_test.rs` | `rust-bridge` | Tests for the native `Valor` bridge. |
| `crates/norma/json.rs` | `rust-bridge` | Native Rust JSON implementation for current generated Rust output. |
| `crates/norma/toml.rs` | `rust-bridge` | Native Rust TOML implementation for current generated Rust output. |
| `crates/norma/yaml.rs` | `rust-bridge` | Native Rust YAML implementation for current generated Rust output. |
| `crates/norma/hal/mod.rs` | `rust-bridge` | Native Rust HAL module export surface. |
| `crates/norma/hal/arca.rs` | `rust-bridge` | Native Rust database support for the host-effect `arca` contract. |
| `crates/norma/hal/consolum.rs` | `rust-bridge` | Native Rust console support for the host-effect `consolum` contract. Must remain available during migration. |
| `crates/norma/hal/processus.rs` | `rust-bridge` | Native Rust process/env/cwd support for the host-effect `processus` contract. |
| `crates/norma/hal/solum.rs` | `rust-bridge` | Native Rust filesystem/path support for the host-effect `solum` contract. |

## Consolum Host-Effect Contract

`stdlib/norma/hal/consolum.fab` is the first host-effect proof surface. Its
current functions map to canonical syscall identities by pactum/function path:

| Contract Member | Canonical Syscall | Direction |
| --- | --- | --- |
| `hauri` | `consolum:hauri` | stdin bytes, sync |
| `hauriet` | `consolum:hauriet` | stdin bytes, async |
| `lege` | `consolum:lege` | stdin text line, sync |
| `leget` | `consolum:leget` | stdin text line, async |
| `funde` | `consolum:funde` | stdout bytes, sync |
| `fundet` | `consolum:fundet` | stdout bytes, async |
| `scribe` | `consolum:scribe` | stdout line, sync |
| `scribet` | `consolum:scribet` | stdout line, async |
| `dic` | `consolum:dic` | stdout text without newline, sync |
| `dicet` | `consolum:dicet` | stdout text without newline, async |
| `mone` | `consolum:mone` | stderr warning line, sync |
| `monet` | `consolum:monet` | stderr warning line, async |
| `vide` | `consolum:vide` | stderr/debug line, sync |
| `videbit` | `consolum:videbit` | stderr/debug line, async |
| `estTerminale` | `consolum:estTerminale` | stdin TTY predicate |
| `estTerminaleOutput` | `consolum:estTerminaleOutput` | stdout TTY predicate |

These identities are source-contract identities, not strict-mode manifest
requirements yet. A later slice can expose them through the existing host
manifest shape after deciding argument/result metadata.

## Language Output Semantics

The language-level output forms `nota`, `vide`, and `mone` should be understood
as host IO:

- `nota` aligns with stdout line output, currently closest to `consolum:scribe`.
- `mone` aligns with stderr warning output, currently closest to `consolum:mone`.
- `vide` aligns with debug/inspection stderr output, currently closest to
  `consolum:vide`.

Current native Rust codegen may continue to lower these directly to Rust output
helpers or macros while the Rust backend remains the stable execution path.
That direct path is not a separate ownership category.

## Deferred Decisions

- Whether `pressura` belongs permanently in `core-language` or moves to
  target-runtime support when streaming handles become concrete.
- Whether `lista.specimen`, `lista.specimina`, and `lista.miscita` should depend
  on `aleator` host randomness or become explicit deterministic target-runtime
  operations.
- Whether path-string helpers inside `solum` should split into a pure path
  module instead of staying under a host-effect filesystem contract.
- How strict-mode manifests should encode `consolum` argument and result
  metadata.
- Whether future host manifests use `consolum:*`, a shorter alias such as
  `io:*`, or both. Epic 6.1 records `consolum:*` as the canonical contract path.
