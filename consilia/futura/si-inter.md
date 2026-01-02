# Set Membership: `inter`

## Summary

Add `inter` (among) as an infix operator for set membership checks.

## Motivation

Current syntax for checking if a value matches one of several options is verbose:

```fab
si n.nomen == "verum" aut n.nomen == "falsum" ergo redde BIVALENS
si status == "pending" aut status == "active" aut status == "paused" ergo allow()
```

The variable repeats for each alternative, and `aut` chains add noise. This is especially problematic in guard clauses where brevity matters:

```fab
# Too long for a one-liner guard
si n.nomen == "verum" aut n.nomen == "falsum" ergo redde BIVALENS
```

## Proposed Syntax

```fab
si n.nomen inter ["verum", "falsum"] ergo redde BIVALENS
si status inter ["pending", "active", "paused"] ergo allow()
```

Reads as: "if the name is among 'verum', 'falsum'."

## Semantics

`inter` tests whether the left operand equals any element in the right operand collection:

```fab
x inter [a, b, c]  # equivalent to: x == a aut x == b aut x == c
```

The right operand must be an array literal or array expression. Equality uses `==` (value equality).

## Negation

Use `non` prefix:

```fab
si status non inter ["banned", "deleted"] ergo allow()
```

## Type Constraints

Both operands must have compatible types:

```fab
si age inter [18, 21, 65] { }           # numerus inter lista<numerus> ✓
si name inter ["alice", "bob"] { }      # textus inter lista<textus> ✓
si count inter ["one", "two"] { }       # numerus inter lista<textus> ✗ type error
```

## Comparison with `intra`

| Operator | Meaning        | Use Case        | Example                |
| -------- | -------------- | --------------- | ---------------------- |
| `intra`  | within (range) | bounds checking | `si x intra 0..100`    |
| `inter`  | among (set)    | set membership  | `si x inter [1, 2, 3]` |

Both are Latin prepositions with distinct spatial metaphors:

- _intra_ = inside a bounded region
- _inter_ = among discrete items

## Target Codegen

| Target     | Output                                   |
| ---------- | ---------------------------------------- |
| TypeScript | `["verum", "falsum"].includes(n.nomen)`  |
| Python     | `n.nomen in ["verum", "falsum"]`         |
| Rust       | `["verum", "falsum"].contains(&n.nomen)` |
| Zig        | `for` loop or `std.mem.indexOfScalar`    |
| C++        | `std::ranges::contains` or `std::find`   |

## Alternatives Considered

### Overload `est` for set membership

```fab
si n.nomen est "verum", "falsum" ergo redde BIVALENS
```

Rejected: `est` already means identity/instanceof check. Overloading adds ambiguity.

### Bare comma list without brackets

```fab
si n.nomen inter "verum", "falsum" ergo redde BIVALENS
```

Tempting for brevity, but conflicts with expression parsing. The brackets make the set explicit and allow computed sets:

```fab
fixum allowed = ["pending", "active"]
si status inter allowed ergo process()
```

### Method form

```fab
si ["verum", "falsum"].continet(n.nomen) ergo redde BIVALENS
```

Valid but inverts the natural reading. "Is name among these?" reads better than "do these contain name?" for guard clauses.

## Recommendation

Implement `inter` as an infix operator. The value-centric form matches the guard clause mental model: "if this value is among these options, bail early."
