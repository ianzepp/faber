# Morphologia - Verb Conjugation as Semantic Dispatch

Latin verb morphology encodes semantic information. The compiler parses conjugation endings to derive behavior flags (mutation, async, streaming) and dispatches to stem-based code generators.

**Status:** Design proposal (POC implemented in Rivus)

---

## The Problem

Modern programming requires expressing multiple variants of the same operation:

| Variant       | JavaScript        | Python         | Rust                  |
| ------------- | ----------------- | -------------- | --------------------- |
| Mutate, sync  | `arr.sort()`      | `list.sort()`  | `vec.sort()`          |
| Copy, sync    | `[...arr].sort()` | `sorted(list)` | `vec.iter().sorted()` |
| Mutate, async | ???               | ???            | ???                   |
| Copy, async   | ???               | ???            | ???                   |
| Stream/lazy   | `arr.values()`    | `iter(list)`   | `vec.iter()`          |
| Async stream  | `for await...`    | `async for...` | `Stream` trait        |

Languages handle this inconsistently:

- **Naming conventions**: `sort`/`sorted`, `reverse`/`reversed`, `readFileSync`/`readFile`
- **Separate APIs**: sync vs async modules, eager vs lazy collections
- **Method suffixes**: `_mut`, `_async`, `_iter`

Each approach requires memorizing arbitrary conventions that vary by operation and domain.

---

## The Solution

Latin verb conjugation already encodes these semantics:

| Form               | Latin Name            | Ending                   | Semantics                      |
| ------------------ | --------------------- | ------------------------ | ------------------------------ |
| Imperative         | Imperativus           | `-a`, `-e`, `-i`         | Command: mutate now            |
| Perfect Participle | Participium Perfectum | `-ata`, `-ita`, `-ta`    | Completed: return result       |
| Future Indicative  | Futurum Indicativum   | `-abit`, `-ebit`, `-iet` | Will do: async mutate          |
| Future Participle  | Participium Futurum   | `-atura`, `-itura`       | About to produce: async result |

This maps directly to a semantic matrix:

|                 | Sync                             | Async                               |
| --------------- | -------------------------------- | ----------------------------------- |
| **Mutates**     | `-a/-e/-i` (imperative)          | `-abit/-ebit` (future indicative)   |
| **Returns new** | `-ata/-ita` (perfect participle) | `-atura/-itura` (future participle) |

The conjugation ending tells you everything:

- `filtra` → mutates in place, synchronous
- `filtrata` → returns new collection, synchronous
- `filtrabit` → mutates in place, asynchronous
- `filtratura` → returns new collection, asynchronous

---

## Why This Matters: The IO Layer

For in-memory collections, morphology provides modest value—7 operations with meaningful imperative/participle pairs. The real payoff comes with IO-bound types where **async is the default** and **streaming is common**.

### Stdlib Domains

| Type         | Latin       | Domain     | Primary Mode      | Morphology Value |
| ------------ | ----------- | ---------- | ----------------- | ---------------- |
| **mathesis** | "learning"  | Math       | Sync              | None             |
| **tempus**   | "time"      | Date/time  | Sync              | None             |
| **textus**   | "woven"     | Strings    | Sync              | Low              |
| **lista**    | "list"      | Arrays     | Sync              | Medium           |
| **tabula**   | "table"     | Maps       | Sync              | Medium           |
| **copia**    | "abundance" | Sets       | Sync              | Medium           |
| **solum**    | "ground"    | Filesystem | Async + streaming | **High**         |
| **caelum**   | "sky"       | Network    | Async + streaming | **High**         |
| **arca**     | "chest"     | Database   | Async + streaming | **High**         |
| **nucleus**  | "kernel"    | System/IPC | Async + streaming | **High**         |

### IO Operations Matrix

For IO types, every operation naturally has multiple variants:

#### solum (Filesystem)

| Operation      | Sync      | Async        | Generator   |
| -------------- | --------- | ------------ | ----------- |
| Read file      | `lege`    | `leget`      | `legens`    |
| Write file     | `scribe`  | `scribet`    | `scribens`  |
| List directory | `enumera` | `enumerabit` | `enumerans` |
| File exists    | `exstat`  | `exstabit`   | —           |
| Delete         | `dele`    | `delebit`    | —           |

#### caelum (Network)

| Operation        | Sync | Async   | Generator    |
| ---------------- | ---- | ------- | ------------ |
| HTTP fetch       | —    | `pete`  | `petens`     |
| Send data        | —    | `mitte` | `mittens`    |
| WebSocket listen | —    | —       | `auscultans` |
| Stream response  | —    | —       | `fluit`      |

#### arca (Database)

| Operation   | Sync     | Async       | Generator  |
| ----------- | -------- | ----------- | ---------- |
| Query       | `quaere` | `quaeret`   | `quaerens` |
| Insert      | —        | `inseret`   | —          |
| Update      | —        | `mutabit`   | —          |
| Delete      | —        | `delebit`   | —          |
| Transaction | —        | `transiget` | —          |

#### nucleus (Microkernel)

