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

> Creates explicit scope boundary with optional error handling.

---

*Generated from `fons/parser/index.ts` — do not edit directly.*