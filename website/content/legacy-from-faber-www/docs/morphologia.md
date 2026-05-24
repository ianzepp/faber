---
title: Morphologia
section: docs
order: 50
---

# Morphologia

Latin is an inflected language. Where English relies on word order and helper words, Latin encodes meaning through word endings. A Roman hearing _amo_ knows instantly: first person, singular, present, active, indicative—"I love." Change it to _amavi_ and the meaning shifts: "I have loved." The ending carries the semantics.

Faber Romanus applies this principle to standard library methods. Instead of proliferating method names—`filter`, `filterAsync`, `filterInPlace`, `filterNewAsync`—we use Latin verb conjugations. The stem carries the action; the ending carries the behavior.

This is not decoration. It is compression. One stem, five forms, five behaviors—all learnable from a single pattern.

---

## The Five Forms

Latin verbs conjugate through dozens of forms. Faber uses five, chosen for their semantic clarity and practical utility:

| Form                     | Endings              | Semantics                       |
| ------------------------ | -------------------- | ------------------------------- |
| **Imperativus**          | -a, -e, -i           | Mutates receiver, synchronous   |
| **Perfectum**            | -ata, -ita, -ta, -sa | Returns new value, synchronous  |
| **Futurum Indicativum**  | -abit, -ebit, -iet   | Mutates receiver, asynchronous  |
| **Futurum Activum**      | -atura, -itura       | Returns new value, asynchronous |
| **Participium Praesens** | -ans, -ens           | Streaming/generator             |

The pattern emerges: **imperative forms mutate**, **participle forms allocate**. **Present tense is synchronous**, **future tense is asynchronous**. The grammar encodes the behavior.

---

## Imperativus: The Command Form

The imperative is the command form: _appende!_ ("append!"), _filtra!_ ("filter!"). In Faber, imperative methods **mutate the receiver in place** and execute **synchronously**.

```fab
varia lista<numerus> items = [1, 2, 3]
items.appende(4)           # items is now [1, 2, 3, 4]
items.filtra(|n| n > 2) # items is now [3, 4]
```

The imperative is the most common form for collection manipulation when you don't need the original data preserved. It maps to methods like `push`, `sort`, `reverse` in target languages.

**Endings:** First conjugation verbs use `-a` (_filtra_), second conjugation use `-e` (_appende_, _pone_, _remove_), third conjugation use `-i` (_accipe_).

---

## Perfectum: The Completed Action

The perfect passive participle describes a completed action: _filtrata_ ("having been filtered"), _ordinata_ ("having been ordered"). In Faber, perfect forms **return a new value** without modifying the original, executing **synchronously**.

```fab
fixum lista<numerus> items = [3, 1, 2]
fixum filtered = items.filtrata(|n| n > 1)  # [3, 2], items unchanged
fixum sorted = items.ordinata()              # [1, 2, 3], items unchanged
```

This is functional programming's bread and butter: transformations that preserve immutability. The perfect participle makes the semantics explicit—you're receiving the _result_ of an action, not commanding the action itself.

**Endings:** Most verbs use `-ata` or `-ita` (_filtrata_, _addita_, _mappata_). Some irregular verbs use `-sa` (_inversa_ from _invertere_) or `-ta`.

**Etymology:** The perfect participle is passive—_filtrata_ literally means "that which has been filtered." You receive the filtered thing, not the act of filtering.

---

## Futurum Indicativum: The Async Mutation

The future indicative states what will happen: _filtrabit_ ("it will filter"), _scribet_ ("it will write"). In Faber, future indicative methods **mutate the receiver** and execute **asynchronously**.

```fab
varia lista<datum> results = []
results.appendebit(fetchData())  # async add, mutates results
cede results.filtrabit(pred)  # await async filter, mutates results
```

This form is less common—most async operations return new values rather than mutating in place. But for streaming writes or accumulating async results, the future indicative provides the semantic.

**Endings:** First conjugation uses `-abit` (_filtrabit_), second conjugation uses `-ebit`, third conjugation uses `-iet` (_faciet_).

---

## Futurum Activum: The Async Transformation

The future active participle describes what is about to happen: _filtratura_ ("about to filter"), _lectura_ ("about to read"). In Faber, future active forms **return a new value** and execute **asynchronously**.

```fab
fixum data = cede solum.lectura(path)        # async read, returns content
fixum processed = cede items.filtratura(pred) # async filter, returns new list
```

This is the async equivalent of the perfect participle: functional transformations that happen to be asynchronous. The form signals both the async nature and the non-mutating behavior.

**Endings:** `-atura` for first conjugation, `-itura` for third/fourth conjugation.

---

## Participium Praesens: The Streaming Form

The present participle describes ongoing action: _legens_ ("reading"), _scribens_ ("writing"), _filtrans_ ("filtering"). In Faber, present participle methods produce **streams or generators**.

```fab
itera ex solum.legens(path) fixum chunk {
    processa(chunk)  # process each chunk as it arrives
}

itera ex items.filtrans(pred) fixum item {
    scribe item  # lazy evaluation, processes one at a time
}
```

