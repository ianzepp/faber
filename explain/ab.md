+++
term = "ab"
kind = "keyword"
category = "iteration"
canonical = true
summary = "Selects numeric range iteration in an itera loop."
syntax = "itera ab <range> <binding>"
related = ["itera", "ex", "de", "per", "ab pipeline"]
+++

Selects numeric range iteration in an `itera` loop. Use `ex` or `de` for collections; use `ab` only for a numeric interval.

```fab
itera ab 0‥10 per 2 fixum i {
    scribe i
}
```
