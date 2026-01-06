---
status: completed
updated: 2026-01-06
note: Core de preposition fully implemented. Borrowed return types remain unimplemented.
implemented:
  - Key iteration (de...pro)
  - Borrowed parameters (Zig, C++, Rust)
  - novum...de construction from source
not_implemented:
  - Borrowed return types (-> de Type)
---

# de — Read-Only Reference

**Latin:** "from, concerning"

`de` indicates a read-only relationship. You're reading from or concerning something, not modifying it.

---

## Key Iteration

Iterate over keys **concerning** an object:

```fab
de tabula pro clavis {
    scribe clavis, tabula[clavis]
}
```

Compiles to `for...in`. Uses `de` ("concerning") for read-only key access.

Contrast with `ex`, which iterates values.

---

## Borrowed Parameter (Systems Targets)

The parameter is borrowed **from** the caller. Read-only access.

```fab
functio process(de textus input) {
    # input is borrowed, read-only
    scribe input
}
```

**Target mappings:**
- Zig: `input: []const u8`
- Rust: `input: &str`
- C++: `const std::string&`
- TS/Py: ignored (GC handles memory)

---

## novum...de Construction

Create new instance, taking initial values **from** an expression:

```fab
fixum props = { nomen: "Marcus", aetas: 30 }
fixum person = novum Persona de props
```

The `de` preposition indicates the source object from which fields are copied.

---

## Borrowed Return Type (Not Implemented)

**Status:** Design only. Systems targets only.

The return type is borrowed - caller receives a reference into existing data, not a new allocation:

```fab
functio getName(de User user) -> de textus {
    redde user.name
}
```

**Target mappings:**
- Zig: `fn getName(user: *const User) []const u8`
- Rust: `fn get_name(user: &User) -> &str`
- C++: `const std::string& getName(const User& user)`
- TS/Py: ignored (returns copy/reference as normal)

When multiple `de` parameters exist, use `ex` to specify the source. See `ex.md` for details.

---

## Summary

| Pattern                 | Meaning                      | Status   |
| ----------------------- | ---------------------------- | -------- |
| `de object pro key { }` | Iterate keys from object     | Done     |
| `de Type param`         | Borrowed/read-only parameter | Done     |
| `-> de Type`            | Borrowed return type         | Not done |
| `novum Type de expr`    | Initialize from expression   | Done     |
