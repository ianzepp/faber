# Faber CLI Product Plan

Internal planning artifact for promoting `compilers/radix-rs` from an implementation-named debug CLI into the public-facing packaged Faber compiler command.

Last updated: 2026-04-10

---

## 1. Interpreted Problem

### Claimed problem

Define the public product/CLI surface for the compiler, assuming:

- `radix-rs` becomes the canonical implementation
- the crate upgrades to `clap`
- the command surface takes a clean break from the current debug-first `radix-rs` CLI

### Inferred actual problem

The compiler is reaching the point where the bottleneck is no longer only backend correctness. The next product step is to decide what the public contract of the compiler is:

- what users type
- which commands are stable and public
- which surfaces remain developer-only
- how file/package execution works
- how targets and diagnostics are expressed

### Confidence

High.

---

## 2. Product Decision

### Canonical public command name

Use:

- `faber`

Do not use `radix-rs` as the public product command.

### Internal implementation naming

Keep the compiler implementation named `radix-rs` internally.

This plan does **not** require renaming:

- the `compilers/radix-rs` project
- the Rust crate/package identity
- the internal binary produced during normal development

The `faber` name is the public packaged command exposed later through distribution surfaces such as a Homebrew tap.

### Why

`radix-rs` is the implementation name, not the language/tool identity. The public installed command should be named after the product, while the implementation can keep its internal backend-oriented identity.

### Alternative

`fabercc`

### Why rejected for now

`fabercc` only becomes attractive if the ecosystem is expected to split into multiple low-level compiler executables with a separate top-level `faber` meta-tool. The repo is not there yet. A clean public command should be the shortest obvious name.

---

## 3. Current State

### `radix-rs`

Current CLI in [main.rs](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/main.rs):

- `lex`
- `parse`
- `hir`
- `check`
- `emit`
- `emit-package`

Characteristics:

- hand-rolled `std::env::args()` parsing
- phase-debug orientation
- mixed file/stdin support
- explicit package command
- target-oriented emission

### `nanus-go`

Current CLI in [main.go](/Users/ianzepp/github/ianzepp/faber/compilers/nanus-go/main.go):

- `lex`
- `parse`
- `emit`

Characteristics:

- stdin/stdout microcompiler
- no first-class semantic `check`
- no package command
- minimal options
- implementation aid, not product CLI

### Decision

Promote `radix-rs`, not `nanus-go`, into the canonical implementation behind the public command.

---

## 4. Product Goals

The public `faber` command should:

1. feel like a normal language tool, not an internal compiler lab
2. unify file and package inputs under one model
3. keep target selection explicit
4. keep diagnostics explicit and scriptable
5. separate stable public commands from developer inspection commands
6. avoid compatibility shims that preserve the old `radix-rs` surface forever

---

## 5. Surface Model

Split the CLI into two layers.

### Public user layer

Stable, product-facing commands:

1. `check`
2. `build`
3. `run`
4. `emit`
5. `test`
6. `targets`
7. `doctor`

### Developer inspection layer

Debug/compiler-workbench commands:

1. `lex`
2. `parse`
3. `hir`
4. optionally `ast` later if a non-JSON debug view is useful

### Design rule

The public help output should lead with product verbs. Inspection verbs should exist, but should not define the identity of the tool.

---

## 6. Proposed Command Surface

### `faber check <input>`

Purpose:

- parse + analyze without producing target output

Use cases:

- CI
- editor integration
- fast validation
- target-compatibility checks

Flags:

- `-t, --target <target>`
- `--package`
- `--permissive`
- `--warnings-as-errors`
- `--diagnostic-format <human|short|json>`
- `--color <auto|always|never>`
- `-q, --quiet`
- `-v, --verbose`

Notes:

- `--target` matters because target policy diagnostics may differ
- `--permissive` should remain explicit if kept at all

### `faber build <input>`

Purpose:

- compile a file or package into output files on disk

Use cases:

- local builds
- release builds
- multi-file/package compilation

Flags:

