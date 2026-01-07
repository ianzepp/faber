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

### Async-Generator-First Implementation

For IO types, the **generator form is the base implementation**. Sync and async-batch variants are derived:

```
legens (async generator)     ← Base implementation (streaming)
    ↓ collect stream
leget (async batch)          ← Derived (returns complete data)
    ↓ block until complete
lege (sync batch)            ← Derived (blocks, returns complete data)
```

**Why this inversion matters:**

1. **Streaming is the primitive**: You can always derive batch from stream (collect), but not vice versa
2. **Memory efficiency**: Streaming doesn't require loading everything into memory
3. **Buffer management**: Base impl manages fixed buffers internally; derived forms handle allocation
4. **Zig 0.15 alignment**: The new buffer-based I/O API naturally fits streaming patterns

**Implementation pattern:**

```zig
// legens - base: streams chunks through fixed internal buffer
pub fn legens(path: []const u8) ChunkIterator {
    // Fixed buffer, yields slices, caller processes each chunk
}

// leget - derived: collect stream into allocated memory
pub fn leget(alloc: Allocator, path: []const u8) Future([]u8) {
    // Internally: iterate legens(), append chunks to growing buffer
}

// lege - derived: block and collect
pub fn lege(alloc: Allocator, path: []const u8) []u8 {
    // Internally: poll leget() to completion, return result
}
```

See `consilia/futura/async-generator.md` for the full design.

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

| Category                             | Count | Morphology                       |
| ------------------------------------ | ----- | -------------------------------- |
| Full pairs (imperative + participle) | 7     | `@ radix imperativus, perfectum` |
| Mutate-only                          | 9     | `@ radix imperativus`            |
| Participle-only                      | 20    | `@ radix perfectum`              |
| Read-only                            | 42    | None                             |

Collections benefit modestly from morphology. The system is **designed for IO domains** where async and streaming variants are essential.

---

## Annotation-Based Declaration

Functions declare valid conjugations via annotation:

