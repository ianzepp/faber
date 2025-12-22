# Lexicon Module TODOs

Gap analysis for transitioning from PoC to production compiler.

## Decisions

### 2nd Conjugation Semantic Mapping (2024-12-21)

2nd conjugation (-ēre verbs) reserved for **stateful operations**, distinct from
1st/3rd conjugation transformations:

| Verb | Imperative | Present 3sg | Future 3sg | Maps To |
|------|------------|-------------|------------|---------|
| videre | vide | videt | videbit | console.debug |
| habere | habe | habet | habebit | .has() / .includes() |
| tenere | tene | tenet | tenebit | loop variable binding |
| monere | mone | monet | monebit | console.warn |
| respondere | responde | respondet | respondebit | yield |

Rationale:
- `scribe` (3rd conj) = user output, `vide` (2nd conj) = dev/debug output
- `videt` could be async background logging (fire-and-forget)
- `habet` for containment: `si lista habet numerum` → `if (list.includes(number))`
- `tene` for iterator binding in loops
- `responde` for generator yields, `respondet` for pipeline yields

### 4th Conjugation Semantic Mapping (2024-12-21)

4th conjugation (-ire verbs) reserved for **IO and side-effect operations**:

| Verb | Imperative | Present 3sg | Future 3sg | Maps To |
|------|------------|-------------|------------|---------|
| aperire | aperi | aperit | aperiet | open file/stream/connection |
| finire | fini | finit | finiet | close/dispose resources |
| audire | audi | audit | audiet | listen for events |
| venire | veni | venit | veniet | incoming data/fetch |
| invenire | inveni | invenit | inveniet | .find() / search |
| scire | sci | scit | sciet | type guard / instanceof |

Complete conjugation semantic model:

| Conjugation | Theme | Examples |
|-------------|-------|----------|
| 1st (-are) | Creation | creare |
| 2nd (-ēre) | Output/observation | videre, monere, respondere |
| 3rd (-ere) | Data transformation | mittere, legere, scribere |
| 4th (-ire) | IO / Side effects | aperire, dormire, exire |

### Verb vs Method Philosophy (2024-12-21)

**Benchmark:** If it feels like a Unix command, it's a verb. Otherwise, it's a type method.

Verbs are top-level statements (like shell commands):
```
scribe x              // echo x
inveni x in lista     // grep x file
dormi 5               // sleep 5
aperi "/file"         // open file
```

Methods are operations on objects (dot syntax):
```
lista.habet(x)        // list.includes(x)
lista.pelle(x)        // list.push(x)
textus.frange(":")    // text.split(":")
```

**Removed from verbs (now type methods):**
- habere → `obj.habet(x)` - state query
- tenere → redundant with `pro` loop keyword
- portare → unclear use case

**Added to verbs (very Unix):**
- dormire → `dormi 5` like `sleep 5`
- exire → `exi 0` like `exit 0`

**Current verb inventory:**

| Conj | Verb | Example | Unix Analog |
|------|------|---------|-------------|
| 1st | creare | `crea Rem` | touch, mkdir |
| 2nd | videre | `vide x` | debug output |
| 2nd | monere | `mone x` | warn (stderr) |
| 2nd | respondere | `responde x` | yield, pipe output |
| 3rd | mittere | `mitte x ad y` | curl POST, nc |
| 3rd | legere | `lege "/file"` | cat, read |
| 3rd | scribere | `scribe x` | echo |
| 4th | aperire | `aperi "/file"` | open |
| 4th | finire | `fini x` | close, kill |
| 4th | audire | `audi portum` | nc -l, listen |
| 4th | venire | `veni ex "url"` | curl, wget |
| 4th | invenire | `inveni x in y` | grep, find |
| 4th | scire | `si scit Typum` | test -f |
| 4th | dormire | `dormi 5` | sleep |
| 4th | exire | `exi 0` | exit |

### OOP/Module Keywords (2024-12-21)

Designed for cross-target portability (TS, Zig, Rust, C, WASM):

| Keyword | Meaning | Compiles To |
|---------|---------|-------------|
| genus | kind/struct | TS class, Zig/Rust/C struct |
| pactum | agreement/contract | TS interface, Rust trait |
| publicus | public | pub/public/export |
| (default) | private | private/not exported |
| exporta | export | export/pub |
| importa...ex | import from | import/use/@import |

