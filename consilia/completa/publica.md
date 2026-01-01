# Notae: Annotation-Based Modifiers

Use `@` to enter annotation mode. Everything after `@` until newline is metadata (notae) that modifies the following declaration. The `#` character is reserved exclusively for line comments.

## Rationale

Faber needs visibility modifiers for module exports, but mixing Latin grammar with programming conventions created awkward compromises:

- Prefix modifiers (`publicus genus Foo`) break column-0 keyword scanning
- Postfix modifiers (`genus Foo publicum`) felt inconsistent with existing field syntax
- Inline modifiers cluttered declaration signatures

Solution: Pull all modifiers out of declarations into a separate annotation line.

### Design Principles

**Latin professor**: Annotations use proper Latin with gender agreement.

**Autistic programmer**: One consistent pattern — `@` always means metadata, `#` always means comment, declarations always have clean structure.

## Syntax

The `@` character is only valid as the first non-whitespace character on a line. This keeps parsing simple and ensures annotations are visually distinct from code.

### Basic Annotations

```fab
# This is a comment
# - with
# - Multiple
# lines

@ publicum
genus User {
    textus nomen

    @ privatum
    textus secret

    @ privata
    functio validate() -> bivalens { }
}

@ publica futura
functio fetchUsers() -> lista<User> { }

@ publicus
ordo Status { Active, Inactive }

@ publicum
pactum Serializable {
    @ publica
    functio serialize() -> textus
}
```

### Multiple Annotations

Space-separated on one line:

```fab
@ publica futura cursor
functio streamUsers() -> User { }

@ publicum abstractum
genus Base {
    @ protecta abstracta
    functio process() -> vacuum
}
```

### Annotation + Binding Modifiers

Modifiers that bind names (`cura alloc`) stay in the signature:

```fab
@ publica futura
functio fetch() cura alloc -> Data { }
```

## Gender Agreement

All gender variants are semantically equivalent — `publicum`, `publica`, and `publicus` all mean "public." The compiler accepts any form for any declaration.

However, _correct_ Latin uses gender agreement. The preferred forms:

| Declaration | Gender    | public     | private    | protected   | abstract     |
| ----------- | --------- | ---------- | ---------- | ----------- | ------------ |
| `genus`     | neuter    | `publicum` | `privatum` | `protectum` | `abstractum` |
| `pactum`    | neuter    | `publicum` | `privatum` | `protectum` | —            |
| `functio`   | feminine  | `publica`  | `privata`  | `protecta`  | `abstracta`  |
| `ordo`      | masculine | `publicus` | `privatus` | `protectus` | —            |
| field       | neuter    | `publicum` | `privatum` | `protectum` | —            |

Async/generator modifiers on functions also use feminine: `futura`, `cursor`.

**Wrong gender is cringe, not an error.** The formatter can canonicalize to correct forms. A linter can warn. But `@ publicum` on a `functio` parses and works — it just makes Latin professors wince.

## Available Annotations

### Visibility

- `publicum` / `publica` / `publicus` — exported / public
- `privatum` / `privata` / `privatus` — module-private / member-private
- `protectum` / `protecta` / `protectus` — protected (subclass access)

### Function Modifiers

- `futura` — async function
- `cursor` — generator function

### Declaration Modifiers

- `abstractum` / `abstracta` — abstract class or method
- `generis` — static member (class-level, not instance-level)

### Future Annotations

- `obsoletum "message"` — deprecation warning
- `testum` — test-only code
- `purgatum` — pure function (no side effects)

## Implementation

### 1. Lexer Changes

No new keywords needed. The `@` token switches to annotation parsing mode. Annotation words are parsed as identifiers and validated semantically. The `#` token is used for line comments.

### 2. Parser Changes (`fons/parser/index.ts`)

Add annotation parsing before declarations:

```typescript
function parseAnnotations(): Annotation[] {
    const annotations: Annotation[] = [];

    while (check('AT')) {
        advance(); // consume @
        const notes: string[] = [];

        while (!check('NEWLINE') && !isAtEnd()) {
            notes.push(parseIdentifier().name);
        }

        annotations.push({ notes, position });
    }

    return annotations;
}

function parseTopLevelDeclaration(): Statement {
    const annotations = parseAnnotations();
    const decl = parseDeclaration();
    decl.annotations = annotations;
    return decl;
}
```

### 3. AST Changes (`fons/parser/ast.ts`)

Add annotations to all declaration types:

```typescript
interface Annotation {
    notes: string[];
    position: Position;
}

interface GenusDeclaration {
    // ... existing fields
    annotations?: Annotation[];
}

// Same for FunctioDeclaration, PactumDeclaration, OrdoDeclaration,
// FieldDeclaration, VariaDeclaration, etc.
```

Derive visibility/async/generator/abstract from annotations:

```typescript
function getVisibility(annotations: Annotation[]): Visibility {
    for (const ann of annotations) {
        for (const note of ann.notes) {
            if (['publicum', 'publica', 'publicus'].includes(note)) return 'public';
            if (['privatum', 'privata', 'privatus'].includes(note)) return 'private';
            if (['protectum', 'protecta', 'protectus'].includes(note)) return 'protected';
        }
    }
    return 'private'; // default - explicit @ publicum required for exports
}
```

### 4. Semantic Validation

Validate gender agreement:

```typescript
function validateAnnotations(decl: Declaration): void {
    const gender = getDeclarationGender(decl);

    for (const note of decl.annotations?.flatMap(a => a.notes) ?? []) {
        const noteGender = getAnnotationGender(note);
        if (noteGender && noteGender !== gender) {
            warn(`Annotation '${note}' should use ${gender} form`);
        }
    }
}
```

### 5. Codegen Changes

Extract visibility from annotations and emit target-appropriate code:

**TypeScript:**

```typescript
const visibility = getVisibility(node.annotations);
const exportKeyword = visibility === 'public' ? 'export ' : '';
return `${exportKeyword}class ${node.name.name} { ... }`;
```

**Rust/Zig:** Emit `pub` prefix for public visibility.

**Python:** Ignore or generate `__all__` list.

### 6. Migration from Existing Modifiers

Current prefix modifiers on fields (`privatus textus name`) should migrate to annotation style:

```fab
# Old
genus User {
    privatus textus secret
}

# New
genus User {
    @ privatum
    textus secret
}
```

The parser could support both during transition, with warnings for old style.

## Test Cases

```yaml
# proba/codegen/statements/notae.yaml
cases:
    - name: exported genus
      input: |
          @ publicum
          genus Foo { numerus x }
      ts: |
          export class Foo { x: number; }

    - name: private field
      input: |
          @ publicum
          genus Foo {
              @ privatum
              numerus x
          }
      ts: |
          export class Foo { private x: number; }

    - name: async function
      input: |
          @ publica futura
          functio fetch() -> textus { redde "data" }
      ts: |
          export async function fetch(): Promise<string> { return "data"; }

    - name: abstract class and method
      input: |
          @ publicum abstractum
          genus Base {
              @ publica abstracta
              functio process() -> vacuum
          }
      ts: |
          export abstract class Base { abstract process(): void; }

    - name: gender agreement feminine
      input: |
          @ privata
          functio helper() -> vacuum { }
      ts: |
          function helper(): void { }
```

## Examples

### Complete Module

```fab
# User management module.
# Handles user creation, serialization, and retrieval.

# Pactum for types that can serialize themselves
@ publicum
pactum Serializable {
    @ publica
    functio serialize() -> textus
}

# Core user type.
# Stores identity and credentials.
@ publicum
genus User implet Serializable {
    textus nomen       # display name
    textus email       # primary contact

    @ privatum
    textus passwordHash  # never expose this

    @ privatum generis
    numerus instanceCount: 0  # class-level counter

    # Serialize to JSON-like format
    @ publica
    functio serialize() -> textus {
        redde scriptum("{}: {}", nomen, email)
    }

    # Hash a plaintext password.
    # Uses bcrypt under the hood.
    @ privata
    functio hashPassword(textus plain) -> textus {
        # implementation here
    }

    # Get total instances created
    @ publica generis
    functio getInstanceCount() -> numerus {
        redde User.instanceCount
    }
}

# Fetch a single user by ID
@ publica futura
functio fetchUser(numerus id) -> User? {
    # async database lookup
}

# Stream all users, yielding one at a time
@ publica futura cursor
functio streamUsers() -> User {
    # yields users from cursor
}

# Internal helper, not exported
functio internalHelper() -> vacuum {
    # module-private utility
}
```

## Open Questions

1. **Annotation arguments**: Should annotations support arguments? E.g., `@ obsoletum("Use V2")` or `@ obsoletum "Use V2"`

2. **Multi-line annotations**: Allow annotations to span lines with continuation?

    ```fab
    @ publicum
    @ obsoletum "Use V2"
    genus OldThing { }
    ```

    vs

    ```fab
    @ publicum obsoletum("Use V2")
    genus OldThing { }
    ```

3. **Contextual keywords**: Annotation words should not be reserved globally — `publicum` should be valid as an identifier elsewhere.
