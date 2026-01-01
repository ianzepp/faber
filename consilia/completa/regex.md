---
status: implemented
note: Core regex literals implemented; string methods (quaere, para, muta, mutaOmnes, probat) not yet implemented
updated: 2025-01
---

# Regex Literals

Regex literals use the `sed` keyword with a string pattern and optional flags.

## Syntax

```fab
sed "pattern"
sed "pattern" flags
```

**WHY `sed`:** The Latin word "sed" means "but" — semantically unrelated. However, `sed` (the Unix stream editor) is synonymous with pattern matching for most programmers.

**WHY strings:** Using quoted strings instead of `/pattern/` delimiters:

- No special tokenizer mode — standard string parsing applies
- Escaping already solved — `\d` passes through (unknown escapes are preserved)
- Forward slashes aren't special — `sed "/path/to/file"` just works

**WHY bare flags:** Flags are a bare identifier after the pattern string (no comma). This avoids ambiguity in method calls while keeping syntax clean.

## Flags

Flags are optional, specified as a bare identifier after the pattern:

```fab
sed "pattern"           # no flags
sed "pattern" i         # case insensitive
sed "pattern" ims       # multiple flags
```

| Flag | Meaning                                      |
| ---- | -------------------------------------------- |
| `i`  | Case insensitive                             |
| `m`  | Multiline (`^`/`$` match line boundaries)    |
| `s`  | Dotall (`.` matches newlines)                |
| `x`  | Extended (ignore whitespace, allow comments) |
| `u`  | Unicode                                      |

**No `g` flag:** "Global" (match all vs first) is not a pattern modifier — it's an iteration behavior. The method you call determines this: `quaere` finds first, `para` finds all.

## Examples

```fab
# Simple patterns
fixum digits = sed "\d+"
fixum email = sed "[^@]+@[^@]+"
fixum paths = sed "/usr/local/.*"

# With flags
fixum word = sed "hello" i
fixum lines = sed "^start" im

# In method calls — no ambiguity (flags before comma)
fixum found = text.quaere(sed "\d+")
fixum result = text.muta(sed "old" i, "new")
fixum matches = text.para(sed "\w+")
```

## Design Principles

**Pass-through pattern:** The pattern string is opaque to Faber. The compiler does not validate regex syntax — that's the target language's job. If you write `sed "[broken"`, the target compiler complains.

**One target per project:** Faber compiles to one language at a time. A regex written for a TypeScript project doesn't need to work in Zig. Write patterns appropriate for your target.

## Codegen

TypeScript uses native suffix flags. All other targets get flags injected as inline `(?...)` prefix.

### TypeScript

Native regex literal with suffix flags:

```typescript
// sed "\d+"
/\d+/

// sed "hello" i
/hello/i
```

### Python

Flags injected as inline prefix:

```python
# sed "\d+"
re.compile(r'\d+')

# sed "hello" i
re.compile(r'(?i)hello')
```

### Rust

Flags injected as inline prefix:

```rust
// sed "\d+"
Regex::new(r"\d+").unwrap()

// sed "hello" i
Regex::new(r"(?i)hello").unwrap()
```

### C++

C++ `<regex>` does not support inline flags. Regex literals without flags work; flags require manual translation.

```cpp
// sed "\d+"
std::regex("\\d+")

// sed "hello" i — NOT SUPPORTED
// Use std::regex("hello", std::regex_constants::icase) manually
```

**Limitation:** C++ users should avoid flags in `sed` literals. Use flagless patterns or construct regex manually with `std::regex_constants`.

### Zig

No stdlib regex. Emit pattern as string for use with external library. Flags injected as inline prefix:

```zig
// sed "\d+"
"\\d+"

// sed "hello" i
"(?i)hello"
```

## Parser

The parser recognizes:

1. `sed` keyword
2. String literal (pattern)
3. Optionally: bare identifier (flags — letters only)

No comma between pattern and flags. This distinguishes flags from the next method argument.

```fab
# Unambiguous parsing:
text.muta(sed "old" i, "new")
#        ^^^^^^^^^^  ^^^^^
#        regex arg   string arg
```

## AST

```typescript
interface RegexLiteral {
    type: 'RegexLiteral';
    pattern: string; // Pattern string value
    flags: string; // Flag characters, may be empty
}
```

## Type

The Faber type for regex values: TBD. Candidates:

- `forma` (form, shape)
- `exemplar` (pattern, example)
- `schema` (pattern — Greek loan, common in Latin)

For now, regex literals are expressions with inferred type. Type name can be decided when regex variables/parameters are needed.

## Methods

Regex operations on strings. The method determines iteration behavior (first match vs all matches), not the regex flags.

```fab
# Search — find first match
fixum found = text.quaere(sed "\d+")

# Replace first — substitute first match
fixum result = text.muta(sed "old", "new")

# Replace all — substitute all matches
fixum result = text.mutaOmnes(sed "old", "new")

# Match all — return all matches
fixum matches = text.para(sed "\w+")

# Test — boolean check
fixum valid = text.probat(sed "^\d{4}$")
```

## Preamble

Python codegen needs `import re` when regex literals are used.

Rust codegen needs `use regex::Regex;` when regex literals are used.

## Not Supported

- Slash delimiters (`/pattern/flags`) — use string syntax
- Comma-separated flags — use bare identifier after pattern
- Interpolation in patterns — not planned
- Named groups normalization — target-specific syntax differences remain

## Target Limitations

| Target | Flags Support                        |
| ------ | ------------------------------------ |
| TS/JS  | Full (native suffix)                 |
| Python | Best-effort (inline `(?...)` prefix) |
| Rust   | Best-effort (inline `(?...)` prefix) |
| C++    | None (no inline flag support)        |
| Zig    | Library-dependent                    |

TypeScript is the primary target with full support. Other targets get flags injected as inline `(?imsx)` prefix where the regex engine supports it — behavior may vary. Test your patterns against your target.
