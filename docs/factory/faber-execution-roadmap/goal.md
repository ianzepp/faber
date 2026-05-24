# Goal: Faber Execution Roadmap

**Status**: coordinating goal drafted, not started
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-execution-roadmap/`
**Mode**: long-running roadmap, factory coordination, and multi-session implementation control
**Commit Policy**: Commit after each completed epic phase and validation gate pass

## Summary

Coordinate the current Faber execution work into a long-running factory roadmap: make the exempla Rust e2e corpus truthful, remove retired `ab` collection syntax, stabilize the Rust backend, implement non-strict `ad` capability calls, prove the macOS host syscall model, bridge stable Rust-generated Faber code into Wasm host syscalls, and migrate `norma` toward compiler-core contracts and host-owned capabilities. This document is the umbrella control plane; linked factory docs remain the implementation goals for their specific areas.

## Problem

- The recent planning work produced several related goal documents, but there is not yet one roadmap that explains how they fit together.
- The Rust e2e corpus currently mixes stale syntax, invalid examples, package-bound examples that should live outside `examples/exempla/`, unsupported target features, real backend bugs, and future host/capability work.
- `ab` and `ad` are easy to confuse if they stay embedded in the e2e failure list, but they are different problems: `ab` should be removed, while `ad` should become capability-call syntax.
- The host architecture, syscall/frame model, generated Rust syscall bridge, future MIR lowering, and `norma` direction need to evolve in a coherent order.
- This work will span multiple sessions and likely multiple agents, so each phase needs explicit handoff, validation, and stop rules.

## Goals

- Provide a single factory-ready roadmap for the execution architecture and e2e cleanup work.
- Preserve existing focused factory docs as phase inputs instead of duplicating all implementation detail here.
- Define epics in the order the user requested:
  - Epic 1: steps 1, 2, and 3.
  - Epic 2: steps 4 and 5.
  - Epic 3: step 6.
  - Epic 4: step 7.
  - Epic 5: step 8.
  - Epic 6: step 9.
- Make the roadmap resilient across compaction, interruptions, and multi-session work.
- Allow agent delegation where the task is bounded, reviewable, and does not require hidden architectural judgment.
- Keep the current Rust backend path useful while the Wasm/host path is built additively.
- End with a language/runtime architecture where ordinary Faber core compiles directly, host capabilities route through frame-shaped syscalls, and `norma` no longer implies per-program linked stdlib implementation.

## Non-Goals

- Do not implement the phases in this coordinating goal document.
- Do not replace the focused factory docs with this umbrella doc.
- Do not create a common host crate before the macOS host proves the shape.
- Do not make the future host path block current Rust e2e repair.
- Do not keep retired syntax alive only to improve e2e pass counts.
- Do not migrate all of `norma` before the compiler/backend and host boundaries are proven.
- Do not require strict capability verification for normal authoring builds.

## Ground Truth Researched

- `docs/factory/exempla-rust-e2e/goal.md`: umbrella e2e goal with `71/138` pass baseline, `67` failing files, failure taxonomy, and validation phases.
- `docs/factory/remove-ab-dsl/goal.md`: focused language simplification goal for removing `ab`, `ubi`, `prima`, `ultima`, and `summa` collection-DSL syntax.
- `docs/factory/capability-calls/goal.md`: focused `ad` capability-call goal with permissive unresolved-provider compilation and future strict mode.
- `hosts/macos-arm64/ARCHITECTURE.md`: host architecture direction, Wasm Component Model target, core/capability split, non-strict/strict capability modes, and `norma` direction.
- `hosts/macos-arm64/SYSCALL_MODEL.md`: host syscall/frame model based on Muninn frames/kernel, including `Frame`, routing, sigcalls, and structured errors.
- `hosts/macos-arm64/README.md`: placeholder host crate intent and links to the host architecture/syscall docs.
- `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs`: reference frame protocol material.
- `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs`: reference syscall routing, sigcall registry, cancellation, backpressure, and error model.
- `/Users/ianzepp/work/ianzepp/abbot/daemon/src/kernel/dispatcher.rs` and `/Users/ianzepp/work/ianzepp/abbot/daemon/src/kernel/sigcall_hub.rs`: frame-first syscall dispatcher and outbound sigcall stream reference.
- `/Users/ianzepp/work/ianzepp/monk-os-kernel/src/dispatch/dispatcher.ts` and `/Users/ianzepp/work/ianzepp/monk-os-kernel/src/dispatch/sigcall/registry.ts`: dispatcher-owned syscall/sigcall split where unknown syscall names can route to registered userspace handlers.
- User clarification from 2026-05-24: the macOS host implementation should live in `hosts/macos-arm64`; the Muninn kernel code should be copied/vendorized into the Faber host and edited into a Faber-owned kernel rather than pulled as an external crate dependency.
- `crates/radix/src/hir/nodes.rs`: HIR already preserves `HirStmtKind::Ad(HirAd)`.
- `crates/radix/src/codegen/rust/`: current stable execution path and the first place to prove `ad` -> `__faber_ad` -> `__faber_syscall`.
- `crates/radix/src/mir/`: MIR exists as an eventual canonical lower-target candidate and analysis surface, but the current Epic 5 bridge should not depend on MIR-exclusive Wasm codegen.
- User clarification from 2026-05-24: steps 1, 2, and 3 form Epic 1; steps 4 and 5 form Epic 2; steps 6, 7, 8, and 9 are individual epics.

## Reference Packet

Before starting any epic, inspect:

- `docs/factory/faber-execution-roadmap/goal.md`: this coordinating roadmap.
- `git status --short`: confirm unrelated worktree changes before editing.
- `AGENTS.md`: project rules, autocommit policy, grammar constraints, and Rust-only tooling.
- `EBNF.md`: canonical source grammar before changing examples or parser behavior.
- `crates/radix/src/exempla_e2e_test.rs`: executable corpus discovery and Rust e2e validation.
- `crates/radix/src/tool.rs`: generated-code format/linter helpers and CLI flag semantics.
- `crates/radix/src/hir/`, `crates/radix/src/mir/`, `crates/radix/src/semantic/`, `crates/radix/src/codegen/rust/`: compiler pipeline surfaces affected by later epics.
- `stdlib/norma/` and `crates/norma/`: current stdlib/interface/runtime support split.
- `hosts/macos-arm64/`: future host implementation surface.

Then inspect the focused goal for the active epic:

- Epic 1: `docs/factory/exempla-rust-e2e/goal.md` and `docs/factory/remove-ab-dsl/goal.md`.
- Epic 2: `docs/factory/exempla-rust-e2e/goal.md`.
- Epic 3: `docs/factory/capability-calls/goal.md`.
- Epic 4: `hosts/macos-arm64/ARCHITECTURE.md` and `hosts/macos-arm64/SYSCALL_MODEL.md`.
- Epic 5: `crates/radix/src/codegen/rust/`, current MIR docs/code for future constraints, and host syscall docs/code.
- Epic 6: `hosts/macos-arm64/ARCHITECTURE.md`, `hosts/macos-arm64/SYSCALL_MODEL.md`, `stdlib/norma/`, and `crates/norma/`.

## Constraints And Invariants

- Truthful classification beats pass-count optics.
- Current Rust backend validation stays active until the host/Wasm path can replace it with evidence.
- Source grammar remains type-first and must match `EBNF.md`.
- Do not invent syntax to repair examples.
- Missing type information must be fixed upstream; codegen must not guess.
- `ab` is the retired collection DSL; `ad` is capability-call syntax and the source-level Faber host syscall form.
- `ad` unresolved-provider behavior is allowed in non-strict mode and must fail clearly at runtime.
- Host capability calls should route as frame-shaped syscalls internally.
- The first host implementation should stay in `hosts/macos-arm64`; extract shared host code only after concrete duplication or cross-host pressure exists.
- Do not depend on Muninn as a runtime crate for the Faber host. Copy or reimplement only the relevant Muninn-inspired kernel pieces inside `hosts/macos-arm64`, preserve provenance for any copied source or adapted semantics, and make the result Faber-owned host code.
- `norma` should move toward canonical contracts and interfaces, not per-program linked implementation as the final model.
- `ad` names a capability route. The host decides whether the route is handled by a built-in syscall or by a registered sigcall provider.
- Built-in host syscalls are for small, stable host primitives. Broad or optional IO surfaces should route through sigcall providers instead of growing the host kernel.
- Git commands that create locks must run serially in this repository.

## Supporting Skills

- `warmup`: use at the start of an epic if the active agent lacks fresh repo context.
- `goal-forge`: use only to refine this roadmap or split a new goal document when scope is still fuzzy.
- `delivery`: use to lower each epic phase into a concrete implementation plan before code edits.
- `factory`: use as the outer loop for executing an epic or a set of delivery phases.
- `poker-face`: use after each epic checkpoint to verify the work actually satisfies the epic and did not only move failures. Run poker-face agents with high thinking levels because these gates must catch subtle scope misses, false completion, and cross-epic regressions.
- `zombie-docs`: use when examples, explain corpus, README, or architecture docs may drift during implementation.
- `consequences`: use before changes that alter grammar, HIR/MIR contracts, host ABI, capability contracts, or `norma` classification.
- `clean-break`: use for the `ab` removal epic to avoid carrying compatibility residue.

## Agent Delegation Policy

Agents are appropriate for bounded, reviewable slices:

- read-only corpus classification,
- failure ledgers,
- example migration audits,
- focused parser/HIR/codegen branch removal,
- isolated Rust backend bug clusters,
- docs drift checks,
- host prototype spikes that do not alter compiler contracts.

Agents are not appropriate without a delivery spec and human-readable checkpoint for:

- cross-cutting grammar changes,
- HIR/MIR representation changes,
- Wasm ABI decisions,
- host capability security/grant policy,
- `norma` contract migration decisions,
- deleting exempla or docs without classification evidence.

Parallel agents may run read-only audits or disjoint file edits, but their outputs must be integrated serially. Do not run parallel git-locking operations in this repository.

## Multi-Session Loop

Each epic should follow this loop:

1. Refresh context: read this roadmap, the focused goal doc, current `git status --short`, and the latest relevant validation output.
2. Create or update an epic ledger under the focused factory directory.
3. Lower the next bounded phase through `delivery`.
4. Execute only that phase.
5. Validate with the phase's focused commands.
6. Run a `poker-face` completion gate or equivalent checklist. When using a poker-face agent, configure it for high thinking levels and require it to compare the finished work against this roadmap, the active focused goal, the delivery spec, and validation evidence.
7. Commit the completed phase.
8. Update the ledger with status, remaining failures, commands run, and next recommended phase.
9. Stop cleanly if the next phase would require a new architectural decision.

This loop is designed for compaction and handoff. A future session should be able to resume by reading the roadmap, the active epic ledger, and the latest commit.

## Implementation Shape

### Epic 1: Roadmap, Baseline Ledger, And `ab` Removal

Includes prior steps 1, 2, and 3:

1. Holistic roadmap pass.
2. E2E baseline ledger.
3. Remove `ab` DSL.

Primary docs:

- `docs/factory/faber-execution-roadmap/goal.md`
- `docs/factory/exempla-rust-e2e/goal.md`
- `docs/factory/remove-ab-dsl/goal.md`

Intent:

- Establish the coordinating roadmap and make it durable.
- Classify all exempla before moving, rewriting, removing, or fixing them.
- Remove the dead `ab` collection DSL and migrate/remove its examples.

Agent use:

- Allow agents for corpus classification and `ab` surface inventory.
- Use a single integrating agent for parser/HIR/semantic/codegen removal.

Checkpoint:

- The e2e ledger exists and every original exemplar has a recorded class/disposition.
- `ab` is no longer active collection DSL syntax or executable-backend obligation.
- E2E failures no longer include `ab` as an ordinary Rust backend bug.

### Epic 2: Exempla Boundary And Core Rust Backend Stabilization

Includes prior steps 4 and 5:

4. Enforce the `examples/exempla/` standalone corpus boundary.
5. Stabilize current Rust backend for core language.

Primary doc:

- `docs/factory/exempla-rust-e2e/goal.md`

Intent:

- Make `examples/exempla/` honest as a standalone single-file language-example corpus. Rewrite, move, or remove files that need package structure, helper modules, external crates, host/runtime providers, generated test harnesses, future features, or intentional failure semantics.
- Fix valid-source Rust backend failures in focused clusters: option/nullability, objects/maps, enums/variants, `elige`/`discerne`, structs/methods, iteration/ranges, ownership, conversions, and collection methods.

Agent use:

- Allow agents for failure-cluster diagnosis, corpus relocation/rewrite audits, and isolated backend bug fixes after the exempla boundary decisions are defined.
- Avoid parallel edits to shared Rust codegen/typecheck files unless the delivery spec explicitly assigns disjoint files.

Checkpoint:

- Every remaining `.fab` file under `examples/exempla/` is intended to compile as a single standalone Rust executable through the existing e2e shape.
- Files needing package/dependency/runtime/test context have been rewritten, moved to sibling examples/fixtures, or removed with recorded rationale.
- Valid executable Rust exempla pass or have narrow, recorded backend blockers.
- Rust backend fixes do not guess missing type information.

### Epic 3: Non-Strict `ad` Capability Calls In Current Rust Path

Includes prior step 6:

6. Implement `ad` non-strict Rust behavior.

Primary doc:

- `docs/factory/capability-calls/goal.md`

Intent:

- Treat `ad` as capability-call syntax.
- Make explicit-typed unresolved capability calls compile.
- Emit temporary Rust behavior that fails clearly at runtime when no provider is linked.
- Preserve future strict verification without requiring provider metadata now.

Agent use:

- Allow one focused implementation agent after a delivery spec defines the typecheck, codegen, and e2e behavior.
- Keep `cape`/alternate-exit complexity split if it becomes the blocker.

Checkpoint:

- `examples/exempla/ad/ad.fab` is no longer an unsupported-codegen failure.
- Runtime failure for missing providers is explicit.
- The behavior aligns with the host syscall model but does not require the host to exist yet.

### Epic 4: Faber-Owned macOS Host Kernel Slice

Includes prior step 7:

7. Build minimal macOS host syscall slice.

Primary docs:

- `hosts/macos-arm64/ARCHITECTURE.md`
- `hosts/macos-arm64/SYSCALL_MODEL.md`
- `hosts/macos-arm64/README.md`
- `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs`
- `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs`
- `stdlib/norma/hal/`
- `crates/norma/hal/`

Intent:

- Prove the host model inside the existing `hosts/macos-arm64` crate.
- Treat frames as the durable host/protocol invariant, not as an implementation detail of one process shape.
- Preserve the useful Muninn semantics: `Frame`, `Status`, request correlation, prefix routing, syscalls, sigcalls, cancellation, structured `E_` errors, and unresolved `E_NO_ROUTE`.
- Build or copy only the relevant Muninn-inspired kernel runtime pieces into `hosts/macos-arm64`; do not depend on Muninn as an external crate and do not import unused transport or scheduler machinery just to match Muninn's current crate shape.
- Edit any imported code into a Faber-owned host kernel with Faber naming, docs, tests, and capability/HAL assumptions.
- Add one tiny built-in syscall, such as `host:echo`, to prove frame routing before broader HAL migration.
- Add manifest output for built-in syscalls and registered providers.
- Begin identifying which existing HAL surfaces under `stdlib/norma/hal/` and `crates/norma/hal/` should move to host-owned syscalls, but migrate only the smallest surface needed to prove the slice.
- Keep the first runtime slice small, macOS-local, and additive to the existing compiler/Rust backend path.

Host topology:

- The first runtime proof may be a direct launcher-style host command that routes an in-memory frame through the kernel, because this proves the syscall contract with the least lifecycle policy.
- A later `serve` mode may expose the same frame contract over a local transport such as a Unix domain socket, using JSON for debug/local ergonomics or a compact binary frame stream for production.
- The long-term model can be hybrid: in-process host built-ins and local/remote provider processes both speak the same frame-shaped protocol.
- Wasm/component compilation does not replace frames. A Wasm import can be a small function such as `capability_call(name, args)`, and the host should immediately wrap that import as a `Frame` before routing it.

Agent use:

- Allow a prototype agent for a mechanical Muninn code import only if the delivery spec defines exact source files, target module paths, provenance notes, and expected compile/test fixes.
- Allow read-only agents to inventory Faber HAL surfaces and propose syscall names, but do not let that inventory become a full `norma` migration in this epic.
- Keep the integrating owner responsible for adapting imported kernel semantics, because copied code or copied ideas must become Faber host-owned.
- Do not let this epic create a shared/common host crate.
- Do not add a path, git, or published crate dependency on Muninn for the Faber host runtime.

Suggested first module shape:

```text
hosts/macos-arm64/src/
├── main.rs
├── kernel/
│   ├── mod.rs
│   ├── backpressure.rs
│   ├── error.rs
│   ├── frame.rs
│   ├── kernel.rs
│   ├── pipe.rs
│   ├── router.rs
│   ├── sender.rs
│   ├── sigcall.rs
│   └── syscall.rs
├── hal/
│   ├── mod.rs
│   ├── consolum.rs
│   └── host.rs
└── manifest.rs
```

This shape is a starting point, not a final ABI. The kernel should stay internal to the macOS host until a second host or concrete duplication justifies extraction.

Recommended phase split:

1. `4.1`: Add the Faber-owned frame/kernel route proof inside `hosts/macos-arm64`, with `host:echo`, `E_NO_ROUTE`, manifest output, tests, and provenance notes. This may run entirely in-process from a CLI command.
2. `4.2`: Attach a first Wasm/component import to the same frame router. The import ABI can be smaller than a full frame if it wraps into a frame immediately inside the host.
3. `4.3`: Add daemon/server transport only when provider registration, shared service lifecycle, or multi-process capability routing needs it. The transport must carry the same frame contract rather than introducing a parallel protocol.

Checkpoint:

- The Faber-owned kernel compiles inside `hosts/macos-arm64` with no Muninn runtime dependency.
- Provenance from any Muninn source import or semantic adaptation is recorded in commit history, docs, or module comments clearly enough for future audits.
- The host can route at least one built-in syscall and report one unresolved capability as `E_NO_ROUTE`.
- Manifest output exists and includes the built-in syscall surface.
- The first HAL migration candidate is recorded, with rationale for why it belongs in the host kernel rather than generated Rust support.
- The design remains compatible with both future Wasm component integration and local frame-stream server transport.

### Epic 5: Rust-To-Wasm Faber Host Syscall Bridge

Includes prior step 8:

8. Lower Faber `ad` calls to host syscalls through the stable Rust path.

Primary references:

- `crates/radix/src/hir/nodes.rs`
- `crates/radix/src/codegen/rust/`
- `crates/radix/src/mir/`
- `hosts/macos-arm64/SYSCALL_MODEL.md`
- `hosts/macos-arm64/src/component.rs`
- `hosts/macos-arm64/src/kernel/`

Intent:

- Treat `ad` as the Faber source-level host syscall form: source syntax names a host capability/syscall such as `host:echo`, `fs:read`, or `pg:query`.
- Reuse the stable HIR-to-Rust backend as the first implementation path instead of making MIR or a full Wasm backend the critical path.
- Lower generated Rust `ad` dispatches through a source-form helper such as `__faber_ad(...)`, then through a target-specific syscall helper such as `__faber_syscall(...)`.
- For native Rust output, preserve the current explicit unresolved-provider behavior unless a concrete linked-provider path is added.
- For Rust compiled to Wasm, make `__faber_syscall(...)` call a host import supplied by the macOS host.
- Have the macOS host import wrapper decode or build a request, route it as a `Frame` through `HostKernel`, and return the response.
- Prove both a successful built-in syscall (`host:echo`) and an unresolved provider syscall (`pg:query` producing `E_NO_ROUTE`) from generated Faber code.
- Keep MIR as an eventual canonical lower-target candidate and analysis surface, but do not require MIR-backed Wasm codegen for this epic.

Agent use:

- Allow agents for read-only Rust-to-Wasm tooling research, ABI sketch comparison, and focused host/runtime tests.
- Keep the integrating owner responsible for generated Rust helper shape, host import ABI, and any compiler-driver or package-build changes.
- Do not split overlapping edits across Rust codegen, host component loading, and package build wiring without an explicit delivery spec.

Checkpoint:

- Existing native HIR-to-Rust behavior remains stable, including explicit unresolved behavior for `ad` when no host/provider is linked.
- A Wasm-targeted generated Rust path exists where `ad` dispatches call `__faber_ad(...)`, which delegates to a host syscall import through `__faber_syscall(...)`.
- The host can instantiate or run the generated Wasm artifact and provide the syscall import.
- `ad "host:echo"` reaches the Epic 4 `HostKernel` path and returns a successful frame-derived result.
- `ad "pg:query"` reaches the same host path and returns a structured `E_NO_ROUTE` failure.
- The proof does not require strict provider verification, a final WIT world, a daemon/server transport, full `norma` migration, or MIR-exclusive Wasm codegen.

### Epic 6: `norma` Classification And Migration

Includes prior step 9:

9. Classify `norma` and migrate the first host-effect contract.

Primary references:

- `stdlib/norma/`
- `crates/norma/`
- `stdlib/norma/hal/consolum.fab`
- `crates/norma/hal/consolum.rs`
- `hosts/macos-arm64/ARCHITECTURE.md`
- `hosts/macos-arm64/SYSCALL_MODEL.md`

Intent:

- Classify `norma` with durable ownership labels:
  - `core-language`: pure language/library semantics the compiler or target runtime can own directly.
  - `host-effect`: outside-world effects and IO that semantically route through Faber host syscall capability surfaces.
  - `rust-bridge`: native Rust support kept for current HIR-to-Rust output while the host path matures.
  - `needs-decision`: surfaces that require human review before classification.
- Use existing `stdlib/norma/hal/*.fab` pacta as the first host-effect contract source instead of inventing new syscall annotation syntax.
- Treat `@ externa` HAL functions as syscall-capability contract surfaces by convention; their syscall identities can start from the pactum/function path such as `consolum:scribe`.
- Classify all IO as `host-effect`, including console IO. Native Rust may keep direct output fast paths, but that is backend lowering policy, not a separate `norma` ownership category.
- Start with `stdlib/norma/hal/consolum.fab` as the first proof surface and keep language output such as `nota`, `vide`, and `mone` consistent with host IO semantics.
- Keep host syscall errors generic for now, using the frame/host error shape proven by earlier epics rather than per-syscall error taxonomies.
- Preserve `crates/norma` implementations as `rust-bridge` support where current native Rust output still needs them; do not delete implementations in this epic.
- Keep pure collection/data operations as `core-language` or ordinary built-in type APIs.

Agent use:

- Allow agents for inventory and classification proposals.
- Require human/integrating-agent review before changing contract file layout or moving implementation responsibilities.
- Do not delegate deletion of `stdlib/norma` or `crates/norma` files; deletion is out of scope for this epic.

Recommended first slice:

1. `6.1`: Create an Epic 6 ledger and classify `norma` at module/file level before changing compiler behavior.
2. Inventory `stdlib/norma/` and `crates/norma/` with enough detail to assign `core-language`, `host-effect`, `rust-bridge`, or `needs-decision`.
3. Classify `stdlib/norma/hal/consolum.fab` as `host-effect` and `crates/norma/hal/consolum.rs` as `rust-bridge`.
4. Record canonical syscall identities for existing `consolum` members, such as `consolum:scribe`, `consolum:dic`, `consolum:mone`, `consolum:vide`, and stdin/TTY operations.
5. Record that `nota`, `vide`, `mone`, and related language output semantics are host IO, while native Rust direct output remains allowed as backend lowering policy.
6. Validate the classification docs and ledger for consistency. Do not change compiler lowering, strict-mode behavior, host dependency provisioning, or delete any `norma` files in `6.1`.

Checkpoint:

- Every `norma` surface inspected in the epic has a recorded classification: `core-language`, `host-effect`, `rust-bridge`, or `needs-decision`.
- Existing `stdlib/norma/hal/*.fab` pacta are recognized as host-effect contract sources, with `consolum` classified first.
- `consolum` records canonical syscall identities for its existing interface members without adding new annotation syntax.
- `crates/norma/hal/consolum.rs` remains available as `rust-bridge` support for native Rust output.
- `nota`, `vide`, `mone`, and related console output behavior are treated as host IO semantically while preserving existing native Rust behavior where appropriate.
- Future strict-mode host manifests have a clear contract source, but strict verification itself remains out of scope.
- No `norma` implementation files are deleted as part of this epic.

### Epic 7: Host Sigcall Provider Boundary

Future step added after Epic 6 design review.

Primary references:

- `hosts/macos-arm64/SYSCALL_MODEL.md`
- `hosts/macos-arm64/src/kernel/`
- `/Users/ianzepp/work/ianzepp/monk-os-kernel/src/dispatch/dispatcher.ts`
- `/Users/ianzepp/work/ianzepp/monk-os-kernel/src/dispatch/sigcall/registry.ts`
- `/Users/ianzepp/work/ianzepp/abbot/daemon/src/kernel/dispatcher.rs`
- `/Users/ianzepp/work/ianzepp/abbot/daemon/src/kernel/sigcall_hub.rs`

Intent:

- Stop treating every outside-world interface as a candidate for compiled-in host kernel code.
- Make the host dispatcher choose between built-in syscalls and registered sigcall providers for the same source-level `ad` route shape.
- Keep `ad "provider:operation"` stable as source syntax while moving implementation selection into host manifests, provider registration, and runtime routing.
- Define provider lifecycle, registration, request correlation, streaming responses, cancellation, timeout, and `E_NO_ROUTE` behavior before adding broad IO surfaces such as databases, TUI frameworks, browser automation, or cloud APIs.
- Preserve `consolum:*` as the small built-in proof while moving future HAL/provider work out of the kernel module tree unless a route is truly a host primitive.

Recommended first slice:

1. Define the provider registry data model: exact route ownership, optional prefix ownership, provider id, process/transport handle, version, and declared capabilities.
2. Add a host-internal sigcall dispatcher path that can route an unknown non-built-in call to a registered provider.
3. Add request/response correlation for one external provider process or in-test provider shim.
4. Prove `E_NO_ROUTE` still fires when neither a built-in syscall nor a provider registration exists.
5. Add manifest output that distinguishes built-in syscalls from registered providers.

Checkpoint:

- Host kernel modules contain generic frame, routing, syscall, and sigcall infrastructure, not provider-specific IO implementations by default.
- At least one non-built-in capability route is served by a registered provider or provider shim.
- Built-in `consolum:*` still works as the small host-owned syscall proof.
- Provider registration does not require recompiling the host for every new interface family.
- Runtime response semantics remain frame-shaped and support the eventual streaming/cancellation model.

## Exit Strategy

Decision: included.

- Each epic must leave the repo in a shippable or clearly documented intermediate state.
- Current Rust e2e validation remains the fallback execution proof until the host/Wasm path is proven.
- Host work is additive until a later migration explicitly replaces Rust-backend execution.
- Syntax removal work must preserve legacy diagnostics or explain history when silent removal would confuse users.
- If an epic exposes a deeper language-design issue, stop and split a new factory goal instead of hiding the decision inside implementation.

## Acceptance Criteria

- This roadmap exists and references every focused goal doc needed to resume the work.
- Each requested group is represented as an epic:
  - Epic 1: steps 1, 2, 3.
  - Epic 2: steps 4, 5.
  - Epic 3: step 6.
  - Epic 4: step 7.
  - Epic 5: step 8.
  - Epic 6: step 9.
- Every epic has intent, primary references, agent policy, and checkpoint criteria.
- Epic 7 records the syscall/sigcall provider boundary before broad host/provider interfaces are added.
- The roadmap is suitable for multi-session factory execution without relying on hidden chat context.
- The roadmap preserves the dependency order between e2e cleanup, `ab` removal, Rust backend stabilization, `ad` capability calls, host syscalls, the Rust-to-Wasm host syscall bridge, and `norma` migration.

## Validation

- `git diff --check -- docs/factory/faber-execution-roadmap/goal.md` should pass after edits.
- Review check: every linked doc path in this roadmap exists.
- Review check: every epic has a concrete checkpoint and stop condition through this roadmap or the linked focused goal.
- Review check: no implementation phase requires strict capability verification before non-strict capability calls and host manifest support exist.
- Review check: no phase requires a shared host crate before the macOS host slice proves the model.

## Open Questions

- Should the e2e baseline ledger live only under `docs/factory/exempla-rust-e2e/`, or should this roadmap keep a rollup ledger of epic status?
- Should strict mode follow the Epic 7 provider boundary, or wait until a real host/provider set exists?
- Should the first Wasm boundary expose full frames or a smaller `call(name, args)` wrapper that becomes frames inside the host?

## Future Host Dependency Manifests

Decision note: included, out of current epic scope.

- Provider dependencies should be installed, resolved, and versioned on the host side rather than compiled into each Faber program.
- Normal compilation should continue to accept `ad` calls from local source-declared shapes without requiring a complete provider interface or SDK.
- `requirit.toml` and `requirit.lock` are the likely future surfaces for project host requirements and resolved provider truth.
- Tooling may partially manage `requirit.toml` by scanning `ad` dispatch paths, scaffolding missing provider sections, or warning when a route such as `pg:query` has no declared host requirement.
- Full auto-generation is not assumed, because an `ad` path alone does not decide provider package, version range, grants, config, secrets, or deployment host.
- Host sync/provisioning should eventually read project requirements, install or verify provider packages, bind config/secrets, and expose the route manifest used by runtime dispatch and optional strict checks.
- This is not part of Epic 6; Epic 6 should avoid turning `norma` into the provider dependency catalog.

## Stop Conditions

- Stop if a phase attempts to solve multiple epics at once without a delivery spec.
- Stop if e2e pass counts improve by deleting, moving, or hiding examples before classification/disposition evidence.
- Stop if `ab` removal starts adding compatibility layers instead of retiring the DSL.
- Stop if `ad` implementation starts requiring provider metadata in normal compilation.
- Stop if host work creates a common crate before the macOS host has a working minimal slice.
- Stop if Epic 5 starts replacing the stable HIR-to-Rust path with MIR-exclusive Wasm codegen before the Rust-to-Wasm syscall bridge is proven.
- Stop if later MIR/Wasm lowering decisions are made without updating tests and dumps for the new representation.
- Stop if `norma` migration would remove temporary Rust backend support before current validation has a replacement.
