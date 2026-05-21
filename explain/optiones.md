---
term: "optiones"
kind: "modifier"
category: "entry"
canonical: true
summary: "Binds CLI options metadata to a function declaration."
syntax: "functio <name>(...) optiones <ident> -> <type>"
related:
  - "argumenta"
  - "ad"
---

Binds CLI options metadata to a function declaration.

```fab
functio mitte() optiones Opts -> textus {
redde "ok"
}
```
