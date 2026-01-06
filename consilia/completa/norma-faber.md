---
status: completed
updated: 2026-01-06
note: Standard library annotation system fully implemented. lista migration complete (47 methods, 144 tests passing).
implemented:
  - All 4 stdlib annotations (@ innatum, @ subsidia, @ radix, @ verte)
  - Build-time registry generation (bun run build:norma)
  - All 5 targets (ts, py, rs, cpp, zig)
  - Indexed placeholders (§0, §1) for arg reordering
  - Complete lista.fab with 47 methods
  - Morphology validation (receiver-bound)
remaining:
  - tabula.fab and copia.fab migrations (lista is complete template)
  - Phase 5: Full morphology validation at all call sites
  - Phase 6: Subsidia fallback for complex implementations
  - Phase 7: Real morphology for IO (solum, caelum, arca)
---

# norma-faber - Standard Library in Faber Source

Define the standard library as Faber source files with codegen annotations, replacing scattered TypeScript registries.

## Problem

Current stdlib implementation:

1. **Scattered registries**: `fons/faber/codegen/lista.ts`, `tabula.ts`, etc. define method translations
2. **Not self-documenting**: The API lives in TypeScript, not Faber
3. **Morphology is ad-hoc**: No formal declaration of valid verb forms
4. **N x M sprawl**: Each method needs entries for each target

## Solution

Define stdlib in `fons/norma/*.fab` files with annotations:

| Annotation | Purpose |
|------------|---------|
| `@ innatum` | Maps genus to native types per target |
| `@ subsidia` | External implementation file for complex cases |
| `@ radix` | Declares morphology stem and valid forms |
| `@ verte` | Codegen transformation per target |

---

## Morphology Split: Fake vs Real

**Collections (lista, tabula, copia)**: Fake morphology
- All method variants defined explicitly
- `@ radix` declares valid forms for validation only
- Compiler does NOT generate variants
- Different implementations for `filtra` vs `filtrata`

**IO (solum, caelum, arca)**: Real morphology
- Base form defined (async generator)
- Compiler generates derived forms
- `leget()` = collect `legens()`
- `lege()` = await collect `legens()`

```
Collections:  filtra ≠ filtrata  (different algorithms)
IO:           lege = await(leget()) = await(collect(legens()))
```

---

## Annotation Syntax

### `@ innatum` - Native Type Mapping

```faber
@ innatum ts "Array", py "list", rs "Vec", cpp "std::vector", zig "Lista"
genus lista<T> { }
```

### `@ subsidia` - External Implementation

```faber
@ subsidia zig "subsidia/zig/lista.zig"
@ subsidia rs "subsidia/rs/lista.rs"
genus lista<T> { }
```

When a method has no `@ verte` for a target, compiler:
1. Includes the subsidia file in preamble/imports
2. Emits direct method call to the implementation

### `@ radix` - Morphology Declaration

```faber
@ radix filtr, imperativus, perfectum
functio filtra<T>(...) fit vacuum
functio filtrata<T>(...) fit lista<T>
```

For collections: validation only (rejects undefined forms like `filtratura`).
For IO: compiler generates derived forms from base.

### `@ verte` - Codegen Transformation

Simple method rename:
```faber
@ verte ts "push"
@ verte py "append"
```

Template with `§` placeholders (positional):
```faber
@ verte ts (ego, elem) -> "[...§, §]"
@ verte py (ego, pred) -> "list(filter(§, §))"
```

Indexed placeholders (`§0`, `§1`, etc.) for reordering or repeated args:
```faber
# Reorder: put elem before ego
@ verte ts (ego, elem) -> "[§1, ...§0]"

# Repeated arg: ego used multiple times
@ verte cpp (ego, elem) -> "(std::find(§0.begin(), §0.end(), §1) != §0.end())"
```

Zig with allocator (always last parameter, use indexed to put first in output):
```faber
@ verte zig (ego) -> "§.longitudo()"                    # no alloc
@ verte zig (ego, elem, alloc) -> "§0.adde(§2, §1)"    # alloc before elem
@ verte zig (ego, pred, alloc) -> "§0.filtrata(§2, §1)"
```

---

## Example: lista.fab