| Operation      | Sync      | Async        | Generator   |
| -------------- | --------- | ------------ | ----------- |
| Spawn process  | `genera`  | `generabit`  | —           |
| Wait for child | `expecta` | `expectabit` | —           |
| Send signal    | `signa`   | `signabit`   | —           |
| Listen signals | —         | —            | `signans`   |
| IPC receive    | `accipe`  | `accipiet`   | `accipiens` |

### The Value Proposition

Without morphology, IO operations require ad-hoc naming:

```javascript
// Node.js style
fs.readFileSync(path);
fs.readFile(path, callback);
fs.promises.readFile(path);
fs.createReadStream(path);
```

With morphology, the verb form encodes the behavior:

```faber
solum.lege(path)      // sync read
solum.leget(path)     // async read (Promise)
solum.legens(path)    // streaming read (Generator)
```

The pattern is **consistent across all domains**. Learn the conjugation once, apply everywhere.

---

## Collection Methods Analysis

### lista Methods by Radix

| Stem         | Imperative (mutates) | Participle (returns new) | Notes           |
| ------------ | -------------------- | ------------------------ | --------------- |
| **add-**     | `adde`               | `addita`                 | Full pair       |
| **praepon-** | `praepone`           | `praeposita`             | Full pair       |
| **remov-**   | `remove`             | `remota`                 | Full pair       |
| **decapit-** | `decapita`           | `decapitata`             | Full pair       |
| **filtr-**   | `filtra`             | `filtrata`               | Full pair       |
| **ordin-**   | `ordina`             | `ordinata`               | Full pair       |
| **invert-**  | `inverte`            | `inversa`                | Full pair       |
| **purg-**    | `purga`              | —                        | Mutate only     |
| **mapp-**    | —                    | `mappata`                | Participle only |
| **reduct-**  | —                    | `reducta`                | Participle only |
| **explan-**  | —                    | `explanata`              | Participle only |
| **plan-**    | —                    | `plana`                  | Participle only |
| **sect-**    | —                    | `sectio`                 | Participle only |
| **prim-**    | —                    | `prima`                  | Participle only |
| **ultim-**   | —                    | `ultima`                 | Participle only |
| **unic-**    | —                    | `unica`                  | Participle only |

#### Read-Only Methods (No Morphology)

| Method      | Type       | Meaning       |
| ----------- | ---------- | ------------- |
| `longitudo` | noun       | Length        |
| `primus`    | adjective  | First element |
| `ultimus`   | adjective  | Last element  |
| `continet`  | verb (3sg) | Contains      |
| `vacua`     | adjective  | Is empty      |
| `inveni`    | verb       | Find element  |
| `omnes`     | adjective  | All match     |
| `aliquis`   | pronoun    | Any matches   |
| `summa`     | noun       | Sum           |
| `minimus`   | adjective  | Minimum       |
| `maximus`   | adjective  | Maximum       |

### copia Methods by Radix

| Stem            | Imperative (mutates) | Participle (returns new) | Notes       |
| --------------- | -------------------- | ------------------------ | ----------- |
| **add-**        | `adde`               | —                        | Mutate only |
| **del-**        | `dele`               | —                        | Mutate only |
| **purg-**       | `purga`              | —                        | Mutate only |
| **uni-**        | —                    | `unio`                   | Returns new |
| **intersect-**  | —                    | `intersectio`            | Returns new |
| **differenti-** | —                    | `differentia`            | Returns new |
| **symmetr-**    | —                    | `symmetrica`             | Returns new |

#### Read-Only Methods (No Morphology)

| Method       | Meaning     |
| ------------ | ----------- |
| `habet`      | Has element |
| `longitudo`  | Size        |
| `vacua`      | Is empty    |
| `subcopia`   | Is subset   |
| `supercopia` | Is superset |

### tabula Methods by Radix

| Stem        | Imperative (mutates) | Participle (returns new) | Notes                  |
| ----------- | -------------------- | ------------------------ | ---------------------- |
| **pon-**    | `pone`               | —                        | Mutate only            |
| **del-**    | `dele`               | —                        | Mutate only            |
| **purg-**   | `purga`              | —                        | Mutate only            |
| **confl-**  | `confla`             | —                        | Mutate only (merge)    |
| **invert-** | —                    | `inversa`                | Returns new (swap k/v) |
| **selig-**  | —                    | `selige`                 | Returns new (pick)     |
| **omitt-**  | —                    | `omitte`                 | Returns new (omit)     |

#### Read-Only Methods (No Morphology)

| Method      | Meaning          |
| ----------- | ---------------- |
| `accipe`    | Get value        |
| `accipeAut` | Get or default   |
| `habet`     | Has key          |
| `longitudo` | Size             |
| `vacua`     | Is empty         |
| `claves`    | Keys iterator    |
| `valores`   | Values iterator  |
| `paria`     | Entries iterator |

### Summary

| Category                             | Count | Morphology                        |
| ------------------------------------ | ----- | --------------------------------- |
| Full pairs (imperative + participle) | 7     | `@ radix(imperativus, perfectum)` |
| Mutate-only                          | 9     | `@ radix(imperativus)`            |
| Participle-only                      | 20    | `@ radix(perfectum)`              |
| Read-only                            | 42    | None                              |

