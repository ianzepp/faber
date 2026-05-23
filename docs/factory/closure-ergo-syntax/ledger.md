# Closure Ergo Syntax Ledger

## Implementation Notes

- `∴` lexes to the existing `Ergo` token, so all current single-statement `ergo` tail sites accept the symbolic spelling without a second parser path.
- Compact closure parsing is speculative but bounded to `type name`-shaped starts, preventing ordinary expressions such as `si ok ergo ...` from being reinterpreted as closures.
- Compact closure bodies accept a single expression or `fac { ... } cape ...`; `fac ... dum` is rejected in closure bodies.
- Legacy `clausura ... : ...` remains accepted as a compatibility form.
- Closure AST records explicit `⇥` syntax. Full closure error-channel type semantics remain a follow-up; current HIR closure function types still carry `err: None`.

## Edited Surfaces

- Lexer: `crates/radix/src/lexer/scan.rs`
- Parser: `crates/radix/src/parser/expr.rs`, `crates/radix/src/parser/stmt.rs`
- AST / visitors / preflight scans: `crates/radix/src/syntax/ast.rs`, `crates/radix/src/syntax/visit.rs`, `crates/radix/src/driver/mod.rs`
- Lowering and typecheck: `crates/radix/src/hir/lower/expr.rs`, `crates/radix/src/hir/lower/stmt.rs`, `crates/radix/src/semantic/passes/typecheck/call.rs`
- Faber printer: `crates/radix/src/codegen/faber/expr.rs`
- Docs and examples: `EBNF.md`, `explain/clausura.md`, `explain/therefore.md`, `examples/exempla/clausa/clausa.fab`, `examples/exempla/morphologia/morphologia.fab`

## Validation

- `cargo test -p radix -- --nocapture`
- `cargo run -p faber -- check examples/exempla/clausa/clausa.fab`
- `cargo run -p faber -- check examples/exempla/morphologia/morphologia.fab`
- `cargo run -p radix --bin radix -- emit -t faber examples/exempla/clausa/clausa.fab`
