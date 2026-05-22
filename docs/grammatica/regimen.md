# Regimen

Control flow determines how a program executes: which statements run, in what order, and under what conditions. Faber uses Latin keywords that map intuitively to their English equivalents while reading naturally as Latin prose.

The Latin word _regimen_ means "guidance" or "direction"—literally, the act of steering. In Faber, control flow keywords _steer_ execution through conditionals, loops, and branching constructs.

---

## Conditionals

Conditional statements execute code based on whether a condition evaluates to true or false.

### si (If)

The keyword `si` means "if" in Latin. It introduces a condition that, when true, executes the following block.

```fab
si x > 0 {
    nota "positive"
}
```

The condition must be a boolean expression. If it evaluates to `verum` (true), the block executes. Otherwise, execution continues past the block.

Multiple statements can appear within the block:

```fab
si user.authenticated {
    nota "Welcome back"
    loadUserPreferences(user)
    updateLastLogin(user)
}
```

### sin (Else If)

The keyword `sin` is a classical Latin contraction of `si non`—"but if" or "if however." It chains additional conditions after an initial `si`.

```fab
fixum hour = 14

si hour < 6 {
    nota "Late night"
}
sin hour < 12 {
    nota "Morning"
}
sin hour < 18 {
    nota "Afternoon"
}
sin hour < 22 {
    nota "Evening"
}
```

Each `sin` condition is tested only if all preceding conditions were false. The chain stops at the first true condition.

### secus (Else)

The keyword `secus` means "otherwise" in Latin. It provides a default branch when no preceding condition matched.

```fab
si hour < 12 {
    nota "Morning"
}
sin hour < 18 {
    nota "Afternoon"
}
secus {
    nota "Evening or night"
}
```

A `secus` block always executes if reached—it has no condition to test. It must appear last in a conditional chain.

### Short Forms

Faber provides concise syntax for simple conditionals.

#### ergo (Therefore)

The keyword `ergo` means "therefore" or "thus" in Latin—expressing logical consequence. Use it for one-liner conditionals where a block would be verbose.

```fab
si x > 5 ergo nota "x is big"
```

This is equivalent to:

```fab
si x > 5 {
    nota "x is big"
}
```

The `ergo` form works with `secus` for one-liner if-else:

```fab
si age ≥ 18 ergo nota "Adult" secus ergo nota "Minor"
```

#### ergo redde (Early Return)

The sequence `ergo redde` combines the one-line consequent marker with the ordinary return statement.

```fab
functio classify(numerus x) → textus {
    si x < 0 ergo redde "negative"
    si x ≡ 0 ergo redde "zero"
    redde "positive"
}
```

This is equivalent to:

```fab
functio classify(numerus x) → textus {
    si x < 0 {
        redde "negative"
    }
    si x ≡ 0 {
        redde "zero"
    }
    redde "positive"
}
```

This form excels at guard clauses—conditions that validate input and exit early:

```fab
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ergo redde nihil
    redde a / b
}
```

It works throughout a conditional chain:

```fab
functio grade(numerus score) → textus {
    si score ≥ 90 ergo redde "A"
    sin score ≥ 80 ergo redde "B"
    sin score ≥ 70 ergo redde "C"
    sin score ≥ 60 ergo redde "D"
    secus ergo redde "F"
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
    nota counter
    counter = counter + 1
}
```

The condition is checked before each iteration. If it starts false, the block never executes.

While loops work well for countdown patterns:

```fab
varia numerus countdown = 3

dum countdown > 0 {
    nota "Countdown:", countdown
    countdown = countdown - 1
}
nota "Done!"
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
    nota n
}
```

The verb `itera` is the imperative of _iterare_ ("to repeat, traverse"). The preposition `ex` means "from" or "out of"—the source from which values are drawn. The binding uses `fixum` (immutable) or `varia` (mutable).

This verb-first syntax aligns with other Faber statements like `nota`, `iace`, and `importa`. Where JavaScript writes `for (const item of items)`, Faber writes `itera ex items fixum item`.

The syntax works with any iterable:

```fab
fixum names = ["Marcus", "Julia", "Claudia"]

itera ex names fixum name {
    nota name
}
```

Processing each item:

```fab
fixum values = [10, 20, 30]

itera ex values fixum v {
    fixum doubled = v * 2
    nota doubled
}
```

The one-liner form:

```fab
itera ex numbers fixum n ergo nota n
```

#### Range Expressions

Ranges generate sequences of numbers. Faber provides three range operators:

| Operator | Latin Meaning | End Behavior | Example                             |
| -------- | ------------- | ------------ | ----------------------------------- |
| `..`     | (shorthand)   | exclusive    | `0..5` yields 0, 1, 2, 3, 4         |
| `ante`   | "before"      | exclusive    | `0 ante 5` yields 0, 1, 2, 3, 4     |
| `usque`  | "up to"       | inclusive    | `0 usque 5` yields 0, 1, 2, 3, 4, 5 |

The `..` operator is convenient shorthand matching common programming conventions (Python's `range()`, Rust's `..`):

```fab
itera ex 0..5 fixum i {
    nota i
}
```

The `ante` keyword makes exclusivity explicit—the range stops _before_ the end value:

```fab
itera ex 0 ante 5 fixum i {
    nota i
}
```

The `usque` keyword includes the end value—the range goes _up to and including_ the end:

```fab
itera ex 0 usque 5 fixum i {
    nota i
}
```

For step increments, use `per`:

```fab
itera ex 0..10 per 2 fixum i {
    nota i  # 0, 2, 4, 6, 8
}

itera ex 0 usque 10 per 2 fixum i {
    nota i  # 0, 2, 4, 6, 8, 10
}
```

### itera de (For Each Keys)

The `itera de` construct iterates over keys (property names or indices) rather than values. The syntax reads: "iterate concerning the object, binding each key."

```fab
fixum persona = { nomen: "Marcus", aetas: 30, urbs: "Roma" }

itera de persona fixum clavis {
    nota clavis
}
```

The preposition `de` means "about" or "concerning"—indicating a read-only relationship with the object. You're iterating _concerning_ the object's structure, not extracting its contents.

To access values, use the key with bracket notation:

```fab
itera de persona fixum clavis {
    nota "Key: §, Value: §"(clavis, persona[clavis])
}
```

For arrays, `de` iterates indices:

```fab
fixum numeri = [10, 20, 30]

itera de numeri fixum index {
    nota "Index §: §"(index, numeri[index])
}
```

The distinction between `ex` and `de` mirrors their Latin meanings:

- `itera ex items fixum item` — draw _values_ **from** the collection
- `itera de object fixum key` — inspect _keys_ **concerning** the object

### Async Iteration

For asynchronous streams, use `cede` (await) prefix with `itera`:

```fab
cede itera ex stream fixum chunk {
    nota chunk
}
```

This compiles to `for await...of` in JavaScript/TypeScript.

### Loop Control

Two keywords control loop flow:

#### rumpe (Break)

The keyword `rumpe` is the imperative of _rumpere_ ("to break"). It exits the innermost loop immediately.

```fab
varia i = 0

dum i < 10 {
    si i ≡ 5 {
        rumpe
    }
    nota i
    i = i + 1
}
```

Output: 0, 1, 2, 3, 4 (loop breaks when `i` reaches 5).

In nested loops, `rumpe` exits only the inner loop:

```fab
itera ex 0..3 fixum outer {
    itera ex 0..10 fixum inner {
        si inner ≡ 2 {
            rumpe  # exits inner loop only
        }
        nota "outer=§, inner=§"(outer, inner)
    }
}
```

#### perge (Continue)

The keyword `perge` is the imperative of _pergere_ ("to continue" or "proceed"). It skips to the next iteration.

```fab
itera ex 0..10 fixum i {
    si i % 2 ≡ 0 {
        perge  # skip even numbers
    }
    nota i
}
```

Output: 1, 3, 5, 7, 9 (even numbers skipped).

Like `rumpe`, `perge` affects only the innermost loop.

---

## Branching

Branching statements select one path among several based on a value.

### elige (Switch)

The keyword `elige` is the imperative of _eligere_ ("to choose"). It matches a value against cases.

```fab
fixum status = "active"

elige status {
    casu "pending" {
        nota "Waiting..."
    }
    casu "active" {
        nota "Running"
    }
    casu "done" {
        nota "Completed"
    }
}
```

The keyword `casu` is the ablative of _casus_ ("case" or "instance")—literally "in the case of."

Unlike C-family switch statements, Faber's `elige` does not fall through. Each `casu` is self-contained.

For a default branch, use `ceterum` ("otherwise" or "for the rest"):

```fab
elige code {
    casu 200 {
        nota "OK"
    }
    casu 404 {
        nota "Not Found"
    }
    ceterum {
        nota "Unknown status"
    }
}
```

One-liner cases use `ergo`:

```fab
elige status {
    casu "pending" ergo nota "waiting"
    casu "active" ergo nota "running"
    ceterum ergo iace "Unknown status"
}
```

Early returns use `ergo redde`:

```fab
functio statusMessage(numerus code) → textus {
    elige code {
        casu 200 ergo redde "OK"
        casu 404 ergo redde "Not Found"
        casu 500 ergo redde "Server Error"
        ceterum ergo redde "Unknown"
    }
}
```

### discerne (Pattern Match)

The keyword `discerne` is the imperative of _discernere_ ("to distinguish" or "discriminate"). It performs exhaustive pattern matching on tagged union types (_discretio_).

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
functio handle_event(Event e) → nihil {
    discerne e {
        casu Click pro x, y {
            nota "Clicked at §, §"(x, y)
        }
        casu Keypress pro key {
            nota "Key: §"(key)
        }
        casu Quit {
            nota "Goodbye"
        }
    }
}
```

The `pro` keyword destructures variant fields into local bindings. For the `Quit` variant (which has no fields), the block has no `pro` clause.

For simple variants without data, the body can use `ergo`:

```fab
discretio Status { Active, Inactive, Pending }

functio describe(Status s) → textus {
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
        redde l.nomen ≡ r.nomen
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

The keyword `custodi` is the imperative of _custodire_ ("to guard" or "watch over"). It groups early-exit conditions at the start of a function.

```fab
functio divide(numerus a, numerus b) → numerus {
    custodi {
        si b ≡ 0 {
            redde 0
        }
    }

    redde a / b
}
```

The `custodi` block creates a visual separation between validation and main logic. All precondition checks cluster together, making the function's requirements explicit.

Multiple guards in one block:

```fab
functio processValue(numerus x) → numerus {
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
functio sqrt(numerus n) → numerus {
    custodi {
        si n < 0 {
            iace "Cannot compute square root of negative number"
        }
    }

    redde computeSqrt(n)
}
```

The `ergo redde` form works within `custodi`:

```fab
functio clamp(numerus value, numerus min, numerus max) → numerus {
    custodi {
        si value < min ergo redde min
        si value > max ergo redde max
    }

    redde value
}
```

Input validation patterns:

```fab
functio createUser(textus name, textus email, numerus age) → textus {
    custodi {
        si name ≡ nihil aut name ≡ "" {
            redde "Error: name required"
        }
        si email ≡ nihil aut email ≡ "" {
            redde "Error: email required"
        }
        si age < 13 {
            redde "Error: must be 13 or older"
        }
        si age > 120 {
            redde "Error: invalid age"
        }
    }

    redde "User created: §"(name)
}
```

### adfirma (Assert)

The keyword `adfirma` is the imperative of _adfirmare_ ("to affirm" or "assert"). It checks runtime invariants.

```fab
adfirma x > 0
```

If the condition is false, execution halts with an assertion error.

Add a message for clarity:

```fab
adfirma x > 0, "x must be positive"
adfirma name ≠ "", "name must not be empty"
```

Assertions differ from guards:

- Guards handle expected edge cases gracefully (return early, throw recoverable errors)
- Assertions catch programming errors (bugs) that should never occur in correct code

Use `adfirma` for conditions that indicate bugs if violated:

```fab
functio processArray(lista<numerus> items) {
    adfirma items ≠ nihil, "items must not be null"
    adfirma items.longitudo > 0, "items must not be empty"

    # ... process items
}
```

---

## Summary

| Keyword    | Latin Meaning    | Purpose                |
| ---------- | ---------------- | ---------------------- |
| `si`       | "if"             | Conditional branch     |
| `sin`      | "but if"         | Else-if branch         |
| `secus`    | "otherwise"      | Else branch            |
| `ergo`     | "therefore"      | One-liner consequent   |
| `ergo redde` | "therefore return" | One-liner early return |
| `dum`      | "while"          | While loop             |
| `ex`       | "from, out of"   | Iteration source       |
| `de`       | "concerning"     | Key iteration source   |
| `pro`      | "for"            | Iteration binding      |
| `rumpe`    | "break!"         | Exit loop              |
| `perge`    | "continue!"      | Skip to next iteration |
| `elige`    | "choose!"        | Value switch           |
| `casu`     | "in the case of" | Switch case            |
| `ceterum`  | "otherwise"      | Switch default         |
| `discerne` | "distinguish!"   | Pattern match          |
| `custodi`  | "guard!"         | Guard clause block     |
| `adfirma`  | "affirm!"        | Runtime assertion      |
