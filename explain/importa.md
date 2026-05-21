---
term: "importa"
kind: "keyword"
category: "package"
canonical: true
summary: "Imports names from another module or package source."
syntax: "importa ex <source> privata <name>"
examples:
  - "examples/exempla/importa/importa.fab"
aliases:
  - "import"
related:
  - "incipit"
  - "functio"
---

Use `importa` at module scope to bring external names into the current file.

```fab
importa ex "utils" privata helper
```
