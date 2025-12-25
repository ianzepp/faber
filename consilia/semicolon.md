# Semicolon Support

Latin: *semicolon* is modern punctuation, no classical equivalent.

## Summary

Semicolons are **optional statement separators**. They enable multiple statements per line but are never required.

## Motivation

Block lambdas are naturally terse. Forcing multi-line for simple cases is awkward:

```
// Without semicolons: must be multi-line
items.mappata(pro x {
    fixum y = transform(x)
    redde y
})

// With semicolons: single line is natural
items.mappata(pro x { fixum y = transform(x); redde y })
```

Secondary motivation: programmer comfort. Developers from C-family languages may prefer explicit statement boundaries, even when optional.

## Behavior

1. **Separates statements** - allows multiple statements per line
2. **Never required** - newlines continue to work as implicit separators
3. **Trailing semicolon is valid** - `stmt;` is fine, no empty statement created
4. **Multiple semicolons collapse** - `stmt;; stmt` treated as single separator

## Examples

```
// Block lambda on one line
button.onClick(pro { setup(); doThing() })

// Promise chain with inline error handling
fetch(url).then(pro data { validate(data); redde process(data) })

// Style preference at line ends (purely optional)
fixum nomen = "Marcus";
fixum aetas = 30;

// Multiple statements in guard blocks
custodi {
    si x == nihil { log("null x"); redde nihil }
}
```

## What This Is Not

This is **not** semicolon-terminated syntax. There is no:
- Automatic Semicolon Insertion (ASI)
- "Missing semicolon" errors
- Required semicolons anywhere

The language remains newline-friendly. Semicolons are purely additive for those who want them.

## Implementation

Two parser locations consume optional semicolons:

1. **`parseBlockStatement()`** - between statements within `{ }`
2. **`parseProgram()`** - between top-level statements

The tokenizer already emits `SEMICOLON` tokens. The parser simply consumes them as optional separators via `while (match('SEMICOLON')) {}` after each statement.

## Decision History

- **2024-12**: Approved. Motivated by single-line block lambda use case (`pro { stmt; redde result }`). Secondary motivation: style preference for explicit delimiters.
