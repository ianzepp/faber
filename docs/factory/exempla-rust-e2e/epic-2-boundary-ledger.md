# Epic 2 Boundary Relocation Ledger

**Created**: 2026-05-24  
**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`  
**Delivery Spec**: `docs/factory/faber-execution-roadmap/epic-2-phase-1-delivery.md`

This ledger records files moved out of `examples/exempla/` during Epic 2 Phase 1. The intent is to keep `examples/exempla/` as a standalone single-file Rust e2e corpus while preserving non-standalone material as fixtures.

## Summary

| Class | Count | Destination |
| --- | ---: | --- |
| Runtime/HAL dependency | 5 | `examples/fixtures/exempla-boundary/runtime-hal/` |
| Package/CLI shape | 2 | `examples/fixtures/exempla-boundary/package-cli/` |
| Import/helper-library shape | 2 | `examples/fixtures/exempla-boundary/imports/` |
| External dependency | 1 | `examples/fixtures/exempla-boundary/regex/` |
| Proba/test/package fixture | 14 | `examples/fixtures/exempla-boundary/proba/` |
| Declaration-only/non-executable | 3 | `examples/fixtures/exempla-boundary/declaration-only/` |
| Stale or invalid source | 5 | `examples/fixtures/exempla-boundary/stale-source/` |
| Unsupported Rust target feature | 5 | `examples/fixtures/exempla-boundary/unsupported-rust-target/` |
| Total `.fab` files relocated | 37 |  |

`ad/ad.fab` intentionally remains in `examples/exempla/` because Epic 3 is the linked capability-call implementation and its checkpoint names that exemplar directly.

## Relocations

| Original path | New path | Reason |
| --- | --- | --- |
| `examples/exempla/hal/aleator.fab` | `examples/fixtures/exempla-boundary/runtime-hal/aleator.fab` | Requires `norma`/HAL runtime provider context |
| `examples/exempla/hal/consolum.fab` | `examples/fixtures/exempla-boundary/runtime-hal/consolum.fab` | Requires `norma`/HAL runtime provider context |
| `examples/exempla/hal/json.fab` | `examples/fixtures/exempla-boundary/runtime-hal/json.fab` | Requires `norma`/HAL runtime provider context |
| `examples/exempla/hal/processus.fab` | `examples/fixtures/exempla-boundary/runtime-hal/processus.fab` | Requires `norma`/HAL runtime provider context |
| `examples/exempla/hal/yaml.fab` | `examples/fixtures/exempla-boundary/runtime-hal/yaml.fab` | Requires `norma`/HAL runtime provider context |
| `examples/exempla/cli/main.fab` | `examples/fixtures/exempla-boundary/package-cli/main.fab` | Requires package/module CLI command structure |
| `examples/exempla/cli/commands/greet.fab` | `examples/fixtures/exempla-boundary/package-cli/commands/greet.fab` | Package command module, not standalone executable source |
| `examples/exempla/importa/auxilia.fab` | `examples/fixtures/exempla-boundary/imports/auxilia.fab` | Helper module for import demonstration |
| `examples/exempla/importa/importa.fab` | `examples/fixtures/exempla-boundary/imports/importa.fab` | Requires sibling module/import assembly |
| `examples/exempla/expressionis/expressionis.fab` | `examples/fixtures/exempla-boundary/regex/expressionis.fab` | Requires external `regex` crate |
| `examples/exempla/proba/proba.fab` | `examples/fixtures/exempla-boundary/proba/proba.fab` | Test-harness semantics, not standalone executable shape |
| `examples/exempla/proba/modificatores.fab` | `examples/fixtures/exempla-boundary/proba/modificatores.fab` | Test modifiers and async/future test semantics |
| `examples/exempla/proba/packages/failing/` | `examples/fixtures/exempla-boundary/proba/packages/failing/` | Package fixture with `faber.toml` |
| `examples/exempla/proba/packages/ignored/` | `examples/fixtures/exempla-boundary/proba/packages/ignored/` | Package fixture with `faber.toml` |
| `examples/exempla/proba/packages/passing/` | `examples/fixtures/exempla-boundary/proba/packages/passing/` | Package fixture with `faber.toml` |
| `examples/exempla/proba/packages/selection-failure/` | `examples/fixtures/exempla-boundary/proba/packages/selection-failure/` | Package/test selection negative fixture |
| `examples/exempla/proba/packages/selectors/` | `examples/fixtures/exempla-boundary/proba/packages/selectors/` | Package fixture with selectors |
| `examples/exempla/proba/packages/solum/` | `examples/fixtures/exempla-boundary/proba/packages/solum/` | Package fixture requiring runtime/package context |
| `examples/exempla/proba/packages/suite/` | `examples/fixtures/exempla-boundary/proba/packages/suite/` | Package suite fixture |
| `examples/exempla/omitte/omitte.fab` | `examples/fixtures/exempla-boundary/proba/omitte.fab` | Test skip semantics, not ordinary executable example |
| `examples/exempla/postpara/postpara.fab` | `examples/fixtures/exempla-boundary/proba/postpara.fab` | Test teardown semantics |
| `examples/exempla/praepara/praepara.fab` | `examples/fixtures/exempla-boundary/proba/praepara.fab` | Test setup semantics |
| `examples/exempla/futurum/futurum.fab` | `examples/fixtures/exempla-boundary/proba/futurum.fab` | Test future/async fixture semantics |
| `examples/exempla/figendum/figendum.fab` | `examples/fixtures/exempla-boundary/proba/figendum.fab` | Historical test fixture, not standalone language example |
| `examples/exempla/annotatio/annotatio.fab` | `examples/fixtures/exempla-boundary/declaration-only/annotatio.fab` | Emits declaration-only Rust functions without executable bodies |
| `examples/exempla/annotatio/grammatica-nova.fab` | `examples/fixtures/exempla-boundary/declaration-only/grammatica-nova.fab` | Mixes annotations/import declarations and package-only surfaces |
| `examples/exempla/externa/externa.fab` | `examples/fixtures/exempla-boundary/declaration-only/externa.fab` | FFI/declaration-only example without standalone executable body |
| `examples/exempla/tempta/in-functione.fab` | `examples/fixtures/exempla-boundary/stale-source/in-functione.fab` | Uses retired `tempta` syntax |
| `examples/exempla/tempta/tempta.fab` | `examples/fixtures/exempla-boundary/stale-source/tempta.fab` | Uses retired `tempta` syntax |
| `examples/exempla/literalis/literalis.fab` | `examples/fixtures/exempla-boundary/stale-source/literalis.fab` | Unknown identifiers/stale source |
| `examples/exempla/qua/qua.fab` | `examples/fixtures/exempla-boundary/stale-source/qua.fab` | Emits placeholder type hole; not a valid standalone exemplar |
| `examples/exempla/lege/lege.fab` | `examples/fixtures/exempla-boundary/stale-source/lege.fab` | Input/read behavior is not standalone deterministic e2e source |
| `examples/exempla/custodi/validatio.fab` | `examples/fixtures/exempla-boundary/unsupported-rust-target/validatio.fab` | Validation/alternate-exit policy is unsupported on Rust target |
| `examples/exempla/fac/cape.fab` | `examples/fixtures/exempla-boundary/unsupported-rust-target/cape.fab` | `cape` alternate-exit behavior unsupported on Rust target |
| `examples/exempla/fac/fac.fab` | `examples/fixtures/exempla-boundary/unsupported-rust-target/fac.fab` | `fac` effect/alternate-exit behavior unsupported on Rust target |
| `examples/exempla/functio/exitus.fab` | `examples/fixtures/exempla-boundary/unsupported-rust-target/exitus.fab` | Failable function exit policy unsupported on Rust target |
| `examples/exempla/iace/iace.fab` | `examples/fixtures/exempla-boundary/unsupported-rust-target/iace.fab` | Throw/raise behavior unsupported on Rust target |

## Validation Notes

Generated package `target/` directories were not preserved in the new fixture tree. They are build artifacts, not source fixtures.

After relocation, `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reported `59/100` files passing. That was the Phase 1 boundary baseline, not the current state.

Current closeout state on 2026-05-24:

- `examples/exempla/` contains `100` `.fab` files.
- `examples/fixtures/exempla-boundary/` contains `37` relocated `.fab` fixtures.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `99/100` files passing.
- The only current expected failure is `examples/exempla/ad/ad.fab`, deferred to Epic 3 capability-call support.
- The Rust e2e harness now asserts the full expected corpus state instead of only checking `salve-munde.fab`.

The relocated boundary classes no longer contribute HAL, regex, import, package, proba, `tempta`, declaration-only, or standalone-input failures to the executable corpus.
