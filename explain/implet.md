---
term: "implet"
kind: "keyword"
category: "type"
canonical: true
summary: "Declares that a type implements one or more pactum contracts."
syntax: "genus <name> implet <interface-list> <block>"
aliases:
  - "implements"
related:
  - "genus"
  - "pactum"
---

Declares that a type implements one or more pactum contracts.

```fab
genus Circle implet Drawable {
functio draw() { }
}
```
