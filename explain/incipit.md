+++
term = "incipit"
kind = "keyword"
category = "function"
canonical = true
summary = "Declares the synchronous program entry point."
syntax = "incipit <block>"
examples = ["examples/exempla/incipit/incipit.fab"]
aliases = ["main", "entry"]
related = ["functio", "importa"]
+++

Use `incipit` for code that should run when the compiled program starts.

```fab
incipit {
scribe "Salve, munde!"
}
```
