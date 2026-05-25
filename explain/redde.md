+++
term = "redde"
kind = "keyword"
category = "transfer"
canonical = true
summary = "Returns a value from a function."
syntax = "redde <expression>"
aliases = ["return"]
related = ["→", "⇥", "∴", "iace", "tacet"]
+++

Returns a value from a function. The enclosing bodyful function must declare an explicit normal return type with `→`; a function that omits `→` is effect-only and cannot use `redde`.

```fab
functio parse(textus input) → numerus ⇥ textus {
    si input ≡ "" ∴ iace "empty"
    redde 42
}
```

`redde` exits through the normal `→` return path. In a function with a `⇥`
alternate-exit contract, use `iace` to leave through the recoverable error path.
