+++
term = "ad"
kind = "keyword"
category = "endpoint"
canonical = true
summary = "Requests a host capability."
syntax = "ad <path> (<args>) [→ <type> <name> [ut <alias>]] [⇥ <type>] [<block>] [cape <name> <block>]"
related = ["argumenta", "exitus"]
+++

Requests a host or provider capability. Normal compilation is permissive:
capability names do not need to be linked while compiling, but an unresolved
capability fails explicitly at runtime.

```fab
ad "fasciculus:lege" ("hello.txt") → textus content ⇥ CapabilityError {
}
```
