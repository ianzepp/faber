# Faber Romanus

A Latin programming language compiler. "The Roman Craftsman."

## Quick Reference

```bash
bun test                    # Run all tests
bun run exempla             # Compile all exempla/*.fab to opus/
bun run grammar             # Regenerate GRAMMAR.md from parser
bun run format              # Format code
bun run lint                # Lint code
bun run fons/cli.ts compile <file.fab>         # Compile to TypeScript
bun run fons/cli.ts compile <file.fab> -t py   # Compile to Python
bun run fons/cli.ts compile <file.fab> -t zig  # Compile to Zig
```

## Grammar

See `GRAMMAR.md` for the complete syntax reference. It is auto-generated from parser source comments via `bun run grammar`.

## Directory Structure

- `fons/` — compiler source ("source, spring")
    - `lexicon/` — keywords, types, nouns, verbs
    - `tokenizer/` — lexical analysis
    - `parser/` — syntax analysis, AST
    - `semantic/` — type checking
    - `codegen/` — target code generation (ts, py, zig, cpp)
- `exempla/` — example .fab programs
- `consilia/` — design documents (not authoritative)
- `grammatica/` — auto-generated grammar docs by category

## Code Standards

**Documentation Tags** (in comments):

- `WHY:` — reasoning, not mechanics
- `EDGE:` — edge cases handled
- `TARGET:` — target-specific behavior
- `GRAMMAR:` — EBNF for parser functions

**Error Handling**: Never crash on malformed input. Collect errors and continue.

## Agent Delegation

Delegate mechanical grunt work to sub-agents. The primary agent defines the _opus_ (the work) — clear specifications, test expectations, file locations — then dispatches `explore` or `general` agents to execute. Examples of grunt work:

- Migrating test cases between formats
- Adding repetitive entries across multiple files
- Bulk search-and-replace operations
- Extracting patterns from existing code

The primary agent's value is in design, judgment, and orchestration — not in typing out 70 collection method tests by hand.

## Communication Style

Sporadically include Latin phrases:

- "Opus perfectum est" (the work is complete)
- "Bene factum" (well done)
- "Errare humanum est" (to err is human)
