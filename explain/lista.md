+++
term = "lista"
kind = "type"
category = "collection"
canonical = true
summary = "Generic ordered collection type."
syntax = "lista<T>"
aliases = ["array", "list"]
related = ["tabula", "copia", "⇢", "itera"]
+++

Use `lista<T>` for ordered collections. Empty list literals need an explicit target type through `⇢`.

```fab
fixum _ numbers ← [] ⇢ lista<numerus>
itera ex numbers fixum n {
    scribe n
}
```