This is the most distinctive form. Where imperative and perfect handle batch operations, the present participle handles streaming data—reading files chunk by chunk, processing infinite sequences, transforming data lazily.

**Endings:** First conjugation uses `-ans` (_filtrans_), other conjugations use `-ens` (_legens_, _scribens_).

**Etymology:** The present participle is active and ongoing—_legens_ means "one who is reading" or "while reading." The action is in progress, not completed.

---

## Semantic Flags

Each morphological form maps to semantic flags that the compiler uses for optimization and validation:

| Form                 | mutare | async | reddeNovum | allocatio |
| -------------------- | :----: | :---: | :--------: | :-------: |
| Imperativus          |  yes   |  no   |     no     |    no     |
| Perfectum            |   no   |  no   |    yes     |    yes    |
| Futurum Indicativum  |  yes   |  yes  |     no     |    no     |
| Futurum Activum      |   no   |  yes  |    yes     |    yes    |
| Participium Praesens |   no   |  no   |     no     |    no     |

- **mutare**: Method modifies the receiver in place
- **async**: Method returns a Promise/Future
- **reddeNovum**: Method returns a new value (doesn't modify receiver)
- **allocatio**: Method allocates memory (relevant for Zig target)

---

## The `@ radix` Annotation

Standard library methods declare their morphological forms using the `@ radix` annotation:

```fab
@ radix append, imperativus, perfectum
functio appende(T elem) -> vacuum

@ radix append, imperativus, perfectum
functio appendita(T elem) -> lista<T>
```

This declares that stem `append` supports two forms:

- `appende` (imperativus): mutates, sync
- `appendita` (perfectum): returns new, sync

If you call `lista.appenditura(x)`, the compiler warns: "form 'futurum_activum' not declared for stem 'append'." The morphology system catches typos and undefined operations.

---

## Collections vs I/O

The same morphological system serves different semantic purposes depending on the domain:

### Collections (lista, tabula, copia)

For collections, morphology distinguishes **mutation vs allocation**:

```fab
varia items = [1, 2, 3]

# Imperative: mutate in place
items.appende(4)           # items = [1, 2, 3, 4]
items.filtra(|n| n > 2) # items = [3, 4]

# Perfect: return new collection
fixum with4 = items.addita(4)       # new list, items unchanged
fixum big = items.filtrata(|n| n > 2) # new list, items unchanged
```

### I/O Operations (solum)

For I/O, morphology distinguishes **sync vs async vs streaming**:

```fab
# Imperative: synchronous, blocking
fixum content = solum.lege(path)

# Future: asynchronous, returns Promise
fixum content = cede solum.leget(path)

# Present participle: streaming
itera ex solum.legens(path) fixum chunk {
    processa(chunk)
}
```

The same grammatical system, two semantic interpretations. Collections care about mutation; I/O cares about timing.

---

## Irregular Verbs

Latin has irregular verbs, and so does Faber. The most common irregularity is in the perfect participle:

| Verb      | Regular Would Be | Actual Form | Reason           |
| --------- | ---------------- | ----------- | ---------------- |
| invertere | invertita        | inversa     | Latin _inversus_ |
| scribere  | scribita         | scripta     | Latin _scriptus_ |

The `@ radix` annotation handles this by declaring the actual stem:

```fab
# Stem is 'inver', not 'invert', because participle is 'inversa'
@ radix inver, perfectum
functio inversa() -> lista<T>
```

The compiler's stem-guided parsing handles these cases: when validating `inversa`, it knows to look for stem `inver` + `-sa`, not `invert` + `-a`.

---

## Why Not Just Use Different Names?

A reasonable question. Why `appende`/`appendita` instead of `append`/`appendNew` or `push`/`pushed`?

**Compression.** Five forms from one stem means five behaviors encoded in a learnable pattern. Once you know the pattern, you can predict the meaning of unfamiliar methods.

**Consistency.** Every collection, every I/O operation, every domain uses the same morphological system. No need to remember that arrays use `push` but strings use `append`.

**Target transparency.** `lista.appende(x)` compiles to `list.push(x)` in TypeScript, `list.append(x)` in Python, `vec.push(x)` in Rust. The Latin is the abstraction; the target language is the implementation.

**Etymology as documentation.** _Filtrata_ literally means "that which has been filtered"—a filtered thing. The grammar teaches the semantics.

---

## Summary

Latin morphology is not an affectation. It is a compression scheme: one stem, multiple endings, multiple behaviors. The conjugation table becomes an API:

| I want to...           | Form                 | Example               |
| ---------------------- | -------------------- | --------------------- |
| Mutate in place, sync  | Imperativus          | `items.appende(x)`    |
| Get new value, sync    | Perfectum            | `items.addita(x)`     |
| Mutate in place, async | Futurum Indicativum  | `items.appendebit(x)` |
| Get new value, async   | Futurum Activum      | `items.additura(x)`   |
| Stream/generate        | Participium Praesens | `items.appendens(x)`  |

The compiler validates your choices. The grammar guides your intuition. The Latin carries the meaning.

