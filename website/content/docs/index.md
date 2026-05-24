+++
title = "Language Reference"
description = "Faber Romanus language reference, CLI, targets, and guides. Sources of truth live in the repository (EBNF.md, explain/, docs/, examples/)."
order = 10
+++

# Language & Tool Reference

This section is the curated public documentation for Faber.

**Primary sources in this repository (do not duplicate prose here without good reason):**

- [EBNF.md](../EBNF.md) — The formal grammar and specification commentary.
- [explain/](../explain/) — Embedded per-concept reference corpus. `faber explain <topic>` surfaces these.
- [docs/](../docs/) — Release notes, epics, factory plans, and project documentation.
- [examples/exempla/](../examples/exempla/) — Curated example programs.
- [stdlib/norma/](../stdlib/norma/) — Standard library definitions and @ verte target mappings.
- CLI behavior: `cargo run -p faber -- --help` and `faber <subcommand> --help`.
- Target matrix: `cargo run -p faber -- targets`.

## Planned Pages

- [Grammar Reference](grammar.md) — Synced from EBNF.md
- [Fundamenta](fundamenta.md) — Core concepts
- [Typi](typi.md) — Type system (`T ∪ nihil`, `sponte`, `ignotum`, etc.)
- [Operatores](operatores.md) — Operators
- [Structurae](structurae.md) — Data structures (lista, tabula, genus, etc.)
- [Regimen](regimen.md) — Control flow (si, dum, itera, elige, etc.)
- [Functiones](functiones.md) — Functions and annotations
- [Importa](importa.md) — Modules and imports
- [Errores](errores.md) — Error handling (fac/cape, throw, etc.)
- [CLI](cli.md) — The `faber` and `radix` command-line interfaces
- [Targets](targets.md) — Supported backends and their capabilities
- [Packages & Manifests](manifest.md) — `faber.toml`, `faber init`, `faber build` etc.
- [Testing](test.md) — `faber test` framework
- [Examples](examples.md) — Curated programs with explanations

**Note:** Many of these will be thin curated wrappers or direct imports/generations during the content refresh pass. The goal is correctness against the live compiler, not exhaustive hand-written duplication.

See the [website refresh plan](faber-website-refresh-plan.md) for the detailed migration checklist.
