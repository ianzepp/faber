# Faber Website Refresh Plan

Date: 2026-05-22

This plan captures the discovery pass for bringing the Faber website back in line with the current `../faber` project. It is intentionally pre-implementation: the next goal session should use this as the working checklist.

Direction: migrate the useful contents of `faber-www` back into `../faber` under a dedicated `website/` folder. The website is documentation for Faber, not a separate application with an independent lifecycle.

## Repository Direction

Prefer a monorepo layout:

```text
faber/
  crates/
  docs/
  examples/
  explain/
  stdlib/
  website/
    content/
    templates/
    styles/
    dist/        # generated, ignored unless deployment needs otherwise
```

Reasons:

- Website correctness depends on the exact Faber source tree: `README.md`, `EBNF.md`, `docs/grammatica`, `examples`, `explain`, release docs, and live CLI behavior.
- Docs and compiler behavior should be changed, reviewed, and committed together.
- Site generation can use repo-local paths instead of fragile sibling-repo sync assumptions.
- CI can validate the site against the same code revision being published.
- A dedicated `website/` folder keeps presentation assets contained without treating the docs as a separate product.

The standalone `faber-www` repository should become a migration source and then be archived, frozen, or redirected after the `../faber/website` build and deployment path works.

## Source Of Truth

Use `../faber` as the source of truth for current Faber behavior.

Confirmed current shape:

- `../faber` is a Rust workspace with `crates/faber`, `crates/radix`, and `crates/norma`.
- `faber` is the user-facing project and package tool.
- `radix` is the compiler-developer inspection tool.
- `norma` is the Rust runtime support crate and `stdlib/norma` contains Faber standard library definitions.
- Current release shown by the local CLI is `faber 0.35.0`.
- Live target output is:
  - `rust`: check/build/run/package yes, primary backend.
  - `go`: check/build yes, run/package no, file emission only.
  - `ts`: check/build yes, run/package no, file emission only.
  - `faber`: check/build yes, run/package no, canonical pretty-print target.
- Current docs live under `../faber/docs/grammatica`, with `../faber/EBNF.md`, `../faber/explain`, `../faber/examples/exempla`, and `../faber/examples/automation` as adjacent source material.
- Current package manifest is `faber.toml`, not `faber.fab`.
- The v0.34 line split Faber/Radix; v0.35 adds `faber test`, expanded `faber explain`, `sponte`, `fixus`, and `T ∪ nihil`.

Secondary context:

- `../faber-archivum` preserves deprecated bootstrap, self-hosting, TypeScript reference, rivus, nanus, old tests, and old scripts. It is archaeology, not current website truth.
- `../faber-consilia` contains design notes and future/planning material.
- `../faber-trials` contains LLM learnability research harness material; the website may still link to or summarize it, but it should not be presented as the primary current project surface.
- `../faber-grammars-zed` currently has only a starter README signal from this quick pass.

## Current Standalone Website State

Confirmed:

- `faber-www` is a Bun-based static site generator with Markdown content under `content/`, layout in `templates/layout.html`, and CSS in `styles/main.css`.
- `package.json` still defines `build` as `bun run sync:content && bun run build/generate.ts`.
- The `build/*.ts` files are deleted in the current worktree, so `bun run build` fails with `Module not found "build/sync-grammar.ts"`.
- Deployment uploads `dist` to GitHub Pages but does not currently run a build step.
- Current tracked site content is mostly hand/synced Markdown and old generated pages.
- The old generator used `gray-matter`, which assumes YAML-style front matter by default (delimited by `---`). Current Faber direction is TOML front matter delimited by `+++` (consistent with the `explain/` corpus), so any replacement generator should parse TOML metadata intentionally.

Existing local changes before this plan:

- Deleted: `build/generate.ts`
- Deleted: `build/sync-compilers.ts`
- Deleted: `build/sync-examples.ts`
- Deleted: `build/sync-grammar.ts`
- Deleted: `build/sync-research.ts`

Those deletions were pre-existing in the workspace and should not be reverted blindly.

## Major Drift To Correct During Migration

### Build And Publishing

- Move the buildable site source into `../faber/website`.
- Do not preserve `package.json`/Bun as the default plan; the site is static documentation and should use the Faber repo toolchain where practical.
- Prefer a small Rust-owned generator or `xtask`-style command inside `../faber` for first implementation.
- Keep the generator architecture simple enough that it can be ported to Faber later when filesystem/path/Markdown/TOML support is mature enough.
- Decide whether content under `website/content` is curated Markdown, generated copies from root docs, or a mix.
- Replace YAML-front-matter assumptions (`---`) with TOML front matter parsing (`+++ ... +++`).
- Add or move GitHub Pages workflow into `../faber` so it builds `website/dist` before upload, or document that `dist` must be committed/generated elsewhere.
- Regenerate `llms.txt` and `faber-complete.md` from current content if the site keeps those artifacts.