```faber
@ radix imperativus, perfectum, futurum_activum
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
// From: @ radix imperativus, perfectum, futurum_activum

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

- Add `@ radix ...` annotation parsing
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

## GPT Notes (Final Review)

**Verdict:** The design is worth pursuing and is strongly aligned with the Latin-first thesis, but it only becomes “industrial quality” if morphology is implemented as **typed, opt-in semantic dispatch** (not lexical macro rewriting). Morphology should determine _what value you get back_ (sync/async/generator + mutate/copy), never inject hidden control flow.

### Where This Design Is Truly Strong

#### 1. Latin Grammar as Semantic Compression

The mapping from conjugation/aspect to execution model is not naming-convention theater; it is genuinely expressive:

- **Imperative** reads as “do this now” → sync + mutating.
- **Future indicative** reads as “will do” → async + mutating.
- **Perfect participle** can be interpreted as “the thing-having-been-X’d” → returns a new value.
- **Future participle** reads as “about to produce” → async + returns a new value.

This is exactly the kind of “learn one rule, apply everywhere” pattern that reduces API surface area without adding syntactic noise.

#### 2. IO Domains Are the Real Payoff

The filesystem/network/db/kernel layer naturally has sync/async/streaming variants for almost every operation. Morphology gives you a consistent, low-entropy naming scheme that avoids the Node-style explosion of ad-hoc suffixes.

#### 3. The Stdlib/User Boundary Is Acceptable

A split where:

- stdlib uses morphology (Latin names by design)
- user code uses `fit`/`fiet`/`fiunt`/`fient` (arbitrary names)

…is fine as long as the compiler’s implementation stays non-magical and mechanically certain. The key is that both paths must compile to the same underlying semantics in each backend.

### Non-Negotiables (Must Fix Before Expanding)

#### 1. No Hidden `await`

Morphology may change _return type shape_ (e.g., `Promise<T>`/Future), but the user must still explicitly consume it via `cede`/`figendum`/`variandum` (and generator loops). Hidden `await` breaks correctness and defeats explicit control-flow principles.

#### 2. Receiver Type Binding (Opt-In)

Dispatch must not be based solely on the method’s spelling. The compiler must verify the receiver is a morphology-enabled type and that the stem/conjugation is declared for that type. Otherwise user-defined methods that happen to share spellings will miscompile.

#### 3. Single `@ radix` Syntax (No Parenthesized Variant)

Use the line-based syntax consistently:

```faber
@ radix imperativus, perfectum, futurum_indicativum, futurum_activum
functio filtra<T>(...) fit vacuum { ... }
```

In Rivus, parse the comma-separated identifiers only while tokens remain on the same source line as the `@` annotation.

#### 4. Naming Hygiene: `radix` Means Stem

The term `radix` already means “stem” throughout the proposal and POC. The annotation lists **conjugations/forms**, not stems. Avoid naming any AST field `radices` if it stores conjugation names; use something like `formae` / `conjugationes` / `morphologia`.

#### 5. Document the Semantic Extensions Honestly

Even if the system uses Latin-inspired endings, it is still a compiler feature with deliberate semantics. Be explicit that:

- participles are interpreted substantively (“produce the filtered thing”), not purely descriptively
- generator mapping is a semantic choice
- the POC currently does suffix matching (which is fine) rather than full Latin morphological analysis

### A Concrete Contract (What `@ radix` Should Mean)

- `@ radix ...` marks a declaration site as morphology-enabled and declares which **forms** are valid.
- At call sites, the compiler:
    1. checks whether the receiver type is morphology-enabled
    2. parses the called method name into `(stem, form)`
    3. validates `(stem, form)` against the receiver’s declared set
    4. lowers to the correct backend implementation

If the receiver is not morphology-enabled, the call is treated as an ordinary method call (no “lexical hijacking”).

### On Partial Coverage (Collections)

Partial coverage for collections can be okay if it is intentional and documented as “high-value verbs only”. If it is accidental, it should be completed or removed—silent gaps undermine predictability.

### Next Implementation Step (Rivus)

To keep Rivus self-hosting and mechanically certain, the first end-to-end milestone should be:

1. Parse `@ radix imperativus, ...` (line-based)
2. Store structured annotation data on declarations
3. Build a per-type registry during semantic analysis
4. Enforce receiver-bound validation at call sites
5. Remove hidden-await codegen in TS morphology helpers

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
        @ radix quaer, futurum_indicativum
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

GPT notes correctly warn: "Avoid `@ radix` syntax drift — keep a single argument syntax."

**Concern:** If `@ radix` supports multiple syntaxes, parsers will diverge over time. Choose one syntax and stick to it.

The preferred form (line-based) is cleaner:

```faber
@ radix imperativus, perfectum, futurum_activum
functio filtra<T>(...) fit vacuum { ... }
```

Any alternate variant tends to create ambiguity; the line-based form avoids this parsing complexity.

---

### Open Questions

#### 1. Stem Derivation vs Registry

Why not derive stems algorithmically instead of maintaining `estRadixListae()` registries? Irregular stems (`invert-` vs `invers-`) are suffix changes, not arbitrary mutations. Latin conjugation is systematic — first conjugation verbs always have `-a` imperatives and `-ata` perfect participles. Why manually encode what grammar defines?

**Alternative approach:** Define conjugation patterns by conjugation class (1st, 2nd, 3rd, 3rd-io, 4th) rather than per-verb stems. Let compiler conjugate verbs by class. This would be truly "mechanically certain" — no manual stem mapping needed.

#### 2. Validation Strategy with `@ radix`

How does `@ radix` interact with morphology? Does compiler validate that `filtra` actually implements `imperativus` variant, or does it just generate variants automatically?

**Scenario:** If I write:

```faber
@ radix imperativus
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
@ radix stems fero et lat, imperativus, perfectum
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

---

## Opus 4.5 Notes

**Verdict:** The design is **conceptually sound and linguistically grounded**, but the POC has critical bugs that must be fixed before expansion. More importantly, the design makes deliberate semantic extensions to Latin grammar that should be explicitly acknowledged rather than presented as pure morphological parsing.

### Where All Reviews Converge

Having reviewed GPT Notes, Gemini 3, and GLM 4.7, I note strong consensus on three points:

#### 1. Hidden `await` Is a Showstopper

Every review flags this. The POC in `radices.fab` emits patterns like:

```faber
scriptum("await (async () => {{ §.push(§); }})()", ...)
```

This violates Faber's core principle of explicit control flow. If `filtrabit` silently injects `await`, you cannot use it in a `fit` function without generating invalid target code. **All reviews agree:** morphology determines return type (`Promise<T>` vs `T`), not control flow. The user must explicitly `cede` or assign to `figendum`.

#### 2. Type Binding Is Missing

Gemini 3's `Calculator.adde()` example is the clearest articulation: current dispatch checks `estRadixListae("add")` without verifying the receiver type. A user-defined `genus Calculator` with method `adde(n: numerus)` would incorrectly compile to `calculator.push(n)`. This is a bug, not a design flaw, but it's critical.

#### 3. The Core Insight Is Valid

All reviews affirm that Latin conjugation genuinely encodes the semantic axes modern programming struggles with. This isn't naming-convention theater — the mapping from Future Indicative (`-abit`) to async mutation and Future Participle (`-atura`) to async result is grammatically truthful.

---

### Where I Diverge from Other Reviews

#### On Partial Collection Coverage

**GLM 4.7 says:** "7 of 78 methods? Either commit fully or drop it."

