---
status: completed
updated: 2026-01-06
implemented:
  - Mutation block (in object { fields })
  - Mutable parameters (Zig, C++, Rust)
---

# in — Mutable Target

**Latin:** "in, into"

`in` indicates reaching into something to modify it.

---

## Mutation Block

Reach **into** the object to modify its fields:

```fab
in user {
    nomen = "Marcus"
    aetas = 31
}
```

The `in` block provides temporary write access to an object's fields without the `object.field = value` syntax.

**Compiles to:**

```typescript
user.nomen = "Marcus";
user.aetas = 31;
```

---

## Mutable Parameter (Systems Targets)

The parameter is mutably borrowed. The function modifies what was passed **in**:

```fab
functio append(in lista<textus> items, textus value) {
    items.adde value
}
```

**Target mappings:**
- Zig: `items: *std.ArrayList([]const u8)`
- Rust: `items: &mut Vec<String>`
- C++: `std::vector<std::string>&`
- TS/Py: ignored (reference semantics by default)

---

## Summary

| Pattern                   | Meaning                    | Status |
| ------------------------- | -------------------------- | ------ |
| `in object { mutations }` | Mutate object's fields     | Done   |
| `in Type param`           | Mutably borrowed parameter | Done   |
