+++
term = "lista"
kind = "type"
category = "collection"
canonical = true
summary = "Generic ordered collection type."
syntax = "lista<T>"
aliases = ["array", "list"]
related = ["tabula", "copia", "∷", "itera"]
+++

Use `lista<T>` for ordered collections. Empty lists use `vacua` with an explicit declared type.

```fab
fixum lista<numerus> numbers ← vacua
itera ex numbers fixum n {
    scribe n
}
```
