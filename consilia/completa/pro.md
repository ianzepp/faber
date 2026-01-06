---
status: completed
updated: 2026-01-06
implemented:
  - Iteration binding (ex...pro, de...pro)
  - Lambda parameters (pro x: expr)
  - Variant field binding (discerne)
  - Response binding (with ad - future)
---

# pro — Iteration Binding

**Latin:** "for, on behalf of"

`pro` introduces named bindings in control flow. It says "for each of these, call it..."

---

## Iteration Binding

**For** each element, call it `item`:

```fab
ex items pro item {
    scribe item
}
```

Works with both `ex` (values) and `de` (keys):

```fab
ex items pro item { ... }      # iterate values
de object pro key { ... }      # iterate keys
```

The binding can use `pro` (preposition) or `fit`/`fiet` (verbs):

```fab
ex items pro item { }    # for each item
ex items fit item { }    # becomes item (sync)
ex stream fiet item { }  # will become item (async)
```

---

## Lambda Parameter

**For** parameter `x`, compute the body:

```fab
fixum double = pro x: x * 2
fixum add = pro a, b: a + b
```

The `pro` keyword introduces the lambda parameter list, followed by `:` and the expression body.

---

## Variant Field Binding

**For** fields `x` and `y`, bind them from the matched variant:

```fab
discerne event {
    casu Click pro x, y { scribe x, y }
    casu KeyPress pro key { scribe key }
}
```

The `pro` keyword extracts fields from the matched discriminated union variant.

---

## Response Binding (Future)

Bind the response **as** `resp`:

```fab
ad "https://api.example.com/users" ("GET") fiet Response pro resp {
    scribe resp.status
}
```

**Status:** Depends on `ad` statement implementation. See `futura/ad.md`.

---

## Summary

| Pattern                     | Meaning                  | Status   |
| --------------------------- | ------------------------ | -------- |
| `ex source pro var { }`     | Bind each element as var | Done     |
| `de object pro key { }`     | Bind each key as var     | Done     |
| `pro params: expr`          | Lambda with params       | Done     |
| `casu Variant pro fields`   | Bind variant fields      | Done     |
| `ad ... pro var { }`        | Bind response as var     | Not done |
