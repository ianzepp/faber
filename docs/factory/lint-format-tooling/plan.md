# Lint and Format Tooling Factory Plan

**Status**: design captured, not scheduled
**Created**: 2026-05-23
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/lint-format-tooling/`
**Mode**: developer tooling design / compiler diagnostics and formatter follow-up
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber has correctness checking and some lint-like warnings, but it does not yet have a distinct user-facing linter or automatic formatter.

The design should keep the boundary mechanical:

- `check` answers whether the program is valid Faber.
- `lint` critiques valid Faber for suspicious, stale, non-canonical, or mechanically weak forms.
- `format` rewrites source into canonical layout without changing semantics.

## Current Reality

### Lint Foundation Exists

`crates/radix/src/semantic/passes/lint.rs` already runs as semantic pass 7. It emits warnings for:

- unused variables, functions, and imports,
- unreachable code,
- unnecessary `⇢` casts,
- explicit `ignotum` annotations,
- shadowing as a hard error.

Borrow analysis also emits mode warnings for `de` / `in` / `ex` misuse, and target-specific warnings exist elsewhere, such as Rust `cura "arena"` no-op warnings.

So Faber already has compiler-backed lint judgments. What it does not have is a stable user-facing lint command, stable lint slugs, policy configuration, or local suppression syntax.

### Formatter Foundation Exists, But Is Not User-Safe Yet

`crates/radix/src/codegen/faber/mod.rs` implements a canonical Faber pretty-printer backend. `radix emit -t faber` can normalize Faber output and is useful for round-trip tests.

That backend is HIR-based and explicitly loses comments and original formatting. It is a good canonicalizer, but it should not become an in-place `faber format --write` formatter until formatting can preserve comments and trivia.

## Tool Boundary

### `faber check`

`check` is truth:

```bash
faber check
```

It should report errors that affect compiler honesty:

- parse errors,
- unresolved names,
- type mismatches,
- invalid assignments,
- borrow/move errors,
- exhaustiveness errors,
- unsupported target behavior.

`check` may continue to show important warnings, but warning policy should not be its main job.

### `faber lint`

`lint` is judgment:

```bash
faber lint
```

It should report named judgments over valid code:

- unused bindings,
- unreachable code,
- non-canonical aliases,
- legacy syntax still temporarily accepted,
- suspicious broad `ignotum`,
- suspicious or ambiguous flow/effect usage,
- target portability risks,
- stale suppressions.

Lint findings should have stable slugs so users can configure, suppress, search, and track them.

### `faber format`

`format` is shape:

```bash
faber format --check
faber format --write
```

It should enforce canonical whitespace, indentation, brace style, line breaks, and canonical spelling preferences without changing program meaning.

The first safe formatter should be token/CST-preserving. HIR pretty-print may support `format --canonical` or internal normalization, but not ordinary in-place formatting until comments survive.

## Lint Definition

A Faber lint is:

> A named compiler-backed judgment over valid Faber code, where the judgment explains a mechanical risk rather than a personal style preference.

Each lint should have:

```text
slug:        hygiene.unused-binding
category:    hygiene
default:     warn
scope:       item | block | statement | expression
reason:      binding is never read
fix:         remove it, prefix it, or use it
```

Suggested categories:

| Category | Purpose |
|----------|---------|
| `mechanica` | Violates or weakens Faber's mechanical laws |
| `hygiene` | Unused, unreachable, stale, or confusing code |
| `claritas` | Valid but unclear code |
| `legacy` | Accepted old syntax with a canonical replacement |
| `effectus` | Suspicious flow, error, async, or generator behavior |
| `target` | Target-specific portability or backend warnings |
| `periculum` | Likely bug, stronger than style |

Example slugs:

```text
hygiene.unused-binding
hygiene.unreachable-code
hygiene.unused-suppression
claritas.explicit-ignotum
legacy.ascii-arrow
legacy.clausura-colon
mechanica.noncanonical-colon-field
effectus.cede-ambiguous
target.rust-cura-arena-noop
```

Slugs should be boring English, not Latin. They are tool protocol, config keys, search terms, and CI output. User-facing messages may still use Faber vocabulary.

## Lint Policy

Project-level policy should use plain config values:

```toml
[lint]
"mechanica" = "deny"
"legacy" = "warn"
"claritas.explicit-ignotum" = "allow"
"hygiene.unused-binding" = "warn"
```

Policy levels:

| Level | Meaning |
|-------|---------|
| `allow` | Do not report this lint by default |
| `warn` | Report as a warning |
| `deny` | Report as an error |

Hard compiler errors are not lints. If a finding affects type safety, name resolution, ownership validity, codegen correctness, or parser truth, it belongs in `check` and cannot be downgraded through lint policy.

## Source Suppression

Use `tacet` for local lint suppression:

```fab
@ lint tacet "hygiene.unused-binding"
functio probe() {
    fixum numerus scratch ← 1
}
```

Meaning:

> This named lint is deliberately silent in the annotated scope.

This is intentionally tied to the existing Faber concept:

```fab
tacet
```

In executable code, `tacet` means "do nothing, deliberately." In lint control, `@ lint tacet` means "say nothing about this lint here, deliberately."

The suppression must name a slug. Do not add a bare blanket form:

```fab
@ lint tacet
```

Category suppression may be allowed later, but should be visibly broad:

```fab
@ lint tacet "hygiene.*"
```

Optional reasons may be useful:

```fab
@ lint tacet "hygiene.unused-binding" quia "kept for debugger attachment"
functio probe() {
    fixum numerus scratch ← 1
}
```

If the suppressed lint no longer fires, the linter should warn:

```text
warning[hygiene.unused-suppression]: lint suppression did not suppress anything
```

That warning can itself be suppressed only explicitly:

```fab
@ lint tacet "hygiene.unused-suppression"
```

## Formatter Design

The formatter should eventually be comment-preserving and source-aware:

- preserve comments,
- preserve block-string content,
- normalize indentation,
- enforce Stroustrup brace style,
- prefer canonical glyphs where the parser can prove semantic equivalence,
- preserve deliberate compact forms when they fit,
- expand crowded compact forms into `fac` blocks when needed,
- never rewrite through HIR in a way that drops trivia.

The existing Faber codegen target should remain valuable as:

- canonical output for generated Faber,
- round-trip testing,
- `format --canonical` experiments,
- parser/codegen stabilization checks.

It should not be the default in-place formatter until comment preservation is solved.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
|-------|------|------|------------|
| 0 | Design confirmation | Confirm `check` / `lint` / `format` boundaries and `@ lint tacet` suppression. | Plan approved. |
| 1 | Inventory | Audit existing warnings, diagnostic codes, lint tests, Faber pretty-printer, and CLI command surfaces. | Ledger maps current warnings to proposed slugs. |
| 2 | Stable lint slugs | Add stable lint slug metadata to existing warnings. | Current lint warnings render with slugs. |
| 3 | `faber lint` command | Add user-facing lint command over existing analysis. | `faber lint` reports warnings without requiring build output. |
| 4 | Lint policy config | Add project-level `allow` / `warn` / `deny` policy. | Config can deny a warning or allow a warning by slug/category. |
| 5 | Source suppression parser | Parse `@ lint tacet "slug"` annotations. | Suppression syntax parses but may not yet affect all scopes. |
| 6 | Suppression semantics | Apply suppressions by annotated scope and report unused suppressions. | Local suppression works and stale suppressions warn. |
| 7 | Formatter check mode | Add `faber format --check` using safe formatting infrastructure. | Check mode reports drift without rewriting. |
| 8 | Formatter write mode | Add comment-preserving `faber format --write`. | Rewrites source while preserving comments and block strings. |
| 9 | Docs and examples | Update README, explain entries, EBNF/spec commentary, and CLI help. | Users can distinguish check/lint/format and suppress lints deliberately. |

## Open Questions

- Should `faber check` continue to emit all warnings, or should warnings move behind `faber lint` and `--warnings` modes?
- Should lint policy live in `faber.toml`, a future separate tooling file, or both?
- Should source-level suppression be available before project-level policy exists?
- Should wildcard suppressions such as `"hygiene.*"` be allowed in the first implementation?
- Should `@ lint tacet` attach to arbitrary statements, only declarations/blocks, or every AST node with annotations?
- Should the formatter expose both `format --write` and `format --canonical` if the latter is HIR-based and comment-dropping?
- Should `legacy` lints become hard parser errors automatically after a migration window?

## First Useful Slice

The first useful implementation should not start with formatter rewriting. It should start with named lints:

1. assign stable slugs to existing warnings,
2. add `faber lint`,
3. add tests proving slugs are stable,
4. document `check = truth`, `lint = judgment`, `format = shape`.

`@ lint tacet` should come after slugs exist. Suppressing an unnamed warning is not mechanical.

---

*Design maxim: `tacet` is deliberate silence. The program may do nothing deliberately; the linter may also say nothing deliberately, but only when the silence is named.*
