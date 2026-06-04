# Library Import Provenance Factory Plan

**Status**: planned
**Created**: 2026-06-04
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/library-import-provenance/`
**Mode**: hard-cut import syntax / library provenance / Rust runtime bridge correction
**Commit Policy**: Commit after each completed phase and validation gate pass
**Related Plans**:
[`../stdlib-data-formats/plan.md`](../stdlib-data-formats/plan.md),
[`../norma-http-hal/plan.md`](../norma-http-hal/plan.md)

## Interpreted Problem

Faber has started building a real Norma standard library surface, but the
current implementation still blurs three separate concepts:

1. the source import specifier written by a Faber program,
2. the provider and module identity of the imported library,
3. the target-specific Rust runtime path used by codegen.

The slash form currently used in newer tests and docs, such as
`"norma/hal/http"`, makes `norma` look like the first segment of a normal
module path instead of a provider namespace. That is too weak for future
package imports and too easy to confuse with local package modules.

The Rust bridge also has an identity bug: selected Norma calls and HTTP
interfaces are recognized by local names or interface shape. A user-defined
binding called `http`, or a user-defined `pactum Replicatio` with matching
method names, can be treated as Norma runtime code even when it was not imported
from Norma. That is not a documentation problem; it is a provenance problem.

The correct fix is a hard cut to provider-qualified import specifiers and a
target-neutral library identity carried through resolution, HIR, and codegen.

## Approved Source Contract

Built-in and package-library imports use provider-qualified specifiers:

```fab
importa ex "norma:json" privata json
importa ex "norma:toml" privata toml
importa ex "norma:hal/http" privata http
importa ex "norma:hal/consolum" privata consolum
```

The provider is the text before the first colon. The module path is the text
after the colon, split by `/`.

Local package imports keep their existing relative path shape:

```fab
importa ex "./models" privata models
importa ex "../shared/config" privata config
```

Future package providers should fit the same source shape:

```fab
importa ex "sqlite:client" privata sqlite
importa ex "ian.math:statistica" privata stat
```

This phase is hard cut only:

- reject old built-in slash forms such as `"norma/json"`,
  `"norma/toml"`, and `"norma/hal/http"`;
- do not add a resolver compatibility alias;
- do not add a warning-only deprecation window;
- do not silently reinterpret `norma/...` as `norma:...`;
- update active tests, fixtures, examples, and current docs to the colon form.

Diagnostics for old built-in Norma paths should be direct:

```text
built-in Norma imports use provider syntax; write "norma:hal/http"
```

Historical factory ledgers may continue to mention the old spelling as baseline
truth, but current guidance and active examples must use the provider-qualified
form.

## Provider Specifier Rules

Provider-qualified specifiers are library imports, not relative file imports.

Rules:

- A specifier beginning with `./` or `../` is a local relative import and must
  not be parsed as a provider import.
- A library specifier contains exactly one provider separator, the first `:`.
- The provider segment must be non-empty.
- The module path segment must be non-empty.
- The module path uses `/` as the internal path separator.
- Empty path segments are invalid.
- For this factory pass, only the built-in `norma` provider is implemented.
- Unknown providers produce a provider-resolution diagnostic, not a file-path
  fallback.

The formal grammar already shows `norma:hal/consolum`; implementation and
tests need to be brought back into alignment with that contract.

## Provenance Model

Library identity must be represented explicitly and carried target-neutrally.
Do not use source spelling, local binding name, method names, or Rust module
paths as identity.

The implementation should introduce or converge on data shaped like this:

```rust
struct LibraryIdentity {
    provider: LibraryProvider,
    module_path: Vec<String>,
}

enum LibraryProvider {
    BuiltinNorma,
    Package(String),
}

struct LibraryBinding {
    local_def_id: DefId,
    identity: LibraryIdentity,
}

