+++
term = "privata"
kind = "keyword"
category = "modifier"
canonical = true
summary = "Marks an import as local to the current module."
syntax = "importa ex <source> privata <name>"
aliases = ["private"]
related = ["publica", "importa"]
+++

In `importa`, `privata` keeps the imported name local to the current module. The active compiler still parses `@ privata`, but member visibility is not a stable genus model.

```fab
importa ex "./types" privata Helper
```
