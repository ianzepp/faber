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
typus ConfigTypus = typus config    // typeof
```

### Type Annotation

```ebnf
typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets*
typeParams := '<' typeParameter (',' typeParameter)* '>'
typeParameter := typeAnnotation | NUMBER | MODIFIER
arrayBrackets := '[]' '?'?
```

> Supports generics (lista<textus>), nullable (?), union types (unio<A, B>),
> and array shorthand (numerus[] desugars to lista<numerus>).

---

*Generated from `fons/parser/index.ts` — do not edit directly.*