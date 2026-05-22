# Automation Example

This package is a Faber CLI example modeled after the sibling `../automations/` repository.

It is intentionally a skeleton. The first goal is to exercise Faber's current CLI/package shape while documenting the runtime gaps that need to close before a faithful automation executor is practical.

## Current Commands

```text
automation inventory list
automation inventory show <id>
automation inventory check <id>
automation runner dry-run <id>
```

The current command bodies print placeholder output. The fixture files under `fixtures/` define the reference data shape for later parsing and dry-run behavior.

## Validate

From the repository root:

```bash
cargo run --manifest-path Cargo.toml -p faber -- check --package examples/automation/main.fab
cargo run -p faber -- emit -t rust --package examples/automation/main.fab
```

Runnable CLI generation is Rust-only in the active compiler. See `PLAN.md` for the staged path from this skeleton to a closer executor port.