Collections benefit modestly from morphology. The system is **designed for IO domains** where async and streaming variants are essential.

---

## Annotation-Based Declaration

Functions declare valid conjugations via annotation:

```faber
@ radix(imperativus, perfectum, futurum_activum)
functio filtra<T>(praedicatum: functio(T) fit bivalens) {
    # Single implementation - compiler generates variants
}
```

The compiler:

1. Parses the stem from the function name (`filtra` → `filtr-`)
2. Validates called conjugations against declared variants
3. Generates target code with appropriate semantics per variant

### Morphology as Single Source of Truth

Currently, Faber uses return type modifiers to indicate async/generator semantics:

| Modifier | Meaning                 |
| -------- | ----------------------- |
| `fit`    | Sync return             |
| `fiet`   | Async return (Promise)  |
| `fiunt`  | Sync generator (yields) |
| `fient`  | Async generator         |

With morphology in place, this creates redundancy:

```faber
# Redundant - saying the same thing twice
functio filtratura<T>(...) fiet lista<T> { ... }
#        ^^^^^^^^^ async       ^^^^ async
```

**Two systems coexist:**

Morphology only applies to Latin-named stdlib functions. User code uses arbitrary names (English, German, etc.) that don't follow Latin conjugation patterns.

| Context                                 | Async/generator mechanism           |
| --------------------------------------- | ----------------------------------- |
| Latin stdlib (lista, solum, arca, etc.) | Morphology (`-atura`, `-ans`, etc.) |
| User-defined functions                  | `fit`/`fiet`/`fiunt`/`fient`        |

```faber
# STDLIB: Morphology encodes semantics
lista.filtratura(predicate)   # -atura → async, returns new
solum.legens(path)            # -ens → generator, streaming

# USER CODE: fit/fiet/fiunt/fient required
functio fetchUserData(id: numerus) fiet User {
    # fiet indicates async - no morphology available
}

functio iteratePages(url: textus) fiunt Pagina {
    # fiunt indicates generator - no Latin conjugation
}
```

| Conjugation                         | Semantics          | Stdlib equivalent |
| ----------------------------------- | ------------------ | ----------------- |
| Imperative (`-a/-e/-i`)             | Sync, mutates      | `fit` (default)   |
| Perfect Participle (`-ata/-ita`)    | Sync, returns new  | `fit`             |
| Future Indicative (`-abit/-ebit`)   | Async, mutates     | `fiet`            |
| Future Participle (`-atura/-itura`) | Async, returns new | `fiet`            |
| Present Participle (`-ans/-ens`)    | Generator          | `fiunt`           |
| Future + Generator                  | Async generator    | `fient`           |

**For stdlib:** Morphology is the source of truth. The `@ radix` annotation declares valid conjugations.

**For user code:** `fit`/`fiet`/`fiunt`/`fient` remain essential. No deprecation.

### Generated Output (TypeScript)

```typescript
// From: @ radix(imperativus, perfectum, futurum_activum)

// imperativus: filtra() - mutates in place, sync
filtra(predicate: (x: T) => boolean): void {
    for (let i = this.length - 1; i >= 0; i--) {
        if (!predicate(this[i])) this.splice(i, 1);
    }
}

// perfectum: filtrata() - returns new, sync
filtrata(predicate: (x: T) => boolean): T[] {
    return this.filter(predicate);
}

// futurum_activum: filtratura() - returns new, async
async filtratura(predicate: (x: T) => Promise<boolean>): Promise<T[]> {
    const results = await Promise.all(
        this.map(async x => ({ x, keep: await predicate(x) }))
    );
    return results.filter(r => r.keep).map(r => r.x);
}
```

### Generated Output (Zig)

```zig
// imperativus: filtra() - mutates in place
pub fn filtra(self: *Lista(T), predicate: fn(T) bool) void {
    // in-place filter
}

// perfectum: filtrata() - returns new, needs allocator
pub fn filtrata(self: Lista(T), allocator: Allocator, predicate: fn(T) bool) !Lista(T) {
    var result = Lista(T).init(allocator);
    // copy and filter
    return result;
}
```

### No Annotation = No Morphology

Functions without `@ radix` compile directly with no conjugation variants:

```faber
# tempus - no morphology, just regular functions
functio nunc() fit Tempus {
    # returns current time
}

functio addHoras(t: Tempus, h: numerus) fit Tempus {
    # always returns new (immutable type)
}
```

---

## Morphology Flags

```faber
genus MorphologiaFlagga {
    bivalens mutare       # Modifies receiver in place
    bivalens async        # Returns Promise/Future
    bivalens reddeNovum   # Returns new collection
    bivalens allocatio    # Needs allocator (Zig target)
    bivalens generator    # Yields items (streaming)
}
```

### Flag Mapping by Conjugation

