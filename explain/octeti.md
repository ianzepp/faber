+++
term = "octeti"
kind = "type"
category = "type"
canonical = true
summary = "Primitive byte-buffer type."
syntax = "octeti"
aliases = ["bytes"]
related = ["textus", "lista"]
+++

Use `octeti` for raw byte data at I/O and binary boundaries.

```fab
functio digest(octeti data) → textus {
    redde "hash"
}
```
