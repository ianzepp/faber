---
title: nanus-go (Go)
description: Go compiler implementation
section: compilers
order: 1
---

# nanus-go (Go)

**Nanus** (Latin: "dwarf") is a minimal Faber compiler targeting only the language subset needed to compile **rivus**, the self-hosting Faber compiler.

This is a Go port of `fons/nanus-ts/` with the same behavior and structure.

## Why Nanus Exists

Faber implements 139 distinct language features. Rivus only uses 84 of them (~60%). Nanus exists to:

1. **Prove the minimal viable subset** - What's actually needed for a real compiler?
2. **Provide a clean reference** - 2,200 lines vs faber's 27,000
3. **Enable fast iteration** - Simpler codebase for bootstrap development
4. **Identify rivus complexity** - Features nanus struggles with may be unnecessarily complex in rivus

## Architecture

```
lexer.go    (~240 lines)  source -> tokens
parser.go   (~1150 lines) tokens -> AST
emitter.go  (~480 lines)  AST -> TypeScript
ast.go      (~190 lines)  type definitions
errors.go   (~50 lines)   error formatting
index.go    (~60 lines)   orchestration
cmd/nanus   (~65 lines)   command-line interface
```

Design principles:
- **Single file per phase** - No over-abstraction
- **Switch over visitors** - Pattern match on `node.tag`
- **Data-oriented** - Plain objects, no class hierarchies
- **Explicit over clever** - Repetition is fine if it's clear

## Status

Work in progress. Currently handles:
- Variable declarations (`varia`, `fixum`)
- Functions (sync and async)
- Classes (`genus`), interfaces (`pactum`), enums (`ordo`), tagged unions (`discretio`)
- Control flow (`si`, `dum`, `ex..fixum`, `elige`, `discerne`)
- Most expressions and operators

Known gaps being addressed:
- Keywords used as field names (`asynca`, `generator`)
- Some edge cases in type annotation parsing

## Usage

```bash
# Compile a .fab file to TypeScript
cd fons/nanus-go

go run ./cmd/nanus compile input.fab

# Output to file
go run ./cmd/nanus compile input.fab -o output.ts

# From stdin
echo 'scribe "hello"' | go run ./cmd/nanus compile
```

## Relationship to Faber and Rivus

```
faber   - Reference compiler (TypeScript, 27k lines, full language)
rivus   - Bootstrap compiler (Faber, self-hosting, ~60% of language)
nanus   - Minimal compiler (TypeScript, 2.2k lines, rivus subset only)
```

Nanus is intentionally limited. If rivus needs a feature nanus doesn't support, the first question is: "Can rivus be simplified instead?"
