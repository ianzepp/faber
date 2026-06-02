# Target Support Matrix Factory Goal

**Status**: ready for future factory assignment  
**Created**: 2026-06-02  
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Factory Artifact Dir**: `docs/factory/target-support-matrix/`  
**Compiler Metadata Dir**: `crates/radix/targets/`  
**Primary Goal**: TOML-authored, compiler-audited target feature support metadata  
**Commit Policy**: commit after each completed phase with passing focused and checkpoint validation

## Objective

Create a compiler-owned target support matrix that is easy to edit in TOML,
useful for docs and factory planning, and audited against real compiler behavior.

The support matrix should answer questions such as:

- Does Rust support `ad`, and at what tier?
- Is Go failure on optional parameters a backend bug or declared unsupported
  target policy?
- Which examples prove TypeScript support for collections?
- Which Wasm examples are only compile-valid versus runnable?

The matrix must not become a stale hand-maintained checklist. TOML entries are
authoritative only when the target-support audit can prove their stated examples
reach the declared tier or fail with the declared diagnostic.

## Design Direction

Use one common feature registry plus one TOML file per target:

```text
crates/radix/targets/
├── features.toml
├── rust.toml
├── go.toml
├── ts.toml
├── wasm-text.toml
└── llvm-text.toml
```

The per-target files should avoid nested `[targets.foo]` prefixes. Each file is
already scoped to one target.

Representative target file shape:

```toml
id = "go"
name = "Go"
backend = "hir"
extension = "go"

[capabilities]
check = true
build = true
run = false
package = false

[features.ad]
support = "unsupported"
reason = "No Go capability runtime or frame ABI exists yet."

[[features.ad.examples]]
path = "examples/exempla/ad/ad.fab"
expected_tier = "unsupported-diagnostic"
diagnostic_contains = "ad is not supported for Go targets"
primary = true
```

Representative common feature registry:

```toml
[ad]
label = "Capability calls"
kind = "effect"

[optional_parameters]
label = "Optional/default parameters"
kind = "function"

[structured_cape]
label = "Structured cape handlers"
kind = "control"
```

## Support Levels

The initial support-level vocabulary should be small:

- `supported`: target is expected to satisfy the declared example tier.
- `unsupported`: target intentionally rejects the feature with a clear diagnostic.
- `runtime-required`: generated code may compile, but behavior depends on a
  target runtime, host import, provider ABI, or external capability.
- `compile-only`: target can emit/compile the feature but cannot honestly run it
  yet.
- `experimental`: support exists but is not stable enough to gate as fully
  supported.

Do not use the matrix to hide ordinary implementation debt. If a target should
support a feature but currently emits invalid code, keep it as a failing backend
bug until a target-policy decision says otherwise.

## Expected Tiers

Example expectations should use target-appropriate tiers:

- `frontend`: lex/parse/semantic analysis succeeds.
- `emit`: target output is produced.
- `format`: generated target code formats.
- `typecheck`: generated target code passes target typechecking.
- `compile`: generated target code compiles or validates.
- `run`: generated output executes without unexpected failure.
- `behavior`: output matches `.expected` or explicit metadata.
- `unsupported-diagnostic`: compilation fails with the declared diagnostic.
- `runtime-failure`: generated output reaches runtime and fails with an expected
  target-runtime reason.

The audit should allow each target to map these tiers onto its real toolchain.
For example, Go may treat `compile` as `go run` compile success before program
execution, while Wasm may split validation, instantiation, and execution in its
own e2e harness.

## Non-Negotiable Rules

- TOML is the editable source of truth for target support claims.
- TOML is not trusted unless the audit passes.
- Every `supported`, `runtime-required`, `compile-only`, or `experimental`
  feature should have at least one example unless there is a documented reason.
- Every `unsupported` feature should have either an example proving the explicit
  diagnostic or a documented reason why no exemplar exists yet.
- Unknown feature keys fail validation.
- Missing example paths fail validation.
- Expected failures that start passing fail validation until metadata is updated.
- Supported examples that regress fail validation.
- The audit must distinguish target policy gaps from backend bugs.
- Do not require non-Rust target support files to be complete until their
  backend factory sessions have produced truthful baselines.

## Factory Phases

This goal is intentionally staged. The first phases create the skeleton and prove
the approach with Rust. Go, Wasm, and TypeScript target files should wait until
their active factory sessions have finished or reached stable baseline ledgers.

### Phase 0: Schema and Skeleton

Create the target metadata directory and schema parser/validator without trying
to fully classify every target.

Deliverables:

- `crates/radix/targets/features.toml`
- `crates/radix/targets/rust.toml` with a small initial subset
- Optional placeholder files for other targets only if they are clearly marked
  incomplete and excluded from strict support audits.
- Rust structs for parsing the TOML using existing repo dependencies or a small
  dependency addition if justified.
- Validation that feature keys exist, support levels are recognized, example
  paths exist, and required fields are present.

Checkpoint:

```bash
cargo test -p radix target_support_schema -- --nocapture
cargo test -p radix
```

### Phase 1: Rust Support Audit Proof

Use Rust as the first audited target because the Rust e2e harness is already
truthful and currently passes the exemplar corpus.

Deliverables:

- Expand `crates/radix/targets/rust.toml` with high-value target-divergent
  features:
  - `ad`
  - alternate exits / `iace`
  - structured `cape`
  - async / `futura` / `cede`
  - cursor
  - optional parameters
  - collections
  - dynamic `ignotum`
  - package build support
- Add `target_support_audit` coverage that runs or reuses target-tier probes for
  Rust examples.
