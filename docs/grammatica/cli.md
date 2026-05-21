# CLI Framework

**Status**: Implemented for the active `radix-rs` Rust target.

Faber supports declarative CLI programs with `@ cli`, `@ optio`, `@ operandus`, `@ imperium`, and `@ imperia`.
The compiler builds a validated CLI IR, typechecks `args.<name>` access, and emits self-contained Rust argument
parsing, help, version, subcommand dispatch, and package-local module mounts.

Runnable CLI code generation is currently Rust-only. TypeScript, Go, and Faber targets reject runnable CLI programs
before emitting misleading code. Package CLI compilation is also Rust-only.

## Entry Points

```fab
incipit { ... }
incipit argumenta args { ... }
incipit argumenta args exitus args.code { ... }
incipit argumenta args exitus 5 { ... }
```

`@ cli` programs must use `incipit argumenta <ident>`. The binding receives a typed record containing parsed options
and operands.

## Program Annotations

```fab
@ cli "tool"
@ versio "1.0.0"
@ descriptio "Tool description"
@ optio verbose brevis "v" longum "verbose" typus bivalens ubique
incipit argumenta args {}
```

| Annotation     | Placement       | Meaning                                              |
| -------------- | --------------- | ---------------------------------------------------- |
| `@ cli`        | root `incipit`  | Marks the program as a CLI and gives the binary name |
| `@ versio`     | root `incipit`  | Enables `--version` output                           |
| `@ descriptio` | root or command | Adds help text                                       |
| `@ imperia`    | root `incipit`  | Mounts commands from an imported module              |

Top-level `@ optio ... ubique` and `@ operandus ... ubique` values are global. They are merged into every command's
`args` object, including mounted commands.

## Options

```fab
@ optio <ident> [brevis <string>] [longum <string>] [typus <type>] [descriptio <string>] [ubique] [vel <value>]
```

Examples:

```fab
@ optio verbose brevis "v" longum "verbose" typus bivalens descriptio "Enable verbose output"
@ optio output brevis "o" longum "output" typus textus descriptio "Output path"
@ optio limit longum "limit" typus numerus vel 100 descriptio "Maximum rows"
@ optio color longum "color" vel "auto" descriptio "Color mode"
```

Rules:

- The binding name comes first.
- `typus` defaults to `textus`.
- `typus bivalens` creates a boolean flag.
- Boolean flags default to `falsum` when no `vel` is provided.
- Non-boolean options without `vel` are optional in the generated Rust type.
- At least one of `brevis` or `longum` is required.
- `brevis` must be one character and must not include `-`.
- `longum` must not include leading `--` or whitespace.
- The historical type-first form, such as `@ optio textus output ...`, is not supported.
- The historical bare `bivalens` modifier, such as `@ optio verbose bivalens ...`, is not supported. Use
  `typus bivalens`.

Supported CLI types today:

| Faber type       | Rust parser behavior                             |
| ---------------- | ------------------------------------------------ |
| `textus`         | string                                           |
| `numerus`        | `i64` parse                                      |
| `fractus`        | `f64` parse                                      |
| `bivalens`       | boolean flag or bool parse                       |
| `octeti`         | string-backed placeholder in current CLI parsing |
| `ignotum`        | string-backed placeholder                        |
| `lista<textus>`  | repeated/rest string values                      |
| `lista<numerus>` | repeated/rest numeric values                     |

Structured types, enums, choices, maps, and user-defined records are not CLI-parsed yet.

## Operands

```fab
@ operandus [ceteri] <type> <ident> [descriptio <string>] [ubique] [vel <value>]
```

Examples:

```fab
@ operandus textus input descriptio "Input file"
@ operandus ceteri textus files descriptio "Additional files"
@ operandus numerus id descriptio "Record id"
```

Rules:

- Fixed operands are assigned from positional arguments in declaration order.
- `ceteri` collects the remaining positional arguments and must be the final operand.
- Only one `ceteri` operand is allowed.
- `ubique` operands must be declared on the root `@ cli` entry point.
- Operand bindings share the same flat `args` record as options.

