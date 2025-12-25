# Importa

Module system: imports and exports.

## Exempla

- `exempla/importa/`

---

## Syntax

### Import Declaration

```ebnf
importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (identifierList | '*')
identifierList := IDENTIFIER (',' IDENTIFIER)*
```

**Examples:**

```fab
ex norma importa scribe, lege
ex "norma/tempus" importa nunc, dormi
ex norma importa *
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*