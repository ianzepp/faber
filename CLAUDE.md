# Faber Romanus

A Latin programming language compiler. "The Roman Craftsman."

## Project Philosophy

- **Compiler as tutor**: Error messages teach Latin grammar
- **Accessibility over purity**: Lower barriers, no gatekeeping
- **Incremental discovery**: Architecture emerges from decisions, not upfront planning

## Stack

- Runtime: Bun
- Language: TypeScript
- Targets: TypeScript (default), Zig

## Commands

```bash
bun run src/cli.ts compile <file.la>          # Compile to TypeScript
bun run src/cli.ts compile <file.la> -t zig   # Compile to Zig
bun test                                       # Run tests
```

## Code Standards

See `docs/code-standards.md` for comprehensive guidelines. Key points:

**Module Headers**: Every file needs a header block documenting:
- Compiler phase (lexical, syntactic, semantic, codegen)
- Input/output contract
- Invariants that must hold

**Documentation Tags**:
- `WHY:` - Explain reasoning, not mechanics
- `EDGE:` - Document edge cases handled
- `TARGET:` - Note target-specific behavior
- `GRAMMAR:` - EBNF for parser functions

**Error Handling**:
- Never crash on malformed input
- Collect errors and continue parsing
- Error messages should teach Latin syntax

**Testing**:
- Test each compiler phase independently
- Include error recovery tests

## Architecture Decisions

Tracked in `docs/decisions.md`. Each decision is numbered and dated.
