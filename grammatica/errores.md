# Errores

Error handling: try/catch, throw, panic, and scoped error handling.

## Exempla

- `exempla/errores/`

---

## Syntax

### Throw Statement

```ebnf
throwStmt := ('iace' | 'mori') expression
```

> Two error severity levels:
> iace (throw!) → recoverable, can be caught
> mori (die!)   → fatal/panic, unrecoverable

### Try Statement

```ebnf
tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
```

> 'tempta' (attempt/try), 'cape' (catch/seize), 'demum' (finally/at last).

### Catch Clause

```ebnf
catchClause := 'cape' IDENTIFIER blockStmt
```

### Fac Block Statement

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)?
```

> 'fac' (do/make) creates an explicit scope boundary for grouping
> statements with optional error handling via 'cape' (catch).
> Useful for:
> - Scoped variable declarations
> - Grouping related operations with shared error handling
> - Creating IIFE-like constructs

**Examples:**

```fab
fac { fixum x = computeValue() }
fac { riskyOperation() } cape e { scribe e }
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*