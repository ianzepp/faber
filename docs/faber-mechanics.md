# Faber Mechanics Review: `radix-rs`

Review mode only. No compiler code changes were made.

Update:
- This started as a read-only mechanics pass.
- The highest-priority findings around inline lowering and Rust-target exception handling have since been implemented.
- Parser recovery boundaries and parser-local regression coverage have since been improved.
- `PARSE022` now acknowledges inline exits instead of teaching an outdated subset of the grammar.
- The remaining sections below are preserved as review context, but some items are now resolved.

Scope:
- `compilers/radix-rs`
- Five review phases: parser control flow and recovery, AST fidelity, diagnostics quality, resolve/lowering boundary, and late semantic passes

Method:
- Read grammar in `EBNF.md` first
- Inspect the active `radix-rs` pipeline and recent repository direction
- Review each layer in order using the `faber-mechanics` stance: grammar fidelity, local disambiguation, explicit phase boundaries, recovery quality, and preservation of source distinctions

Validation:
- Original review was read-only
- Follow-up parser mechanics work was validated with `cargo test --manifest-path compilers/radix-rs/Cargo.toml`

## Baseline

`radix-rs` is the active compiler center of gravity in this repository.

Evidence:
- CI is scoped to the crate in `/Users/ianzepp/github/ianzepp/faber/.github/workflows/ci.yml`
- Root scripts bias toward `radix-rs` in `/Users/ianzepp/github/ianzepp/faber/package.json`
- Recent git history is dominated by `radix` HIR/codegen/resolve work followed by repo reorganization

One documentation drift is already visible:
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/lib.rs:17` documents `Semantic` before `HIR Lowering`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/README.md:7` correctly places `lower` inside the semantic pass sequence
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/main.rs:12` also reflects the phase-debug CLI view more accurately than `lib.rs`

This is not a runtime bug, but it is mechanical drift at the public contract level.

## Phase 1: Parser Control Flow And Recovery

Mechanical boundary:
- Parser token discipline, predictive disambiguation, and malformed-input recovery

What looks solid:
- The parser is still hand-written and locally predictive in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/mod.rs`
- `parse_ergo_body` keeps the body forms visible instead of collapsing them immediately in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/stmt.rs:97`
- `cape` parsing is centralized in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/stmt.rs:715`

Findings:
- Resolved: recovery synchronization in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/mod.rs` now stops at the full statement restart surface instead of a stale subset, and block parsing now treats `}` as a distinct recovery boundary. This closes the bug where a missing body could consume the next valid statement before recovery had a chance to stop.
- Improved: parser-local regression coverage now exists in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/mod_test.rs`, including direct tests for recovery boundaries and broad keyword-surface parsing. This is no longer a zero-coverage surface.
- `parse_secus_stmt` allows a bare expression-statement fallback when neither block, inline return, nor `ergo` is present in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/stmt.rs:139`. This is permissive and may be correct, but it means `secus` is less structurally explicit than `si` bodies. That asymmetry should be intentional and documented if kept.

Assessment:
- The parser still reads like the grammar in most places, and the worst recovery drift has been corrected. The remaining parser mechanics work is now less about missing basic defenses and more about deepening malformed-input coverage and deciding whether permissive `secus` fallback is intentional.

## Phase 2: AST Fidelity And Syntax Contracts

Mechanical boundary:
- Preserve distinctions the source already made explicit, without forcing downstream reconstruction

What looks solid:
- Control-flow bodies remain structurally distinct as `Block`, `Ergo`, and `InlineReturn` in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/syntax/ast.rs:421`
- `SecusClause` also preserves whether the source used `sin`, block form, statement form, or inline return in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/syntax/ast.rs:436`
- `CapeClause`, `CeterumDefault`, and case-arm spans are all explicit

Findings:
- The AST preserves the distinctions correctly, but several of those distinctions are not honored later. The AST is doing its job; the downstream phases are where blurring starts.
- `InlineReturn` deliberately separates `reddit`, `iacit`, `moritor`, and `tacet` in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/syntax/ast.rs:428`. That is the right source contract and should not be collapsed early.

Assessment:
- The syntax layer is not the main problem. It is carrying more truth than later phases currently respect. That makes it a stable source-of-truth anchor for future mechanics passes.

## Phase 3: Diagnostics Quality

Mechanical boundary:
- Diagnostics should teach the real language rule that failed, not merely report generic parser confusion

