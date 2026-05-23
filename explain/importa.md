+++
term = "importa"
kind = "keyword"
category = "package"
canonical = true
summary = "Imports names from another module or package source."
syntax = "importa ex <source> <privata|publica> (<name> [ut <alias>] | * ut <alias>)"
examples = ["examples/exempla/importa/importa.fab"]
aliases = ["import"]
related = ["ex", "privata", "publica", "ut"]
+++

Use `importa` at module scope to bring names into the current file. `privata` imports locally; `publica` re-exports. Wildcard imports require `* ut <alias>`.

```fab
importa ex "utils" privata helper
importa ex "./types" publica User
importa ex "lodash" privata * ut _
```
