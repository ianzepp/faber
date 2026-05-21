# Faber/Radix Tool Split Factory Ledger

**Factory Run**: faber-radix-tool-split per `plan.md`
**Started**: 2026-05-21
**Completed**: 2026-05-21
**Target Repo**: /Users/ianzepp/work/ianzepp/faber
**Intended Release**: v0.34.0

## Phases

| Phase | Name | Status | Notes |
| ----- | ---- | ------ | ----- |
| 0 | Preflight | completed | Baseline captured; branch ahead 4 |
| 1 | Workspace promotion | completed | `radix/` container removed; root workspace |
| 2 | Node/Bun retirement | completed | Removed package.json, bun.lock, TS scripts, norma-ts |
| 3 | Radix binary extraction | completed | `radix` bin in `crates/radix/src/bin/radix.rs` |
| 4 | Faber crate scaffold | completed | `crates/faber` with check/build/targets + stubs |
| 5 | Package ownership move | completed | `faber::package` owns discovery/orchestration |
| 6 | Compatibility aliases | completed | faber lex/parse/hir/emit/cli-ir delegate to radix tool |
| 7 | Release and docs | completed | README, AGENTS, CI, release workflow, release notes |
| 8 | Validation | completed | `./scripta/ci` PASS |

## Validation Gate

```bash
./scripta/ci  # fmt, test, clippy, release builds — PASS
```

## Open Decisions (unchanged from plan)

- Release archive ships both `faber` and `radix` tarballs (implemented in workflow).
- `faber emit` remains a compatibility alias for at least v0.34.
- Package graph lives in `crates/faber/src/package.rs` (no shared crate added).
