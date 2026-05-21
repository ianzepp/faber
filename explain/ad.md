---
term: "ad"
kind: "keyword"
category: "endpoint"
canonical: true
summary: "Declares an HTTP endpoint."
syntax: "ad <path> (<args>) [-> [<type>] pro <name> [ut <alias>]] [<block>] [cape <name> <block>]"
related:
  - "argumenta"
  - "exitus"
---

Declares an HTTP endpoint.

```fab
ad "/salve" (request) -> textus pro res {
}
```