What looks solid:
- Parser and semantic diagnostics are cataloged centrally in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/catalog.rs`
- Error-code discipline is explicit and stable

Findings:
- Resolved: `PARSE022` in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/catalog.rs:122` now mentions inline exits, which matches the grammar accepted by `parse_ergo_body` in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/stmt.rs:97`.
- `ParseErrorKind` is fairly granular in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/error.rs`, but a large amount of parser behavior still routes through generic `Expected` or `MissingBlock`. That is serviceable, not polished.
- Late semantic finalization uses `MissingTypeAnnotation` as a catch-all for unresolved inference outcomes in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs:225`, `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs:248`, `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs:274`, and `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs:304`. That blurs true missing annotations together with deeper inference failures or unsupported lowering outcomes.

Assessment:
- Diagnostics are structurally organized. The parser help text is closer to the true grammar now, but late-phase error classification still drifts away from the real failure cause.

## Phase 4: Resolve And Lowering Boundary

Mechanical boundary:
- Preserve syntax-phase truth into HIR and only normalize where the semantics are genuinely equivalent

What looks solid:
- Resolve still walks the AST directly and preserves lexical scoping explicitly in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/resolve.rs`
- Lowering keeps its own local-binding scope and synthetic `DefId` discipline in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/mod.rs`

High-severity findings:
- Resolved: inline `iacit` and `moritor` were being lowered as returns. They now lower to `Throw` and `Panic`, and `tacet` now remains a no-op instead of becoming `Redde(None)`.
- Resolved: Rust-target compilation now rejects exception constructs directly instead of drifting into partial pseudo-support.
- `resolve` walks type aliases twice: once through ordinary `resolve_stmt` processing and again through `resolve_alias_types` in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/resolve.rs:65` and `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/resolve.rs:79`. That may be functionally tolerable, but it weakens the phase contract by mixing ordinary name resolution with special fixpoint alias lowering instead of drawing a cleaner boundary.
- `cape` is preserved in the AST and resolve layer, but several lowerings flatten it into extra statements rather than preserving it as a first-class control-flow construct. For example, `lower_cape_clause_stmts` spills the catch block directly into an output statement list in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs:714`. That may be sufficient for some backends, but it hides the fact that the source described exceptional control flow.
- `lower_ad` is still explicitly a stub lowered as a tuple placeholder in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs:433`. The comment is honest, but it means the HIR contract is knowingly incomplete for this construct.
- `lower_fac` is also explicitly incomplete, lowering `fac ... dum` to a block with a trailing placeholder `dum` expression in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs:522`. This is another case where the syntax layer is ahead of the HIR contract.
- `lower_discerne` still degrades multi-subject matches and multiple patterns into error paths or tuple placeholders in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs:616` and `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs:638`. Again, the code is honest about the limitation, but the mechanical contract is incomplete.

Assessment:
- This is still the most important layer to revisit after parser recovery. The worst semantic collapse is fixed, but several constructs remain stubbed or flattened too aggressively.

## Phase 5: Typecheck And Late Semantic Passes

Mechanical boundary:
- Late passes should consume a faithful HIR contract, not guess around missing source distinctions

What looks solid:
- The type checker is internally disciplined and explicit about inference finalization in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs`
- Dedicated tests exist for `collect`, `resolve`, `typecheck`, `borrow`, `exhaustive`, and `lint`

Findings:
- Because lowering collapses some control-flow distinctions and leaves several constructs as placeholders or error expressions, late passes are partly forced into damage control rather than analysis over a crisp intermediate form.
- Finalization in the type checker reports unresolved types as `MissingTypeAnnotation` even when the deeper cause may be unsupported lowering or an earlier semantic blur. This is visible in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs:301`. That makes late semantic failures less teachable than they could be.
- The crate README still lists several semantic TODOs in `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/README.md:15`, which aligns with what the code shows: the late semantic layers are not the first place to pursue mechanical cleanup. They depend on the syntax-to-HIR contract becoming more exact first.

Assessment:
- The late phases are not the main source of mechanical drift. They are downstream consumers of an HIR layer that is still partially lossy and partially stubbed.

## Cross-Phase Summary

Main strengths:
- The repository has clearly made `radix-rs` the primary compiler path
- The AST remains source-faithful in the important control-flow areas
- Diagnostics are centralized and the semantic pass structure is explicit

Main weaknesses:
- Parser recovery is better, but malformed nested-input coverage is still shallow
- Parser-local regression coverage now exists, but it is still young compared with the semantic and codegen surfaces
- Exception constructs are now handled more honestly, but `cape`/`tempta` still are not modeled as calmly as the rest of the language
- Several constructs are knowingly stubbed or partially lowered, which pushes ambiguity into later passes

Recommended order for future mechanics work:
1. HIR lowering correctness for catch/control-flow constructs
2. Parser malformed-input depth beyond the current recovery and keyword-surface tests
3. Resolve/lowering contract cleanup for aliases and partially lowered constructs
4. Late semantic refinement after the HIR boundary is trustworthy
5. Public-doc drift cleanup in crate-level docs

## Concrete Follow-Up Targets

Highest-priority files for the next review or implementation pass:
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/mod.rs`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/hir/lower/stmt.rs`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/stmt.rs`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser/mod_test.rs`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/semantic/passes/typecheck.rs`
- `/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/lib.rs`

Opus nondum perfectum est, sed linea fracturae nunc clarior est.
