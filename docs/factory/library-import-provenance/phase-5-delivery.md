# Phase 5 Delivery: Docs And Fixtures

## Objective

Update active docs, guidance, and fixtures so current built-in Norma library
imports use provider-qualified syntax.

## Implementation

- Updated active related factory plans to teach provider-qualified imports:
  - `docs/factory/norma-http-hal/plan.md`
  - `docs/factory/stdlib-data-formats/plan.md`
  - `docs/factory/requirit-package-manager/plan.md`
- Updated the Norma HAL expansion pattern to require provider/import
  provenance keyed by `DefId` and forbid runtime identity from local receiver
  names, interface names, or method lists.
- Updated the active Norma HTTP runtime comment to refer to `norma:hal/http`.

## Scan Results

Remaining old built-in slash spellings are intentional:

- historical ledgers or completed phase records documenting previous behavior;
- the current library-import-provenance plan's negative examples and baseline
  discussion;
- the Phase 1 negative test source proving `norma/hal/http` is rejected;
- local relative import examples such as `./norma/json`, which are explicitly
  not provider imports.

## Validation

Run during the phase:

```bash
rg -n 'importa ex "norma/(json|toml|hal/http|hal/consolum)|`norma/(json|toml|hal/http|hal/consolum)`|"norma/(json|toml|hal/http|hal/consolum)"' . -g '*.md' -g '*.rs' -g '*.fab'
rg -n 'name/shape|method-list|method list|receiver names|receiver-name|runtime interface recognition|exact HIR shape|local receiver names' docs crates/radix/src/codegen/rust -g '*.md' -g '*.rs'
```

Result: scans showed no active guidance recommending old slash-form built-in
imports. Remaining matches are intentional historical, negative-test, or
relative-local cases.