- `-t, --target <target>`
- `-o, --out-dir <dir>`
- `--package`
- `--diagnostic-format <human|short|json>`
- `--color <auto|always|never>`
- `-q, --quiet`
- `-v, --verbose`

Notes:

- `build` should be the default production-oriented verb
- this replaces the conceptual need for `emit-package`

### `faber run <input>`

Purpose:

- compile and execute a file or package

Use cases:

- examples
- local experimentation
- language onboarding

Flags:

- `-t, --target <target>`
- `--package`
- `--diagnostic-format <human|short|json>`
- `--color <auto|always|never>`
- `-- <program args...>`

Notes:

- only enable for targets with a runnable path
- if runtime support differs by target, fail explicitly

### `faber emit <input>`

Purpose:

- produce emitted target source, primarily for inspection or integration

Use cases:

- backend debugging
- generated-source inspection
- golden-test workflows

Flags:

- `-t, --target <target>`
- `-o, --out <file>`
- `--stdout`
- `--package`
- `--diagnostic-format <human|short|json>`

Notes:

- `emit` remains useful, but should read as an advanced command rather than the top-level product story

### `faber test <input>`

Purpose:

- run Faber test/proba surfaces as a first-class product command

Use cases:

- project-local test runs
- CI

Flags:

- `-t, --target <target>`
- `--package`
- `--filter <pattern>`
- `--diagnostic-format <human|short|json>`
- `-q, --quiet`
- `-v, --verbose`

Notes:

- this should become the user-facing wrapper around `proba` execution semantics
- implementation can start narrow and grow later

### `faber targets`

Purpose:

- show supported targets and capability notes

Output should include:

- target names
- whether they support `check`
- whether they support `build`
- whether they support `run`
- whether they have explicit gated surfaces

### `faber doctor`

Purpose:

- environment and toolchain validation

Possible checks:

- compiler binary version
- selected target availability
- runtime tool presence where needed
- output directory/config sanity

---

## 7. Developer Commands

These remain supported, but clearly positioned as internal/developer tooling:

- `faber lex <input>`
- `faber parse <input>`
- `faber hir <input>`

Rules:

1. keep JSON output support
2. do not let these commands dictate the public product identity
3. keep them documented under a separate “developer inspection” help section

---

## 8. Commands to Remove or Replace

### Remove as public surface

- `emit-package`

### Replace with

- `build <dir>`
- `emit <dir> --package`

### Why

File-vs-package is input shape, not a top-level verb. A stable CLI should not multiply commands for what is fundamentally the same operation.

### Do not preserve forever

- the `radix-rs` name as the public packaged command contract
- the current help text ordering
- manual target parsing semantics if `clap` makes them more explicit

---

## 9. Input Model

### Accepted inputs

The new CLI should accept:

- a single `.fab` file
- a package directory

### Resolution rules

1. if input is a file, compile that file
2. if input is a directory, compile the package rooted there
3. if the path is ambiguous or package detection is incomplete, `--package` forces package mode

### Why

This makes the product mental model simpler:

- users choose the operation
- the tool determines the input kind

They do not need separate verbs for single-file and package compilation.

---

## 10. Target Model

Targets should become a typed `clap::ValueEnum`, with one canonical vocabulary:

- `rust`
- `go`
- `ts`
- `faber`

Possible short aliases can exist, but they should be secondary.

### Recommendation

Accept aliases:

- `rs` -> `rust`
- `fab` -> `faber`

But print canonical names in help and diagnostics.

### Why

This reduces drift between help text, docs, and implementation.

---

## 11. Global Flags

Recommended global flags:

- `-v, --verbose`
- `-q, --quiet`
- `--color <auto|always|never>`
- `--diagnostic-format <human|short|json>`

Optional later:

- `--cwd <path>`
- `--config <file>`

### Decision

Prefer `--diagnostic-format` over a single overloaded `--json`.

### Why

`--json` is too narrow once there are multiple machine-readable output shapes. Diagnostics need an explicit format contract.

