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

The package is manifest-backed by `faber.toml`; use the directory path for normal commands.

## Validate

From the repository root:

```bash
cargo run -p faber -- check examples/automation
cargo run -p faber -- build examples/automation
cargo run -p faber -- run examples/automation -- inventory list
cargo run -p faber -- run examples/automation -- runner dry-run sample-automation
cargo run -p faber -- emit -t rust --package examples/automation
```

Runnable CLI generation is Rust-only in the active compiler. See `PLAN.md` for the staged path from this skeleton to a closer executor port.
