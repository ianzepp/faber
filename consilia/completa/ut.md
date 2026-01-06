---
status: partial
updated: 2026-01-06
note: Aliasing implemented for imports and destructuring. Parameter aliases remain unimplemented.
implemented:
  - Import alias (importa name ut alias)
  - Destructuring alias (ex obj fixum field ut alias)
not_implemented:
  - Parameter internal names (Type external ut internal)
---

# ut — Aliasing/Renaming

**Latin:** "as, like"

`ut` renames identifiers. It says "call this AS that name."

**Note:** Type casting uses `qua`, not `ut`. The `ut` preposition is reserved for aliasing/renaming.

---

## Import Alias

Import `scribe` **as** `s`:

```fab
ex norma importa scribe ut s, lege ut l
s "Salve!"
```

Multiple imports can have aliases:

```fab
ex norma importa scribe ut print, lege ut read
```

---

## Destructuring Alias

Extract field `nomen` and bind it **as** `n`:

```fab
ex persona fixum nomen ut n, aetas ut a
scribe n, a
```

Note the brace-less syntax — destructuring uses the same comma-separated pattern as imports.

Works with nested destructuring:

```fab
ex response fixum data ut d, status ut s
```

---

## Parameter Alias (Not Implemented)

**Status:** Design only.

External name `from`, internally known **as** `source`:

```fab
functio move(de Point[] from ut source, in Point[] to ut dest) {
    # 'source' and 'dest' are internal names
    # 'from' and 'to' are external (callsite) names
}
```

This allows parameter names to read well at the callsite while using different names internally.

---

## Summary

| Pattern                       | Meaning                 | Status   |
| ----------------------------- | ----------------------- | -------- |
| `importa name ut alias`       | Import alias            | Done     |
| `ex obj fixum field ut alias` | Destructuring alias     | Done     |
| `Type external ut internal`   | Parameter internal name | Not done |
