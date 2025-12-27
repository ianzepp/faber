---
status: partial
targets: [ts]
note: Parser/AST and TS codegen done; Python/Rust/Zig codegen and `curator` type not yet implemented
updated: 2024-12
---

# Cura - Resource Management

## Implementation Status

| Feature                | Status   | Notes                       |
| ---------------------- | -------- | --------------------------- |
| `cura ... fit` syntax  | Done     | Parser + AST                |
| `cura cede` async      | Done     | Async resource acquisition  |
| `cura ... cape` errors | Done     | Optional catch clause       |
| TypeScript codegen     | Done     | try/finally with solve?.()  |
| `curator` interface    | Not Done | Semantic validation         |
| `solve` method check   | Not Done | Type system integration     |
| Built-in curators      | Not Done | File, socket, etc. (stdlib) |
| Python codegen         | Not Done | with statement              |
| Rust codegen           | Not Done | RAII / Drop                 |
| Zig codegen            | Not Done | defer                       |

## Overview

Automatic resource cleanup via scoped ownership. Any resource that needs cleanup — file handles, database connections, network sockets, locks — can be managed with `cura`.

## Etymology

- `cura` - "care, concern, attention" — care for a resource within scope
- `curator` - "one who cares for" — interface for managed resources
- `solve` - "release, free" — cleanup action

---

## Basic Pattern

```
cura <acquire> fit <binding> {
    <use binding>
}
// automatically cleaned up
```

**Reads as:** "Care for [resource] as [name] { use it }"

---

## Examples

### File Descriptor

```
cura aperi "data.bin" fit fd {
    fixum data = lege fd, 1024
    process(data)
}
// fd closed
```

### Database Connection

```
cura connect(db_url) fit conn {
    fixum users = conn.query("SELECT * FROM users")
    redde users
}
// conn closed
```

### Transaction

```
cura conn.transaction() fit tx {
    tx.insert(record)
    tx.commit()
}
// tx rolled back if not committed
```

### Network Socket

```
cura socket("localhost", 8080) fit sock {
    sock.mitte(request)
    fixum response = sock.accipe()
}
// sock closed
```

### Lock / Mutex

```
cura mutex.lock() fit guard {
    shared_counter += 1
}
// lock released
```

### Nested Resources

```
cura aperi "input.txt" fit input {
    cura aperi "output.txt", { modus: "w" } fit output {
        pro linea in lege input ut lineae {
            inscribe output, process(linea)
        }
    }
    // output closed
}
// input closed
```

---

## Error Handling

Cleanup runs even on error:

```
cura aperi "data.bin" fit fd {
    riskyOperation(fd)
} cape err {
    mone "Operation failed:", err
}
// fd still closed
```

Errors during cleanup itself:

```
cura acquire() fit resource {
    use(resource)
} cape err {
    // err could be from use() OR from cleanup
    mone "Error:", err
}
```

---

## The Curator Interface

Resources implement the `curator` interface:

```
pactum curator {
    functio solve() -> vacuum    // cleanup action
}
```

**Etymology:** `solve` = "release, free"

### Built-in Curators

| Type            | `solve` action              |
| --------------- | --------------------------- |
| File descriptor | Close file                  |
| DB connection   | Disconnect                  |
| Transaction     | Rollback (if not committed) |
| Socket          | Close connection            |
| Lock            | Release lock                |
| Arena/allocator | Free memory                 |

---

## Memory Allocators

An allocator is a resource, similar to a file descriptor:

| Aspect         | File Descriptor                   | Allocator                        |
| -------------- | --------------------------------- | -------------------------------- |
| What it is     | Handle to OS resource             | Handle to memory source          |
| Obtained from  | `open()`, `socket()`              | `arena()`                        |
| Passed to      | `read(fd, ...)`, `write(fd, ...)` | `list.adde(...)` (implicitly)    |
| Lifetime       | Must close when done              | Must deinit when done            |
| Scoped pattern | `cura open(path) fit fd { ... }`  | `cura arena() fit alloc { ... }` |

Both are "capabilities" you obtain, pass around, and release.

### Scoped Allocators

The existing `cura expr fit name { }` syntax works for allocators:

```fab
cura arena() fit alloc {
    varia items: textus[] = []
    items.adde("test")  // uses alloc implicitly
}
// arena freed, all allocations released
```

### The `curator` Type

Functions that need an allocator declare a `curator` parameter:

