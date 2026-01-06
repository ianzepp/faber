---
status: archived
updated: 2026-01-06
note: Reference document for hypothetical TS-to-Faber migration tool. Rivus bootstrap was written from scratch in Faber instead of auto-migrating from TS.
---

# TypeScript Language Gaps

This document catalogs TypeScript language features that cannot be directly transformed to Faber. These represent the "error cases" for a TS-to-Faber migration tool.

## Classification

Features are classified by severity:

- **Error**: No Faber equivalent exists. Transformer must reject.
- **Deferred**: Could be added to Faber later (target-gated).
- **Synthesize**: Can be transformed to Faber constructs with some rewriting.
- **Strip**: Can be safely removed (TS-only, no runtime effect).
- **Done**: Implemented in Faber AST (codegen support may vary by target).

---

## Class Features

| TS Feature                       | Severity   | Notes                                                                    |
| -------------------------------- | ---------- | ------------------------------------------------------------------------ |
| `extends` (class inheritance)    | **Done**   | Use `sub` keyword. Target-gated: TS/Py/C++ only; error on Rust/Zig.      |
| `super()` calls                  | Synthesize | Use `supra()`. Target-gated: TS/Py/C++ only.                             |
| `abstract class`                 | **Done**   | Use `abstractus genus`. Target-gated: TS/Py/C++ only.                    |
| `abstract method`                | **Done**   | Use `abstractus functio`. Target-gated: TS/Py/C++ only.                  |
| `protected` modifier             | **Done**   | Use `protectus`. Target-gated: TS/Py/C++ only.                           |
| `readonly` modifier              | Strip      | No runtime effect; Faber uses `fixum` for immutability at binding level. |
| Constructor parameter properties | Synthesize | `constructor(public x: number)` → explicit field + assignment in `creo`. |
| `static` blocks                  | Error      | No Faber equivalent.                                                     |
| `#private` fields (ES private)   | Synthesize | Transform to `privatus` field.                                           |

---

## Type System Features

| TS Feature                                       | Severity   | Notes                                                              |
| ------------------------------------------------ | ---------- | ------------------------------------------------------------------ |
| Generic constraints (`T extends Foo`)            | Synthesize | Use `<T implet Pactum>` or `<T sub Genus>`. All targets supported. |
| Conditional types (`T extends U ? A : B`)        | Error      | No Faber equivalent.                                               |
| Mapped types (`{ [K in keyof T]: ... }`)         | Error      | No Faber equivalent.                                               |
| `infer` keyword                                  | Error      | No Faber equivalent.                                               |
| Template literal types                           | Error      | No Faber equivalent.                                               |
| Intersection types (`A & B`)                     | Error      | Faber has `unio<A,B>` but no intersection.                         |
| `keyof` operator                                 | Error      | No Faber equivalent.                                               |
| `typeof` (type position)                         | Synthesize | Faber has `typus x = typus y` for typeof.                          |
| Index signatures (`[key: string]: T`)            | Error      | Faber `aperit` is not implemented.                                 |
| Utility types (`Partial<T>`, `Pick<T, K>`, etc.) | Error      | Would need stdlib type aliases.                                    |
| `satisfies` keyword                              | Strip      | No runtime effect.                                                 |
| `as const` assertions                            | Strip      | No runtime effect; Faber infers literal types differently.         |

---

## Module Features

| TS Feature                           | Severity   | Notes                                                      |
| ------------------------------------ | ---------- | ---------------------------------------------------------- |
| `namespace`                          | Error      | No Faber equivalent.                                       |
| `module` (legacy)                    | Error      | No Faber equivalent.                                       |
| `declare` statements                 | Strip      | No runtime effect.                                         |
| `declare module` (ambient)           | Strip      | No runtime effect.                                         |
| `export =` (CommonJS)                | Error      | Faber uses ES module syntax only.                          |
| `import =` (CommonJS)                | Error      | Faber uses ES module syntax only.                          |
| Re-exports (`export { x } from 'y'`) | Synthesize | Transform to import + export.                              |
| `export default`                     | Synthesize | Transform to named export.                                 |
| Dynamic `import()`                   | Synthesize | Use `ex "path" importabit modulus`. All targets supported. |

