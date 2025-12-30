# Structurae

Data structures: classes (genus), objects, member access, and instantiation.

## Exempla

- `exempla/structurae/`

---

## Syntax

### Genus Declaration

```ebnf
genusDecl := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
genusMember := fieldDecl | methodDecl
```

> Latin 'genus' (kind/type) for data structures.
> 'sub' (under) for inheritance - child is under parent.
> 'implet' (fulfills) for implementing pactum interfaces.
> 'abstractus' for abstract classes that cannot be instantiated.

### Genus Member

```ebnf
genusMember := fieldDecl | methodDecl
fieldDecl := ('privatus' | 'protectus')? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
methodDecl := ('privatus' | 'protectus')? 'generis'? 'abstractus'? ('futura' | 'cursor')* 'functio' ...
```

> Distinguishes between fields and methods by looking for 'functio' keyword.
> Fields are public by default (struct semantics), use 'privatus' for private.
> 'protectus' for protected visibility (subclass access).
> 'abstractus' for abstract methods (no body, must be overridden).

### Pactum Declaration

```ebnf
pactumDecl := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
```

> Latin 'pactum' (agreement/contract) for interfaces.
> Defines method signatures that genus types can implement via 'implet'.

**Examples:**

```fab
pactum Legibilis { functio lege() -> textus }
pactum Mappabilis<T, U> { functio mappa(T valor) -> U }
```

### Pactum Method

```ebnf
pactumMethod := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause?
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
```

> Method signatures without bodies. Same syntax as function declarations
> but terminates after return type (no block).

### Call

```ebnf
call := primary (callSuffix | memberSuffix | optionalSuffix | nonNullSuffix)*
callSuffix := '(' argumentList ')'
memberSuffix := '.' IDENTIFIER | '[' expression ']'
optionalSuffix := '?.' IDENTIFIER | '?[' expression ']' | '?(' argumentList ')'
nonNullSuffix := '!.' IDENTIFIER | '![' expression ']' | '!(' argumentList ')'
```

> Handles function calls, member access, and computed member access.
> Left-associative via loop (obj.a.b parsed as (obj.a).b).
> 
> OPTIONAL CHAINING: ?. ?[ ?( return nihil if object is nihil
> NON-NULL ASSERTION: !. ![ !( assert object is not nihil

### Argument List

```ebnf
argumentList := (argument (',' argument)*)?
argument := 'sparge' expression | expression
```

### Primary

```ebnf
primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
| 'ego' | 'verum' | 'falsum' | 'nihil'
| '(' (expression | arrowFunction) ')'
```

> Latin literals: verum (true), falsum (false), nihil (null).
> 'ego' (I/self) is the self-reference keyword (like 'this' in JS).
> Parenthesized expressions require lookahead to distinguish from arrow functions.

### Identifier

```ebnf
identifier := IDENTIFIER
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*