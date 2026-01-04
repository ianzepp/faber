# Morphologia - Verb Conjugation as Semantic Dispatch

Latin verb morphology encodes semantic information. The compiler parses conjugation endings to derive behavior flags (mutation, async, streaming) and dispatches to stem-based code generators.

**Status:** Design proposal (POC implemented in Rivus)

---

## The Problem

Modern programming requires expressing multiple variants of the same operation:

| Variant | JavaScript | Python | Rust |
|---------|------------|--------|------|
| Mutate, sync | `arr.sort()` | `list.sort()` | `vec.sort()` |
| Copy, sync | `[...arr].sort()` | `sorted(list)` | `vec.iter().sorted()` |
| Mutate, async | ??? | ??? | ??? |
| Copy, async | ??? | ??? | ??? |
| Stream/lazy | `arr.values()` | `iter(list)` | `vec.iter()` |
| Async stream | `for await...` | `async for...` | `Stream` trait |

Languages handle this inconsistently:
- **Naming conventions**: `sort`/`sorted`, `reverse`/`reversed`, `readFileSync`/`readFile`
- **Separate APIs**: sync vs async modules, eager vs lazy collections
- **Method suffixes**: `_mut`, `_async`, `_iter`

Each approach requires memorizing arbitrary conventions that vary by operation and domain.

---

## The Solution

Latin verb conjugation already encodes these semantics:

| Form | Latin Name | Ending | Semantics |
|------|------------|--------|-----------|
| Imperative | Imperativus | `-a`, `-e`, `-i` | Command: mutate now |
| Perfect Participle | Participium Perfectum | `-ata`, `-ita`, `-ta` | Completed: return result |
| Future Indicative | Futurum Indicativum | `-abit`, `-ebit`, `-iet` | Will do: async mutate |
| Future Participle | Participium Futurum | `-atura`, `-itura` | About to produce: async result |

This maps directly to a semantic matrix:

|  | Sync | Async |
|--|------|-------|
| **Mutates** | `-a/-e/-i` (imperative) | `-abit/-ebit` (future indicative) |
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

| Type | Latin | Domain | Primary Mode | Morphology Value |
|------|-------|--------|--------------|------------------|
| **mathesis** | "learning" | Math | Sync | None |
| **tempus** | "time" | Date/time | Sync | None |
| **textus** | "woven" | Strings | Sync | Low |
| **lista** | "list" | Arrays | Sync | Medium |
| **tabula** | "table" | Maps | Sync | Medium |
| **copia** | "abundance" | Sets | Sync | Medium |
| **solum** | "ground" | Filesystem | Async + streaming | **High** |
| **caelum** | "sky" | Network | Async + streaming | **High** |
| **arca** | "chest" | Database | Async + streaming | **High** |
| **nucleus** | "kernel" | System/IPC | Async + streaming | **High** |

### IO Operations Matrix

For IO types, every operation naturally has multiple variants:

#### solum (Filesystem)

| Operation | Sync | Async | Generator |
|-----------|------|-------|-----------|
| Read file | `lege` | `leget` | `legens` |
| Write file | `scribe` | `scribet` | `scribens` |
| List directory | `enumera` | `enumerabit` | `enumerans` |
| File exists | `exstat` | `exstabit` | — |
| Delete | `dele` | `delebit` | — |

#### caelum (Network)

| Operation | Sync | Async | Generator |
|-----------|------|-------|-----------|
| HTTP fetch | — | `pete` | `petens` |
| Send data | — | `mitte` | `mittens` |
| WebSocket listen | — | — | `auscultans` |
| Stream response | — | — | `fluit` |

#### arca (Database)

| Operation | Sync | Async | Generator |
|-----------|------|-------|-----------|
| Query | `quaere` | `quaeret` | `quaerens` |
| Insert | — | `inseret` | — |
| Update | — | `mutabit` | — |
| Delete | — | `delebit` | — |
| Transaction | — | `transiget` | — |

#### nucleus (Microkernel)

| Operation | Sync | Async | Generator |
|-----------|------|-------|-----------|
| Spawn process | `genera` | `generabit` | — |
| Wait for child | `expecta` | `expectabit` | — |
| Send signal | `signa` | `signabit` | — |
| Listen signals | — | — | `signans` |
| IPC receive | `accipe` | `accipiet` | `accipiens` |

### The Value Proposition

Without morphology, IO operations require ad-hoc naming:

