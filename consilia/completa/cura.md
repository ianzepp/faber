---
status: completed
updated: 2026-01-06
note: Core cura mechanism fully implemented. Well-known curator kinds (liber, transactio, mutex, conexio) will be added as standard library features, not language changes.
implemented:
  - cura statement syntax and semantics (all targets)
  - Generic resource management with solve() cleanup
  - Arena allocators (zig only; stripped on GC targets)
  - Page allocators (zig only; stripped on GC targets)
  - Curator type annotations (zig, rs)
  - fit/fiet binding (sync/async)
  - Catch clause support
tests: 12 tests covering arena, page, generic resources
---

# Cura - Resource Management

## Overview

`cura` provides scoped resource management with explicit intent:

1. **Resource cleanup** — file handles, connections, locks get released on scope exit
2. **Allocator scoping** — memory allocators for non-GC targets (Zig, C++, Rust)

For GC targets (TypeScript, Python), allocators are irrelevant. The compiler ignores allocator-related constructs entirely.

---

## Etymology

| Keyword   | Meaning                       | Role                             |
| --------- | ----------------------------- | -------------------------------- |
| `cura`    | "care, concern"               | Block that manages a resource    |
| `curator` | "one who cares for, steward"  | Generic resource manager pactum  |
| `curatum` | "having been cared for" (PPP) | Bound resource / allocator param |
| `fit`     | "it becomes" (sync)           | Sync resource binding            |
| `fiet`    | "it will become" (async)      | Async resource binding           |
| `solve`   | "release, free"               | Cleanup method                   |

---

## Syntax

```ebnf
curaStmt := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
curatorKind := 'arena' | 'page'
```

The curator kind is **optional**. When omitted, the resource expression is required and generic cleanup (`solve()`) is used. When present (`arena` or `page`), it declares an allocator scope.

- `pro` / `fit` — sync resource binding
- `fiet` — async resource binding

**Note:** Additional curator kinds (`liber`, `transactio`, `mutex`, `conexio`) are designed but not yet implemented. Currently only `arena` and `page` are recognized.

---

## Well-Known Curators

| Curator      | Latin meaning    | Use case              | Expression required? | Status      |
| ------------ | ---------------- | --------------------- | -------------------- | ----------- |
| (omitted)    | —                | Generic resource      | yes                  | Implemented |
| `arena`      | sand, open space | Arena allocator       | no                   | Implemented |
| `page`       | pagina           | Page allocator        | no                   | Implemented |
| `liber`      | book, document   | File handle           | yes                  | Planned     |
| `transactio` | transaction      | DB transaction        | yes                  | Planned     |
| `mutex`      | mutual exclusion | Lock guard            | yes                  | Planned     |
| `conexio`    | connection       | Network/DB connection | yes                  | Planned     |

---

## Examples

### Allocators (expression optional)

```fab
cura arena fit mem {
    varia items: textus[] = []
    items.adde("hello")
    items.adde("world")
}
# mem freed, all allocations released

cura page fit mem {
    fixum buffer = allocate(1024)
}
```

Nested allocators:

```fab
cura arena fit outer {
    varia a: textus[] = []
    a.adde("one")

    cura arena fit inner {
        varia b: textus[] = []
        b.adde("two")
    }
    # inner freed

    a.adde("three")
}
# outer freed
```

### Files

```fab
cura liber aperi("data.txt") fit fd {
    fixum data = lege(fd)
    process(data)
}
# fd closed automatically

cura liber aperi("config.json") fit File fd {
    # explicit type annotation
    fixum config = parse(fd)
}
```

### Connections

```fab
cura conexio db.connect(url) fiet conn {
    fixum users = cede conn.query("SELECT * FROM users")
    redde users
}
# conn closed automatically
```

### Transactions

```fab
cura transactio conn.begin() fiet tx {
    tx.execute("INSERT INTO users (name) VALUES (?)", name)
    tx.execute("UPDATE stats SET count = count + 1")
}
# commit on success, rollback on error
```

### Locks

```fab
cura mutex lock.acquire() fit guard {
    shared_counter += 1
}
# lock released automatically
```

### Generic (custom curator)

```fab
cura curator TempFile.create() fit tmp {
    inscribe(tmp.path, data)
}
# tmp.solve() called on exit
```

### Error handling

```fab
cura liber aperi("data.txt") fit fd {
    riskyOperation(fd)
} cape err {
    mone("Operation failed:", err)
}
# fd still closed
```

---

## The Curator Pactum

Resources implement cleanup via the `Curator` pactum:

```fab
pactum Curator {
    functio solve() -> vacuum
}
```

Custom resources:

```fab
genus TempFile implet Curator {
    @ privatum
    textus path

    functio solve() {
        dele(ego.path)
    }
}

cura curator TempFile.create() fit tmp {
    inscribe(tmp.path, data)
}
# temp file deleted via solve()
```

---

## Allocator Annotations

### `curatum` in function signatures

When a function needs an explicit allocator parameter:

```fab
functio buildList(curatum mem, textus prefix) -> textus[] {
    varia items: textus[] = []
    items.adde(prefix)
    redde items
}
```

**When to use:** Rarely. Most code should rely on `cura` blocks. Use `curatum` params when:

- Writing library code that must work with caller-provided allocators
- Interfacing with low-level systems code
- You need fine-grained control over which allocator a function uses

**Target behavior:**

- Zig: `curatum mem` becomes `mem: std.mem.Allocator`
- TS/Py: parameter stripped entirely (GC handles memory)

### `curatum` at callsites

Override the allocator for a single call:

```fab
cura arena fit temp {
    fixum a = buildList("foo")              # uses temp
    fixum b = buildList("bar") curatum other # uses other
}
```

**Symmetry:**

| Context   | Syntax                        | Meaning                              |
| --------- | ----------------------------- | ------------------------------------ |
| Signature | `functio f(curatum mem, ...)` | Function receives allocator as `mem` |
| Callsite  | `f() curatum alloc`           | Use `alloc` for this call            |

---

## Default Behavior

Outside any `cura` block, there's an implicit default allocator. For Zig, this maps to a global arena or page allocator. For GC targets, it's ignored.

```fab
# Top-level code, no explicit cura block
varia items: textus[] = []
items.adde("hello")           # uses default allocator
```

Most Faber code never mentions allocators at all.

---

## Target Mappings

### TypeScript

Sync resource (`cura liber X fit Y`):

```typescript
const fd = open('data.txt');
try {
    const data = read(fd);
    process(data);
} finally {
    fd.solve?.();
}
```

Async resource (`cura conexio X fiet Y`):

```typescript
const conn = await db.connect(url);
try {
    const users = await conn.query('SELECT * FROM users');
    return users;
} finally {
    conn.solve?.();
}
```

Allocator constructs (`curatum` params/callsites, `cura arena/page` blocks) are stripped entirely.

### Python

```python
with open("data.txt") as fd:
    data = fd.read()
    process(data)
```

Allocator constructs stripped.

### Zig

```zig
var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
defer arena.deinit();
const alloc = arena.allocator();

var items = std.ArrayList([]const u8).init(alloc);
defer items.deinit();
try items.append("hello");
```

The `curatorStack` determines which allocator name appears in generated code.

### Rust

```rust
{
    let fd = File::open("data.txt")?;
    let data = read(&fd)?;
    process(data);
}
// fd dropped via RAII
```

---

## Design Decisions

### Why explicit curator kinds?

Following Zig's philosophy: explicit is better than implicit for resource management. Saying `cura liber` instead of just `cura` tells the reader exactly what kind of resource is being managed, and lets the compiler verify the expression matches.

### Why `fit` vs `fiet`?

Latin conjugation of `fio` ("to become"):

- `fit` — present tense, "it becomes" (sync)
- `fiet` — future tense, "it will become" (async)

The verb signals whether resource acquisition is sync or async.

### Why `liber` for files?

The Latin `fasciculus` (file/bundle) is too long. `liber` (book, document) is short and captures the concept of a readable/writable document.

### Why sync-only cleanup?

Async cleanup creates ordering problems:

- What if cleanup awaits something that fails?
- Nested async cleanup order is undefined
- Most resources have sync close anyway

If async cleanup is truly needed, do it explicitly before scope exit.

### Why `solve` for cleanup?

Latin `solve` means "release, free" — fitting for releasing resources. Short and clear.

---

## Implementation Notes

### Zig Codegen

The `curatorStack` tracks the active allocator name:

```typescript
let curatorStack: string[] = ['alloc'];

function getCurator(): string {
    return curatorStack[curatorStack.length - 1] ?? 'alloc';
}
```

Push on:

- `cura arena/page fit X` block entry
- Function entry when a `curatum` param exists
- `curatum` callsite (for that call only, then pop)

### GC Target Codegen

Simply ignore:

- `curatum` parameters (strip from signature and callsites)
- `curatum` callsite annotations (strip entirely)
- `cura arena/page` blocks (strip entirely — no allocator concept)
- Other `cura` blocks: generate try/finally with solve?.() cleanup
