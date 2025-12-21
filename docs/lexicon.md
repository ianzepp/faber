# Lexicon Design

The lexicon is the vocabulary database for Faber Romanus. The morphology module uses it to:

1. Recognize word stems from inflected forms
2. Determine grammatical properties (case, number, gender, tense, person)
3. Validate that forms are correct for their declension/conjugation pattern

---

## Noun Declensions

Latin has 5 declension patterns. We'll start with the most common.

### 2nd Declension (masculine, -us)

Most of our type names and variables will follow this pattern.

| Case | Singular | Plural |
|------|----------|--------|
| Nominative | -us | -i |
| Accusative | -um | -os |
| Genitive | -i | -orum |
| Dative | -o | -is |
| Ablative | -o | -is |

Example: `nuntius` (message)
- `nuntius` (nom) - the message [subject]
- `nuntium` (acc) - the message [direct object]
- `nuntii` (gen) - of the message
- `nuntio` (dat) - to/for the message
- `nuntio` (abl) - by/with the message

### 2nd Declension (neuter, -um)

| Case | Singular | Plural |
|------|----------|--------|
| Nominative | -um | -a |
| Accusative | -um | -a |
| Genitive | -i | -orum |
| Dative | -o | -is |
| Ablative | -o | -is |

Example: `datum` (data)
- Nom/Acc identical (neuter pattern)

### 1st Declension (feminine, -a)

| Case | Singular | Plural |
|------|----------|--------|
| Nominative | -a | -ae |
| Accusative | -am | -as |
| Genitive | -ae | -arum |
| Dative | -ae | -is |
| Ablative | -a | -is |

Example: `tabula` (table)

### 3rd Declension (various)

More complex — stems don't follow a simple pattern. Need explicit stem in lexicon.

| Case | Singular | Plural |
|------|----------|--------|
| Nominative | (varies) | -es |
| Accusative | -em | -es |
| Genitive | -is | -um |
| Dative | -i | -ibus |
| Ablative | -e | -ibus |

Example: `conexio` (connection), stem `conexion-`
- `conexio` (nom)
- `conexionem` (acc)
- `conexionis` (gen)
- `conexioni` (dat)
- `conexione` (abl)

---

## Verb Conjugations

### 1st Conjugation (-are)

Example: `creare` (to create)

| Form | Ending | Example |
|------|--------|---------|
| Infinitive | -are | creare |
| Present 1s | -o | creo |
| Present 2s | -as | creas |
| Present 3s | -at | creat |
| Present 1p | -amus | creamus |
| Present 2p | -atis | creatis |
| Present 3p | -ant | creant |
| Future 1s | -abo | creabo |
| Future 3s | -abit | creabit |
| Imperative | -a | crea |

### 2nd Conjugation (-ere, long e)

Example: `habere` (to have)

### 3rd Conjugation (-ere, short e)

Example: `mittere` (to send)

| Form | Ending | Example |
|------|--------|---------|
| Infinitive | -ere | mittere |
| Present 1s | -o | mitto |
| Present 3s | -it | mittit |
| Future 3s | -et | mittet |
| Imperative | -e | mitte |

### 4th Conjugation (-ire)

Example: `audire` (to hear)

---

## Lexicon Entry Structure

```typescript
interface NounEntry {
  stem: string           // e.g., "nunti"
  declension: 1 | 2 | 3 | 4 | 5
  gender: "masculine" | "feminine" | "neuter"
  meaning: string        // English meaning for errors
  category?: "type" | "keyword" | "user"
}

interface VerbEntry {
  stem: string           // e.g., "mitt"
  conjugation: 1 | 2 | 3 | 4
  meaning: string
  category?: "keyword" | "user"
}

interface KeywordEntry {
  latin: string
  javascript: string
  type: "control" | "declaration" | "operator" | "value"
}
```

---

## Core Vocabulary

### Keywords (indeclinable)

| Latin | JS | Category |
|-------|-----|----------|
| `si` | `if` | control |
| `aliter` | `else` | control |
| `dum` | `while` | control |
| `pro` | `for` | control |
| `in` | `in` | preposition |
| `ex` | `of` | preposition |
| `cum` | (with) | preposition |
| `ad` | (to) | preposition |
| `et` | `&&` | operator |
| `aut` | `\|\|` | operator |
| `non` | `!` | operator |
| `esto` | `let` | declaration |
| `fixum` | `const` | declaration |
| `functio` | `function` | declaration |
| `futura` | `async` | modifier |
| `redde` | `return` | control |
| `verum` | `true` | value |
| `falsum` | `false` | value |
| `nihil` | `null` | value |

### Built-in Types (nouns, 2nd/3rd declension)

| Latin | JS | Declension | Gender |
|-------|-----|------------|--------|
| Textus | String | 4th | masc |
| Numerus | Number | 2nd | masc |
| Bivalens | Boolean | 3rd | — |
| Lista | Array | 1st | fem |
| Tabula | Map | 1st | fem |
| Copia | Set | 1st | fem |
| Res | Object | 3rd | fem |
| Functio | Function | 3rd | fem |
| Promissum | Promise | 2nd | neut |
| Erratum | Error | 2nd | neut |

### Common Verbs

| Latin | JS/Meaning | Conjugation |
|-------|------------|-------------|
| mittere | send | 3rd |
| legere | read | 3rd |
| scribere | write | 3rd |
| creare | create | 1st |
| delere | delete | 2nd |
| capere | catch/get | 3rd |
| reddere | return/give back | 3rd |
| vocare | call | 1st |
| facere | make/do | 3rd |

---

## Design Decisions

1. **User-defined identifiers**: Require Latin morphology. LLMs write the code anyway.

2. **Verb form depth**: Just sync/async distinction:
   - Present tense / Imperative → sync
   - Future tense → async (returns Promissum)

## Design Decisions (continued)

3. **Irregular verbs**: Reject. Use regular alternatives only.

| Avoid | Use instead | Conjugation |
|-------|-------------|-------------|
| `esse` (to be) | `existere` (to exist) | 3rd |
| `ire` (to go) | `procedere` (to proceed) | 3rd |
| `ferre` (to carry) | `portare` (to carry) | 1st |
| `velle` (to want) | `optare` (to choose) | 1st |
| `posse` (to be able) | `valere` (to be able) | 2nd |

Rationale: Easier morphology parsing, consistent patterns, good enough for v1.

## Future Ideas

1. **Prophetic perfect tense for memoization** — Perfect tense (`misit` = "has sent") could indicate cached/memoized functions. "It has already happened" = return cached result. Not core, but interesting.
