LLM Guidance:
  1. Use `faber` for packages and day-to-day workflows; use `radix` for single-file
     compiler phase inspection when you do not need `faber.toml` or Cargo integration.
  2. Discover commands:
     faber --help
     faber <command> --help
  3. Establish project shape before compiling:
     faber init my-app
     faber targets
  4. Check and emit before run/test on a new package:
     faber check .
     faber build .
  5. Language reference (glyphs, keywords, grammar terms):
     faber explain --list
     faber explain functio
     faber explain --json functio
  6. Compatibility aliases (`lex`, `parse`, `hir`, `cli-ir`, `emit`) match `radix`
     for single-file inspection; package paths still prefer explicit `faber` commands.

Common flows:
  New package:
    faber init hello
    faber check hello
    faber build hello
    faber run hello
    faber test hello

  Check or build current directory:
    faber check .
    faber build . -t rust -o dist/
    faber build . --release

  Run with arguments forwarded after `--`:
    faber run . -- --flag value

  Test selection (Faber probanda metadata maps to Cargo harness):
    faber test .
    faber test . --name my_case
    faber test . --suite suite/path
    faber test . --tag slow -- --nocapture

  Explain corpus:
    faber explain --list
    faber explain --category keywords
    faber explain --search return
    faber explain ≡
    faber explain --json functio

  Single-file check (non-package):
    faber check examples/exempla/salve-munde.fab

  Package-aware check/emit (directory or faber.toml):
    faber check path/to/pkg
    faber emit -t rust path/to/pkg

  Inspect phases without a package (radix-compatible aliases):
    faber lex examples/exempla/salve-munde.fab
    faber parse examples/exempla/salve-munde.fab

Output contract:
  - `build`: path of written artifact on stdout; diagnostics on stderr
  - `check` / `emit` (package): compiler diagnostics on stderr; emit writes source
  - `run` / `test`: forwards child process exit code; build diagnostics on stderr
  - `init`: manifest path on stdout
  - `explain`: human text on stdout; `--json` for one term only
  - `lex`, `parse`, `hir`, `cli-ir`: JSON on stdout (same as `radix`)
  - `targets`: capability rows on stdout
  - errors: stderr with hints (e.g. `faber explain --list`)