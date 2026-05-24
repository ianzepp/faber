---
title: Regimen
section: docs
order: 14
---

# Regimen

Control flow determines how a program executes: which statements run, in what order, and under what conditions. Faber uses Latin keywords that map intuitively to their English equivalents while reading naturally as Latin prose.

The Latin word *regimen* means "guidance" or "direction"—literally, the act of steering. In Faber, control flow keywords *steer* execution through conditionals, loops, and branching constructs.

---

## Conditionals

Conditional statements execute code based on whether a condition evaluates to true or false.

### si (If)

The keyword `si` means "if" in Latin. It introduces a condition that, when true, executes the following block.

```fab
si x > 0 {
    scribe "positive"
}
```

The condition must be a boolean expression. If it evaluates to `verum` (true), the block executes. Otherwise, execution continues past the block.

Multiple statements can appear within the block:

```fab
si user.authenticated {
    scribe "Welcome back"
    loadUserPreferences(user)
    updateLastLogin(user)
}
```

### sin (Else If)

The keyword `sin` is a classical Latin contraction of `si non`—"but if" or "if however." It chains additional conditions after an initial `si`.

```fab
fixum hour = 14

si hour < 6 {
    scribe "Late night"
}
sin hour < 12 {
    scribe "Morning"
}
sin hour < 18 {
    scribe "Afternoon"
}
sin hour < 22 {
    scribe "Evening"
}
```

Each `sin` condition is tested only if all preceding conditions were false. The chain stops at the first true condition.

### secus (Else)

The keyword `secus` means "otherwise" in Latin. It provides a default branch when no preceding condition matched.

```fab
si hour < 12 {
    scribe "Morning"
}
sin hour < 18 {
    scribe "Afternoon"
}
secus {
    scribe "Evening or night"
}
```

A `secus` block always executes if reached—it has no condition to test. It must appear last in a conditional chain.

### Short Forms

Faber provides concise syntax for simple conditionals.

#### ergo (Therefore)

The keyword `ergo` means "therefore" or "thus" in Latin—expressing logical consequence. Use it for one-liner conditionals where a block would be verbose.

```fab
si x > 5 ergo scribe "x is big"
```

This is equivalent to:

```fab
si x > 5 {
    scribe "x is big"
}
```

The `ergo` form works with `secus` for one-liner if-else:

```fab
si age >= 18 ergo scribe "Adult" secus scribe "Minor"
```

#### reddit (Early Return)

The keyword `reddit` means "it returns" in Latin (third person singular of *reddere*, "to give back"). It combines `ergo` with `redde` for early return patterns.

```fab
functio classify(numerus x) -> textus {
    si x < 0 reddit "negative"
    si x == 0 reddit "zero"
    redde "positive"
}
```

This is equivalent to:

```fab
functio classify(numerus x) -> textus {
    si x < 0 {
        redde "negative"
    }
    si x == 0 {
        redde "zero"
    }
    redde "positive"
}
```

The `reddit` form excels at guard clauses—conditions that validate input and exit early:

```fab
functio divide(numerus a, numerus b) -> numerus? {
    si b == 0 reddit nihil
    redde a / b
}
```

It works throughout a conditional chain:

```fab
functio grade(numerus score) -> textus {
    si score >= 90 reddit "A"
    sin score >= 80 reddit "B"
    sin score >= 70 reddit "C"
    sin score >= 60 reddit "D"
    secus reddit "F"
}
```

---

## Loops

Loops repeat a block of code, either a fixed number of times or until a condition changes.

### dum (While)

The keyword `dum` means "while" or "as long as" in Latin. It executes a block repeatedly while a condition remains true.

```fab
varia numerus counter = 0

dum counter < 5 {
    scribe counter
    counter = counter + 1
}
```

The condition is checked before each iteration. If it starts false, the block never executes.

While loops work well for countdown patterns:

```fab
varia numerus countdown = 3

dum countdown > 0 {
    scribe "Countdown:", countdown
    countdown = countdown - 1
}
scribe "Done!"
```

The one-liner form uses `ergo`:

```fab
dum i > 0 ergo i = i - 1
```

### itera ex (For Each Values)

The `itera ex` construct iterates over values in a collection. The syntax follows Faber's verb-first pattern: "iterate from items, binding each item."

```fab
fixum numbers = [1, 2, 3, 4, 5]

itera ex numbers fixum n {
    scribe n
}
```

The verb `itera` is the imperative of *iterare* ("to repeat, traverse"). The preposition `ex` means "from" or "out of"—the source from which values are drawn. The binding uses `fixum` (immutable) or `varia` (mutable).

This verb-first syntax aligns with other Faber statements like `scribe`, `iace`, and `importa`. Where JavaScript writes `for (const item of items)`, Faber writes `itera ex items fixum item`.

The syntax works with any iterable:

```fab
fixum names = ["Marcus", "Julia", "Claudia"]

itera ex names fixum name {
    scribe name
}
```

Processing each item:

```fab
fixum values = [10, 20, 30]

itera ex values fixum v {
    fixum doubled = v * 2
    scribe doubled
}
```