struct LibraryItem {
    def_id: DefId,
    identity: LibraryIdentity,
    exported_name: String,
    kind: LibraryItemKind,
}
```

The exact Rust names can follow local style, but the invariants are mandatory:

- every imported library module binding has provider identity keyed by `DefId`;
- every declaration injected from a library interface has item provenance keyed
  by its own `DefId`;
- target codegen receives this information through HIR, analysis metadata, or a
  side table with stable `DefId` keys;
- target backends decide runtime linkage from provider identity, not from Faber
  syntax strings;
- source aliases change local names only, not provider identity.

Example:

```fab
importa ex "norma:hal/http" privata http ut rete
```

If aliasing is supported by the existing import grammar for this declaration
shape, `rete.petet(...)` must still lower to the Norma HTTP runtime because the
receiver binding's `DefId` has `BuiltinNorma + ["hal", "http"]` provenance. A
user-defined `rete` value must not lower that way.

## Rust Codegen Contract

The Rust backend must consume provider provenance.

Replace receiver-name matching like:

```rust
fn norma_runtime_module_path(receiver_name: &str) -> Option<&'static str>
```

with a DefId/provider-aware lookup shaped like:

```rust
fn rust_runtime_module_for_library_binding(
    def_id: DefId,
    libraries: &LibraryRegistry,
) -> Option<&'static str>
```

The mapping belongs in Rust backend linkage metadata:

```text
BuiltinNorma + ["json"]        -> norma::json
BuiltinNorma + ["toml"]        -> norma::toml
BuiltinNorma + ["hal", "http"] -> norma::hal::http
BuiltinNorma + ["hal", "consolum"] -> norma::hal::consolum
```

The backend may use source names for generated local variable names, but never
for deciding whether a call is a Norma runtime call.

## Runtime Interface Type Contract

HTTP runtime interface mapping must also use provenance.

Current shape-based recognition such as `Replicatio` plus method-list matching
is unsafe. A local user interface with the same name and methods is still a user
interface.

The intended identity mapping is:

```text
BuiltinNorma + ["hal", "http"] + "http"        -> elide module interface
BuiltinNorma + ["hal", "http"] + "Replicatio" -> norma::hal::http::Replicatio
BuiltinNorma + ["hal", "http"] + "Rogatio"    -> norma::hal::http::Rogatio
BuiltinNorma + ["hal", "http"] + "Servitor"   -> norma::hal::http::Servitor
```

Method-list checks may remain only as defensive assertions that the imported
interface file still matches the expected runtime ABI. They must not decide
identity. If a method-list assertion fails, codegen should report an internal
library ABI mismatch or targeted diagnostic rather than silently treating a
different interface as the runtime type.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline Ledger | Record current slash-form imports, grammar state, resolver behavior, and string/shape bridge bug. | Ledger captures truth before edits. |
| 1 | Source Syntax Hard Cut | Make provider-qualified `norma:...` the only accepted built-in Norma import syntax. | Colon imports resolve; slash built-ins reject with targeted diagnostics. |
| 2 | Provider Metadata Model | Add target-neutral library binding and item provenance keyed by `DefId`. | Resolver and semantic tests can inspect provider identity independent of aliases. |
| 3 | Runtime Call Bridge By Provenance | Replace receiver-name runtime call bridging with provider-aware lookup. | Aliased Norma imports bridge; user-defined same-name bindings do not. |
| 4 | Runtime Type Mapping By Provenance | Replace HTTP interface name/shape identity with imported library item provenance. | User-defined `Replicatio` is not mapped to `norma::hal::http::Replicatio`. |
| 5 | Docs And Fixtures | Update active docs, fixtures, and package examples to `provider:module/path`. | No active guidance recommends slash-form built-in imports. |
| 6 | Validation Gate | Run focused and broad checks. | Required Cargo, Faber, and fixture commands pass. |

Progress artifacts should be written in this directory as each phase runs:

- Phase 0: `phase-0-delivery.md`, `ledger.md`
- Phase 1: `phase-1-delivery.md`
- Phase 2: `phase-2-delivery.md`
- Phase 3: `phase-3-delivery.md`
- Phase 4: `phase-4-delivery.md`
- Phase 5: `phase-5-delivery.md`
- Phase 6: `phase-6-delivery.md`

## Phase Details

### Phase 0: Baseline Ledger

Record:

- `git status --short`;
- current `EBNF.md` import examples;
- current package tests using `"norma/json"`, `"norma/toml"`, and
  `"norma/hal/http"`;
- current `crates/faber/src/library.rs` built-in resolver behavior;
- current Rust runtime bridge in
  `crates/radix/src/codegen/rust/expr/call/runtime.rs`;
- current HTTP runtime interface mapping in `crates/radix/src/codegen/rust`;
- a repro showing a user-defined `Replicatio` or `http` identity collision if
  still present.

No source behavior changes in this phase.

### Phase 1: Source Syntax Hard Cut

Steps:

- Update the library import parser/resolver to parse provider-qualified
  specifiers.
- Implement `norma:...` resolution to `stdlib/norma/...`.
- Reject `norma/...` as an old built-in Norma spelling.
- Ensure unknown provider diagnostics do not fall through to local file-path
  resolution.
- Update active package tests and fixtures to the colon form.
- Add tests for accepted and rejected import specifiers.

Checkpoint:

- `importa ex "norma:json" privata json` resolves.
- `importa ex "norma:hal/http" privata http` resolves.
- `importa ex "norma/json" privata json` fails with the targeted hard-cut
  diagnostic.
- `importa ex "./norma/json" privata local` remains a local relative import.

### Phase 2: Provider Metadata Model

Steps:

- Introduce a target-neutral representation for resolved library providers and
  module paths.
- Attach library binding identity to the local import binding `DefId`.
- Attach library item provenance to declarations appended or synthesized from
  stdlib interface files.
- Preserve this metadata through collect, resolve, lower, typecheck, and codegen
  boundaries without target-specific Rust paths.
- Add focused tests proving alias imports preserve provider identity.

Checkpoint:

- Code can ask whether a specific `DefId` came from `BuiltinNorma +
  ["hal", "http"]`.
- No codegen path needs to inspect the local binding name to know the provider.

### Phase 3: Runtime Call Bridge By Provenance

Steps:

- Replace `norma_runtime_module_path(receiver_name)` with a lookup keyed by the
  resolved receiver binding.
- Map provider identities to Rust runtime modules in the Rust backend.
- Preserve existing JSON, TOML, console, and HTTP runtime call behavior.
- Add negative tests where user-defined `json`, `toml`, `consolum`, or `http`
  values do not lower to Norma runtime modules.

Checkpoint:

- `importa ex "norma:hal/http" privata http ut rete` lowers
  `rete.petet(...)` to `norma::hal::http::petet(...)`.
- A user-defined local `http.petet(...)` call is not bridged.

### Phase 4: Runtime Type Mapping By Provenance

Steps:

- Replace HTTP runtime interface recognition by name and method list with
  imported library item provenance.
- Map only Norma HTTP imported `Replicatio`, `Rogatio`, and `Servitor` to
  concrete `norma::hal::http` Rust types.
- Keep method-list comparison only as an ABI sanity assertion if useful.
- Add negative tests for user-defined interfaces with colliding names and
  matching methods.

Checkpoint:

- A Faber source file defining its own `pactum Replicatio` emits a normal user
  trait or interface representation.
- A Faber source file importing `norma:hal/http` emits
  `norma::hal::http::Replicatio` where the stdlib type is referenced.

### Phase 5: Docs And Fixtures

Steps:

- Update active package fixtures to the provider-qualified syntax.
- Update current docs and examples that present canonical built-in Norma import
  syntax.
- Update the Norma HAL expansion pattern to require provider provenance and
  forbid name/shape runtime identity.
- Leave historical baseline ledgers alone when they intentionally document old
  behavior.

Checkpoint:

- Active docs teach `norma:json`, `norma:toml`, and `norma:hal/http`.
- No active fixture relies on the old slash spelling.

### Phase 6: Validation Gate

Required focused checks:

```bash
cargo test -p faber library
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
cargo test -p radix http_hal_calls_emit_norma_runtime_bridge_and_concrete_response_type
cargo test -p radix runtime
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

