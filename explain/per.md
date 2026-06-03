+++
term = "per"
kind = "keyword"
category = "range"
canonical = true
summary = "Sets the step for a range."
syntax = "<range> per <expression>"
related = ["ante", "usque"]
+++

Sets the step for a range.

```fab
itera ab 0‥10 per 2 fixum i {
    scribe i
}
```