## Single-Command CLIs

Use a single-command CLI when the program has no subcommands.

```fab
@ cli "echo"
@ versio "0.1.0"
@ descriptio "Display a line of text"
@ optio newline brevis "n" longum "newline" typus bivalens descriptio "Print trailing newline"
@ operandus ceteri textus strings descriptio "Text to print"
incipit argumenta args {
    scribe args.strings
}
```

`--help`, `-h`, and `--version` are generated automatically. `--version` is generated only when `@ versio` exists.

## Subcommand CLIs

`@ imperium` on a function creates a dispatchable subcommand. Command handlers may use `argumenta <ident>` to receive
their parsed values.

```fab
@ cli "jobs"
@ descriptio "Job control"
@ optio verbose brevis "v" longum "verbose" typus bivalens ubique
incipit argumenta args {}

@ imperium "list"
@ alias "ls"
@ descriptio "List jobs"
@ optio limit longum "limit" typus numerus vel 20
functio list() argumenta args -> vacuum {
    scribe args.verbose
    scribe args.limit
}

@ imperium "show"
@ operandus numerus id
functio show() argumenta args -> vacuum {
    scribe args.verbose
    scribe args.id
}
```

Usage:

```text
jobs --verbose list --limit 5
jobs ls
jobs show 42
```

Rules:

- Subcommand CLIs must keep the root `incipit` body empty.
- Command function parameters are not supported for CLI dispatch. Use `argumenta args`.
- Command functions without `argumenta` can dispatch, but cannot access parsed CLI values.
- Nested command paths use `/`, for example `@ imperium "config/set"`.
- Root help lists globals and commands.
- Command help lists globals, command-local options, and operands.
- Missing or unknown commands exit with code `2`.

## Module Mounts

`@ imperia` mounts command endpoints from an explicitly imported package-local module.

```fab
importa ex "./commands/greet" privata * ut greet

@ cli "example"
@ imperia "greet" ex greet
incipit argumenta args {}
```

Mounted module:

```fab
@ imperium "hello"
@ alias "hi"
@ operandus textus name
functio hello() argumenta args -> vacuum {
    scribe scriptum("Hello, §!", args.name)
}
```

Usage:

```text
example greet hello Ian
example greet hi Ian
```

Rules:

- Mount targets must be package-local wildcard import aliases.
- Mounted modules do not declare their own `@ cli`.
- The compiler does not scan every imported file automatically; only explicit `@ imperia` mounts are exposed.
- The mounted canonical path is `mount-prefix + local-imperium-path`.
- Mounted aliases are mount-local. `@ imperia "jobs" ex jobs` plus `@ alias "set"` dispatches as `jobs set`, not
  root-level `set`.
- Root globals are available inside mounted command `args`.
- Command-local bindings that collide with root globals are compile errors.
- Mounted modules may not declare `ubique` options or operands.

## Inspection

Use `cli-ir` to inspect the normalized CLI model:

```bash
cargo run --manifest-path Cargo.toml -p faber -- cli-ir examples/exempla/cli/main.fab
```

Use `emit` or `build` for runnable Rust output:

```bash
cargo run -p faber -- emit -t rust --package examples/exempla/cli/main.fab
cargo run -p faber -- build -t rust --package examples/exempla/cli/main.fab
```

For package projects, prefer a `faber.toml` manifest with `[paths]` pointing at the CLI entry file. See
[`manifest.md`](manifest.md) for the package manifest format.

## Current Limits

- Runnable CLI codegen is Rust-only.
- Package CLI codegen is Rust-only.
- Parser helper code is emitted inline; there is no `norma::cli` runtime module yet.
- Shell completions, choice/enum option parsing, and interactive CLI features are future work.
- `optiones <ident>` remains historical/unimplemented CLI syntax. Use `argumenta <ident>` for command handler args.

## Historical Notes

Earlier reference compilers had TypeScript CLI codegen and accepted older option shapes. Those forms are archive
material now. The active compiler uses the binding-first `@ optio` grammar documented above.
