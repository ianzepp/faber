+++
term = "inter"
kind = "keyword"
category = "collection"
canonical = true
summary = "Checks whether a value appears in a collection."
syntax = "<expression> inter <collection>"
related = ["intra"]
+++

Checks whether a value appears in a collection.

```fab
si status inter ["pending", "active"] {
scribe "valid"
}
```
