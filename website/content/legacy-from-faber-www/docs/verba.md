---
title: Verba
section: docs
order: 50
---

# Verba

Reserved keywords in Faber Romanus. This reference catalogs all Latin keywords used throughout the language, organized by their grammatical and functional role.

**Implementation synchronization:** When the compiler recognizes a new keyword, it must appear in:

- `fons/nanus-ts/lexicon.ts` - Bootstrap compiler (TypeScript)
- `fons/rivus/lexicon/verba.fab` - Self-hosted compiler
- This document

---

## Control Flow

Keywords for directing program flow through conditionals, loops, and branching:

| Verbum     | Meaning     | Usage                                  |
| ---------- | ----------- | -------------------------------------- |
| `si`       | if          | Begin conditional branch               |
| `sin`      | else if     | Chained conditional                    |
| `secus`    | else        | Default branch; also `:` in ternary    |
| `dum`      | while       | Loop while condition holds             |
| `fac`      | do          | Do-while loop prefix                   |
| `pro`      | for         | Loop variable binding (with `ex`/`de`) |
| `elige`    | switch      | Value-based branching                  |
| `casu`     | case        | Branch within `elige`                  |
| `ceterum`  | default     | Fallback branch in `elige`             |
| `ergo`     | then        | Consequence marker (one-liner)         |
| `rumpe`    | break       | Exit loop                              |
| `perge`    | continue    | Skip to next iteration                 |
| `redde`    | return      | Return value from function             |
| `reddit`   | then return | Sugar for `ergo redde`                 |
| `custodi`  | guard       | Early-exit guard clause                |
| `adfirma`  | assert      | Runtime assertion                      |
| `discerne` | match       | Pattern matching on `discretio`        |

**Entry points:**

| Verbum     | Meaning    | Usage                            |
| ---------- | ---------- | -------------------------------- |
| `incipit`  | main       | Synchronous program entry point  |
| `incipiet` | async main | Asynchronous program entry point |

---

## Testing

Keywords for organizing test suites and cases:

| Verbum        | Meaning          | Usage                              |
| ------------- | ---------------- | ---------------------------------- |
| `probandum`   | describe         | Test suite declaration (gerundive) |
| `proba`       | test/it          | Individual test case               |
| `praepara`    | beforeEach       | Run before each test               |
| `praeparabit` | async beforeEach | Async setup before each test       |
| `postpara`    | afterEach        | Run after each test                |
| `postparabit` | async afterEach  | Async teardown after each test     |
| `omnia`       | all              | Modifier: beforeAll/afterAll       |
| `omitte`      | skip             | Skip this test                     |
| `futurum`     | todo             | Mark test as pending               |

---

## Error Handling

Keywords for managing errors and cleanup:

| Verbum   | Meaning    | Usage                                                         |
| -------- | ---------- | ------------------------------------------------------------- |
| `tempta` | try        | Begin error-handling block                                    |
| `cape`   | catch      | Handle thrown error                                           |
| `demum`  | finally    | Cleanup block (runs regardless)                               |
| `iace`   | throw      | Throw recoverable error                                       |
| `iacit`  | then throw | Sugar for `ergo iace` (also used as a post-function modifier) |
| `mori`   | panic      | Fatal/unrecoverable error                                     |

---

## Resource Management

Keywords for scoped resource and allocator management:

| Verbum  | Meaning | Usage                      |
| ------- | ------- | -------------------------- |
| `cura`  | care    | Scoped resource management |
| `arena` | arena   | Arena allocator kind       |
| `page`  | page    | Page allocator kind        |

See `cura.md` in `consilia/completa/` for full details on resource management.

---

## I/O and Debugging

Keywords for input, output, and diagnostic operations:

| Verbum     | Meaning     | Usage                        |
| ---------- | ----------- | ---------------------------- |
| `scribe`   | print       | Write to stdout              |
| `vide`     | debug       | Debug output                 |
| `mone`     | warn        | Warning output               |
| `lege`     | read        | Read input                   |
| `lineam`   | line        | With `lege`: read one line   |
| `scriptum` | format      | Create formatted string      |
| `cede`     | await/yield | Await promise or yield value |