---

## 12. Clap Structure

Recommended high-level structure:

```rust
#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true, value_enum, default_value = "human")]
    diagnostic_format: DiagnosticFormat,

    #[arg(long, global = true, value_enum, default_value = "auto")]
    color: ColorMode,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(clap::Subcommand)]
enum Command {
    Check(CheckArgs),
    Build(BuildArgs),
    Run(RunArgs),
    Emit(EmitArgs),
    Test(TestArgs),
    Targets,
    Doctor,
    Lex(InputArgs),
    Parse(InputArgs),
    Hir(InputArgs),
}
```

Shared argument structs:

- `InputArgs`
- `TargetArgs`
- `OutputArgs`

### Why

This keeps:

- target parsing consistent
- help text consistent
- package/file behavior centralized

---

## 13. Compatibility Policy

This migration should be a clean break.

### Recommendation

1. keep `radix-rs` as the internal implementation/compiler project name
2. introduce `faber` as the canonical packaged user command
3. keep any direct `radix-rs` user entrypoint only for a short transition period if needed
4. print explicit deprecation notices from the old user-facing entrypoint if one ships
5. remove old compatibility paths after the transition window

### Do not do

- preserve every old `radix-rs` subcommand spelling forever
- keep `emit-package` alive indefinitely
- silently reinterpret old commands without explicit diagnostics

### Why

The repo already has enough compiler complexity. Carrying multiple CLI personalities forward would create product ambiguity and maintenance drag.

---

## 14. Rollout Plan

### Stage A: Introduce typed CLI parsing

Scope:

- adopt `clap`
- preserve current behavior internally while moving parsing to typed structs
- keep internal implementation naming unchanged during the parser migration

Exit criteria:

- existing commands still function
- target parsing is centralized
- help output is generated by `clap`
- no crate/package rename is required to complete the stage

### Stage B: Reshape public verbs

Scope:

- add `build`, `run`, `test`, `targets`, `doctor`
- unify file/package handling
- remove `emit-package` from the main surface

Exit criteria:

- product-facing verbs exist and are documented
- old package-specific command is deprecated or removed

### Stage C: Separate public vs developer help

Scope:

- public help leads with product verbs
- phase-debug verbs move into a distinct section or wording

Exit criteria:

- `faber --help` reads like a language tool, not a compiler lab shell

### Stage D: Rename and release

Scope:

- ship `faber` binary
- optionally keep `radix-rs` as a transitional alias
- update docs/scripts/README examples

Exit criteria:

- docs and scripts primarily reference `faber`
- the public command surface is stable enough to announce

---

## 15. Script and Docs Consequences

This CLI break will require coordinated updates in:

- root `package.json` scripts
- compiler README files
- docs that still teach `radix-rs`
- examples that show old command names
- CI commands that invoke the binary directly

### Important rule

Update examples and scripts to the new public name intentionally. Do not leave a split-brain state where docs say `faber` but scripts still treat `radix-rs` as primary.

---

## 16. Open Questions

1. Should `run` default to a preferred target, or require `--target` until runtime policy is settled?
2. Should `test` be implemented immediately, or land as a reserved command with explicit “not yet implemented” status?
3. Should `faber` support a config file in the first public release, or defer that to keep the break small?
4. Should developer commands remain top-level, or move under a namespace like `faber debug parse` later?

Current bias:

- require explicit behavior rather than guessing
- keep debug commands top-level for now
- defer config files until the basic product surface is stable

---

## 17. Recommendation

The next CLI wave should do this:

1. promote `radix-rs` into a public `faber` binary
2. migrate argument parsing to `clap`
3. define the stable public verbs as:
   - `check`
   - `build`
   - `run`
   - `emit`
   - `test`
   - `targets`
   - `doctor`
4. keep `lex`, `parse`, and `hir` as explicit developer inspection commands
5. delete `emit-package` from the long-term public surface

That gives the project a compiler product surface that matches where the implementation is going, without dragging the debug-first CLI shape into the public contract forever.