---

## Decorator Features

| TS Feature           | Severity | Notes                |
| -------------------- | -------- | -------------------- |
| Class decorators     | Error    | No Faber equivalent. |
| Method decorators    | Error    | No Faber equivalent. |
| Property decorators  | Error    | No Faber equivalent. |
| Parameter decorators | Error    | No Faber equivalent. |

---

## Function Features

| TS Feature            | Severity   | Notes                                                                |
| --------------------- | ---------- | -------------------------------------------------------------------- |
| Function overloads    | Synthesize | Keep only implementation signature.                                  |
| `this` parameter      | Synthesize | Use `ego` as parameter type and/or return type. Enables fluent APIs. |
| `asserts` return type | Strip      | No runtime effect.                                                   |
| `is` type predicates  | Strip      | No runtime effect (use `est` for runtime checks).                    |

---

## Control Flow

| TS Feature         | Severity   | Notes                                          |
| ------------------ | ---------- | ---------------------------------------------- |
| `for` (C-style)    | Synthesize | Transform to `dum` (while) or `ex 0..n pro i`. |
| `do...while`       | Synthesize | Transform to `dum` with condition at end.      |
| Labeled statements | Error      | No Faber equivalent for `break label`.         |
| `with` statement   | Error      | Deprecated in strict mode anyway.              |

---

## Expression Features

| TS Feature                     | Severity   | Notes                                                                         |
| ------------------------------ | ---------- | ----------------------------------------------------------------------------- |
| `delete` operator              | Error      | Use `tabula.dele(k)` for dynamic keys. No dynamic property ops on `objectum`. |
| `void` operator                | Synthesize | Transform to expression + `nihil`.                                            |
| `in` operator (property check) | Synthesize | Could use object method or `habet`.                                           |
| `instanceof`                   | Synthesize | Maps to `est` type check.                                                     |
| `typeof` (runtime)             | Synthesize | Maps to `est` type check.                                                     |
| Comma operator                 | Synthesize | Split into separate statements.                                               |
| `++`/`--` operators            | Synthesize | Transform to `x = x + 1`.                                                     |
| `<<=`, `>>=` operators         | Error      | Explicitly excluded from Faber.                                               |

---

## Literal Features

| TS Feature               | Severity   | Notes                                                  |
| ------------------------ | ---------- | ------------------------------------------------------ |
| RegExp literals          | Synthesize | Maps to `sed` type (designed, pending implementation). |
| Tagged template literals | Error      | No Faber equivalent.                                   |
| BigInt literals          | Synthesize | Maps to `magnus` type.                                 |

---

## JSX

| TS Feature            | Severity | Notes                          |
| --------------------- | -------- | ------------------------------ |
| JSX elements          | Error    | No Faber equivalent currently. |
| JSX fragments         | Error    | No Faber equivalent currently. |
| JSX spread attributes | Error    | No Faber equivalent currently. |

---

## Summary Statistics

| Severity   | Count | Action                            |
| ---------- | ----- | --------------------------------- |
| Error      | 25    | Reject with clear error message   |
| Synthesize | 20    | Transform to equivalent Faber     |
| Strip      | 7     | Remove (no runtime effect)        |
| **Done**   | 4     | Already implemented in Faber AST  |

---

## Migration Strategy

For a TS codebase migration:

1. **Run transformer** with `--check` mode first to get error report.
2. **Address errors** by refactoring TS code or marking as manual-conversion.
3. **Review synthesized** transforms for semantic correctness.
4. **Stripped** features are safe to ignore.

The goal is not 100% TS coverage, but practical migration for codebases that don't rely heavily on advanced type-level programming.