```faber
# fons/norma/lista.fab
# Standard library lista<T> definition

# ============================================================================
# TYPE & EXTERNAL IMPLEMENTATIONS
# ============================================================================

@ innatum ts "Array", py "list", rs "Vec", cpp "std::vector", zig "Lista"
@ subsidia zig "subsidia/zig/lista.zig"
genus lista<T> { }

# ============================================================================
# ADDING ELEMENTS
# ============================================================================

@ radix add, imperativus, perfectum
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte cpp "push_back"
@ verte zig (ego, elem, alloc) -> "§.adde(§, §)"
functio adde<T>(ego lista<T>, elem: T) fit vacuum

@ verte ts (ego, elem) -> "[...§, §]"
@ verte py (ego, elem) -> "[*§, §]"
@ verte rs (ego, elem) -> "{ let mut v = §.clone(); v.push(§); v }"
@ verte cpp (ego, elem) -> "[&]{ auto v = §; v.push_back(§); return v; }()"
@ verte zig (ego, elem, alloc) -> "§.addita(§, §)"
functio addita<T>(ego lista<T>, elem: T) fit lista<T>

# ============================================================================
# REMOVING ELEMENTS
# ============================================================================

@ radix remov, imperativus, perfectum
@ verte ts "pop"
@ verte py "pop"
@ verte rs (ego) -> "§.pop()"
@ verte cpp (ego) -> "[&]{ auto v = §.back(); §.pop_back(); return v; }()"
@ verte zig (ego) -> "§.remove()"
functio remove<T>(ego lista<T>) fit T?

@ verte ts (ego) -> "§.slice(0, -1)"
@ verte py (ego) -> "§[:-1]"
@ verte rs (ego) -> "§[..§.len().saturating_sub(1)].to_vec()"
@ verte zig (ego, alloc) -> "§.remota(§)"
functio remota<T>(ego lista<T>) fit lista<T>

# ============================================================================
# FUNCTIONAL METHODS
# ============================================================================

@ radix filtr, imperativus, perfectum
@ verte ts (ego, pred) -> "(() => { for (let i = §.length - 1; i >= 0; i--) { if (!(§)(§[i])) §.splice(i, 1); } })()"
@ verte py (ego, pred) -> "§[:] = [x for x in § if (§)(x)]"
@ verte cpp (ego, pred) -> "§.erase(std::remove_if(§.begin(), §.end(), [&](auto& x) { return !(§)(x); }), §.end())"
functio filtra<T>(ego lista<T>, praedicatum: functio(T) fit bivalens) fit vacuum

@ verte ts "filter"
@ verte py (ego, pred) -> "list(filter(§, §))"
@ verte rs (ego, pred) -> "§.iter().filter(§).cloned().collect::<Vec<_>>()"
@ verte cpp (ego, pred) -> "(§ | std::views::filter(§) | std::ranges::to<std::vector>())"
@ verte zig (ego, pred, alloc) -> "§.filtrata(§, §)"
functio filtrata<T>(ego lista<T>, praedicatum: functio(T) fit bivalens) fit lista<T>

@ radix ordin, imperativus, perfectum
@ verte ts "sort"
@ verte py "sort"
@ verte rs "sort"
@ verte cpp (ego) -> "std::ranges::sort(§)"
@ verte zig (ego) -> "§.ordina()"
functio ordina<T>(ego lista<T>) fit vacuum

@ verte ts (ego) -> "[...§].sort()"
@ verte py (ego) -> "sorted(§)"
@ verte rs (ego) -> "{ let mut v = §.clone(); v.sort(); v }"
@ verte cpp (ego) -> "[&]{ auto v = §; std::ranges::sort(v); return v; }()"
@ verte zig (ego, alloc) -> "§.ordinata(§)"
functio ordinata<T>(ego lista<T>) fit lista<T>

# ============================================================================
# PROPERTIES (no morphology)
# ============================================================================

@ verte ts (ego) -> "§.length"
@ verte py (ego) -> "len(§)"
@ verte rs (ego) -> "§.len()"
@ verte cpp (ego) -> "§.size()"
@ verte zig (ego) -> "§.longitudo()"
functio longitudo<T>(ego lista<T>) fit numerus

@ verte ts (ego) -> "§.length === 0"
@ verte py (ego) -> "len(§) == 0"
@ verte rs (ego) -> "§.is_empty()"
@ verte cpp (ego) -> "§.empty()"
@ verte zig (ego) -> "§.vacua()"
functio vacua<T>(ego lista<T>) fit bivalens

# ============================================================================
# AGGREGATION
# ============================================================================

@ verte ts (ego) -> "§.reduce((a, b) => a + b, 0)"
@ verte py (ego) -> "sum(§)"
@ verte rs (ego) -> "§.iter().sum::<i64>()"
@ verte cpp (ego) -> "std::accumulate(§.begin(), §.end(), decltype(§[0]){})"
@ verte zig (ego) -> "§.summa()"
functio summa<T>(ego lista<T>) fit T
```

