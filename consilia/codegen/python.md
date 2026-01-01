---
status: implemented
targets: [py]
updated: 2024-12
---

# Python Target Notes

Python is a high-priority compilation target for Faber due to its accessibility and popularity in education. The language's dynamic nature and significant whitespace present unique challenges, but Python 3.10+ features provide good alignment with Faber's syntax.

## Why Python is a High-Priority Target

1. **Educational reach** - Python is often the first programming language taught; Latin syntax could engage classical education
2. **Dynamic typing** - Like TypeScript output, type hints are optional annotations that don't affect runtime
3. **No compilation step** - Immediate feedback matches Faber's accessibility goals
4. **Rich ecosystem** - Libraries for everything; easy to integrate with existing code

## Direct Mappings

| Faber                   | Python                   | Notes                                             |
| ----------------------- | ------------------------ | ------------------------------------------------- |
| `varia`                 | `=`                      | Assignment (no `let`, Python has no const)        |
| `fixum`                 | `=`                      | Assignment (convention: UPPER_CASE for constants) |
| `functio`               | `def`                    | Function declaration                              |
| `futura`                | `async def`              | Async function                                    |
| `cede`                  | `yield`/`await`          | Context-dependent                                 |
| `genus`                 | `class`                  | With auto-merge constructor pattern               |
| `pactum`                | `Protocol`               | From `typing` module (3.8+)                       |
| `implet`                | inheritance              | Class implements protocol via `(Protocol)`        |
| `ego`                   | `self`                   | Explicit in method signatures                     |
| `novum`                 | direct call              | No `new` keyword: `Class()` not `new Class()`     |
| `novum X {}`            | `Class({})`              | Constructor with dict overrides                   |
| `si`/`secus`           | `if`/`elif`/`else`       | Note: `sin` -> `elif`                       |
| `dum`                   | `while`                  | While loop                                        |
| `ex...pro`              | `for...in`               | Iteration                                         |
| `elige`                 | `match`                  | Pattern matching (3.10+)                          |
| `tempta`/`cape`/`demum` | `try`/`except`/`finally` | Full support                                      |
| `iace`                  | `raise Exception`        | Throw exception                                   |
| `mori`                  | `raise SystemExit`       | Fatal/unrecoverable                               |
| `adfirma`               | `assert`                 | Assertion                                         |
| `redde`                 | `return`                 | Return statement                                  |
| `scribe`                | `print()`                | Console output                                    |
| `vide`                  | `print("[DEBUG]", ...)`  | Debug output                                      |
| `mone`                  | `print("[WARN]", ...)`   | Warning output                                    |
| `verum`/`falsum`        | `True`/`False`           | Boolean literals                                  |
| `nihil`                 | `None`                   | Null value                                        |

## Type Mappings

| Faber          | Python         | Notes                           |
| -------------- | -------------- | ------------------------------- |
| `textus`       | `str`          | String type                     |
| `numerus`      | `int`          | Integer                         |
| `bivalens`     | `bool`         | Boolean                         |
| `nihil`        | `None`         | Null type                       |
| `vacuum`       | `None`         | Void return                     |
| `T[]`     | `list[T]`      | List type (3.9+ bracket syntax) |
| `tabula<K,V>`  | `dict[K, V]`   | Dictionary type                 |
| `copia<T>`     | `set[T]`       | Set type                        |
| `promissum<T>` | `Awaitable[T]` | Async result                    |
| `erratum`      | `Exception`    | Error type                      |
| `cursor<T>`    | `Iterator[T]`  | Iterator type                   |
| `T?`           | `T \| None`    | Nullable (3.10+ union syntax)   |
| `T \| U`       | `T \| U`       | Union types (3.10+)             |

## Implementation Status