| Form               | mutare | async | reddeNovum | allocatio | generator |
| ------------------ | :----: | :---: | :--------: | :-------: | :-------: |
| Imperative         |  yes   |  no   |     no     |    no     |    no     |
| Perfect Participle |   no   |  no   |    yes     |    yes    |    no     |
| Future Indicative  |  yes   |  yes  |     no     |    no     |    no     |
| Future Participle  |   no   |  yes  |    yes     |    yes    |    no     |
| Present Participle |   no   |  no   |     no     |    no     |    yes    |
| Future + Generator |   no   |  yes  |     no     |    no     |    yes    |

---

## Zig Codegen: Morphology and State Machines

Zig removed native async/await in version 0.11. Faber compiles async/generator functions to state machines using the Responsum protocol (see `consilia/futura/zig-async.md`).

### Three Paths to Async

| Source                   | Example                       | Zig Support                |
| ------------------------ | ----------------------------- | -------------------------- |
| `futura`/`cursor` + `->` | `futura functio fetch() -> T` | **Error** (TS/Python only) |
| Return verb              | `functio fetch() fiet T`      | Supported                  |
| Morphology               | `solum.leget(path)`           | Supported                  |

Both morphology and return verbs compile to the **same state machine pattern**:

```faber
# STDLIB: Morphology-based
figendum data = solum.leget(path)        # -et → Future
ex solum.legens(path) pro chunk { ... }  # -ens → Iterator

# USER CODE: Verb-based
functio fetchUser(id: numerus) fiet User { ... }  # fiet → Future
functio iterateRows() fiunt Row { ... }           # fiunt → Iterator
```

### Conjugation to State Machine Mapping

| Conjugation                               | Return Verb | Zig Output                         |
| ----------------------------------------- | ----------- | ---------------------------------- |
| Future Indicative (`-abit`/`-ebit`/`-et`) | `fiet`      | `Future` with `poll()`             |
| Future Participle (`-atura`/`-itura`)     | `fiet`      | `Future` with `poll()`             |
| Present Participle (`-ans`/`-ens`)        | `fiunt`     | `Iterator` with `next()`           |
| Future + Generator                        | `fient`     | `AsyncIterator` with `poll_next()` |

### Generated Structure

Both morphology-based and verb-based async produce identical Zig structures:

```zig
// From lista.filtratura(pred) OR from functio foo() fiet T
const FilterFuture = struct {
    state: FilterState,

    pub fn poll(self: *@This(), ctx: *AsyncContext) Responsum(Lista(T)) {
        switch (self.state) {
            .start => |s| { ... },
            .awaiting => |s| { ... },
        }
    }
};

// From solum.legens(path) OR from functio foo() fiunt T
const ReadIterator = struct {
    state: ReadState,

    pub fn next(self: *@This()) Responsum(Chunk) {
        switch (self.state) {
            .reading => |s| { ... },
            .done => return .done,
        }
    }
};
```

### Detection Unification

The compiler detects async/generator semantics from either source:

```
┌─────────────────┐     ┌─────────────────┐
│ Morphology      │     │ Return Verb     │
│ parseMethodum() │     │ fiet/fiunt/fient│
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     ▼
         ┌───────────────────────┐
         │ Async/Generator Flag  │
         │ on Function/Call      │
         └───────────┬───────────┘
                     ▼
         ┌───────────────────────┐
         │ State Machine Codegen │
         │ (same for both paths) │
         └───────────────────────┘
```

### Allocator Threading

Zig state machines need allocators for:

- State struct allocation
- Result collection allocation (for `-atura` forms that return new collections)

The `allocatio` flag from morphology signals this requirement:

```faber
# filtratura needs allocator: async=true, reddeNovum=true, allocatio=true
figendum filtered = lista.filtratura(curator, predicate)
```

```zig
pub fn filtratura(
    self: Lista(T),
    allocator: Allocator,      // from allocatio flag
    predicate: fn(T) bool,
) FilterFuture(T) {
    return FilterFuture(T).init(allocator, self, predicate);
}
```

### Open Questions (Zig-specific)

1. **Nested futures**: How does a morphology-based call inside a `fiet` function affect state machine composition?

2. **Cancellation**: Present participle generators (`-ens`) imply iteration—what cleanup happens if iteration stops early?

3. **Error propagation**: Do morphology-based async calls use Responsum `.err` or Zig error unions (`!T`)?

See `consilia/futura/zig-async.md` for full state machine design.

---

## Current Implementation (POC)

The proof-of-concept in Rivus demonstrates the core pattern:

### Parser: `fons/rivus/parser/morphologia.fab`

Parses method names to extract stem and flags:

```faber
@ publica
functio parseMethodum(textus nomen) fit RadixEtFlagga? {
    # Check endings longest-first
    # -atura/-itura (5 chars) - future active participle
    # -abit/-ebit (4 chars) - future indicative
    # -ata/-ita/-ta (3 chars) - perfect passive participle
    # -a/-e/-i (1 char) - imperative
}
```

### Stem Registry: `fons/rivus/codegen/radices.fab`

Maps stems to code generators:

