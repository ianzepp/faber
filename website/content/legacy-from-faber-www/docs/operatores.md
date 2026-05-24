---
title: Operatores
section: docs
order: 12
---

# Operatores

Operators in Faber Romanus embody the language's central philosophy: making implicit programming concepts explicit through linguistic structure. Where most languages rely solely on symbols, Faber offers Latin keywords alongside familiar operators, letting programmers choose the form that best expresses their intent.

This dual nature is not redundancy. Latin keywords read like natural language, making code flow more clearly when read aloud or processed by language models. Symbolic operators remain available for those who prefer terseness or familiarity. The choice is stylistic, not semantic; both forms compile identically.

---

## Arithmetic

Standard mathematical operators work as expected:

```fab
fixum sum = 10 + 5       # addition
fixum diff = 10 - 5      # subtraction
fixum prod = 10 * 5      # multiplication
fixum quot = 10 / 5      # division
fixum rem = 10 % 3       # modulo (remainder)
```

The `+` operator also handles string concatenation:

```fab
fixum greeting = "salve" + " mundus"
```

Unary negation uses the minus sign as a prefix:

```fab
fixum x = 5
fixum neg = -x           # -5
```

There are no increment (`++`) or decrement (`--`) operators. Use compound assignment instead.

---

## Comparison

Relational operators compare values and return boolean results:

```fab
fixum isLess = 5 < 10           # less than
fixum isGreater = 10 > 5        # greater than
fixum isLessOrEqual = 5 <= 5    # less than or equal
fixum isGreaterOrEqual = 10 >= 10
```

For equality, Faber follows JavaScript's distinction between loose and strict comparison:

```fab
fixum loose = 10 == "10"        # true (type coercion)
fixum strict = 10 === 10        # true (same type and value)
fixum notEqual = 10 != 5
fixum strictNotEqual = 10 !== "10"
```

In practice, prefer strict equality (`===`, `!==`) to avoid subtle bugs from type coercion.

---

## Logical Operators

Logical operators combine boolean expressions. Faber offers both symbolic and Latin forms.

### Latin Forms

The Latin keywords read naturally in conditional expressions:

```fab
fixum both = verum et verum       # and
fixum either = falsum aut verum   # or
fixum negated = non flag          # not
```

Etymology:
- `et` means "and" in Latin, the same word that gives us "et cetera" (and the rest)
- `aut` means "or" in the exclusive sense (one or the other)
- `non` means "not"

### Symbolic Forms

The familiar C-style operators remain available:

```fab
fixum both = verum && verum
fixum either = falsum || verum
fixum negated = !flag
```

**Style guidance:** Prefer Latin forms (`et`, `aut`, `non`) over symbols. The Latin reads more clearly and avoids the visual ambiguity between `!` (logical not) and `!.` (non-null assertion).

Short-circuit evaluation works as expected. In `falsum et expensiveCheck()`, the function never executes because the first operand is already false.

---

## Nullish Operations

### vel (Nullish Coalescing)

The `vel` operator provides a default value when the left operand is `nihil` (null):

```fab
fixum textus? maybeName = nihil
fixum name = maybeName vel "default"   # "default"
```

Unlike `aut` (logical or), `vel` only triggers on null, not on falsy values:

```fab
0 vel 5           # 0 (zero is not null)
"" vel "default"  # "" (empty string is not null)
nihil vel 5       # 5
```

Etymology: `vel` is the Latin inclusive "or" (meaning "or if you prefer"). In Faber, it carries the sense of "or else use this alternative."

**Note:** The JavaScript `??` operator is not available. Use `vel` instead.

### Null Checks

Faber provides unary prefix operators for checking null state:

```fab
fixum textus? maybe = nihil

scribe nihil maybe       # verum (is null)
scribe nonnihil maybe    # falsum (is not null)
```

For checking whether a value is null or empty (strings, arrays, collections):

```fab
scribe nulla maybe       # verum (null or empty)
scribe nonnulla maybe    # falsum (has content)
```

The distinction matters:
- `nihil`/`nonnihil` check strictly for null
- `nulla`/`nonnulla` check for null OR empty (length zero)

---

## Type Checking

### est (Type Check)

The `est` operator tests whether a value is of a particular type or is null:

```fab
fixum numerus? maybeValue = nihil
fixum isNull = maybeValue est nihil    # verum
```

For negative type checking:

```fab
fixum isNotNull = maybeValue non est nihil
```

Etymology: `est` is the Latin verb "is" (third person singular of "esse", to be). The phrase `x est nihil` reads naturally as "x is nothing."

### Boolean Checks

Faber allows testing boolean values with `verum` and `falsum` as prefix operators:

```fab
fixum enabled = verum
fixum isTrue = verum enabled     # strict check that enabled is true
fixum isFalse = falsum disabled  # strict check that disabled is false
```

---

## Ternary Expressions

Conditional expressions select between two values based on a condition.

### Symbolic Style

The familiar question-mark-colon syntax:

```fab
fixum max = a > b ? a : b
fixum grade = score >= 90 ? "A" : score >= 80 ? "B" : "C"
```

### Latin Style

The `sic ... secus` form reads as "thus ... otherwise":

```fab
fixum max = a > b sic a secus b
```

Etymology:
- `sic` means "thus" or "so" (as in "sic semper tyrannis")
- `secus` means "otherwise" or "differently"

The Latin form works well when the condition and branches are short. For complex nested conditions, symbolic style may be clearer. Do not mix the two forms in a single expression.

---

## Ranges

Range operators create sequences of numbers for iteration.

### Exclusive Ranges

The `..` operator creates a range that excludes the end value:

```fab
itera ex 0..10 fixum i {
    scribe i    # 0, 1, 2, ..., 9
}
```

The Latin keyword `ante` ("before") means the same thing:

```fab
itera ex 0 ante 10 fixum i {
    scribe i    # 0, 1, 2, ..., 9
}
```

### Inclusive Ranges

The `usque` operator includes the end value:

```fab
itera ex 0 usque 10 fixum i {
    scribe i    # 0, 1, 2, ..., 10
}
```

Etymology: `usque` means "up to" or "as far as" in Latin, implying inclusion of the destination.

### Stepped Ranges

The `per` modifier controls the step size:

```fab
itera ex 0..10 per 2 fixum i {
    scribe i    # 0, 2, 4, 6, 8
}
```

For descending ranges, use a negative step:

```fab
itera ex 10..0 per -1 fixum i {
    scribe i    # 10, 9, 8, ..., 1
}
```

Etymology: `per` means "by" or "through" in Latin.

### Range Containment

The `intra` operator tests whether a value falls within a range:

```fab
si age intra 18 usque 65 {
    scribe "working age"
}

si value intra 0..100 {
    scribe "valid percentage"
}
```

Etymology: `intra` means "within" in Latin.

---

## Set Membership

The `inter` operator tests whether a value appears in a collection:

```fab
si status inter ["pending", "active", "paused"] {
    scribe "valid status"
}
```

Etymology: `inter` means "among" or "between" in Latin.

---

## Type Conversion

Faber provides postfix operators for converting values between primitive types. These operators use the Latin perfect passive participle form (`-atum`), meaning "having been made into."

### numeratum (to integer)

Parse a string to an integer, or convert other types to integer:

```fab
fixum n = "42" numeratum              # 42
fixum hex = "ff" numeratum<i32, Hex>  # 255 (hexadecimal)
fixum bin = "101" numeratum<u8, Bin>  # 5 (binary)
fixum oct = "777" numeratum<i32, Oct> # 511 (octal)
```

Without a fallback, parse failure causes a panic. Use `vel` to provide a default:

```fab
fixum n = "bad" numeratum vel 0       # 0 (fallback on parse failure)
fixum n = userInput numeratum vel -1  # -1 if input is not a valid number
```

Etymology: from `numerus` (number) + `-atum` (made into).

### fractatum (to float)

Parse a string to a floating-point number:

```fab
fixum f = "3.14" fractatum            # 3.14
fixum f = "bad" fractatum vel 0.0     # 0.0 (fallback)
```

Etymology: from `fractus` (broken, fraction) + `-atum` (made into).

### textatum (to string)

Convert any value to its string representation. This conversion is infallible:

```fab
fixum s = 42 textatum                 # "42"
fixum s = 3.14 textatum               # "3.14"
fixum s = verum textatum              # "verum"
```