- Ensure unsupported or runtime-required Rust entries are verified by explicit
  diagnostics or expected runtime behavior.

Checkpoint:

```bash
cargo test -p radix target_support_audit -- --nocapture
cargo test -p radix exempla_rust_e2e -- --ignored --nocapture
cargo test -p radix
```

### Phase 2: Reporting Surface

Expose the TOML-backed support matrix in developer-facing output.

Deliverables:

- Extend `radix targets` or add `radix targets --verbose` if CLI shape permits.
- Print support levels by feature for each loaded target.
- Keep the existing concise target capability output stable unless a CLI design
  phase explicitly changes it.
- Add a deterministic text or JSON-ish test fixture for the report.

Checkpoint:

```bash
cargo test -p radix cmd_targets -- --nocapture
cargo run -p radix --bin radix -- targets
cargo test -p radix
```

### Phase 3: Generated Docs Table

Generate or maintain a Markdown support table from TOML metadata.

Deliverables:

- A generated or checked-in Markdown table under `docs/` or
  `docs/factory/target-support-matrix/`.
- Columns for Rust, Go, TypeScript, Wasm text, and LLVM text once target files
  exist.
- Symbols or text for `supported`, `unsupported`, `runtime-required`,
  `compile-only`, and `experimental`.
- A test or script that catches docs drift if the table is checked in.

Checkpoint:

```bash
cargo test -p radix target_support
./scripta/check-markers
```

### Phase 4: Go Target File After Go Factory Stabilizes

Wait until the Go codegen factory has produced a truthful baseline and any
initial harness-honesty fixes.

Inputs:

- `/Users/ianzepp/work/ianzepp/faber-go-codegen`
- `docs/factory/go-codegen/baseline-ledger.md`
- Current Go e2e expected-failure metadata and pass/fail clusters.

Deliverables:

- `crates/radix/targets/go.toml`
- Go feature examples with expected tiers.
- Audit integration for Go target tiers.

Checkpoint:

```bash
cargo test -p radix target_support_audit -- --nocapture
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
cargo test -p radix
```

### Phase 5: Wasm Target File After Wasm Factory Baseline

Wait until the Wasm codegen factory has established a tiered e2e harness and
baseline ledger.

Inputs:

- `/Users/ianzepp/work/ianzepp/faber-wasm-codegen`
- `docs/factory/wasm-codegen/baseline-ledger.md`
- The final tier names and harness command chosen by that session.

Deliverables:

- `crates/radix/targets/wasm-text.toml`
- Wasm feature examples with tiered expectations.
- Audit integration that understands Wasm compile-valid versus instantiate/run
  tiers.

Checkpoint:

```bash
cargo test -p radix target_support_audit -- --nocapture
cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture
cargo test -p radix
```

### Phase 6: TypeScript Target File After TS Factory Baseline

Wait until the TypeScript codegen factory has established its e2e harness and
toolchain baseline.

Inputs:

- `/Users/ianzepp/work/ianzepp/faber-ts-codegen`
- `docs/factory/ts-codegen/baseline-ledger.md`
- The final TypeScript formatter/typechecker/runtime command choices.

Deliverables:

- `crates/radix/targets/ts.toml`
- TypeScript feature examples with expected tiers.
- Audit integration for emit, format, typecheck, run, and behavior tiers.

Checkpoint:

```bash
cargo test -p radix target_support_audit -- --nocapture
cargo test -p radix exempla_ts_e2e -- --ignored --nocapture
cargo test -p radix
```

### Phase 7: LLVM Text and Future Low-Level Targets

Classify LLVM text only after the MIR/Wasm work clarifies how low-level probe
targets should report support.

Deliverables:

- `crates/radix/targets/llvm-text.toml`
- Minimal audited feature subset, probably focused on MIR compile-valid probes.
- Explicit notes that this is a probe target, not native executable codegen.

Checkpoint:

```bash
cargo test -p radix mir
cargo test -p radix target_support_audit -- --nocapture
```

## Data Model Guidance

Prefer plain, boring TOML. Avoid embedding Rust enum names in metadata.

Suggested target entry shape:

```toml
[features.optional_parameters]
support = "supported"
reason = "Optional/default parameters lower for this target."

[[features.optional_parameters.examples]]
path = "examples/exempla/functio/optionalis.fab"
expected_tier = "run"
primary = true
```

Suggested unsupported entry shape:

```toml
[features.structured_cape]
support = "unsupported"
reason = "Structured cape handlers are not emitted by this backend yet."

[[features.structured_cape.examples]]
path = "examples/fixtures/target-support/structured-cape.fab"
expected_tier = "unsupported-diagnostic"
diagnostic_contains = "structured cape handlers"
primary = true
```

If a feature lacks a good current exemplar, add a small target-support fixture
under an explicit fixture directory rather than distorting the public exempla
corpus.

## Completion Criteria

This factory goal is complete when:

- The TOML schema is compiler-owned and validated.
- Rust support metadata is audited against real behavior.
- `radix targets` or an equivalent report can show support levels from TOML.
- At least one generated or drift-checked Markdown table exists.
- Go, Wasm, and TypeScript phases are either completed from their stabilized
  factory baselines or explicitly left pending with documented blockers.
- The audit fails loudly on stale support claims.

## Deferred Work

- Do not add Zig, Swift, COBOL, or other future targets in this goal unless a
  separate backend factory has been created.
- Do not move all target-policy checks out of Rust code immediately. The TOML
  matrix can first audit and report policy, then later feed driver diagnostics
  where it clearly reduces duplication.
- Do not require every parser keyword to have an entry on day one. Start with
  features that affect target divergence and e2e classification.