```faber
@ publica
functio generaListaMethodum(
    textus radix,
    textus obj,
    lista<textus> args,
    MorphologiaFlagga flagga
) fit textus? {
    elige radix {
        casu "add" reddit genAdd(obj, args, flagga)
        casu "filtr" reddit genFiltr(obj, args, flagga)
        casu "ordin" reddit genOrdin(obj, args, flagga)
        # ...
    }
}
```

### Call Dispatch: `fons/rivus/codegen/ts/expressia/index.fab`

At call sites, check for morphology-based dispatch:

```faber
casu VocatioExpressia ut e {
    # Check if callee is member access with recognized verb
    fixum parsed = parseMethodum(methodNomen)
    si nonnihil parsed {
        si estRadixListae(parsed.radix) {
            redde generaListaMethodum(parsed.radix, obj, args, parsed.flagga)
        }
    }
    # Fall through to normal call handling
}
```

### Limitations of POC

1. **No validation**: All conjugations accepted if stem is recognized
2. **No annotations**: Stems hardcoded in registry, not declared on functions
3. **TS-only**: Zig codegen not implemented for async variants
4. **Collections only**: IO types not yet implemented

---

## Implementation Roadmap

### Phase 1: Annotation Support (Current)

- Add `@ radix(...)` annotation parsing
- Validate conjugations against declared variants
- Error on unrecognized morphology

### Phase 2: Multi-Target Codegen

- Zig codegen with allocator threading
- Async patterns for each target
- Generator/streaming support

### Phase 3: IO Types

- Implement solum (filesystem) with full morphology
- Implement caelum (network) with streaming
- Implement arca (database) with cursors
- Implement nucleus (microkernel) with IPC

### Phase 4: Faber Migration

- Port morphology system from Rivus to Faber
- Migrate lista/tabula/copia to stem-based codegen
- Remove duplicated method entries

---

## Open Questions

1. **Generator conjugations**: Latin present participle (`filtrans` = "filtering") could encode generators. Is this the right mapping?

2. **Irregular verbs**: Some Latin verbs have irregular participles (`fero` → `latus`). Restrict to regular verbs or handle exceptions?

3. **Zig async**: Zig doesn't have async/await. What's the codegen pattern? Callbacks? Event loop integration?

4. **Error messages**: When morphology parsing fails, show parsed stem or original method name?

5. **Property access**: `items.longitudo` (no parens) doesn't go through call dispatch. Need type info for property translation.

---

## GPT Notes

- **Rivus-only direction**: Treat `fons/faber` as a static reference; implement and dogfood morphology in `fons/rivus` only (parser → AST → semantic → codegen). The roadmap item “Phase 4: Faber Migration” is likely obsolete under this strategy.

- **Preferred `@ radix` surface syntax**: Use a line-based form, not parentheses:

    ```faber
    @ radix imperativus, perfectum, futurum_indicativum, futurum_activum
    functio filtra<T>(...) fit vacuum { ... }
    ```

    In Rivus, parse the comma-separated identifiers only while tokens remain on the same source line as the `@` annotation.

- **Avoid `@ radix(...)` drift**: A prior attempt implemented parenthesized args and was reverted. Don’t reintroduce parentheses syntax unless you explicitly want to support both forms long-term.

- **Model annotations as data, not side effects**: Rivus currently “executes” a couple annotations in the statement parser (`@ futura`, `@ abstractum`). For morphology, preserve annotations on declarations (top-level functions and genus/pactum methods) so semantic/codegen can consult them uniformly.

- **Naming: don’t call conjugation lists `radices`**: In this design, `radix` already means “stem”. Store the declared conjugations/forms under a name like `formae`, `conjugationes`, or `morphologia` to avoid confusion once a real stem registry exists.

- **Centralize annotation parsing**: `pactum`/`genus` member parsers should reuse the same annotation parsing logic as the top-level statement parser to avoid divergence.

- **Registry + validation**: Use `@ radix ...` declarations to build a stem registry (per receiver type and/or module). At call sites where `parseMethodum()` recognizes morphology:
    - error if the stem is not declared as morphology-enabled
    - error if the conjugation is not in the declared set
    - keep error messages stem-aware (show parsed stem + allowed conjugations)

- **Avoid hidden `await` in codegen**: The current Rivus POC emits `await` inside generated expressions for async morphology variants. This can generate invalid JS in non-async contexts and bypass existing “await outside async” checks. Prefer: morphology determines return type (Promise vs value), but awaiting remains explicit via `figendum`/`variandum` (and `cede` inside `fiet`/`fient`).

- **Irregular stems**: The POC already hints at stem irregularity (e.g., `invert-` vs `invers-`). If morphology is driven by declared stems, allow a declaration to pin the canonical stem (or list alternate stems) rather than relying purely on stripping suffixes.

## Gemini 3 Opinion: Morphologia & The Latin-First Thesis

**Verdict:** The proposal is **strongly aligned** with the language's core thesis. It leverages Latin's high-density inflectional grammar to solve a real engineering problem (API proliferation). However, the implementation strategy carries significant risks regarding type safety and ecosystem bifurcation that must be addressed to maintain the "industrial quality" principle.

### 1. The Strong Case: Latin Grammar as a Compression Algorithm