**I disagree.** The design document is honest that collections are secondary — morphology is designed for IO domains. The 7 full pairs (`filtra/filtrata`, `ordina/ordinata`, etc.) are high-traffic operations where API proliferation hurts most. Partial coverage is acceptable **if documented as intentional**.

However, the specific gaps are puzzling:

- `adde` exists but not `addita` ("added" → new list with element)?
- `mappata` exists but not `mappa` ("map in place")?

These feel like incomplete implementation rather than deliberate exclusion. If `mappa` is semantically invalid (mapping always produces a new collection), document that reasoning. If `addita` was simply never implemented, either complete it or remove `adde` from morphology.

#### On Democratizing Morphology

**Gemini 3 says:** Allow users to define morphology for their own types, or stdlib feels "magical."

**I disagree.** The value of morphology is **consistency** — learn once, apply everywhere. If users define their own `@ radix` mappings with arbitrary stems, you lose the "one pattern" benefit and create a Tower of Babel.

The stdlib/user split is a feature, not a bug:

- **Stdlib** uses Latin names by design; morphology applies naturally.
- **User code** uses whatever names developers choose (English, German, domain-specific); `fit`/`fiet`/`fiunt`/`fient` handle this correctly.

Forcing users to write Latin to get morphology benefits would contradict "LLM-first, human-readable." The LLM writes stdlib-style code; humans approve it. User-defined functions with English names don't need morphology — they need clear `fiet` declarations.

#### On Generator Semantics

**GLM 4.7 says:** Present participle is "linguistically tenuous."

**Gemini 3 says:** Gerund might be better, but participle is "the best pragmatic fit."

**My view:** The gerundive (`-andus`/`-endus`) is semantically closer than either. Present participle (`legens` = "reading") describes ongoing action. Gerundive (`legendus` = "to be read" / "needing to be read") describes something **available for an action** — closer to a generator that yields items-to-be-consumed.

But I acknowledge this may be over-engineering. Present participle is recognizable; gerundive is obscure even to Latin scholars. If the choice is between linguistic purity and practical learnability, choose learnability. Document the semantic extension and move on.

---

### Unique Observations

The following points were not raised by other reviews:

#### 1. Vocabulary Purity: `mappare` Is Not Latin

The verb `mappare` does not exist in classical, medieval, or ecclesiastical Latin. This is English "map" with a Latin-looking suffix. For a language whose thesis is "Latin grammar as semantic machinery," this creates dissonance.

| Current   | Problem   | Alternative                                                            |
| --------- | --------- | ---------------------------------------------------------------------- |
| `mappata` | Not Latin | `translata` (from `transferre`, "carry across")                        |
| `mappata` | Not Latin | `conversa` (from `convertere`, "turn together")                        |
| `mappata` | Not Latin | `mutata` (from `mutare`, "change") — conflicts with mutation semantics |

If the thesis is "Latin encodes semantics," the vocabulary should follow. `transferre` → `transfera`/`translata` would mean "carry across" — semantically accurate for transforming collection elements.

Similarly, `filtrare` is medieval Latin (from `filtrum` = felt used for filtering), not classical. This is defensible — medieval Latin is still Latin — but should be acknowledged.

#### 2. The `-sa` Suffix Reveals Heuristic Matching

In `morphologia.fab:119-120`:

```faber
si suffix2 == "ta" aut suffix2 == "sa"
    reddit { radix: nomen.sectio(0, longitudo - 2), flagga: FLAGGA_PERFECTUM }
```

This catches `inversa` (from `invertere`), but `-sa` is not a standard participle suffix. The perfect passive participle of `invertere` is `inversus/-a/-um`. The `-sa` ending is the feminine nominative singular form, not a suffix category.

This reveals that the parser is doing **heuristic suffix matching**, not true morphological parsing. The system works, but calling it "morphology" overstates the mechanism. It's pattern matching on known forms with Latin-inspired naming.

This isn't necessarily bad — pattern matching is simpler and more predictable than full morphological analysis. But the documentation should be honest: "morphology-inspired dispatch" rather than "morphological parsing."

#### 3. Participle Semantics Are Stretched

Latin perfect passive participle (`-ata/-ita/-ta`) means "having been X'd" — it describes the **state after action**, not "produce a new thing."

| Latin Form | Literal Meaning        | Design's Meaning        |
| ---------- | ---------------------- | ----------------------- |
| `filtrata` | "having been filtered" | "returns filtered copy" |
| `ordinata` | "having been arranged" | "returns sorted copy"   |

The design effectively treats participles as **substantivized adjectives** ("the filtered thing" / "the sorted thing"), which then implies "a new collection in that state." This interpretation is defensible but requires a semantic leap.

Document this as a deliberate extension: "We interpret the perfect participle as 'produce the thing-that-has-been-X'd' rather than 'describe something already X'd.'" Don't present it as direct Latin-to-code mapping.

#### 4. Read-Only Method Collision Risk