The one-liner form:

```fab
itera ex numbers fixum n ergo scribe n
```

#### Range Expressions

Ranges generate sequences of numbers. Faber provides three range operators:

| Operator | Latin Meaning | End Behavior | Example |
|----------|---------------|--------------|---------|
| `..` | (shorthand) | exclusive | `0..5` yields 0, 1, 2, 3, 4 |
| `ante` | "before" | exclusive | `0 ante 5` yields 0, 1, 2, 3, 4 |
| `usque` | "up to" | inclusive | `0 usque 5` yields 0, 1, 2, 3, 4, 5 |

The `..` operator is convenient shorthand matching common programming conventions (Python's `range()`, Rust's `..`):

```fab
itera ex 0..5 fixum i {
    scribe i
}
```

The `ante` keyword makes exclusivity explicit—the range stops *before* the end value:

```fab
itera ex 0 ante 5 fixum i {
    scribe i
}
```

The `usque` keyword includes the end value—the range goes *up to and including* the end:

```fab
itera ex 0 usque 5 fixum i {
    scribe i
}
```

For step increments, use `per`:

```fab
itera ex 0..10 per 2 fixum i {
    scribe i  # 0, 2, 4, 6, 8
}

itera ex 0 usque 10 per 2 fixum i {
    scribe i  # 0, 2, 4, 6, 8, 10
}
```

### itera de (For Each Keys)

The `itera de` construct iterates over keys (property names or indices) rather than values. The syntax reads: "iterate concerning the object, binding each key."

```fab
fixum persona = { nomen: "Marcus", aetas: 30, urbs: "Roma" }

itera de persona fixum clavis {
    scribe clavis
}
```

The preposition `de` means "about" or "concerning"—indicating a read-only relationship with the object. You're iterating *concerning* the object's structure, not extracting its contents.

To access values, use the key with bracket notation:

```fab
itera de persona fixum clavis {
    scribe scriptum("Key: §, Value: §", clavis, persona[clavis])
}
```

For arrays, `de` iterates indices:

```fab
fixum numeri = [10, 20, 30]

itera de numeri fixum index {
    scribe scriptum("Index §: §", index, numeri[index])
}
```

The distinction between `ex` and `de` mirrors their Latin meanings:
- `itera ex items fixum item` — draw *values* **from** the collection
- `itera de object fixum key` — inspect *keys* **concerning** the object

### Async Iteration

For asynchronous streams, use `cede` (await) prefix with `itera`:

```fab
cede itera ex stream fixum chunk {
    scribe chunk
}
```

This compiles to `for await...of` in JavaScript/TypeScript.

### Loop Control

Two keywords control loop flow:

#### rumpe (Break)

The keyword `rumpe` is the imperative of *rumpere* ("to break"). It exits the innermost loop immediately.

```fab
varia i = 0

dum i < 10 {
    si i == 5 {
        rumpe
    }
    scribe i
    i = i + 1
}
```

Output: 0, 1, 2, 3, 4 (loop breaks when `i` reaches 5).

In nested loops, `rumpe` exits only the inner loop:

```fab
itera ex 0..3 fixum outer {
    itera ex 0..10 fixum inner {
        si inner == 2 {
            rumpe  # exits inner loop only
        }
        scribe scriptum("outer=§, inner=§", outer, inner)
    }
}
```

#### perge (Continue)

The keyword `perge` is the imperative of *pergere* ("to continue" or "proceed"). It skips to the next iteration.

```fab
itera ex 0..10 fixum i {
    si i % 2 == 0 {
        perge  # skip even numbers
    }
    scribe i
}
```

Output: 1, 3, 5, 7, 9 (even numbers skipped).

Like `rumpe`, `perge` affects only the innermost loop.

---

## Branching

Branching statements select one path among several based on a value.

### elige (Switch)

The keyword `elige` is the imperative of *eligere* ("to choose"). It matches a value against cases.

```fab
fixum status = "active"

elige status {
    casu "pending" {
        scribe "Waiting..."
    }
    casu "active" {
        scribe "Running"
    }
    casu "done" {
        scribe "Completed"
    }
}
```

The keyword `casu` is the ablative of *casus* ("case" or "instance")—literally "in the case of."

Unlike C-family switch statements, Faber's `elige` does not fall through. Each `casu` is self-contained.

For a default branch, use `ceterum` ("otherwise" or "for the rest"):

```fab
elige code {
    casu 200 {
        scribe "OK"
    }
    casu 404 {
        scribe "Not Found"
    }
    ceterum {
        scribe "Unknown status"
    }
}
```

One-liner cases use `ergo`:

```fab
elige status {
    casu "pending" ergo scribe "waiting"
    casu "active" ergo scribe "running"
    ceterum iace "Unknown status"
}
```

Early returns use `reddit`:

```fab
functio statusMessage(numerus code) -> textus {
    elige code {
        casu 200 reddit "OK"
        casu 404 reddit "Not Found"
        casu 500 reddit "Server Error"
        ceterum reddit "Unknown"
    }
}
```

### discerne (Pattern Match)

