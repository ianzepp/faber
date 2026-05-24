# Exempla Rust E2E Baseline Ledger

**Created**: 2026-05-24
**Roadmap Epic**: Epic 1
**Baseline Source**: `docs/factory/exempla-rust-e2e/goal.md`

This ledger accounts for the 138 `.fab` files collected under `examples/exempla/` at the start of Epic 1. It preserves the original `71/138` Rust e2e baseline while separating corpus truth from backend repair work.

After Epic 1 removes `ab/ab.fab`, the ignored Rust e2e harness reports `71/137` exempla passing. The remaining failures are Epic 2+ work and no longer include the retired `ab` DSL exemplar.

## Class Summary

| Class | Count | Meaning |
| --- | ---: | --- |
| Removed stale source | 1 | Retired syntax exemplar removed from executable corpus |
| Standalone Rust executable, baseline pass | 59 | Already passed the ignored Rust e2e harness in the baseline |
| Standalone Rust executable, backend/semantic failure | 40 | Valid or likely-valid source exposing current Rust backend gaps |
| Harness/package/dependency/runtime shape | 15 | Needs Cargo/package/runtime/dependency-aware validation, not raw standalone `rustc` |
| Test/package fixture | 8 | Belongs to package/test validation rather than ordinary executable file collection |
| Unsupported/future Rust target feature | 8 | Active or aspirational language surface without current Rust execution support |
| Stale, invalid, or declaration-only source | 7 | Needs source correction, relocation, or non-executable classification |
| Total original baseline | 138 | All original exempla accounted for |

## Removed Stale Source

First failure reason: retired collection DSL should not be kept alive for e2e pass-count optics.

- `ab/ab.fab`

## Harness, Package, Dependency, Or Runtime Shape

First failure reason: standalone `rustc` is the wrong execution shape or misses package/runtime/dependency context.

- `cli/main.fab`
- `expressionis/expressionis.fab`
- `hal/aleator.fab`
- `hal/consolum.fab`
- `hal/json.fab`
- `hal/processus.fab`
- `hal/yaml.fab`
- `importa/auxilia.fab`
- `importa/importa.fab`
- `proba/packages/failing/src/main.fab`
- `proba/packages/ignored/src/main.fab`
- `proba/packages/passing/src/main.fab`
- `proba/packages/selectors/src/main.fab`
- `proba/packages/solum/src/main.fab`
- `proba/packages/suite/src/main.fab`

## Test Or Package Fixtures

First failure reason: test harness or package-selection semantics, not standalone executable semantics.

- `proba/modificatores.fab`
- `proba/packages/selection-failure/src/main.fab`
- `proba/proba.fab`
- `omitte/omitte.fab`
- `postpara/postpara.fab`
- `praepara/praepara.fab`
- `futurum/futurum.fab`
- `figendum/figendum.fab`

## Unsupported Or Future Rust Target Features

First failure reason: current Rust target policy does not yet support the represented feature.

- `ad/ad.fab`
- `custodi/validatio.fab`
- `fac/cape.fab`
- `fac/fac.fab`
- `functio/exitus.fab`
- `iace/iace.fab`
- `annotatio/annotatio.fab`
- `annotatio/grammatica-nova.fab`

## Stale, Invalid, Or Declaration-Only Source

First failure reason: source is not currently a valid standalone executable Rust exemplar.

- `cli/commands/greet.fab`
- `externa/externa.fab`
- `literalis/literalis.fab`
- `qua/qua.fab`
- `tempta/in-functione.fab`
- `tempta/tempta.fab`
- `lege/lege.fab`

## Standalone Rust Executable, Backend Or Semantic Failure

First failure reason: baseline compile/link/runtime failure appears to be a compiler, lowering, codegen, type, or ownership issue rather than corpus shape.

- `assignatio/assignatio.fab`
- `clausa/clausa.fab`
- `conversio/conversio.fab`
- `destructura/destructura.fab`
- `destructura/objectum.fab`
- `discerne/discerne.fab`
- `elige/ceterum.fab`
- `elige/elige.fab`
- `elige/ergo-redde.fab`
- `elige/in-functione.fab`
- `finge/finge.fab`
- `functio/optionalis.fab`
- `functio/typicus.fab`
- `genus/creo.fab`
- `genus/methodi.fab`
- `incipiet/incipiet.fab`
- `innatum/innatum.fab`
- `inter/inter.fab`
- `itera/cursor-iteratio.fab`
- `itera/de.fab`
- `itera/in-functione.fab`
- `itera/intervallum-gradus.fab`
- `itera/intervallum.fab`
- `itera/nidificatus.fab`
- `lista/lista.fab`
- `membrum/membrum.fab`
- `mori/mori.fab`
- `morphologia/morphologia.fab`
- `objectum/objectum.fab`
- `optionalis/optionalis.fab`
- `ordo/ordo.fab`
- `pactum/pactum.fab`
- `praefixum/praefixum.fab`
- `redde/redde.fab`
- `si/ergo-redde.fab`
- `si/est.fab`
- `ternarius/ternarius.fab`
- `varia/destructura.fab`
- `vel/vel.fab`
- `vocatio/vocatio.fab`

## Standalone Rust Executable, Baseline Pass

These files were not listed in the baseline failure set and remain ordinary executable Rust corpus candidates unless a later harness-truth phase reclassifies them with stronger evidence.

- `abstractus/abstractus.fab`
- `adfirma/adfirma.fab`
- `adfirma/in-functione.fab`
- `ante/ante.fab`
- `aut/aut.fab`
- `binarius/binarius.fab`
- `cede/cede.fab`
- `ceteri/ceteri.fab`
- `cura/cura.fab`
- `cura/nidificatus.fab`
- `custodi/custodi.fab`
- `demum/demum.fab`
- `discretio/discretio.fab`
- `dum/conditio-complexa.fab`
- `dum/dum.fab`
- `dum/in-functione.fab`
- `ego/ego.fab`
- `est/est.fab`
- `et/et.fab`
- `fixum/fixum.fab`
- `functio/functio.fab`
- `functio/recursio.fab`
- `futura/futura.fab`
- `generis/generis.fab`
- `genus/genus.fab`
- `implet/implet.fab`
- `incipit/functionibus.fab`
- `incipit/incipit.fab`
- `intra/intra.fab`
- `itera/ex.fab`
- `mone/mone.fab`
- `nexum/nexum.fab`
- `nota/gradus.fab`
- `nota/nota.fab`
- `novum/novum.fab`
- `omnia/omnia.fab`
- `per/per.fab`
- `perge/perge.fab`
- `privatus/privatus.fab`
- `protectus/protectus.fab`
- `publicus/publicus.fab`
- `rumpe/rumpe.fab`
- `salve-munde.fab`
- `scriptum/scriptum.fab`
- `sed/sed.fab`
- `si/ergo.fab`
- `si/nidificatus.fab`
- `si/secus.fab`
- `si/si.fab`
- `si/sin.fab`
- `sparge/sparge.fab`
- `sub/sub.fab`
- `typus/typus.fab`
- `unarius/unarius.fab`
- `usque/usque.fab`
- `varia/typicus.fab`
- `varia/varia.fab`
- `variandum/variandum.fab`
- `vide/vide.fab`

## Epic 2 Handoff

The next epic should teach the harness to use these classes instead of treating every file as a standalone `rustc` unit. Some classifications above are intentionally conservative and should be revised only with stronger validation evidence.
