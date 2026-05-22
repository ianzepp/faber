+++
term = "scriptum"
kind = "keyword"
category = "format"
canonical = true
summary = "Creates a formatted string with `§` placeholders."
syntax = "\"<template>\"(<args>...)"
related = ["lege", "lineam"]
+++

Creates a formatted string with `§` placeholders. The canonical source form is string-template application:

```fab
fixum _ greeting = "Salve, §!"(name)
```

The explicit desugared form remains available as `scriptum("<template>", args...)`.