| Feature              | Status  | Notes                                  |
| -------------------- | ------- | -------------------------------------- |
| Variables            | Done    | `varia`/`fixum` both become assignment |
| Type annotations     | Done    | Optional type hints                    |
| Functions            | Done    | `def` with params and return type      |
| Async functions      | Done    | `async def`                            |
| Generator functions  | Done    | With `yield`                           |
| Async generators     | Done    | `async def` with `yield`               |
| Control flow         | Done    | `if`/`elif`/`else`, `while`, `for`     |
| Pattern matching     | Done    | `match`/`case` (3.10+)                 |
| `genus`              | Done    | `class` with auto-merge `__init__`     |
| `pactum`             | Done    | `Protocol` class                       |
| Computed fields      | Done    | `@property` decorator                  |
| Exception handling   | Done    | `try`/`except`/`finally`               |
| `ego`                | Done    | -> `self`                              |
| Imports              | Done    | `import` and `from...import`           |
| Lista methods        | Done    | Full registry (34 methods)             |
| Intrinsics           | Done    | `_scribe`, `_radix`, etc.              |
| Arrow functions      | Partial | Simple lambdas only                    |
| Template literals    | Done    | f-strings                              |
| Object destructuring | Done    | Multiple assignments                   |

## Design Decisions

### Indentation-Based Blocks

Python uses significant whitespace rather than braces. The codegen tracks indentation depth and emits 4-space indents per level (PEP 8 standard).

```python
# Generated from Faber
def salve(nomen: str) -> str:
    if len(nomen) > 0:
        return f"Salve, {nomen}!"
    else:
        return "Salve!"
```

### Auto-Merge Constructor

Faber's `genus` generates a Python `__init__` that accepts an `overrides` dict, matching the TypeScript pattern:

```python
class persona:
    nomen: str = "anonymous"
    aetas: int = 0

    def __init__(self, overrides: dict = {}):
        if 'nomen' in overrides:
            self.nomen = overrides['nomen']
        if 'aetas' in overrides:
            self.aetas = overrides['aetas']
        self._creo()  # if user defined creo

    def _creo(self):
        # User's creo code here
        pass
```

This allows: `novum persona { nomen: "Marcus" }` -> `persona({"nomen": "Marcus"})`

### Protocol for Interfaces

Faber's `pactum` maps to Python's `typing.Protocol` (structural subtyping):

```python
class Nominatus(Protocol):
    def nomen(self) -> str: ...
```

Classes implementing a protocol don't need explicit inheritance - Python uses duck typing. The `implet` clause generates inheritance syntax for documentation purposes.

### `cede` Context Sensitivity

Like TypeScript, `cede` maps to `yield` inside generator functions and `await` inside async functions. The codegen tracks context via an `inGenerator` flag.

### Lista Methods

Full implementation of 34 lista methods using Python idioms:

| Latin       | Python             | Example Output                      |
| ----------- | ------------------ | ----------------------------------- |
| `adde`      | `.append()`        | `items.append(5)`                   |
| `addita`    | spread             | `[*items, 5]`                       |
| `primus`    | index              | `items[0]`                          |
| `ultimus`   | negative index     | `items[-1]`                         |
| `longitudo` | `len()`            | `len(items)`                        |
| `continet`  | `in` operator      | `(5 in items)`                      |
| `filtrata`  | comprehension      | `[x for x in items if pred(x)]`     |
| `mappata`   | comprehension      | `[f(x) for x in items]`             |
| `reducta`   | `functools.reduce` | `functools.reduce(fn, items, init)` |
| `inversa`   | slice              | `items[::-1]`                       |
| `ordinata`  | `sorted()`         | `sorted(items)`                     |
| `prima`     | slice              | `items[:n]`                         |
| `ultima`    | slice              | `items[-n:]`                        |
| `omitte`    | slice              | `items[n:]`                         |
| `coniunge`  | `.join()`          | `", ".join(items)`                  |
| `omnes`     | `all()`            | `all(pred(x) for x in items)`       |
| `aliquis`   | `any()`            | `any(pred(x) for x in items)`       |

### Range Expressions

Faber's ranges are exclusive, matching Python's `range()` directly:

```
// Faber: 0..10 is exclusive [0, 10)
ex 0..10 pro i { scribe i }

# Python: direct mapping
for i in range(0, 10):
    print(i)
```

With step:

```
// Faber
ex 0..10 per 2 pro i { scribe i }

# Python
for i in range(0, 10, 2):
    print(i)
```

