# Fundamenta

Basic language elements: variables, constants, literals, and output.

## Exempla

- `exempla/fundamenta/`

---

## Syntax

### Variable Declaration

```ebnf
varDecl := ('varia' | 'fixum') (objectPattern '=' expression | typeAnnotation IDENTIFIER | IDENTIFIER) ('=' expression)?
```

> Type-first syntax: "fixum textus nomen = value" or "fixum nomen = value"
> Latin 'varia' (let it be) for mutable, 'fixum' (fixed) for immutable.

### Object Pattern

```ebnf
objectPattern := '{' patternProperty (',' patternProperty)* '}'
patternProperty := 'ceteri'? IDENTIFIER (':' IDENTIFIER)?
```

**Examples:**

```fab
{ nomen, aetas }              // extract nomen and aetas
{ nomen: localName, aetas }   // rename nomen to localName
{ nomen, ceteri rest }        // extract nomen, collect rest
NOT SUPPORTED (will produce parser errors):
{ ...rest }    // JS spread syntax
{ *rest }      // Python unpack syntax
{ **rest }     // Python kwargs syntax
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*