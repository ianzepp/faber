# Caretaker

## Active Queue

### member-visibility-contract-decision
- type: housekeeping
- status: pending
- priority: medium
- size: small
- discovered: 2026-05-07
- source: caretaker loop
- next slice: Decide whether `EBNF.md` should keep member-visibility annotations and `generis functio` as language contract, or whether `radix-rs` docs/spec should narrow to the currently implemented public-by-default plus static-field surface before more tutorial edits.
- notes: `radix-rs` rejects `generis functio`, field-position visibility annotations fail to parse, method-position annotations parse inconsistently with older `-um` fixture spellings, and lowering/codegen have no dedicated member-visibility model yet.
