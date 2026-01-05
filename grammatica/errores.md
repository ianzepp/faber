# Errores

Error handling: try/catch, throw, panic, and scoped error handling.

## Exempla

- `exempla/errores/`

---

## Syntax

### Fac Block Statement

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
```

> 'fac' (do/make) creates an explicit scope boundary for grouping
> statements with optional error handling via 'cape' (catch).
> When followed by 'dum', creates a do-while loop where the body
> executes at least once before the condition is checked.
> Useful for:
> - Scoped variable declarations
> - Grouping related operations with shared error handling
> - Creating IIFE-like constructs
> - Do-while loops (body executes first, then condition checked)

**Examples:**

```fab
fac { fixum x = computeValue() }
fac { riskyOperation() } cape e { scribe e }
fac { process() } dum hasMore()
fac { process() } cape e { log(e) } dum hasMore()
```

---

*Generated from `fons/faber/parser/index.ts` — do not edit directly.*