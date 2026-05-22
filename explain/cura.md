+++
term = "cura"
kind = "keyword"
category = "resource"
canonical = true
summary = "Starts a Zig allocator scope."
syntax = 'cura "<allocator-kind>" fixum _ <alloc> <block>'
related = ["arena"]
+++

Starts a Zig allocator scope. Active examples avoid allocator scopes while Zig codegen is unsupported; parser and driver tests cover the syntax.

```fab
cura "arena" fixum _ alloc {
scribe "managed"
}
```
