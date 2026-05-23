# Radix Diagnostics Mode Factory Ledger

**Phase Set Source**: `docs/radix-diagnostics-plan.md`
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber-radix-diagnostics`
**Delivery Spec Directory**: `docs/factory/radix-diagnostics-mode`
**Current Phase**: completion audit
**Completed Phases**:

- Phase 1: `radix check --diagnostics` expanded diagnostics records
- Phase 2: `radix emit --diagnostics` expanded diagnostics records
- Cheap follow-through: file-mode `faber check --diagnostics` via shared `CheckArgs` and `CheckCommand`

**Pending Phases**:

- Package-mode `faber check --diagnostics`, if a future slice chooses to define package-wide expanded reporting

**Checkpoint Policy**: each phase must preserve normal output, add deterministic diagnostics-mode output, and pass focused tests plus the relevant crate test subset.
**Commit Policy**: commit each completed phase slice from this worktree after verification and poker-face audit pass.
**Agent Policy**: no subagents for the initial slice; the affected surface is compact enough for direct supervision.
**Correctness Policy**: verify phase attribution is explicit at diagnostic construction points and missing optional fields do not panic in expanded rendering.
**Poker Face Policy**: compare implementation against the saved phase delivery spec and `docs/radix-diagnostics-plan.md`; require at least 85% completion before committing.
**Granularity Policy**: start with the recommended first issue, then extend only where existing shared structs make it a low-risk continuation.
**Open Questions**: none blocking.