The core insight—that Latin conjugation encodes the _exact_ semantic axes modern programming struggles with (Sync/Async, Mutate/Copy)—is brilliant.

- **Authenticity:** Using _Future Indicative_ (`-abit`) for async mutation ("it will do X") and _Future Participle_ (`-atura`) for async result ("about to produce X") isn't just a naming convention; it's syntactically truthful.
- **Economy:** It reduces API surface area. Instead of learning 4 methods (`sort`, `toSorted`, `sortAsync`, `toSortedAsync`), the user learns 1 root (`ordin-`) and applies standard grammar. This is the definition of a "Latin-first" advantage.

### 2. Critical Flaw: Lexical Dispatch vs. Type Safety

The current POC (and to some extent the proposal) relies on **lexical dispatch**.

- **The Risk:** In `fons/rivus/codegen/ts/expressia/index.fab`, dispatch happens if `parseMethodum` finds a valid stem _regardless of the object's type_.
- **Scenario:** If a user defines `genus Calculator` with a method `adde(n: numerus)`, the compiler currently sees `add-` + imperative. It checks `estRadixListae("add")` -> true. It then generates `calculator.push(n)`, which will crash at runtime.
- **Requirement:** Morphology **must** be tied to the type system. The compiler cannot assume `adde` implies `lista`. It must verify the receiver is a type that _opted in_ to the `add-` stem via `@ radix`.

### 3. The "Hidden Await" Danger

The proposal notes this in "GPT Notes", but it bears repeating: **morphology cannot auto-inject `await`**.

- **The Problem:** If `solum.leget(path)` compiles to `await fs.readFile(...)`, using it in a synchronous `fit` function creates invalid TypeScript (`await` is only valid in `async` functions).
- **The Fix:** Morphology should determine the **Return Type** (e.g., returns `Future<T>`), not the control flow. The user must still explicitly handle the async result (using `cede` or assigning to `figendum`). The "verb form determines behavior" principle should mean "determines what object I get back," not "injects invisible control flow."

### 4. The Ecosystem Bifurcation (Stdlib vs. User)

The proposal accepts a split world:

- **Stdlib:** Elegant Latin morphology (`solum.leget`).
- **User Code:** Clunky auxiliary verbs (`functio fetch() fiet User`).

**Criticism:** This weakens the thesis. If Latin is the "interface for humans," users should be able to write Latin-interface code.

- **Suggestion:** Allow users to define morphology mapping for their own types easily.
    ```faber
    genus Repository {
        # Maps 'quaer-' stem to this implementation
        @ radix(quaer-, futurum_indicativum)
        functio findUser(id) { ... }
    }
    ```
    If the language treats Latin morphology as a privileged feature of the standard library only, it becomes a "magic" system rather than a language feature.

### 5. Generator Semantics (`-ans`/`-ens`)

Mapping _Present Participle_ (adjective: "reading [thing]") to _Generator_ (iterator) is acceptable but slightly loose.

- **Latin Nuance:** A participle describes a state. `legens` = "one who is reading".
- **Alternative:** The _Gerund_ (`-endum`) might be more "process-oriented" ("for reading"), but the Participle is likely the best pragmatic fit for "a thing that yields values." Sticking with `-ans/-ens` is defensible.

### Summary of Recommendations

1.  **Strict Type Binding:** Do not implement global stem registries. Morphology dispatch must be scoped to the specific type definition (via `@ radix` or similar).
2.  **Explicit Control Flow:** `filtrabit` should return a `Promise`/`Future`, not auto-await. The user must still write `cede lista.filtrabit(...)` or `figendum x = lista.filtrabit(...)`. This preserves the "Mechanically Certain" principle.
3.  **Democratize Morphology:** Ensure the `@ radix` annotation is robust enough for user-land code, preventing the standard library from feeling "magical."
4.  **Irregularity Strategy:** Embrace Latin irregularity. If `fero` is the verb, allow defining `latus` as the participle stem explicitly in the annotation, rather than trying to algorithmically derive it.

---

## GLM 4.7 Notes

**Verdict:** Morphologia is a **strong concept** aligned with the Latin-first thesis, but has **critical implementation issues** that violate core principles. The idea is sound — POC needs fundamental fixes.

### Strengths

#### 1. Deeply Latin

Verb conjugation encoding mutation/async/generator semantics is authentically Latin and mechanically certain. The mapping is elegant:

| Form                                | Latin Meaning    | Code Semantics     |
| ----------------------------------- | ---------------- | ------------------ |
| `-a/-e/-i` (imperative)             | "do this!"       | Sync, mutates      |
| `-ata/-ita` (perfect participle)    | "done"           | Sync, returns new  |
| `-abit/-ebit` (future indicative)   | "will do"        | Async, mutates     |
| `-atura/-itura` (future participle) | "will have done" | Async, returns new |

This is not "naming convention pretending to be Latin" — it leverages actual grammatical structure. The correspondence between tense/aspect and execution model is conceptually sound.

#### 2. IO Domain Excellence

The real payoff is IO types where **every operation has 3-4 natural variants**:

```faber
solum.lege(path)      # sync read
solum.leget(path)     # async read (Promise)
solum.legens(path)    # streaming read (Generator)
```

This solves a real problem elegantly — no more `fs.readFileSync` vs `fs.readFile` vs `fs.createReadStream` inconsistency. The design correctly identifies that "collections benefit modestly" — morphology's power is in IO-bound domains where async/streaming is the norm.

#### 3. Consistent Pattern

Learn conjugation once, apply everywhere. No memorizing `sort`/`sorted`, `reverse`/`reversed`, `readFileSync`/`readFile`. This is exactly what "LLM-readable" promises — patterns so consistent deviation feels like a bug.

#### 4. Preserves User Flexibility

Morphology for Latin stdlib + `fit`/`fiet`/`fiunt`/`fient` for user code is the right boundary. User code uses English/German/Arbitrary names anyway — morphology wouldn't apply. The design correctly refuses to deprecate return verbs (morphologia.md:314-315).

---

### Critical Issues

#### 1. Hidden `await` Violates "Mechanically Certain"

**POC generates hidden `await` in async variants** (radices.fab:31, 84, 91):

```faber
# POC does this:
genAdd: scriptum("await (async () => {{ §.push(§); }})()", ...)
```

This bypasses explicit `figendum`/`cede` semantics and can generate invalid JS in non-async contexts. The GPT notes correctly flag this ("Avoid hidden await in codegen"). **Async semantics must be explicit.**

**Fix:** Morphology determines return type (`Promise<T>` vs `T`), but awaiting remains explicit via `figendum`/`cede`. The function body shouldn't emit `await`. The "verb form determines behavior" principle should mean "determines what object I get back," not "injects invisible control flow."

---

#### 2. Stem Irregularity Violates "Mechanically Certain"

The design admits stems need manual mapping (morphologia.fab:139-151):

```faber
casu "invert" reddit verum    # present: inverte
casu "invers" reddit verum    # participle: inversa
casu "inver" reddit verum     # participle with -sa: inversa
```

This contradicts the "mechanically certain" principle — compiler should deduce stems, not maintain manual mapping tables.

**Concern:** If every irregular verb needs hardcoded handling, the system isn't actually parsing Latin morphology. It's pattern-matching on known forms. The proposal claims morphological dispatch, but implementation is lookup table dispatch.

**Question:** Why can't stem be derived from function name without a registry? `inverte` → strip `-e` → `invert-`, `inversa` → strip `-a` → `invers-`. The irregularity is in the ending, not the stem. Latin has systematic conjugation patterns — why is stem irregularity handled manually?

---

#### 3. Partial Coverage Creates Inconsistency

Only **7 of 78 collection methods** have full imperative/participle pairs:

| Category        | Count | Morphology                |
| --------------- | ----- | ------------------------- |
| Full pairs      | 7     | `filtra`/`filtrata`       |
| Mutate-only     | 9     | `adde` (no `addita`???)   |
| Participle-only | 20    | `mappata` (no `mappa`???) |
| Read-only       | 42    | `longitudo`               |

This creates cognitive load: which methods support morphology? Which don't? A "mechanically certain" system should not have arbitrary gaps.

**Concern:** The proposal says "collections benefit modestly" — but then why include them at all? If the system is **designed for IO domains**, why half-implement collections? This is the worst of both worlds: complexity of partial implementation without full payoff.

**Option 1:** Drop collection morphology entirely. Keep collections using explicit `fit`/`fiet`/`fiunt`/`fient` for method bodies. Focus morphology on types where it provides full coverage (solum, caelum, arca, nucleus).

**Option 2:** Complete the implementation. Implement missing `addita`, `mappa`, `reducta` variants. Either commit fully or not at all — partial coverage undermines the "consistent pattern" strength.

---

#### 4. Two Parallel Async Systems

**Both coexist simultaneously** (morphologia.md:266-302):

| Context              | Async Mechanism                         |
| -------------------- | --------------------------------------- |
| Stdlib (Latin names) | Morphology (`-atura`, `-abit`, `-ens`)  |
| User code            | Return verbs (`fiet`, `fiunt`, `fient`) |

```faber
# STDLIB
lista.filtratura(predicate)   # -atura → async

# USER CODE
functio fetch() fiet User     # fiet → async
```