The design shows `primus` (first element), `ultimus` (last element), `maximus` (maximum), `minimus` (minimum) as read-only methods. These are adjectives ending in `-us`.

But `-us` is also the masculine nominative singular of many verb forms. If someone creates a stdlib method `validus` ("valid check") or `certus` ("certainty check"), the morphology parser might attempt to parse it as a conjugated verb.

The solution is in the design (no `@ radix` = no morphology), but this creates a footgun: a stdlib designer might accidentally name a method with a pattern that looks conjugatable, triggering unexpected morphology dispatch.

**Recommendation:** The parser should only attempt morphology on methods where the receiver type has `@ radix` declarations. This is the type-binding fix Gemini 3 identified, but the collision risk for read-only methods specifically wasn't highlighted.

#### 5. Conjugation Class Could Replace Stem Tables

GLM 4.7 asks: "Why not derive stems algorithmically?" but doesn't provide a concrete alternative.

Latin verbs fall into five conjugation classes with systematic patterns:

| Class  | Infinitive | Imperative | Perfect Participle  |
| ------ | ---------- | ---------- | ------------------- |
| 1st    | `-are`     | `-a`       | `-atus/-ata`        |
| 2nd    | `-ere`     | `-e`       | `-itus/-ita`        |
| 3rd    | `-ere`     | `-e`       | `-tus/-ta` (varies) |
| 3rd-io | `-ere`     | `-e`       | `-tus/-ta`          |
| 4th    | `-ire`     | `-i`       | `-itus/-ita`        |

Instead of maintaining per-verb stem tables (`invert-`, `invers-`, `inver-`), the `@ radix` annotation could declare conjugation class:

```faber
@ radix classis tertia, imperativus, perfectum
functio inverte<T>() fit vacuum { ... }
```

The compiler would then derive:

- Imperative: `inverte` (3rd conjugation → `-e`)
- Perfect participle: `inversus` → `inversa` (feminine, for collections)

This is "mechanically certain" — conjugation classes are systematic. The irregularity isn't in the stem; it's in the participle formation, which follows class rules.

**Caveat:** This requires the compiler to know Latin conjugation patterns. It trades one complexity (stem tables) for another (grammar tables). But grammar tables are static and well-documented; stem tables grow with the API.

---

### Recommendations

#### Critical (Must Fix)

1. **Remove hidden `await` from codegen.** Morphology determines return type; control flow remains explicit via `cede`/`figendum`.

2. **Bind morphology dispatch to receiver type.** Do not dispatch based solely on method name. Verify the receiver has `@ radix` declarations for the parsed stem.

3. **Document semantic extensions.** Acknowledge that:
    - Perfect participle is interpreted as "produce the X'd thing" (substantivized adjective)
    - Present participle for generators is a pragmatic mapping, not strict grammar
    - The parser does suffix matching, not full morphological analysis

#### Important (Should Address)

4. **Audit vocabulary for Latin authenticity.** Replace `mappata` with a real Latin verb (`translata`, `conversa`). Document medieval Latin forms like `filtrare`.

5. **Complete or justify partial coverage.** Either implement `addita`, `mappa`, etc., or document why specific operations don't have full pairs. "We chose not to implement X because Y" is better than silent gaps.

6. **Choose one `@ radix` syntax.** Use the line-based form only (and enforce it consistently in parser, docs, and examples).

#### Optional (Consider)

7. **Explore conjugation-class derivation.** Instead of per-verb stem tables, declare conjugation class and let the compiler derive forms. This is more "mechanically certain" but requires grammar knowledge in the compiler.

8. **Consider gerundive for generators.** If present participle feels weak, gerundive (`-andus`/`-endus`) means "to be X'd" — semantically closer to "yields items to be consumed." But this adds complexity and obscurity.

---

### Summary

The design is **sound in conception**. Latin verb morphology genuinely encodes the semantic axes (sync/async, mutate/copy, eager/streaming) that modern languages express through ad-hoc naming conventions. The IO domain focus is strategically correct — that's where the payoff is highest.

The POC has **critical bugs** (hidden `await`, missing type binding) that must be fixed before expansion. These are implementation issues, not design flaws.

The design makes **deliberate semantic extensions** to Latin grammar (participles as substantivized adjectives, present participle for generators) that should be documented as such. Calling it "morphological parsing" overstates the mechanism; "morphology-inspired dispatch" is more accurate.

The vocabulary includes **non-Latin forms** (`mappare`) that undermine the Latin-first thesis. This is fixable with better verb choices.

The **stdlib/user split** is a feature, not a bug. Morphology works for Latin-named stdlib; `fit`/`fiet`/`fiunt`/`fient` work for arbitrarily-named user code. Don't democratize morphology — the consistency of the stdlib pattern is the value.

**Overall:** Proceed with implementation, but fix the critical bugs first and be honest in documentation about what the system actually does.
