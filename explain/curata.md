+++
term = "curata"
kind = "modifier"
category = "function"
canonical = true
summary = "Marks a function as requiring an allocator binding."
syntax = "functio <name>(...) curata <ident> → <type>"
related = ["cura", "arena"]
+++

Marks a function as requiring an allocator binding.

```fab
functio greet(textus name) curata alloc → textus {
redde name
}
```