---

## File Structure

```
fons/
  norma/
    lista.fab        # lista<T> definition + codegen
    tabula.fab       # tabula<K,V> definition + codegen
    copia.fab        # copia<T> definition + codegen
    solum.fab        # Filesystem IO (real morphology)
    caelum.fab       # Network IO (real morphology)
    arca.fab         # Database IO (real morphology)

  subsidia/
    zig/
      lista.zig      # Zig implementation (Latin-named wrapper)
      tabula.zig
      copia.zig
    rs/
      lista.rs       # Future: Rust implementation
```

---

## Compiler Flow

```
fons/norma/lista.fab
        │
        ▼
   Parse annotations
        │
        ├── @ innatum  → type registry
        ├── @ subsidia → preamble/import registry
        ├── @ radix    → morphology registry (validation)
        └── @ verte    → codegen registry

At call site:

    items.filtrata(pred)
           │
           ▼
    Validate: "filtrata" is valid form for radix "filtr" ✓
           │
           ▼
    Look up @ verte for target
           │
           ▼
    Emit: items.filter(pred)           // ts
    Emit: list(filter(pred, items))    // py
    Emit: items.filtrata(alloc, pred)  // zig
```

---

## Implementation Plan

Both compilers (Faber and Rivus) implement support in parallel phases.

### Phase 1: Annotation Parsing

| Faber | Rivus |
|-------|-------|
| Parse `@ innatum`, `@ subsidia`, `@ radix`, `@ verte` | Same |
| Store in AST annotation nodes | Same |
| Add to existing annotation infrastructure | Add to existing annotation infrastructure |

**Deliverable:** Both compilers parse all four annotation types without errors.

### Phase 2: Write norma/*.fab Files

Shared work (not compiler-specific):

- Create `fons/norma/lista.fab` with full annotation coverage
- Create `fons/norma/tabula.fab`
- Create `fons/norma/copia.fab`
- Validate syntax parses in both compilers

**Deliverable:** Complete stdlib definitions in Faber source.

### Phase 3: Registry Loading

| Faber | Rivus |
|-------|-------|
| Load `norma/*.fab` at startup | Same |
| Build `@ innatum` → type registry | Same |
| Build `@ subsidia` → preamble registry | Same |
| Build `@ verte` → codegen registry | Same |
| Build `@ radix` → morphology registry | Integrate with existing `morphologia.fab` |

**Deliverable:** Both compilers build internal registries from norma files.

### Phase 4: Codegen Integration

| Faber | Rivus |
|-------|-------|
| Replace `fons/faber/codegen/lista.ts` with registry lookups | Wire codegen to use registries |
| Replace `tabula.ts`, `copia.ts` similarly | Same |
| Delete old TypeScript registry files | N/A (no TS registries) |
| All targets: ts, py, rs, cpp, zig | Start with ts, expand |

**Deliverable:** Method calls use `@ verte` templates from norma files.

### Phase 5: Morphology Validation

| Faber | Rivus |
|-------|-------|
| Add `@ radix` validation at call sites | Already has `parseMethodum()` - integrate |
| Error on undefined forms | Same |
| Port morphology logic from Rivus? | Morphology is source of truth |

**Deliverable:** `items.filtratura(pred)` errors if `futurum_activum` not in `@ radix`.

### Phase 6: Subsidia Fallback

| Faber | Rivus |
|-------|-------|
| When no `@ verte` for target, check `@ subsidia` | Same |
| Include subsidia file in preamble | Same |
| Emit direct method call | Same |

**Deliverable:** Complex methods delegate to target-language implementations.

### Phase 7: Real Morphology (IO) - Future

| Faber | Rivus |
|-------|-------|
| Async generator support all targets | Same |
| Compiler generates derived forms from base | Same |
| `leget()` = collect `legens()` | Same |
| `lege()` = await collect | Same |

**Prerequisite:** `fient` (async generator return) working across all targets.

**Deliverable:** IO stdlib (solum, caelum, arca) with real morphology.

---

## Progress

### Completed

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Annotation Parsing | ✅ Complete | Both Faber and Rivus parse all four annotations |
| Vertical Slice | ✅ Complete | `adde`/`addita` working via norma registry |
| All 5 Targets Wired | ✅ Complete | ts, py, rs, cpp, zig check norma registry |
| Build-time Generation | ✅ Complete | `bun run build:norma` generates from .fab files |
| Indexed Placeholders | ✅ Complete | `§0`/`§1` syntax for reordering and repeated args |
| Full lista Migration | ✅ Complete | 47 methods in norma, 5 reserved keywords in fallback |