### Homepage Positioning

Current homepage says Faber is an LLM-oriented IR compiling to Zig, Rust, C++, TypeScript, or Python. That is stale.

Replace with current positioning:

- Faber Romanus is a Latin programming language and compiler centered on a Rust workspace.
- Faber is currently a user-facing project/package tool, with Rust as the primary package backend.
- Radix is the compiler inspection/developer tool.
- Go, TypeScript, and Faber output are real file-emission targets, but not package/run targets.
- Avoid presenting Zig, Python, and C++ as active current targets.
- Install section should use the current Homebrew instruction from `../faber/README.md`: `brew install ianzepp/tap/faber`.
- GitHub link should be checked against the current repo name before publishing. The site currently links `ianzepp/faber-romanus`; local repo name is `faber`.

### Compiler Pages

The `content/compilers/*` section is mostly obsolete.

Replace the old model:

- `faber` is not a TypeScript reference compiler anymore.
- `rivus`, `nanus-ts`, and `nanus-go` are archive/bootstrap history and should move to an archive/history page or be removed from primary navigation.
- A current architecture section should explain:
  - `crates/faber`: package/project CLI.
  - `crates/radix`: compiler pipeline and `radix` binary.
  - `crates/norma`: Rust runtime support.
  - `stdlib/norma`: Faber standard library definitions and `@ verte` metadata.
  - `explain`: embedded `faber explain` corpus.

### Language Docs

After migration, the site should import docs from repo-local `docs/grammatica`, not the old `fons/grammatica` path baked into the deleted sync script.

Priority docs to replace from source:

- `website/content/docs/grammar.md` from `EBNF.md`.
- `website/content/docs/fundamenta.md`
- `website/content/docs/typi.md`
- `website/content/docs/operatores.md`
- `website/content/docs/structurae.md`
- `website/content/docs/regimen.md`
- `website/content/docs/functiones.md`
- `website/content/docs/importa.md`
- `website/content/docs/errores.md`
- `website/content/docs/cli.md`
- `website/content/docs/targets.md`
- Add missing current docs:
  - manifest/package docs from `docs/grammatica/manifest.md`
  - testing docs from `docs/grammatica/test.md`
  - explain docs from `docs/grammatica/explain.md` if kept public

Specific content drift found:

- Old docs use broad multi-target matrices with Zig/Python/C++ as active surfaces.
- Old CLI docs use historical `@ optio bivalens n ...` syntax; current docs use `@ optio <ident> ... typus bivalens`.
- Old target docs describe Python/Zig/C++ support; current target contract is rust/go/ts/faber.
- Old examples include rivus/nanus and old manifest references such as `faber.fab`.
- Current docs prefer `nota` for neutral diagnostic output; `scribe` remains a compatibility alias.
- Current grammar uses `→`, `@ futura`, `@ cursor`, `sponte`, `fixus`, and `T ∪ nihil`; old content still contains older return and optionality forms in many places.

### Examples

Do not trust the current generated `content/docs/examples*.md` as current truth.

Options:

- Regenerate examples from repo-local `examples/exempla`.
- Add a smaller curated examples page based on current examples, with a link to source.
- Add an automation/package example page from `examples/automation` once the package story is stable.

The current examples pages are very large, include stale rivus/bootstrap examples, and will be hard to audit manually.

### Research

The research pages should be repositioned.

Options:

- Keep as "Research" but make clear it is a historical/adjacent learnability study, not the current compiler status.
- Link to `../faber-trials` material if public-facing.
- Remove from primary nav if the immediate website goal is current project documentation.

Avoid retaining homepage claims like "trials show 96-98%" unless the underlying published methodology and current framework are deliberately refreshed.

### Navigation And Information Architecture

Recommended primary nav after refresh:

- Overview
- Quick Start
- CLI
- Packages
- Targets
- Language Reference
- Examples
- Standard Library
- Architecture
- Research or Archive, if retained

The current `compilers` nav should not remain a primary section unless it is rewritten as current architecture/history.

## Implementation Phases

1. Create the monorepo website home.
   - Add `../faber/website/`.
   - Move or copy only useful source assets from `faber-www`: current content worth preserving, templates, styles, and this plan.
   - Do not move broken build state as-is unless it is useful archaeological context.
   - Add ignore rules for generated `website/dist` unless deployment intentionally requires committed output.

