# Automation Example Gaps

This document tracks the first gaps exposed by trying to model the sibling `../automations/` executor as a Faber CLI app.

The point is not to hide shortcomings. The point is to name them clearly enough that each one can become a scoped compiler, stdlib, runtime, or example task.

## 1. Package-Aware Checking

Current observation:

- `cargo run --manifest-path radix/Cargo.toml -p radix -- check examples/automation/main.fab` checks the root file.
- `cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t rust --package examples/automation/main.fab` is the meaningful gate for mounted command modules.

Gap:

The `check` command does not currently provide the same obvious package-level confidence as Rust package emit.

Possible directions:

- Add `check --package <entry.fab>`.
- Make `check` follow package-local imports when the input is a package entry point.
- Keep package validation documented as `emit -t rust --package` until a proper package check exists.

Near-term recommendation:

Keep using Rust package emit as the package gate for this example, then decide whether `check --package` belongs in the compiler CLI.

## 2. Front Matter Extraction

Current observation:

The production automation files are Markdown files with TOML-like front matter:

```markdown
---
id = "sample-automation"
status = "PAUSED"
---

Prompt body...
```

Faber has TOML parsing declarations, but not a standard Markdown front matter extraction helper.

Gap:

Before `toml.solve()` can parse metadata, Faber code needs to split a Markdown file into:

- metadata text
- prompt body
- invalid or missing front matter diagnostics

Possible directions:

- Implement a small local helper in `examples/automation`.
- Add a reusable `norma` front matter or Markdown utility.
- Change automation records to store metadata in a stricter sidecar file.

Near-term recommendation:

Implement the local helper first. Promote to stdlib only after the shape proves reusable.

## 3. Filesystem Traversal

Current observation:

The production executor scans `*/SKILL.md`. The `solum` HAL has directory listing, path joining, existence checks, reads, writes, and metadata.

Gap:

There is no high-level glob or recursive traversal helper in the current Faber stdlib surface.

Possible directions:

- Handwrite one-level scanning for `*/SKILL.md` in the example.
- Add a reusable `solum.glob()` or `solum.inveni()` API later.
- Keep traversal shallow because the production automation repo uses a deliberately shallow shape.

Near-term recommendation:

Handwrite shallow scanning. Do not add glob semantics until a second real use case needs them.

## 4. Time Runtime For Rust

Current observation:

Scheduling needs current epoch time, elapsed interval checks, lock aging, and timestamped log names. The `tempus` stdlib declaration exists, but Rust runtime support is not currently present.

Gap:

Full schedule parity should not be implemented until the Rust target can call stable time functions through `norma:hal/tempus`.

Possible directions:

- Add Rust `norma::hal::tempus` support for epoch seconds and milliseconds.
- Start with only the functions needed by the automation executor.
- Delay async sleep and callback scheduling until a separate use case needs them.

Near-term recommendation:

For Stage 2, avoid real schedule enforcement. For Stage 4, add the minimal Rust time runtime slice.

## 5. Process Result API

Current observation:

The production executor needs:

- command arguments
- cwd
- stdout log path
- stderr capture/filtering
- exit code
- success/failure branching

The current process surface is useful, but `processus.exsequi()` returns stdout and does not model the full result object needed by the executor.

Gap:

Faber needs a status-aware process API before a real automation runner can update state only after successful completion.

Possible directions:

- Add a new function that returns a result record with `codex`, `stdout`, and `stderr`.
- Extend process spawning so callers can redirect stdout and stderr to files.
- Keep simple `exsequi()` unchanged for shell-output convenience.

Near-term recommendation:

Do not overload `exsequi()`. Add a separate status-aware API when implementing the real runner.

## 6. Atomic State And Locking

Current observation:

The production executor uses:

- `state.json`
- lock files
- stale lock cleanup
- temp-file-then-rename state writes

Faber has enough filesystem basics to model part of this, but no explicit atomic-write or lock-file helper.

Gap:

Correct lock/state behavior needs either disciplined example-local code or a small stdlib helper.

Possible directions:

- Implement simple example-local lock files first.
- Add `solum.scribeAtomice()` for temp-write-and-rename.
- Add a lock helper only if several examples or tools need the same pattern.

Near-term recommendation:

Keep Stage 3 side-effect-free. Add lock and state writes only after time and process-result support exist.

## First Discussion Set

The first useful decisions are:

1. Should Stage 2 stay local and educational, or should it start hardening stdlib APIs immediately?
2. Should package-level validation become `check --package`?
3. Should front matter parsing remain example-local for now?
4. What should the process result record look like?
5. What is the minimum Rust `tempus` runtime slice needed for scheduler parity?