**Skipped (TS-only or unnecessary):**
- `extends`/`extendit` → use composition instead of inheritance
- `implements`/`implet` → structural typing handles this
- `protected`/`protectus` → only TS supports it
- `static`/`stabilis` → defer for later
- `default`/`praefinitum` → part of export syntax, not separate keyword

## High Priority (Blocking Features)

### Verb Conjugations
- [x] Add 2nd conjugation (-ēre verbs: videre, habere, tenere)
- [x] Add 4th conjugation (-ire verbs: audire, venire, scire)

### Keywords for OOP/Modules
- [x] `genus` → struct/class
- [x] `pactum` → interface/trait
- [x] `exporta` → export
- [x] `publicus` → public visibility

### Access Modifiers
- [x] `publicus` → public (default is private)

### Error Diagnostics
- [x] Return structured errors instead of `null`
- [x] Distinguish unknown stem vs invalid ending
- [ ] Add position information for error messages
- [ ] "Did you mean?" suggestions for near-misses

**Decision (2024-12-21):** Parse functions now return `LexiconError` instead of `null`:

```typescript
interface LexiconError {
    error: 'unknown_stem' | 'invalid_ending';
    word: string;
    stem?: string;   // Present if stem matched but ending failed
    ending?: string; // The ending that didn't match
}
```

Examples:
- `parseNoun('asdfgh')` → `{ error: 'unknown_stem', word: 'asdfgh' }`
- `parseNoun('nuntixx')` → `{ error: 'invalid_ending', word: 'nuntixx', stem: 'nunti', ending: 'xx' }`

No extra computation - we already knew why parsing failed, just stopped throwing it away.

### Fix Incorrect Latin
- [ ] `Res` stem is "R" but should be "re-" (5th declension irregular)
- [ ] `Functio` stem "Function" isn't Latin - consider "Actio" or "Operatio"
- [ ] Add 3rd declension neuter variant (tempus/temporis pattern)

## Medium Priority (Correctness)

### 5th Declension
- [ ] Add declension5Endings table
- [ ] Support res, dies, spes patterns
- [ ] Update getEndingsForDeclension()

### Verb Forms
- [ ] Infinitive forms (-re) for function references
- [ ] Present participle (-ns/-ntis) for callbacks/closures
- [ ] Past participle (-tus/-ta/-tum) for completed state

### Type System
- [ ] Tuple type (Iunctum?)
- [ ] Union type syntax
- [ ] Intersection type syntax
- [ ] Literal types
- [ ] Actually parse type modifiers (Naturalis, Proprius, etc.)

### Performance
- [ ] Replace linear array scan with trie structure
- [ ] Longest-match-first for stem disambiguation
- [ ] Cache parsed results for repeated identifiers

### Additional Keywords
- [ ] `ut` → as (type casting, import aliases)
- [ ] `genus` → typeof
- [ ] `cede` → yield (generators)

## Low Priority (Polish)

### Past Tenses
- [ ] Imperfect (-bam, -bas, etc.) - "was doing"
- [ ] Perfect (-i, -isti, etc.) - "did"
- [ ] Pluperfect (-eram, -eras, etc.) - "had done"

### Passive Voice
- [ ] Present passive (-or, -ris, -tur, etc.)
- [ ] Map to different code patterns (event handlers?)

### Subjunctive Mood
- [ ] Present subjunctive
- [ ] Imperfect subjunctive
- [ ] Use for conditionals, wishes, purpose clauses

### Vocative Case
- [ ] Add to declension tables
- [ ] Use for REPL/interactive commands

### Infrastructure
- [ ] Vocabulary registration API for user-defined words
- [ ] Declension/conjugation inference from principal parts
- [ ] Form generation (reverse of parsing)
- [ ] Compound word decomposition

## Test Coverage Gaps

- [ ] 1st declension nouns (Lista, Tabula patterns)
- [ ] 3rd declension nominative singular (Cursor pattern)
- [ ] 4th declension full paradigm (Textus)
- [ ] All neuter patterns systematically
- [ ] Plural forms for each declension
- [ ] Type modifier parsing
- [ ] Edge cases: empty strings, long words, Unicode
- [ ] All verb persons/numbers for each conjugation
