---
status: completed
updated: 2026-01-06
implemented:
  - Type assertion (expr qua Type)
---

# qua — Type Assertion

**Latin:** "as, in the capacity of, by which"

`qua` asserts a value's type. It says "treat this in the capacity of."

---

## Type Assertion

Treat the result **as** `textus`:

```fab
fixum value = getData() qua textus
```

**Note:** This is type assertion (casting), not type conversion. The value is not transformed, only its type is asserted to the compiler.

---

## In Expressions

Works in any expression context:

```fab
fixum body = response.body qua objectum
fixum num = input qua numerus
```

---

## Difference from `ut`

| Keyword | Purpose       | Example                   |
| ------- | ------------- | ------------------------- |
| `qua`   | Type casting  | `value qua textus`        |
| `ut`    | Name aliasing | `importa name ut alias`   |

`qua` changes the compiler's understanding of a value's type.
`ut` gives a new name to an existing binding.

---

## Target Mappings

| Target     | `expr qua Type`       |
| ---------- | --------------------- |
| TypeScript | `expr as Type`        |
| Python     | `expr` (ignored)      |
| Rust       | `expr as Type`        |
| C++        | `static_cast<Type>`   |
| Zig        | `@as(Type, expr)`     |

---

## Summary

| Pattern         | Meaning                   | Status |
| --------------- | ------------------------- | ------ |
| `expr qua Type` | Assert expression as type | Done   |
