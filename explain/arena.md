+++
term = "arena"
kind = "keyword"
category = "resource"
canonical = true
summary = "Selects the arena allocator inside a cura block."
syntax = 'cura "arena" fixum _ <alloc> <block>'
related = ["cura"]
+++

Selects the arena allocator inside a Zig allocator scope.

```fab
cura "arena" fixum _ alloc {
scribe "allocated"
}
```