---

## Declarations

Keywords for declaring variables, types, and structures:

### Variables

| Verbum  | Meaning | Usage             |
| ------- | ------- | ----------------- |
| `varia` | let     | Mutable binding   |
| `fixum` | const   | Immutable binding |

### Functions and Instantiation

| Verbum    | Meaning  | Usage                         |
| --------- | -------- | ----------------------------- |
| `functio` | function | Function declaration          |
| `novum`   | new      | Object instantiation          |
| `finge`   | form     | Construct `discretio` variant |

### Types and Modules

| Verbum      | Meaning         | Usage                            |
| ----------- | --------------- | -------------------------------- |
| `importa`   | import          | Import from module               |
| `exporta`   | export          | Export from module               |
| `typus`     | type            | Type alias; also `typeof` on RHS |
| `genus`     | class/struct    | Data structure with methods      |
| `pactum`    | interface/trait | Contract/interface               |
| `ordo`      | enum            | Enumeration                      |
| `discretio` | tagged union    | Discriminated union type         |

---

## Modifiers

Keywords that modify declarations, members, or execution behavior:

### Function Modifiers

| Verbum      | Meaning        | Usage                                                         |
| ----------- | -------------- | ------------------------------------------------------------- |
| `futura`    | async          | Async function annotation (`@ futura`)                        |
| `cursor`    | generator      | Generator function annotation (`@ cursor`)                    |
| `curata`    | managed        | Post-parameter modifier: allocator binding (`curata NAME`)    |
| `errata`    | errored        | Post-parameter modifier: error binding (`errata NAME`)        |
| `immutata`  | unchanged      | Post-parameter modifier: const/non-mutating qualifier         |
| `iacit`     | throws         | Post-parameter modifier: throws declaration (target-specific) |
| `prae`      | comptime       | Compile-time type parameter                                   |
| `praefixum` | comptime block | Compile-time evaluation                                       |

### Visibility

| Verbum      | Meaning   | Usage                                 |
| ----------- | --------- | ------------------------------------- |
| `publicus`  | public    | Public visibility (genus modifier)    |
| `publica`   | public    | Public visibility (functio modifier)  |
| `privatus`  | private   | Private field                         |
| `privata`   | private   | Private method                        |
| `protectus` | protected | Protected visibility (TS/Py/C++ only) |

### Type Relationships

| Verbum       | Meaning    | Usage                                  |
| ------------ | ---------- | -------------------------------------- |
| `generis`    | static     | Type-level/static member               |
| `implet`     | implements | Implement interface                    |
| `sub`        | extends    | Class inheritance (TS/Py/C++ only)     |
| `abstractus` | abstract   | Abstract class/method (TS/Py/C++ only) |

### Other

| Verbum  | Meaning | Usage                                |
| ------- | ------- | ------------------------------------ |
| `omnia` | all     | Modifier: beforeAll/afterAll (tests) |

---

## Operators

### Logical

| Verbum | Symbol | Meaning            |
| ------ | ------ | ------------------ |
| `et`   | `&&`   | Logical AND        |
| `aut`  | `\|\|` | Logical OR         |
| `non`  | `!`    | Logical NOT        |
| `vel`  | `??`   | Nullish coalescing |

### Comparison

| Verbum | Symbol | Meaning         |
| ------ | ------ | --------------- |
| `est`  | `===`  | Strict equality |

### Null and Empty Checks

Unary operators that expand to inline checks:

| Verbum        | Meaning           | Generated Code    |
| ------------- | ----------------- | ----------------- |
| `nihil x`     | is null           | `x == null`       |
| `nonnihil x`  | is not null       | `x != null`       |
| `nulla x`     | is empty          | length/size check |
| `nonnulla x`  | has content       | length/size check |
| `negativum x` | less than zero    | `x < 0`           |
| `positivum x` | greater than zero | `x > 0`           |

### Ternary

| Verbum  | Symbol | Usage                  |
| ------- | ------ | ---------------------- |
| `sic`   | `?`    | Then branch in ternary |
| `secus` | `:`    | Else branch in ternary |

