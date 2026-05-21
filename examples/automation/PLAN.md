# Faber Automation Example Plan

## Interpreted Problem

Build `examples/automation/` as a real CLI-shaped Faber example, using `../automations/` as the reference workload. The goal is to expose what Faber can do today and what compiler/runtime support is still missing before a faithful automation executor can replace the shell implementation.

The reference executor scans `*/SKILL.md`, parses TOML-style front matter, checks automation status and recurrence, manages state and locks, builds an agent prompt, dispatches to `codex`, `opencode`, or `ollama`, logs output, and updates state after successful runs.

## Normalized Spec

Create an example package with:

- A Faber CLI entry point under `examples/automation/main.fab`.
- Command modules under `examples/automation/commands/`.
- Fixture automation files under `examples/automation/fixtures/`.
- Documentation that explains how to check, emit, and evolve the example.

The initial implementation should be a skeleton, not a full executor. It should stay inside current language and runtime capabilities.

## Repo-Aware Baseline

Current facts from the active `radix` compiler:

- Declarative CLI syntax with `@ cli`, `@ imperium`, `@ optio`, `@ operandus`, and `@ imperia` exists.
- Runnable CLI generation is Rust-only.
- Rust package compilation supports package-local CLI module mounts.
- `solum`, `processus`, `json`, `toml`, and `yaml` stdlib declarations exist.
- Rust runtime support exists for several HAL modules, including filesystem and process support.
- `tempus` currently lacks Rust runtime support, so accurate wall-clock scheduling and stale-lock aging should not be faked in the first slice.

## Stage Graph

### Stage 1: Inventory CLI

Implement a compiling CLI skeleton with commands for:

- `inventory list`
- `inventory show <id>`
- `inventory check <id>`
- `runner dry-run <id>`

Acceptance:

- Package checks with `cargo run --manifest-path Cargo.toml -p faber -- check --package examples/automation/main.fab`.
- Rust can be emitted with `cargo run -p radix --bin radix -- emit -t rust --package examples/automation/main.fab`.
- README documents current limitations.

### Stage 2: Real Metadata Parsing

Replace placeholder output with actual fixture scanning and metadata parsing:

- Find `*/SKILL.md`.
- Extract front matter.
- Parse metadata with `toml.solve()` or a focused front-matter helper.
- Validate required fields: `id`, `kind`, `execute`, `status`, `rrule`, `model`, `reasoning_effort`, and `cwds`.

Acceptance:

- `inventory list` prints records from fixture data.
- `inventory check` reports missing/invalid fields without crashing.

### Stage 3: Dry-Run Execution Model

Model execution without starting agent processes:

- Load fixture or repository `AGENTS.md`.
- Extract skill body.
- Build the prompt.
- Resolve executor command for `codex`, `opencode`, and `ollama`.
- Print the command, cwd, log path, lock path, and state key that would be used.

Acceptance:

- `runner dry-run <id>` proves prompt assembly and dispatch selection.
- No state, lock, or process side effects happen unless explicitly added later.

### Stage 4: Runtime Hardening For Parity

Add missing compiler/runtime support required by the real executor:

- Rust `norma:hal/tempus` support for current epoch seconds and milliseconds.
- Status-aware process execution that captures exit code and stderr/stdout paths.
- Safer filesystem operations for lock files and atomic state writes.
- Tests for generated Rust that exercises these runtime calls.

Acceptance:

- Faber can express the same control-flow decisions as `../automations/executor.sh` without shelling out for basic logic.

### Stage 5: Parity Executor

Implement real scheduled execution:

- `run --force`
- Status filtering
- BYMINUTE/BYHOUR/BYDAY checks
- Minimum interval enforcement
- Stale lock cleanup
- Per-automation logs
- State update only after success

Acceptance:

- Behavior matches the reference executor for fixture cases.
- The example remains documented as an example, not the production automation executor, until manually promoted.

## Epic Candidates And Scopable Issues

- `automation-example-skeleton`: create the package tree, docs, stub CLI, and fixtures.
- `automation-frontmatter-parser`: implement and test metadata extraction.
- `automation-dry-run-runner`: build prompt assembly and dispatch rendering.
- `rust-tempus-runtime`: add Rust runtime support for `norma:hal/tempus`.
- `processus-status-api`: add status-aware process execution support.
- `automation-parity-fixtures`: encode reference executor behavior as example fixtures.

## Checkpoints

- Checkpoint 1: Skeleton package exists and emits Rust.
- Checkpoint 2: Fixture metadata is parsed and validated.
- Checkpoint 3: Dry-run output mirrors the production executor command shape.
- Checkpoint 4: Runtime gaps are closed in `norma`.
- Checkpoint 5: Real execution is possible behind explicit commands.

## Gate Plan

Use these commands as gates while the example evolves:

```bash
cargo run --manifest-path Cargo.toml -p faber -- check examples/automation/main.fab
cargo run -p radix --bin radix -- emit -t rust --package examples/automation/main.fab
cargo run -p faber -- check --package examples/automation/main.fab
```

When runtime/compiler behavior changes, add targeted Rust tests under `crates/radix/src`.

## Open Questions

- Should the final CLI expose top-level commands (`list`, `show`, `check`, `run`) or keep module-mounted commands (`inventory list`, `runner dry-run`) to better demonstrate `@ imperia`?
- Should production parity use TOML front matter exactly, or should automation skills migrate to a stricter metadata format before Faber becomes the executor?
- What is the right Faber stdlib API for status-aware process execution: extend `processus.exsequi`, add a new return-record function, or model a process handle more fully?
