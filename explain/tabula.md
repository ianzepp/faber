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

Use `tabula<K,V>` for key/value maps. Empty maps use `vacua` with an explicit declared type.

```fab
fixum tabula<textus, numerus> scores ← vacua
itera de scores fixum name {
    scribe name
}
```
