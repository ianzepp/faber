+++
term = "iacit"
kind = "modifier"
category = "function"
canonical = true
summary = "Marks a function as able to throw."
syntax = "functio <name>(...) iacit → <type>"
related = ["iace", "cape", "⇥", "errata"]
+++

Marks a function as able to throw a recoverable value. Prefer an explicit `⇥` alternate-exit type when the thrown value is part of the callable contract.

```fab
functio fallit() iacit → vacuum ⇥ textus {
    iace "err"
}
```
