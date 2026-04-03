# Errores

Error handling in Faber distinguishes between two fundamentally different kinds of failure: recoverable errors that calling code can handle, and fatal errors that indicate unrecoverable conditions. This distinction, common in systems languages like Rust and Zig, makes error handling intentions explicit in the code itself.

The Latin vocabulary reinforces these semantics: `iace` (throw) signals an error you expect callers to catch, while `mori` (die) signals that the program cannot meaningfully continue.

## Exempla

- `exempla/statements/tempta-cape/` - try/catch patterns
- `exempla/statements/fac/` - scoped blocks with error handling
- `exempla/statements/iace/` - throw statements

---

## Try/Catch

The `tempta`/`cape`/`demum` trio corresponds directly to try/catch/finally in other languages. Use this structure when you want to attempt an operation that might fail and handle the failure gracefully.

### tempta (Try)

From Latin *temptare* (to attempt, to try). The `tempta` block wraps code that might throw an error.

```fab
tempta {
    scribe "Attempting operation..."
    riskyOperation()
    scribe "Operation succeeded"
}
```

Code after the block only executes if no error occurs. If an error is thrown and not caught, it propagates to the caller.

### cape (Catch)

From Latin *capere* (to seize, to catch). The `cape` clause binds the thrown error to a variable and executes handler code.

```fab
tempta {
    iace "Something went wrong"
    scribe "This line never runs"
}
cape err {
    scribe "Caught error:", err
}
```

The error variable (`err` in this example) is scoped to the `cape` block. You can name it whatever makes sense for your context.

A `tempta` block can omit `cape` if you only need cleanup behavior via `demum`:

```fab
tempta {
    scribe "Operation succeeds"
}
demum {
    scribe "Cleanup runs anyway"
}
```

### demum (Finally)

From Latin *demum* (at last, finally). The `demum` block contains cleanup code that runs regardless of whether an error occurred.

```fab
tempta {
    scribe "Opening resource..."
    iace "Failed to open"
}
cape err {
    scribe "Error occurred:", err
}
demum {
    scribe "Cleanup: always runs"
}
```

The `demum` block executes after both the `tempta` body and any `cape` handler, whether the operation succeeded or failed. This is essential for resource cleanup: closing files, releasing locks, resetting state.

When a function returns from within a `tempta` block, the `demum` block still executes before the return completes:

```fab
functio withReturnInDemum() -> textus {
    tempta {
        scribe "Starting operation"
        redde "success"
    }
    cape err {
        redde "error"
    }
    demum {
        scribe "Demum runs before return"
    }
}
```

### Nested Try Blocks

`tempta` blocks can nest. Inner errors are caught by inner handlers; if uncaught, they propagate outward:

```fab
tempta {
    scribe "Outer try"

    tempta {
        scribe "Inner try"
        iace "Inner error"
    }
    cape inner {
        scribe "Caught inner:", inner
    }

    scribe "Continues after inner catch"
}
cape outer {
    scribe "Outer catch:", outer
}
```

---

## Throwing Errors

Faber provides two keywords for signaling errors, each with distinct semantics.

### iace (Throw)

From Latin *iacere* (to throw, to hurl). Use `iace` for recoverable errors that calling code can catch and handle.

```fab
iace "Something went wrong"
```

The expression following `iace` becomes the error value. Typically this is a string message, but it can be any expression:

```fab
fixum code = 404
iace scriptum("Error code: {}", code)
```

`iace` throws from the current context and unwinds the stack until a `cape` clause catches it, or propagates to the program's top level.

### mori (Panic)

From Latin *mori* (to die). Use `mori` for fatal errors that indicate the program cannot meaningfully continue.

```fab
mori "Fatal: invariant violated"
```

Unlike `iace`, a `mori` cannot be caught. It terminates execution immediately. Use this for conditions that represent bugs (violated invariants, impossible states) rather than expected failure modes.

### When to Use Each