```javascript
// Node.js style
fs.readFileSync(path)
fs.readFile(path, callback)
fs.promises.readFile(path)
fs.createReadStream(path)
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

| Stem | Imperative (mutates) | Participle (returns new) | Notes |
|------|---------------------|--------------------------|-------|
| **add-** | `adde` | `addita` | Full pair |
| **praepon-** | `praepone` | `praeposita` | Full pair |
| **remov-** | `remove` | `remota` | Full pair |
| **decapit-** | `decapita` | `decapitata` | Full pair |
| **filtr-** | `filtra` | `filtrata` | Full pair |
| **ordin-** | `ordina` | `ordinata` | Full pair |
| **invert-** | `inverte` | `inversa` | Full pair |
| **purg-** | `purga` | — | Mutate only |
| **mapp-** | — | `mappata` | Participle only |
| **reduct-** | — | `reducta` | Participle only |
| **explan-** | — | `explanata` | Participle only |
| **plan-** | — | `plana` | Participle only |
| **sect-** | — | `sectio` | Participle only |
| **prim-** | — | `prima` | Participle only |
| **ultim-** | — | `ultima` | Participle only |
| **unic-** | — | `unica` | Participle only |

#### Read-Only Methods (No Morphology)

| Method | Type | Meaning |
|--------|------|---------|
| `longitudo` | noun | Length |
| `primus` | adjective | First element |
| `ultimus` | adjective | Last element |
| `continet` | verb (3sg) | Contains |
| `vacua` | adjective | Is empty |
| `inveni` | verb | Find element |
| `omnes` | adjective | All match |
| `aliquis` | pronoun | Any matches |
| `summa` | noun | Sum |
| `minimus` | adjective | Minimum |
| `maximus` | adjective | Maximum |

### copia Methods by Radix

| Stem | Imperative (mutates) | Participle (returns new) | Notes |
|------|---------------------|--------------------------|-------|
| **add-** | `adde` | — | Mutate only |
| **del-** | `dele` | — | Mutate only |
| **purg-** | `purga` | — | Mutate only |
| **uni-** | — | `unio` | Returns new |
| **intersect-** | — | `intersectio` | Returns new |
| **differenti-** | — | `differentia` | Returns new |
| **symmetr-** | — | `symmetrica` | Returns new |

#### Read-Only Methods (No Morphology)

| Method | Meaning |
|--------|---------|
| `habet` | Has element |
| `longitudo` | Size |
| `vacua` | Is empty |
| `subcopia` | Is subset |
| `supercopia` | Is superset |

### tabula Methods by Radix

| Stem | Imperative (mutates) | Participle (returns new) | Notes |
|------|---------------------|--------------------------|-------|
| **pon-** | `pone` | — | Mutate only |
| **del-** | `dele` | — | Mutate only |
| **purg-** | `purga` | — | Mutate only |
| **confl-** | `confla` | — | Mutate only (merge) |
| **invert-** | — | `inversa` | Returns new (swap k/v) |
| **selig-** | — | `selige` | Returns new (pick) |
| **omitt-** | — | `omitte` | Returns new (omit) |

#### Read-Only Methods (No Morphology)

| Method | Meaning |
|--------|---------|
| `accipe` | Get value |
| `accipeAut` | Get or default |
| `habet` | Has key |
| `longitudo` | Size |
| `vacua` | Is empty |
| `claves` | Keys iterator |
| `valores` | Values iterator |
| `paria` | Entries iterator |

### Summary

| Category | Count | Morphology |
|----------|-------|------------|
| Full pairs (imperative + participle) | 7 | `@ radix(imperativus, perfectum)` |
| Mutate-only | 9 | `@ radix(imperativus)` |
| Participle-only | 20 | `@ radix(perfectum)` |
| Read-only | 42 | None |

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

| Modifier | Meaning |
|----------|---------|
| `fit` | Sync return |
| `fiet` | Async return (Promise) |
| `fiunt` | Sync generator (yields) |
| `fient` | Async generator |

With morphology in place, this creates redundancy:

```faber
# Redundant - saying the same thing twice
functio filtratura<T>(...) fiet lista<T> { ... }
#        ^^^^^^^^^ async       ^^^^ async
```

**Two systems coexist:**

Morphology only applies to Latin-named stdlib functions. User code uses arbitrary names (English, German, etc.) that don't follow Latin conjugation patterns.

| Context | Async/generator mechanism |
|---------|---------------------------|
| Latin stdlib (lista, solum, arca, etc.) | Morphology (`-atura`, `-ans`, etc.) |
| User-defined functions | `fit`/`fiet`/`fiunt`/`fient` |

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

| Conjugation | Semantics | Stdlib equivalent |
|-------------|-----------|-------------------|
| Imperative (`-a/-e/-i`) | Sync, mutates | `fit` (default) |
| Perfect Participle (`-ata/-ita`) | Sync, returns new | `fit` |
| Future Indicative (`-abit/-ebit`) | Async, mutates | `fiet` |
| Future Participle (`-atura/-itura`) | Async, returns new | `fiet` |
| Present Participle (`-ans/-ens`) | Generator | `fiunt` |
| Future + Generator | Async generator | `fient` |

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

| Form | mutare | async | reddeNovum | allocatio | generator |
|------|:------:|:-----:|:----------:|:---------:|:---------:|
| Imperative | yes | no | no | no | no |
| Perfect Participle | no | no | yes | yes | no |
| Future Indicative | yes | yes | no | no | no |
| Future Participle | no | yes | yes | yes | no |
| Present Participle | no | no | no | no | yes |
| Future + Generator | no | yes | no | no | yes |

---

## Zig Codegen: Morphology and State Machines

Zig removed native async/await in version 0.11. Faber compiles async/generator functions to state machines using the Responsum protocol (see `consilia/futura/zig-async.md`).

### Three Paths to Async

| Source | Example | Zig Support |
|--------|---------|-------------|
| `futura`/`cursor` + `->` | `futura functio fetch() -> T` | **Error** (TS/Python only) |
| Return verb | `functio fetch() fiet T` | Supported |
| Morphology | `solum.leget(path)` | Supported |

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

| Conjugation | Return Verb | Zig Output |
|-------------|-------------|------------|
| Future Indicative (`-abit`/`-ebit`/`-et`) | `fiet` | `Future` with `poll()` |
| Future Participle (`-atura`/`-itura`) | `fiet` | `Future` with `poll()` |
| Present Participle (`-ans`/`-ens`) | `fiunt` | `Iterator` with `next()` |
| Future + Generator | `fient` | `AsyncIterator` with `poll_next()` |

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