### Build-time Generation (commit f297c6d)

Registry is now generated at build time using Faber's parser:

```bash
bun run build:norma
```

Generates:
- `fons/faber/codegen/norma-registry.gen.ts` - TypeScript data (for Faber)
- `fons/rivus/codegen/norma-registry.gen.fab` - Faber code (for Rivus)

The build script (`scripta/build-norma.ts`) uses Faber's actual parser to extract annotations from AST nodes - no hacky regex.

### Current Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ fons/norma/lista.fab          (source of truth - annotations)  │
└─────────────────────────────────────────────────────────────────┘
                              ↓ bun run build:norma
┌─────────────────────────────────────────────────────────────────┐
│ norma-registry.gen.ts (Faber)    norma-registry.gen.fab (Rivus)│
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ fons/faber/codegen/*/expressions/call.ts  (codegen lookup)     │
│   - All 5 targets check norma registry first                   │
│   - Fall back to LISTA_METHODS (hardcoded TypeScript)          │
└─────────────────────────────────────────────────────────────────┘
```

Verified output across all targets:
```
items.adde(4)   → items.push(4)           # ts
                → items.append(4)          # py
                → items.push(4)            # rs
                → items.push_back(4)       # cpp
                → items.adde(alloc, 4)     # zig

items.addita(4) → [...items, 4]           # ts
                → [*items, 4]              # py
                → { let mut v = ... }      # rs
                → [&]{ auto v = ... }()    # cpp
                → items.addita(alloc, 4)   # zig
```

---

## Current State

| Component | Faber | Rivus |
|-----------|-------|-------|
| Annotation parsing | ✅ Yes | ✅ Yes |
| Morphology parsing | No | Yes (`morphologia.fab`) |
| Codegen registries | ✅ norma + TS fallback (5 reserved keywords) | Basic |
| Multi-target codegen | ts, py, rs, cpp, zig | ts |
| subsidia/zig/*.zig | Yes | No |
| norma-registry wired | ✅ All 5 targets | No |
| Build-time generation | ✅ Yes | ✅ Yes (gen.fab) |
| lista methods | ✅ 47 in norma | Uses gen.fab |
| Indexed placeholders | ✅ §0/§1 in templates | ✅ §0/§1 in scriptum |

---

## lista<> Migration ✅ COMPLETE

All 47 lista methods are now in `fons/norma/lista.fab`. Five methods use reserved Faber keywords and fall back to `lista.ts`:

| Category | Methods in norma | Reserved (fallback) |
|----------|------------------|---------------------|
| Adding | adde, addita, praepone, praeposita | - |
| Removing | remove, remota, decapita, decapitata, purga | - |
| Accessing | primus, ultimus, accipe | - |
| Properties | longitudo, vacua | - |
| Searching | continet, indiceDe, inveni, inveniIndicem | - |
| Predicates | omnes, aliquis | - |
| Functional | filtrata, mappata, reducta, explanata, plana, inversa, ordinata, sectio | prima, ultima, omitte |
| Mutating | filtra, inverte | ordina |
| Iteration | perambula, coniunge | - |
| Aggregation | medium, minimus, maximus, minimusPer, maximusPer, numera | summa |
| Lodash-style | congrega, unica, planaOmnia, fragmenta, densa, partire, misce, specimen, specimina | - |
| **Total** | **47 methods** | **5 reserved** |

### Reserved Keywords

These Faber keywords cannot be used as function names:
- `prima` - first N elements (use Faber's `ex lista prima N`)
- `ultima` - last N elements (use Faber's `ex lista ultima N`)
- `omitte` - skip N elements (use Faber's `ex lista omitte N`)
- `ordina` - sort in place (reserved for future use)
- `summa` - sum aggregation (reserved for future use)

### Test Results

All 144 lista tests pass across all 5 targets (ts, py, rs, cpp, zig).

### Remaining Work

- [ ] Deprecate `lista.ts` fallback (keep only 5 reserved methods)
- [ ] Add `tabula.fab` and `copia.fab` definitions
- [ ] Phase 5: Morphology validation at call sites
- [ ] Phase 6: Subsidia fallback for complex implementations

---

## Development Workflow

Step-by-step process for adding methods to the norma system.

### 1. Add annotations to lista.fab

Edit `fons/norma/lista.fab`. Each method needs:

```faber
# Optional: morphology declaration (for validation)
@ radix <stem>, <valid_forms>

# Required: one @ verte per target
@ verte ts "nativeMethod"                           # simple rename
@ verte ts (ego, arg) -> "template with § placeholders"  # template form
@ verte zig (ego, arg, alloc) -> "§.method(§, §)"  # zig needs alloc param

# Required: mark as external (no body)
@ externa
functio methodName()
```

**Template rules:**
- `§` is a positional placeholder (replaced in order)
- `§0`, `§1`, etc. are indexed placeholders (explicit position in values array)
- `ego` = the receiver object (e.g., `items` in `items.adde(4)`) - always index 0
- `alloc` = allocator (Zig only, injected by codegen) - typically last index
- Other params = call arguments in order
- Use indexed placeholders when:
  - Same arg needed multiple times (e.g., `§0.begin()` and `§0.end()`)
  - Args need reordering (e.g., Zig wants `(alloc, elem)` not `(elem, alloc)`)

### 2. Regenerate the registry

```bash
bun run build:norma
```

Output:
```
Found 1 norma file(s): lista.fab
  lista.fab: lista with N method(s)
Generated: .../norma-registry.gen.ts
Generated: .../norma-registry.gen.fab
```

### 3. Test with inline Faber code

Quick test pattern - pipe Faber code directly to compiler:

```bash
# Test single target
echo 'incipit { varia x = [1]; x.adde(2) }' | ./opus/bin/faber compile -t ts

# Test all targets in sequence
for t in ts py rs cpp zig; do
  echo "--- $t ---"
  echo 'incipit { varia x = [1]; x.adde(2) }' | ./opus/bin/faber compile -t $t
done
```

### 4. Verify output

**Expected patterns by target:**

| Target | `adde` (mutating) | `addita` (returns new) |
|--------|-------------------|------------------------|
| ts | `x.push(2)` | `[...x, 2]` |
| py | `x.append(2)` | `[*x, 2]` |
| rs | `x.push(2)` | `{ let mut v = x.clone(); v.push(2); v }` |
| cpp | `x.push_back(2)` | `[&]{ auto v = x; v.push_back(2); return v; }()` |
| zig | `x.adde(alloc, 2)` | `x.addita(alloc, 2)` |

### 5. Check for parse/type errors

```bash
# Validate the .fab file parses
./opus/bin/faber check fons/norma/lista.fab

# Check TypeScript types (no norma-related errors)
npx tsc --noEmit 2>&1 | grep -i norma
```

### 6. Test with a .fab file (optional)

For more complex testing, create a temp file:

```bash
cat > /tmp/test.fab << 'EOF'
incipit {
    varia items = [1, 2, 3]
    items.adde(4)
    fixum extended = items.addita(5)
    scribe items
    scribe extended
}
EOF

./opus/bin/faber compile /tmp/test.fab -t ts
```

### Common Issues

**"0 methods found"**: Check that:
- `genus lista { }` appears before `functio` declarations
- `genus` has `@ innatum` annotation
- `functio` has at least one `@ verte` annotation

**Template not substituting**: Check that:
- Using `§` not `S` for placeholders
- Param count matches placeholder count (for positional `§`)
- `ego` is first param for receiver

**Wrong argument order**: Use indexed placeholders:
- `§0`, `§1`, etc. to explicitly reference values array positions
- Example: `(ego, elem, alloc)` → values = `[ego, elem, alloc]`
- Template `"§0.method(§2, §1)"` → `ego.method(alloc, elem)`

**Repeated arguments in template**: Use indexed placeholders:
- C++ `std::find` needs `ego` four times: `§0.begin(), §0.end(), §1, §0.end()`
- Without indexed: would consume 4 different args (broken)

**Zig allocator undefined**: The test code needs a `cura` block to provide allocator context, or method is being called outside allocation scope.

**Reserved keyword as function name**: These Faber keywords cannot be used:
- `prima`, `ultima`, `omitte`, `ordina`, `summa`
- Keep these methods in the TypeScript fallback registry

---

## Open Questions

1. **Error messages**: When `@ verte` missing for a target, what error? Suggest adding annotation?

2. **Variadic args**: How to handle methods with optional/variadic parameters in templates?

3. **Generic transforms**: Can `@ verte` templates reference type parameters? `"Vec<§T>()"` ?

4. **Inheritance**: If `tabula` extends `lista` behavior, can annotations be inherited?

5. **Testing**: How to test that `@ verte` templates produce valid target code?

---

## Related Documents

- `consilia/morphologia.md` - Verb conjugation as semantic dispatch
- `consilia/futura/stdlib-refactor.md` - Current refactor status
- `consilia/futura/innatum.md` - Native type construction keyword
