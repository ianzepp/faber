+++
term = "curata"
kind = "modifier"
category = "function"
canonical = true
summary = "Marks a function as requiring a Zig allocator binding."
syntax = "functio <name>(...) curata <required> [ut <local>] → <type>"
related = ["cura", "arena"]
+++

Marks a function as requiring a Zig allocator binding. The optional `ut` alias sets the local name visible inside the function body.

```fab
functio greet(textus name) curata alloc ut a → textus {
    redde name
}
```