### Return Type Verbs

Conjugations of _fio_ ("to become") indicating return behavior:

| Verbum  | Async | Generator | Meaning                              |
| ------- | :---: | :-------: | ------------------------------------ |
| `fit`   |  no   |    no     | "it becomes" (sync)                  |
| `fiet`  |  yes  |    no     | "it will become" (async)             |
| `fiunt` |  no   |    yes    | "they become" (sync generator)       |
| `fient` |  yes  |    yes    | "they will become" (async generator) |

### Range Operators

| Verbum  | Meaning | Usage                                |
| ------- | ------- | ------------------------------------ |
| `ante`  | before  | Exclusive range: `0 ante 10` = 0-9   |
| `usque` | up to   | Inclusive range: `0 usque 10` = 0-10 |
| `per`   | through | Iteration step: `0..10 per 2`        |
| `intra` | within  | Range containment: `x intra 0..100`  |
| `inter` | among   | Set membership: `x inter [1, 2, 3]`  |

### Spread and Rest

| Verbum   | Symbol | Usage           |
| -------- | ------ | --------------- |
| `sparge` | `...`  | Spread elements |
| `ceteri` | `...`  | Rest parameters |

---

## Literal Values

Keywords representing constant values:

| Verbum   | Meaning   | Type                       |
| -------- | --------- | -------------------------- |
| `verum`  | true      | Boolean                    |
| `falsum` | false     | Boolean                    |
| `nihil`  | null      | Null (also unary operator) |
| `ego`    | this/self | Self-reference in methods  |

---

## Prepositions

Latin prepositions used in various syntactic contexts:

| Verbum | Meaning    | Contexts                                                      |
| ------ | ---------- | ------------------------------------------------------------- |
| `de`   | from/of    | Key iteration (`itera de`); borrowed reference (Rust/Zig)     |
| `in`   | in/into    | Membership test; mutable reference (Rust/Zig); mutation block |
| `ex`   | from       | Value iteration (`itera ex`); module import (`importa ex`)    |
| `ad`   | to         | Target/destination (planned)                                  |
| `per`  | through    | Iteration step in ranges                                      |
| `qua`  | as (type)  | Type cast: `x qua textus`                                     |
| `ut`   | as (alias) | Rename in import/destructure: `nomen ut n`                    |

---

## Collection DSL

Keywords for collection manipulation (planned):

| Verbum    | Meaning     | Usage                 |
| --------- | ----------- | --------------------- |
| `ab`      | filter from | DSL entry point       |
| `ubi`     | where       | Filter condition      |
| `prima`   | first n     | Take first n elements |
| `ultima`  | last n      | Take last n elements  |
| `summa`   | sum         | Reduce to sum         |
| `ordina`  | sort        | Sort collection       |
| `collige` | pluck       | Extract field values  |
| `grupa`   | group by    | Group by key          |

---

## Regex DSL

| Verbum | Meaning | Usage                                   |
| ------ | ------- | --------------------------------------- |
| `sed`  | regex   | Pattern matching (Unix `sed` reference) |

---

## Planned Keywords

Not yet implemented:

| Verbum  | Meaning | Status                             |
| ------- | ------- | ---------------------------------- |
| `solum` | only    | Test modifier (run only this test) |

---

## Synchronization Checklist

When adding a new keyword to the language:

1. **Add to bootstrap compiler:** `fons/nanus-ts/lexicon.ts`
    - Include the keyword in the lexicon
2. **Add to bootstrap compiler:** `fons/rivus/lexicon/verba.fab`
    - Add to `estVerbum()` function
3. **Categorize if needed:**
    - If statement-starting: add to `estVerbumSententiae()`
    - If genus member: add to `estVerbumGeneris()`
4. **Update this document:** Add to appropriate section above
5. **Update grammatica:** If the keyword affects user-facing syntax, document in relevant `fons/grammatica/*.md` file
6. **Update EBNF:** If grammar rules change, update `EBNF.md`
7. **Add tests:** Create test cases in `fons/proba/` covering the new keyword

