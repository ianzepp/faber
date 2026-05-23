+++
term = "publica"
kind = "keyword"
category = "modifier"
canonical = true
summary = "Marks an import as re-exported from the current module."
syntax = "importa ex <source> publica <name>"
aliases = ["public"]
related = ["privata", "importa"]
+++

In `importa`, `publica` re-exports the imported name. The active compiler still parses `@ publica`, but member visibility is not a stable genus model.

```fab
importa ex "./types" publica Config
```