Required broad checks:

```bash
cargo test -p radix
cargo test -p faber
cargo test -p norma
./scripta/test
```

If focused test names change during implementation, use the closest exact test
targets and record the substitutions in the phase ledger.

## Acceptance Criteria

- `norma:...` is the only accepted built-in Norma import syntax.
- Old slash built-ins are rejected, not warned and accepted.
- Local relative imports continue to work.
- Unknown providers produce provider diagnostics.
- Alias imports preserve library identity.
- Rust runtime call bridging is keyed by provider provenance.
- Rust runtime type mapping is keyed by imported library item provenance.
- User-defined names or matching method lists cannot trigger Norma runtime
  lowering.
- HTTP, JSON, TOML, and console package fixtures still pass after conversion.
- Active docs and examples use the provider-qualified spelling.

## Stop Conditions

Stop the factory phase and fix the architecture if any of these remain true:

- codegen still decides Norma runtime identity from local binding strings;
- codegen still decides HTTP runtime type identity from interface name and
  method list alone;
- `norma/json`, `norma/toml`, or `norma/hal/http` still compile as built-in
  imports;
- provider metadata is stored only in target-specific Rust codegen;
- alias imports lose provider identity;
- unknown providers fall back to local file import behavior.

## Non-Goals

- Do not build a package manager in this pass.
- Do not design a general external registry.
- Do not add transitional compatibility for old built-in slash imports.
- Do not rewrite every historical factory ledger.
- Do not change HTTP runtime behavior except where fixtures need source syntax
  updates.
- Do not change local relative import syntax.

## Factory Notes

This plan intentionally supersedes the slash examples in older active plans.
Where older implementation artifacts say `norma/json` or `norma/hal/http`, the
next implementation phase should treat those as stale surface syntax unless the
text is explicitly documenting historical baseline behavior.

Veritas ante omnia: provider identity must be data in the compiler, not an
accident of names.
