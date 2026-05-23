+++
term = "copia"
kind = "type"
category = "collection"
canonical = true
summary = "Generic set collection type."
syntax = "copia<T>"
aliases = ["set"]
related = ["lista", "tabula", "inter"]
+++

Use `copia<T>` for unique values.

```fab
fixum copia<textus> names ← vacua
si "Marcus" inter names {
    scribe "present"
}
```