2. Add a repo-local site generator.
   - Prefer Rust inside `../faber`, either as a small crate or an `xtask`-style command.
   - Avoid `package.json`, Bun, Node, and `gray-matter` unless a later decision explicitly reverses this.
   - Parse TOML front matter (`+++` delimiters) as the page metadata format (see the `explain/` corpus for the established convention).
   - Render Markdown to HTML, build navigation, copy CSS/static assets, and write `website/dist`.
   - Generate `llms.txt` and `faber-complete.md` from current docs if those artifacts remain published.

3. Establish content import rules.
   - Point grammar import at repo-local `EBNF.md`.
   - Point prose docs at repo-local `docs/grammatica`.
   - Decide whether examples are generated from `examples/exempla`, curated under `website/content`, or both.
   - Preserve or generate TOML front matter (using `+++` delimiters) in `website/content`.

4. Replace stale content.
   - Homepage.
   - Current architecture/compiler pages.
   - Targets, CLI, packages, tests, examples.
   - Remove or demote archive/bootstrap pages.

5. Refresh LLM artifacts.
   - Regenerate `faber-complete.md`.
   - Regenerate `llms.txt`.
   - Ensure descriptions match current target and package surface.

6. Validate.
   - Run the new repo-local site build command.
   - Serve `dist` locally and inspect core pages.
   - Check links to local generated pages.
   - Confirm GitHub Pages workflow can publish a fresh `website/dist`.

7. Retire standalone `faber-www`.
   - Leave a README note or archive marker pointing to `../faber/website`.
   - Decide whether to keep the repo for redirect/deployment compatibility or archive it after the main repo deploys successfully.

## Acceptance Criteria

- The website source lives under `../faber/website`.
- The website builds from a clean `../faber` checkout with documented commands and no sibling-repo dependency.
- The default build path does not require `package.json`, Bun, Node, or YAML front matter (use `+++` TOML front matter instead of `---` YAML).
- The homepage no longer claims active Zig/Python/C++ package targets.
- The docs no longer describe TypeScript Faber, rivus, or nanus as the active compiler stack.
- The target page matches `cargo run -q -p faber -- targets` from the same `../faber` checkout.
- The CLI page matches `cargo run -q -p faber -- --help` and current `docs/grammatica/cli.md`.
- Package docs cover `faber.toml`, `faber init`, `faber build`, `faber run`, and `faber test` with current caveats.
- Archive/bootstrap/research material is either removed from primary navigation or explicitly labeled as historical/secondary.
- `llms.txt` and `faber-complete.md`, if published, are generated from current docs.

## Open Questions

- Should the public repo/link remain `ianzepp/faber-romanus`, or should the website now link to a renamed/public `faber` repository?
- Should the monorepo website folder be named `website/`, `site/`, or `www/`?
- Should the repo-local generator be a dedicated Cargo crate, an `xtask`, or eventually a Faber package once the language/runtime is ready?
- Should research pages remain public in this refresh, or be deferred until the current documentation is correct?
- Should archive pages be retained for historical continuity, or cut entirely from the first refreshed version?
- Should `website/dist` remain ignored and generated in CI, or should it be committed for GitHub Pages upload?

## Generator Decision (after Phase 1 landing)

**Chosen approach:** Dedicated small workspace member `crates/site` (invoked as `cargo run -p site -- build` or eventually wired into `faber site build`).

Rationale:
- Keeps the generator in the Rust-only toolchain (no Bun/Node for the default path, per AGENTS.md and this plan).
- Avoids bloating the published user-facing `faber` binary with markdown rendering + site deps (pulldown-cmark, etc. will live only in the site crate).
- Simple to implement a first version: walk `website/content/**/*.md`, parse TOML frontmatter delimited by `+++` (we already depend on `toml` + `serde` in the workspace; see `explain/` for examples), render bodies with `pulldown-cmark`, perform the trivial `{{title}}` / `{{content}}` / `{{nav}}` / `{{description}}` substitutions into `templates/layout.html`, emit static assets + synthesized `llms.txt` and `faber-complete.md`.
- The site crate can live alongside the other crates; it only needs to be built for docs publishing, not for normal `faber` releases.
- Future path: once the Faber language + norma runtime can comfortably do filesystem + markdown + toml work, the generator logic can be ported to a first-class `faber` package under `examples/` or `stdlib/`, fulfilling the "eventually a Faber package" option.

Interim fallback: if a pure-shell implementation in `scripta/website-build` using only repo tools is faster for v1, we can do that and still call it from CI. But the strong preference is the `crates/site` Rust binary.

This decision unblocks Phase 2 implementation.
