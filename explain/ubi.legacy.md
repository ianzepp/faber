+++
term = "ubi"
kind = "legacy"
category = "collection"
canonical = false
summary = "Retired collection pipeline filter syntax."
syntax = "<retired>"
related = ["ab"]
canonical_term = "lista"
+++

`ubi` was part of the retired `ab` collection pipeline syntax. Use ordinary collection filtering APIs and closures instead.

Filtering now belongs to the collection API rather than a special `ubi` grammar form.

```fab
fixum _ first ← numbers.prima(1)
```