The keyword `discerne` is the imperative of *discernere* ("to distinguish" or "discriminate"). It performs exhaustive pattern matching on tagged union types (*discretio*).

First, define a `discretio` (tagged union):

```fab
discretio Event {
    Click { numerus x, numerus y },
    Keypress { textus key },
    Quit
}
```

Then match against it:

```fab
functio handle_event(Event e) -> nihil {
    discerne e {
        casu Click pro x, y {
            scribe scriptum("Clicked at §, §", x, y)
        }
        casu Keypress pro key {
            scribe scriptum("Key: §", key)
        }
        casu Quit {
            scribe "Goodbye"
        }
    }
}
```

The `pro` keyword destructures variant fields into local bindings. For the `Quit` variant (which has no fields), the block has no `pro` clause.

For simple variants without data, the body can use `ergo`:

```fab
discretio Status { Active, Inactive, Pending }

functio describe(Status s) -> textus {
    discerne s {
        casu Active ergo redde "active"
        casu Inactive ergo redde "inactive"
        casu Pending ergo redde "pending"
    }
}
```

To bind the entire variant (not just its fields), use `ut`:

```fab
discerne left, right {
    casu Primitivum ut l, Primitivum ut r {
        redde l.nomen == r.nomen
    }
    casu _, _ {
        redde falsum
    }
}
```

The underscore `_` is the wildcard pattern—it matches any variant without binding.

The difference between `elige` and `discerne`:
- `elige` matches against primitive values (numbers, strings)
- `discerne` matches against `discretio` variants with destructuring

---

## Guards and Assertions

Guards and assertions enforce invariants—conditions that must hold for the program to proceed correctly.

### custodi (Guard Block)

The keyword `custodi` is the imperative of *custodire* ("to guard" or "watch over"). It groups early-exit conditions at the start of a function.

```fab
functio divide(numerus a, numerus b) -> numerus {
    custodi {
        si b == 0 {
            redde 0
        }
    }

    redde a / b
}
```

The `custodi` block creates a visual separation between validation and main logic. All precondition checks cluster together, making the function's requirements explicit.

Multiple guards in one block:

```fab
functio processValue(numerus x) -> numerus {
    custodi {
        si x < 0 {
            redde -1
        }
        si x > 100 {
            redde -1
        }
    }

    # Main logic, clearly separated from guards
    redde x * 2
}
```

Guards can throw instead of returning:

```fab
functio sqrt(numerus n) -> numerus {
    custodi {
        si n < 0 {
            iace "Cannot compute square root of negative number"
        }
    }

    redde computeSqrt(n)
}
```

The `reddit` shorthand works within `custodi`:

```fab
functio clamp(numerus value, numerus min, numerus max) -> numerus {
    custodi {
        si value < min reddit min
        si value > max reddit max
    }

    redde value
}
```

Input validation patterns:

```fab
functio createUser(textus name, textus email, numerus age) -> textus {
    custodi {
        si name == nihil aut name == "" {
            redde "Error: name required"
        }
        si email == nihil aut email == "" {
            redde "Error: email required"
        }
        si age < 13 {
            redde "Error: must be 13 or older"
        }
        si age > 120 {
            redde "Error: invalid age"
        }
    }

    redde scriptum("User created: §", name)
}
```

### adfirma (Assert)

The keyword `adfirma` is the imperative of *adfirmare* ("to affirm" or "assert"). It checks runtime invariants.

```fab
adfirma x > 0
```

If the condition is false, execution halts with an assertion error.

Add a message for clarity:

```fab
adfirma x > 0, "x must be positive"
adfirma name != "", "name must not be empty"
```

Assertions differ from guards:
- Guards handle expected edge cases gracefully (return early, throw recoverable errors)
- Assertions catch programming errors (bugs) that should never occur in correct code

Use `adfirma` for conditions that indicate bugs if violated:

```fab
functio processArray(lista<numerus> items) {
    adfirma items != nihil, "items must not be null"
    adfirma items.longitudo > 0, "items must not be empty"

    # ... process items
}
```

---

## Summary

| Keyword | Latin Meaning | Purpose |
|---------|--------------|---------|
| `si` | "if" | Conditional branch |
| `sin` | "but if" | Else-if branch |
| `secus` | "otherwise" | Else branch |
| `ergo` | "therefore" | One-liner consequent |
| `reddit` | "it returns" | One-liner early return |
| `dum` | "while" | While loop |
| `ex` | "from, out of" | Iteration source |
| `de` | "concerning" | Key iteration source |
| `pro` | "for" | Iteration binding |
| `fit` | "it becomes" | Sync binding |
| `fiet` | "it will become" | Async binding |
| `rumpe` | "break!" | Exit loop |
| `perge` | "continue!" | Skip to next iteration |
| `elige` | "choose!" | Value switch |
| `casu` | "in the case of" | Switch case |
| `ceterum` | "otherwise" | Switch default |
| `discerne` | "distinguish!" | Pattern match |
| `custodi` | "guard!" | Guard clause block |
| `adfirma` | "affirm!" | Runtime assertion |