### Ternary Expressions

Python's ternary has reversed order from C-style:

```
// Faber: condition ? consequent : alternate
verum ? 1 : 0

# Python: consequent if condition else alternate
1 if True else 0
```

Both `? :` and Latin `sic secus` are supported:

```
x > 5 sic "big" secus "small"
# -> "big" if (x > 5) else "small"
```

## Gaps and Limitations

### Arrow Functions with Block Bodies

Python lambdas cannot contain statements, only expressions. Block-bodied arrow functions should be lifted to named functions:

```
// Faber - block body
pro user {
    si user.aetas < 18 { redde falsum }
    redde user.activus
}

# Python limitation: can't do this in a lambda
# Would need to be a named function
```

Currently, the codegen attempts to treat all arrow functions as expression lambdas, which may produce incorrect output for complex cases.

### Static Fields (`generis`)

Python has class attributes but no true static field syntax. Currently marked partial:

```python
# Would need:
class MyClass:
    count: ClassVar[int] = 0  # typing.ClassVar
```

Not yet implemented in codegen.

### Private Field Convention

Python uses underscore convention rather than access modifiers:

- Default (public) field `nomen` -> `nomen`
- `privatus` field `nomen` -> `_nomen`

The codegen adds underscore prefix for private fields, but Python doesn't enforce this at runtime.

### No True Constants

Python has no `const`. Both `varia` and `fixum` become simple assignment. Convention is to use UPPER_CASE for constants, but this isn't enforced.

### Events (`emitte`/`ausculta`)

Event emission and subscription are not implemented for Python. Would require an event emitter library or asyncio patterns.

### Multiple Named Imports

Parser limitation: comma-separated imports may not parse correctly:

```
ex norma importa scribe, lege  # May fail
```

### Empty `scribe`

`scribe` with no arguments may not parse correctly.

## Python Version Requirements

**Minimum: Python 3.10+**

Required features:

- `match`/`case` statements (3.10)
- `X | Y` union type syntax (3.10)
- `list[T]` generic syntax (3.9, but 3.10 for consistency)

Optional but used:

- `typing.Protocol` (3.8+)
- f-strings (3.6+)
- Type hints (3.5+, but modern syntax 3.10+)

## Intrinsic Mappings

| Faber         | Python                        | Import Needed |
| ------------- | ----------------------------- | ------------- |
| `_scribe`     | `print()`                     | None          |
| `_vide`       | `print(..., file=sys.stderr)` | `sys`         |
| `_mone`       | `warnings.warn()`             | `warnings`    |
| `_lege`       | `input()`                     | None          |
| `_fortuitus`  | `random.random()`             | `random`      |
| `_pavimentum` | `math.floor()`                | `math`        |
| `_tectum`     | `math.ceil()`                 | `math`        |
| `_radix`      | `math.sqrt()`                 | `math`        |
| `_potentia`   | `math.pow()`                  | `math`        |

Note: The codegen does not automatically add required imports. Generated code may need manual import additions for intrinsics.

## Comparison with Other Targets

| Aspect      | TypeScript  | Python         | Zig           |
| ----------- | ----------- | -------------- | ------------- |
| Type safety | Static      | Optional hints | Static        |
| Memory      | GC          | GC             | Manual/Arena  |
| Exceptions  | try/catch   | try/except     | Error unions  |
| Async       | Promise     | asyncio        | Frame-based   |
| Generators  | Native      | Native         | Manual struct |
| Interfaces  | interface   | Protocol       | Duck typing   |
| No `new`    | new Class() | Class()        | Class.init()  |
| Constants   | const       | Convention     | const         |

## Future Considerations

1. **Import generation** - Auto-add required imports for intrinsics (`math`, `sys`, etc.)
2. **Block lambdas** - Lift to named inner functions when block body detected
3. **Static fields** - Implement `ClassVar` for `generis` fields
4. **Events** - Consider asyncio.Queue or simple event emitter pattern
5. **Dataclasses** - Could use `@dataclass` decorator for simpler genus generation
6. **Type checking** - Integration with mypy for generated code validation
