# Epic 6 Ledger: Norma Classification And Migration

## Scope

Epic 6 implements roadmap step 9: classify `norma` and migrate the first
host-effect contract. The first completed slice should be classification-only;
later slices can use the record here to move `consolum` toward host-owned
syscall routing without breaking the current Rust backend.

## 6.1 Norma Classification Baseline

Status: complete pending commit

Delivery spec:

- `docs/factory/faber-execution-roadmap/epic-6/delivery-6.1.md`

Classification artifact:

- `docs/factory/faber-execution-roadmap/epic-6/norma-classification.md`

Inputs inspected:

- `docs/factory/faber-execution-roadmap/goal.md`
- `hosts/macos-arm64/ARCHITECTURE.md`
- `hosts/macos-arm64/SYSCALL_MODEL.md`
- `stdlib/norma/`
- `crates/norma/`
- `hosts/macos-arm64/src/hal/host.rs`
- `hosts/macos-arm64/src/kernel/`
- `hosts/macos-arm64/src/manifest.rs`

Decisions recorded:

- `stdlib/norma/hal/consolum.fab` is `host-effect`.
- `crates/norma/hal/consolum.rs` is `rust-bridge`.
- Existing `@ externa` HAL pacta are host-effect contract sources by
  convention; no new syscall annotation syntax is introduced in 6.1.
- Canonical syscall identities for the current console pactum use the
  `consolum:<member>` form.
- `nota`, `vide`, `mone`, and related language output semantics are host IO at
  the language architecture level; direct native Rust printing remains allowed
  as a backend lowering policy while the Rust path is active.
- Host syscall errors remain generic frame/host errors for now; 6.1 does not
  add per-console error taxonomies.
- No `norma` file is deleted in 6.1.

Out of scope for 6.1:

- compiler lowering changes,
- strict-mode manifest verification,
- host dependency provisioning,
- replacing native Rust output,
- deleting or moving `stdlib/norma` or `crates/norma` files.

Validation:

- `git diff --check -- docs/factory/faber-execution-roadmap/epic-6`
  passed.
- `find docs/factory/faber-execution-roadmap/epic-6 -maxdepth 1 -type f`
  shows the delivery spec, ledger, and classification artifact.
- Every file returned by
  `find stdlib/norma crates/norma -maxdepth 3 -type f` is referenced in
  `norma-classification.md`.
- Content search confirms all current `consolum` function identities are
  recorded with the `consolum:<member>` shape.
- `git status --short` shows only the new Epic 6 docs directory.

Poker-face gate:

- Evaluator mode: self-contained cold pass. No subagent was used because agent
  delegation was not explicitly requested for this turn.
- Original target: roadmap Epic 6 recommended first slice `6.1`.
- Checklist result: satisfied for ledger creation, all-file classification,
  `consolum` host-effect classification, `consolum:<member>` syscall identity
  recording, `crates/norma/hal/consolum.rs` rust-bridge preservation,
  language-output host IO semantics, generic host error policy, and no compiler
  or runtime behavior changes.
- Misses: no high or medium misses found.
- Completion estimate: 100% for 6.1.

Next recommended slice:

- 6.2 should take `stdlib/norma/hal/consolum.fab` as the first executable
  host-effect contract and decide how to expose its identities through the
  existing host manifest without making strict mode mandatory.
