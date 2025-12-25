# Typi

Type system: type annotations, aliases, enums, nullable types, and collections.

## Exempla

- `exempla/typi/`

---

## Syntax

### Type Alias Declaration

```ebnf
typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
```

> Enables creating named type aliases for complex types.

**Examples:**

```fab
typus ID = textus
typus UserID = numerus<32, Naturalis>
```

### Enum Declaration

```ebnf
enumDecl := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
enumMember := IDENTIFIER ('=' (NUMBER | STRING))?
```

> Latin 'ordo' (order/rank) for enumerated constants.

**Examples:**

```fab
ordo color { rubrum, viridis, caeruleum }
ordo status { pendens = 0, actum = 1, finitum = 2 }
```

### Type Annotation

```ebnf
typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets* ('|' typeAnnotation)*
typeParams := '<' typeParameter (',' typeParameter)* '>'
typeParameter := typeAnnotation | NUMBER | MODIFIER
arrayBrackets := '[]' '?'?
```

> Supports generics (lista<textus>), nullable (?), union types (A | B),
> and array shorthand (numerus[] desugars to lista<numerus>).

---

*Generated from `fons/parser/index.ts` — do not edit directly.*