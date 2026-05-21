+++
term = "sparge"
kind = "keyword"
category = "collection"
canonical = true
summary = "Spreads an array or collection literal into its surrounding expression."
syntax = "sparge <expression>"
aliases = ["..."]
related = ["ab", "scriptum"]
+++

Spreads an array or collection literal into its surrounding expression.

```fab
[1, sparge items, 4]
```