```fab
functio buildList(curator memoria, textus prefix) -> textus[] {
    varia items: textus[] = []
    items.adde(prefix)
    redde items
}
```

The parameter name (`memoria`) is user-chosen. Collection methods use the active curator automatically.

#### Etymology

- **curator** — Latin "one who takes care of, steward"
- Derives from **cura** ("care")
- Phonetic symmetry with English "allocator" (same syllable count, similar ending)

### Scope Hierarchy

Each scope has an "active curator" name. Collection methods use whatever name is current. Nested scopes shadow outer ones:

```fab
// main scope: curator = 'alloc' (implicit)
varia outer: numerus[] = []
outer.adde(1)  // uses alloc

cura arena() fit temp {
    // inner scope: curator = 'temp'
    varia inner: numerus[] = []
    inner.adde(2)  // uses temp
}

// back to main scope
outer.adde(3)  // uses alloc again
```

Functions establish their own scope:

```fab
functio process(curator mem, numerus[] data) -> numerus[] {
    // function scope: curator = 'mem'
    varia result: numerus[] = []
    result.adde(42)  // uses mem
    redde result
}
```

### Complete Example

```fab
functio processUsers(curator alloc, User[] users) -> User[] {
    varia result: User[] = []

    ex users pro user {
        si user.active {
            result.adde(user)
        }
    }

    redde result
}

// Usage at top level (implicit arena)
fixum users = getUsers()
fixum active = processUsers(alloc, users)

// Usage with explicit arena
cura arena() fit temp {
    fixum scratch = processUsers(temp, users)
    // scratch freed when block exits
}
```

### Target Mapping

| Target | `curator` becomes                     |
| ------ | ------------------------------------- |
| Zig    | `std.mem.Allocator`                   |
| C++    | `std::pmr::memory_resource*`          |
| Rust   | `&dyn Allocator` or trait bound       |
| TS/Py  | parameter omitted (GC handles memory) |

### Custom Curator

```
genus TempFile implet curator {
    privatum path: textus

    functio solve() {
        dele(ego.path)
    }
}

cura TempFile.create() fit tmp {
    inscribe tmp.path, data
}
// temp file deleted
```

---

## Async Resources

`cura` works with async acquisition:

```
cura cede connect(db_url) fit conn {
    fixum result = cede conn.query(sql)
}
// conn closed
```

Cleanup is always sync (no `cede` in `solve`). Async cleanup would complicate the model and create ordering issues.

---

## Target Mappings

### TypeScript

Uses explicit try/finally:

```typescript
const fd = await open('data.bin');
try {
    const data = await read(fd, 1024);
    process(data);
} finally {
    await fd.close();
}
```

Or with `using` (Stage 3 proposal / TypeScript 5.2+):

```typescript
await using fd = await open('data.bin');
const data = await read(fd, 1024);
process(data);
// auto-disposed
```

### Python

Maps to `with` statement:

```python
with open("data.bin") as fd:
    data = fd.read(1024)
    process(data)
# fd closed
```

### Rust

Maps to RAII / Drop:

```rust
{
    let fd = File::open("data.bin")?;
    let data = read(&fd, 1024)?;
    process(data);
}
// fd dropped, closed
```

Or explicit scope with arena:

```rust
let fd = File::open("data.bin")?;
// ... use fd ...
drop(fd);  // explicit cleanup
```

### Zig

Maps to `defer`:

```zig
const fd = try std.fs.cwd().openFile("data.bin", .{});
defer fd.close();

const data = try fd.read(buffer);
process(data);
// fd closed on scope exit
```

---

## Design Decisions

### Why `solve` for cleanup?

`solve` means "release, free" — fitting for releasing resources. Alternatives considered:

- `claude` — already used for file close
- `purga` — "clean" feels like scrubbing, not releasing
- `libera` — "free" works but `solve` is shorter

### Why interface-based?

The `curator` interface allows any type to be managed with `cura`. This is more flexible than hardcoding specific resource types.

### Why sync-only cleanup?

Async cleanup creates ordering problems:

- What if cleanup awaits something that fails?
- Nested async cleanup order is undefined
- Most resources have sync close anyway

If async cleanup is truly needed, do it explicitly before scope exit.

### Error during cleanup?

If `solve()` fails, the error propagates. The resource may be in an undefined state. This matches Rust's Drop behavior (panics on drop failure are usually fatal).