While justified (user names aren't Latin), this creates **two ways to express the same concept**. The design correctly refuses to deprecate return verbs, but this is a **semantic split**, not a clean boundary.

**Concern:** What if a stdlib method calls a user-defined function? Do async semantics compose? The design mentions Zig state machine composition (morphologia.md:513) but doesn't resolve how `fiet` function calling `filtratura` method handles two async encoding systems.

**Question:** How do morphology-based calls inside `fiet` functions compose? If both generate `Future` structs for Zig, do they use the same state machine pattern? Does the compiler unify them, or are they separate `Future` types that can't interoperate?

---

#### 5. Generator Semantics Are Weak

Present participle (`-ans`/`-ens`) for generators is **linguistically tenuous**:

- `legens` = "reading" (present participle) → streaming/generator?
- Latin uses present participle for ongoing action, but not typically for "producing items one at a time"

The design questions whether this is the right mapping (morphologia.md:620). Latin has richer aspectual distinctions — is present participle truly the best fit for generators?

**Option:** Consider gerund (`legendi` = "for reading") or a different aspectual form if present participle feels weak. However, adding too many verb forms erodes the "consistent pattern" strength. The current choice may be "good enough" but should be explicitly justified in grammar docs.

---

#### 6. Annotation Syntax Must Not Drift

GPT notes correctly warn: "Avoid `@ radix(...)` drift — a prior attempt with parentheses was reverted."

**Concern:** If `@ radix` supports both comma-separated line form and parenthesized form, parsers will diverge over time. Choose **one** syntax and stick to it.

The preferred form (line-based) is cleaner:

```faber
@ radix imperativus, perfectum, futurum_activum
functio filtra<T>(...) fit vacuum { ... }
```

Parenthesized form creates ambiguity: is it `@ radix(imperativus, perfectum)` or `@ radix(imperativus perfectum)`? Line-based avoids this parsing complexity.

---

### Open Questions

#### 1. Stem Derivation vs Registry

Why not derive stems algorithmically instead of maintaining `estRadixListae()` registries? Irregular stems (`invert-` vs `invers-`) are suffix changes, not arbitrary mutations. Latin conjugation is systematic — first conjugation verbs always have `-a` imperatives and `-ata` perfect participles. Why manually encode what grammar defines?

**Alternative approach:** Define conjugation patterns by conjugation class (1st, 2nd, 3rd, 3rd-io, 4th) rather than per-verb stems. Let compiler conjugate verbs by class. This would be truly "mechanically certain" — no manual stem mapping needed.

#### 2. Validation Strategy with `@ radix`

How does `@ radix` interact with morphology? Does compiler validate that `filtra` actually implements `imperativus` variant, or does it just generate variants automatically?

**Scenario:** If I write:

```faber
@ radix(imperativus)
functio filtra<T>(pred: functio(T) fit bivalens) fit vacuum { ... }
```

Does this mean:

- "This function implements imperative variant only, compiler won't generate others"? OR
- "This function defines the implementation, compiler generates all declared variants from this body"?

Current POC doesn't show this validation. Without clear semantics, `@ radix` becomes annotation soup rather than a contract.

#### 3. Zig Async Composition

The design mentions "nested futures" and state machine composition (morphologia.md:513). How do morphology-based calls inside `fiet` functions compose?

```faber
# What happens here?
functio processUser(id: numerus) fiet User {
    fixum user = cede arca.quaeret(id)  # morphology: future participle
    fixum posts = cede solum.legens(user.posts)  # morphology: present participle
    redde user
}
```

Do both `quaeret` and `legens` generate the same `Future` struct pattern? Can `cede` await either one uniformly? Or are there separate state machine types that don't interoperate?

The design claims "morphology and return verbs compile to the same state machine pattern" (morphologia.md:466-484) but doesn't show code. This is a critical gap — Zig has no native async, so composition must be explicit.

#### 4. Irregular Verb Strategy

The design mentions irregular participles like `fero` → `latus` (morphologia.md:622). Will you handle all Latin irregular verbs, or restrict to regular verbs only?

**Concern:** If you manually handle irregulars, you're maintaining a table of exceptions. This defeats the "mechanically certain" principle. If you restrict to regular verbs only, you lose authentic Latin — `fero` is common enough that users will want it.

**Alternative:** Allow users to declare irregular stems explicitly:

```faber
# User defines irregularity mapping
@ radix stems(fero, lat) imperativus, perfectum
functio fere<T>(items: lista<T>) fit vacuum { ... }
```

This shifts complexity to users but keeps compiler simple and mechanical.

#### 5. Partial Methods Rationale

Why include collection morphology with only 7 full pairs? Either drop it (collections use explicit verbs) or complete it (implement missing `addita`, `mappa` variants).

**Counter-argument:** Maybe partial coverage is intentional — morphology only for "high-value" operations (filter, sort, map) where API proliferation is worst. Less common operations (`decapita`, `praepone`) use explicit methods.

**Rebuttal:** But this creates an inconsistent mental model. Why does `filtra` have morphology but `decapita` doesn't? Both are collection operations. Users have to memorize which methods support morphology. This erodes the "consistent pattern" strength.

**Recommendation:** Be explicit about this in docs. If partial coverage is intentional, justify which operations get morphology and which don't. Don't leave it as arbitrary gaps.

---

### Summary of Concerns

1. **Hidden `await` in POC** violates mechanical certainty — must fix before expanding
2. **Manual stem registries** contradict algorithmic parsing — why not derive stems?
3. **Partial collection coverage** creates inconsistency — commit fully or not at all
4. **Two parallel async systems** create semantic split — how do they compose?
5. **Weak generator semantics** — is present participle truly the best mapping?
6. **Annotation drift risk** — choose one syntax and enforce it

The core insight (Latin conjugation encodes semantic axes) is brilliant. The implementation strategy needs to respect the same principles it claims to embody.
