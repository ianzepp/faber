LLM Guidance:
  1. Pick the right tool:
     - `radix` — single-file compiler phase inspection and codegen experiments
     - `faber` — packages (`faber.toml`), build/run/test, and language explain
  2. Discover commands:
     radix --help
     radix <command> --help
     faber --help
  3. Start with capability and shape before mutating:
     radix targets
     radix parse examples/exempla/salve-munde.fab
     radix cli-ir path/to/cli.fab
  4. Prefer inspection JSON commands when automating lexer/parser/HIR/MIR/CLI-IR work.
  5. Use `faber check`, `faber build`, `faber run`, and `faber test` for package workflows.
     `radix` rejects package paths and points you at `faber`.

Common flows:
  Quick sanity on one file:
    radix check examples/exempla/salve-munde.fab
    radix emit -t rust examples/exempla/salve-munde.fab

  Phase debugging (machine-readable stdout):
    radix lex examples/exempla/salve-munde.fab
    radix parse examples/exempla/salve-munde.fab
    radix hir examples/exempla/salve-munde.fab
    radix mir examples/exempla/salve-munde.fab
    radix cli-ir examples/exempla/salve-munde.fab

  Stdin instead of a path (omit path or use `-`):
    cat program.fab | radix check
    cat program.fab | radix emit -t faber

  Expanded diagnostic records:
    radix check --diagnostics main.fab
    radix emit --diagnostics -t rust main.fab

  Permissive check while imports are incomplete:
    radix check --permissive main.fab

  Emit with post-processing (best-effort; warns if tools missing):
    radix emit -t rust --format --linter main.fab

  Inspect Faber `@ cli` surface:
    radix cli-ir src/main.fab

Output contract:
  - `lex`, `parse`, `hir`, `cli-ir`: JSON on stdout; exit 1 on failure
  - `mir`: deterministic text dump on stdout
  - `check`: human lines on stderr; `ok: <file>` on success; exit 1 on failure
  - `check --diagnostics`: expanded diagnostic records on stderr
  - `emit`: generated source on stdout; diagnostics on stderr
  - `targets`: one capability row per backend on stdout
  - errors: action-oriented messages on stderr (package paths redirect to `faber`)