**Use `iace` when:**
- The caller might reasonably recover from this error
- The error represents an expected failure mode (file not found, invalid input, network timeout)
- You want to return error information to the caller
- The program state remains consistent after the error

**Use `mori` when:**
- The error indicates a bug in the program
- An invariant has been violated that should never happen
- Continuing would corrupt data or produce undefined behavior
- The program has reached an impossible state

The distinction follows Rust's philosophy: `iace` is for errors that are part of normal operation (like `Result::Err`), while `mori` is for programmer errors that indicate something has gone fundamentally wrong (like `panic!`).

---

## Scoped Error Handling

Beyond the traditional try/catch pattern, Faber provides `fac` blocks for creating explicit scope boundaries with optional error handling.

### fac Blocks

From Latin *facere* (to do, to make). A `fac` block creates an explicit scope for grouping statements:

```fab
fac {
    fixum x = 42
    scribe x
}
# x is not accessible here
```

Variables declared inside a `fac` block are scoped to that block. This is useful for isolating temporary values or grouping related operations.

### Adding cape to fac

A `fac` block can include a `cape` clause to handle errors from the block's body:

```fab
fac {
    fixum value = riskyComputation()
    scribe value
} cape err {
    scribe "Error occurred:", err
}
```

This is more concise than `tempta`/`cape` when you also want scope isolation, and when you don't need a `demum` clause. The `fac` block with `cape` is equivalent to:

```fab
tempta {
    fixum value = riskyComputation()
    scribe value
}
cape err {
    scribe "Error occurred:", err
}
```

### Do-While with Errors

When a `fac` block is followed by `dum` (while), it creates a do-while loop where the body executes at least once before the condition is checked:

```fab
fac { process() } dum hasMore()
```

This pattern can combine with `cape` for do-while loops with error handling:

```fab
fac {
    processNextItem()
} cape err {
    scribe "Item failed:", err
} dum hasMoreItems()
```

The loop continues as long as `hasMoreItems()` returns true. If an error occurs during `processNextItem()`, the `cape` clause handles it, and then the `dum` condition is checked.

---

## Error Handling in Functions

Functions can handle errors internally or let them propagate to callers.

### Internal Handling

When a function handles errors internally, it returns a fallback value:

```fab
functio safeDivide(numerus a, numerus b) -> numerus {
    tempta {
        si b == 0 {
            iace "Division by zero"
        }
        redde a / b
    }
    cape err {
        scribe "Error:", err
        redde 0
    }
}
```

### Propagation

When a function doesn't catch an error, it propagates to the caller:

```fab
functio divide(numerus a, numerus b) -> numerus {
    si b == 0 {
        iace "Division by zero"
    }
    redde a / b
}

# Caller must handle the error
tempta {
    fixum result = divide(10, 0)
}
cape err {
    scribe "Division failed:", err
}
```

### Cleanup with Resources

The `tempta`/`cape`/`demum` pattern is particularly valuable when working with resources that need cleanup:

```fab
functio processWithCleanup(textus name) {
    varia resource = "pending"

    tempta {
        scribe "Opening:", name
        resource = name

        si name == "" {
            iace "Empty name"
        }

        scribe "Processing:", resource
    }
    cape err {
        scribe "Error:", err
    }
    demum {
        scribe "Closing:", resource
    }
}
```

The `demum` block ensures the resource is cleaned up whether the operation succeeds or fails.

---

## Design Philosophy

Faber's error handling reflects the "compiler as tutor" philosophy. The two-tier system (`iace` vs `mori`) makes the programmer's intent explicit:

- When you write `iace`, you're communicating: "This might fail, and that's okay - handle it."
- When you write `mori`, you're communicating: "This should never happen. If it does, we have a bug."

This distinction helps both human readers and the compiler understand what kind of failure you're anticipating. The Latin vocabulary reinforces the semantics: throwing (iace) implies something catchable, while dying (mori) implies finality.
