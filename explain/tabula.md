+++
term = "tabula"
kind = "type"
category = "collection"
canonical = true
summary = "Generic key/value map type."
syntax = "tabula<K,V>"
aliases = ["map", "dictionary"]
related = ["lista", "copia", "⇢", "itera"]
+++

Use `tabula<K,V>` for key/value maps. Empty map literals need an explicit target type through `⇢`.

```fab
fixum _ scores ← {} ⇢ tabula<textus, numerus>
itera de scores fixum name {
    scribe name
}
```
