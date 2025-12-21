# Architecture Decisions

## Open Questions

These need resolution before or during implementation:

1. ~~**Parser approach**~~ → ADR-004
2. ~~**AST design**~~ → ADR-005
3. ~~**Latin grammar depth**~~ → ADR-003
4. ~~**Error message system**~~ → ADR-002
5. ~~**File extension**: `.fab`~~ → ADR-001

---

## Decisions

### ADR-001: File extension `.fab` (2024-12-20, updated 2024-12-21)

**Status**: Accepted (supersedes original `.fab` decision)

**Context**: Need a file extension for Faber Romanus source files.

**Considered**:
- `.fr` — Too strongly associated with French
- `.ls` — Already used by LiveScript
- `.fabt` — Three characters, prefer two
- `.fab` — Short for "Faber", but three characters
- `.fab` — ISO 639-1 code for Latin, two characters

**Decision**: Use `.fab` for source files.

**Consequences**:
- Conflicts with GNU Libtool archive files (niche, acceptable)
- ISO standard language code provides immediate recognition
- CLI will look for `*.fab` by default

### ADR-002: Error messages as correction hints (2024-12-20)

**Status**: Accepted

**Context**: The "compiler as tutor" concept could range from minimal hints to full grammar lessons.

**Decision**: Limit educational content to correction recommendations based on grammar the compiler actually supports. No lectures, no curriculum — just "you wrote X, did you mean Y?" with brief context.

**Consequences**:
- Error messages stay concise and actionable
- Educational scope is bounded by implemented features
- Grammar explanations only appear when relevant to the specific mistake
- Keeps the compiler focused on compiling, not teaching Latin 101

### ADR-003: Case endings carry semantic meaning (2024-12-20)

**Status**: Accepted

**Context**: Latin grammar depth could range from simple keyword substitution to full morphological awareness.

**Decision**: Case endings will have semantic meaning in Faber Romanus. The compiler will understand Latin morphology and use it to infer intent.

**Cases and potential meanings**:
| Case | Latin role | Code meaning |
|------|-----------|--------------|
| Nominative | subject | return value, caller, the thing acting |
| Accusative | direct object | primary argument, target of action |
| Dative | indirect object | recipient, callback, destination |
| Genitive | possession | property access, "of" relationships |
| Ablative | instrument/means | dependencies, context, "using X" |

**Consequences**:
- Parser must perform morphological analysis, not just tokenization
- Need a lexicon mapping stems to declension patterns
- AST must capture semantic roles, not just syntactic positions
- More complex than keyword substitution, but enables novel semantics
- Verb conjugation is a natural follow-on question (future tense → async?)

### ADR-004: Hand-rolled recursive descent parser (2024-12-20)

**Status**: Accepted

**Context**: Need to choose between parser generators (PEG.js, nearley) and hand-written parsing.

**Decision**: Hand-rolled recursive descent parser with a separate morphology module.

**Rationale**:
- Parser generators handle syntax well but Latin morphology (word-internal analysis) is awkward to express in grammar files
- Need to analyze stems and endings mid-parse, not just token sequences
- Better control over error messages (the tutor aspect)
- Morphology module handles: lexicon lookup, ending analysis, declension patterns
- Recursive descent handles: statement structure, expressions, scope

**Consequences**:
- More code to write upfront
- Full control over parsing and error recovery
- Morphology is a separate, testable module
- No external parser dependencies

### ADR-005: Custom AST with estree transform (2024-12-20)

**Status**: Accepted

**Context**: Need to decide AST representation — use JavaScript's standard (estree) or design custom nodes.

**Decision**: Custom AST representing Latin semantic roles, transformed to estree at codegen boundary.

**Pipeline**:
```
Source (.fab)
    ↓
Parser + Morphology
    ↓
Custom AST (Actio, Accusative, Dative, etc.)
    ↓
Semantic analysis
    ↓
Target-specific codegen
    ↓
    ├── JavaScript
    └── Zig
```

**Custom AST nodes will include**:
- Semantic roles (nominative, accusative, dative, genitive, ablative)
- Verb information (stem, tense, person, number)
- Latin-specific constructs that map to JS patterns

**Consequences**:
- AST reflects language semantics, not just syntax
- Error messages can reference Latin grammar directly
- Need a transform step to estree (additional code)
- Can leverage existing JS codegen tools for final output

### ADR-006: Semantic analysis phase with type resolution (2024-12-21)

**Status**: Accepted

**Context**: The compiler pipeline had a gap between parsing and code generation. Type information from annotations wasn't being used, and codegen was guessing types from literal values. This prevented:
- Type error detection before code generation
- Type-aware code generation (e.g., string concatenation vs numeric addition in Zig)
- Undefined variable detection
- Proper scoping validation

**Decision**: Add a semantic analysis phase between parsing and code generation that:
1. Builds a symbol table with lexical scoping
2. Resolves types from annotations and infers types from expressions
3. Annotates AST nodes with `resolvedType` field
4. Reports semantic errors (type mismatches, undefined variables, etc.)
5. Provides type information to codegen for target-specific decisions

**Architecture**:
```
Source → Tokenizer → Parser → Semantic Analyzer → Codegen
                               ↓
                        - Symbol table
                        - Type resolution
                        - Error collection
                        - AST annotation
```

**Consequences**:
- Codegen can make type-aware decisions (e.g., Zig format specifiers, string concat)
- Type errors caught before code generation
- Foundation for future morphological analysis (Phase B)
- Slightly longer compilation time (additional AST traversal)
- AST nodes are mutable (resolvedType added during analysis)

<!-- Template for decisions:

### ADR-001: Title (YYYY-MM-DD)

**Status**: Accepted | Superseded | Deprecated

**Context**: What prompted this decision?

**Decision**: What did we decide?

**Consequences**: What are the tradeoffs?

-->
