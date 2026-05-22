+++
term = "redde"
kind = "keyword"
category = "transfer"
canonical = true
summary = "Returns a value from a function."
syntax = "redde <expression>"
aliases = ["return"]
related = ["→", "⇥", "ergo", "iace", "tacet"]
+++

Returns a value from a function.

```fab
functio parse(textus input) → numerus ⇥ textus {
    si input ≡ "" ergo iace "empty"
    redde 42
}
```

`redde` exits through the normal `→` return path. In a function with a `⇥`
alternate-exit contract, use `iace` to leave through the recoverable error path.