Etymology: from `textus` (text) + `-atum` (made into).

### bivalentum (to boolean)

Convert any value to a boolean based on "truthiness." This follows the same semantics as `nonnulla`:

```fab
fixum b = 0 bivalentum                # falsum (zero)
fixum b = 42 bivalentum               # verum (non-zero)
fixum b = "" bivalentum               # falsum (empty string)
fixum b = "hello" bivalentum          # verum (non-empty)
fixum b = [] bivalentum               # falsum (empty collection)
fixum b = nihil bivalentum            # falsum (null)
```

Etymology: from `bivalens` (two-valued) + `-um` (made into).

### Chaining Conversions

Conversion operators chain left-to-right like other postfix operators:

```fab
fixum s = "42" numeratum textatum     # "42" -> 42 -> "42"
```

### Radix Types

For `numeratum`, you can specify both the target integer type and the radix (base) for parsing:

| Radix | Base | Example |
|-------|------|---------|
| `Dec` | 10 | `"42" numeratum<i32, Dec>` |
| `Hex` | 16 | `"ff" numeratum<i32, Hex>` |
| `Oct` | 8 | `"777" numeratum<i32, Oct>` |
| `Bin` | 2 | `"101" numeratum<u8, Bin>` |

If no radix is specified, decimal (base 10) is assumed.

---

## Assignment

Simple assignment uses the equals sign:

```fab
varia x = 10
x = 20
```

Compound assignment operators combine an operation with assignment:

```fab
varia counter = 0
counter += 10    # add and assign
counter -= 3     # subtract and assign
counter *= 2     # multiply and assign
counter /= 2     # divide and assign
```

Bitwise compound assignment is also available:

```fab
varia flags = 0b1010
flags &= mask    # bitwise AND and assign
flags |= flag    # bitwise OR and assign
```

---

## Bitwise Operators

Low-level bit manipulation uses symbolic operators for AND, OR, XOR, and NOT:

```fab
fixum flags = 0b1010
fixum mask = 0b1100

fixum bitwiseAnd = flags & mask      # AND
fixum bitwiseOr = flags | mask       # OR
fixum bitwiseXor = flags ^ mask      # XOR
fixum bitwiseNot = ~flags            # NOT (complement)
```

**Precedence note:** Unlike C, bitwise operators in Faber bind tighter than comparison operators. This means:

```fab
flags & mask == 0    # parses as (flags & mask) == 0
```

This matches programmer intent and avoids a common source of bugs in C-family languages.

### Shift Operators

Bit shift operations use Latin postfix keywords rather than `<<` and `>>` symbols. This avoids ambiguity with nested generics like `lista<lista<T>>` where `>>` would otherwise tokenize as a shift operator.

```fab
fixum shifted = 1 sinistratum 4      # left shift: 1 << 4 = 16
fixum halved = 16 dextratum 2        # right shift: 16 >> 2 = 4
```

Etymology:
- `sinistratum` from *sinister* ("left") + *-atum* ("made into")---shifted leftward
- `dextratum` from *dexter* ("right") + *-atum* ("made into")---shifted rightward

The `-atum` suffix follows the same pattern as type conversion operators (`numeratum`, `textatum`), treating the shift as a transformation applied to the value.

---

## Why Both Forms?

Faber's dual operator syntax reflects a deeper design principle: code is read by both humans and machines, and different readers benefit from different representations.

Latin keywords make semantic relationships explicit. When you write `a et b`, the word "and" appears in the code. When you write `maybeName vel "default"`, the sense of "or alternatively" is visible. This clarity benefits code review, documentation, and AI systems that process code as natural language.

Symbolic operators serve programmers who think in terms of established conventions. A `&&` is immediately recognizable to anyone with C-family experience. For quick expressions or dense logic, symbols can be more scannable.

The language does not privilege one form over the other in parsing or semantics. Choose based on context: Latin for clarity, symbols for familiarity. Many programmers find that Latin keywords work best in conditionals and control flow, while symbols suit arithmetic and bitwise operations.

This flexibility embodies Faber's motto of accessibility over purity. The goal is not to force Latin on reluctant programmers, but to offer it as a tool for those who find it clarifying.

