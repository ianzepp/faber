# Fundamenta

Basic language elements: variables, constants, literals, and output.

## Exempla

- `exempla/fundamenta/`

---

## Syntax

### Variable Declaration

```ebnf
varDecl := ('varia' | 'fixum') (typeAnnotation IDENTIFIER | IDENTIFIER) ('=' expression)?
```

> Type-first syntax: "fixum textus nomen = value" or "fixum nomen = value"
> Latin 'varia' (let it be) for mutable, 'fixum' (fixed) for immutable.

### Object Pattern

```ebnf
objectPattern := '{' patternProperty (',' patternProperty)* '}'
patternProperty := IDENTIFIER (':' IDENTIFIER)?
```

**Examples:**

```fab
{ nomen, aetas }
{ nomen: localName, aetas }
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*