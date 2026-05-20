# CLI Framework

**Status**: This document is an **aspirational design specification and historical reference** for Faber's declarative CLI annotation surface. It is not a description of the current `radix-rs` implementation. The surface was implemented (parser + detector + TypeScript code generation for argument parsing, help text, subcommand dispatch, and module mounting) in the earlier reference compiler (`faber-ts`) and had partial support in `rivus`.

The active `radix-rs` compiler only preserves CLI-shaped annotations as generic annotation metadata today. Dedicated AST variants (`Cli`, `Optio(OptioAnnotation)`, and `Operandus(OperandusAnnotation)`) exist in `syntax/ast.rs`, but the parser does not currently populate them, and no lowering, argument parser generation, help formatting, or command dispatch codegen exists in `radix-rs`. Treat this page as a planned contract to implement against, not as a shipped capability matrix.

The richest historical usage example now lives in the archive repository at `../faber-archivum/tests/proba` and related self-hosting/reference CLI material. The active main repo keeps only small illustrative examples under `examples/exempla/cli`.

---

## Planned Grammar Reference

### Entry Point Forms

```
incipit { ... }                              -- plain entry (body runs directly)
incipit argumenta <ident> { ... }            -- binds parsed args to variable
incipit argumenta <ident> exitus <ident> { ... }  -- also binds exit code variable
incipiet ...                                 -- async variant (historical)
```

The `argumenta` form is intended to trigger CLI argument parsing and help generation once CLI lowering exists.

### File-Level / Incipit Annotations

```
@ cli <string>                    -- required to mark a CLI program; value is the binary name
@ versio <string>                 -- program version (emitted for --version / -v)
@ descriptio <string>             -- program-level description for help
@ imperia <string> ex <ident>     -- mount an imported module's commands under the given path
```

`@ imperia` requires a prior wildcard import (`importa ex "..." privata * ut <ident>`).

### Command / Subcommand Annotations (on `functio`)

```
@ imperium <string>               -- marks the function as a CLI subcommand (the string is the command name, supports / for nesting)
@ alias <string>                  -- short alias for the command
@ descriptio <string>             -- help text for this command
```

### Option / Flag Annotations

Two historical syntaxes existed; the second ("new") form is preferred in recent examples.

**Form A (explicit type first):**
```
@ optio <type> <ident> [brevis <string>] [longum <string>] [descriptio <string>]
```
- `<type>`: `bivalens` (flag), `textus`, `numerus`, etc.

**Form B (binding-first, current preference in examples):**
```
@ optio <ident> [brevis <string>] [longum <string>] [bivalens] [descriptio <string>] [ubique] [vel <value>]
```
- Binding name comes first.
- `bivalens` as a bare modifier means boolean flag (no value).
- `ubique` marks the option as global (see Global Options below).
- `vel <value>` provides a default in the planned lowering. The value should be captured and converted to the declared type by the CLI lowering pass.

At least one of `brevis` or `longum` is required unless using the `optiones` bundling path.

### Global Options (`ubique`)

The `ubique` modifier may **only** appear on `@ optio` (and `@ operandus`) annotations attached at the top level — that is, directly before or on the `incipit` that carries the `@ cli` annotation.

```fab
@ cli "vivi"
@ optio textus config longum "config" descriptio "..." ubique
@ optio bivalens json longum "json" descriptio "..." ubique
@ optio textus account longum "account" descriptio "..." ubique

incipit argumenta args {}
```

Rules:
- `ubique` options are automatically available to every command in the program (including commands reached via `@ imperia` module mounts).
- All `ubique` options are merged into the **same flat `args` object** as the command’s local options and operands.
- A name collision between a `ubique` option and a command-local `@ optio` or `@ operandus` is a **compiler error** (reported by the CLI lowering pass).

### Defaults (`vel`)

The `vel <value>` modifier on `@ optio` (and `@ operandus`) declares a CLI-level default.

- If `vel` is **present**, the lowering pass will inject the provided value (after converting the string literal to the declared option type) unless the user supplies the flag on the command line.
- If `vel` is **absent**, the option is optional at the CLI surface. When the user does not provide it, an absent/null value is passed into the `args` object. The command handler may then apply its own `vel` fallback if desired.

Examples:

```fab
@ optio textus interval longum "interval" vel "30s" descriptio "Poll interval for --watch"

@ optio numerus limit longum "limit" vel 100 descriptio "..."

@ optio bivalens strict longum "strict" vel falsum descriptio "..."
```

`vel` values are captured as strings by the parser. The CLI lowering/codegen is responsible for converting them to the declared Faber type (`textus`, `numerus`, `bivalens`, etc.).

`vel` works on both normal options and `ubique` globals.

### CLI-Supported Types

The type position in `@ optio <type>` and `@ operandus <type>` is intended to accept:

- All basic primitive types (`textus`, `numerus`, `bivalens`, `fractus`, `magnus`, `octeti`, etc.)
- `lista<textus>` — for repeatable string-valued options
- `lista<numerus>` — for repeatable numeric options

`vel <value>` defaults are intended to be supported on all of the above types.

**Currently out of scope** (intentionally treated as `textus` for now):

- File paths
- Enums or simple choice types (e.g. folder roles, output formats)
- `lista<bivalens>`
- User-defined `genus` and `tabula<K, V>` types

**Future direction**: Once Faber has a first-class `json` (or equivalent structured data) type together with standard library parsing support, the CLI annotation layer will recognize the type and attempt to parse the argument value automatically.

This scoping keeps the CLI surface focused on what can be reasonably and reliably parsed from a command line while still allowing rich types where they provide clear value (especially repeatable flags).

### Positional Argument Annotations

```
@ operandus [ceteri] <type> <ident> [descriptio <string>] [ubique] [vel <value>]
```

- `ceteri` makes it a rest/variadic collector (must be last; collects into `lista<textus>` in the historical lowering).
- Only one `ceteri` operand allowed per command.
- `ubique` is allowed but rare; the same top-level-only + unified binding + collision rules apply as for options.
- `vel <value>` works the same way as on options (see Defaults section above).
- The same type restrictions apply as documented in the CLI-Supported Types section.

### Function Modifiers Related to CLI

- `optiones <ident>` on a `functio` — bundles all `@ optio` values for that command into a `tabula<textus, ...>` (or equivalent) instead of individual parameters.
- `exitus <ident>` on `incipit argumenta ...` — binds the exit code variable.

---

## File-Level Annotations (Summary Table)

| Annotation     | Arguments                          | Placement          | Meaning |
|----------------|------------------------------------|--------------------|---------|
| `@ cli`        | string (name)                      | before `incipit`   | Marks program as CLI entry point |
| `@ versio`     | string                             | before `incipit`   | `--version` output |
| `@ descriptio` | string                             | before `incipit` or on `@ imperium` / module `incipit` | Help description |
| `@ imperia`    | string (mount path) `ex` ident     | before `incipit`   | Mount submodule commands |

**Note on globals**: `@ optio ... ubique` and `@ operandus ... ubique` are also declared at this top level (see Global Options grammar above).

## Command-Level Annotations

| Annotation     | Arguments                          | Placement             | Meaning |
|----------------|------------------------------------|-----------------------|---------|
| `@ imperium`   | string (command name, `/` allowed) | on `functio`          | Subcommand definition |
| `@ alias`      | string                             | on `@ imperium` fn    | Command alias |
| `@ optio`      | see grammar above (including `vel` and `lista<T>`) | before `@ imperium` fn, or at top level with `ubique` | Flag/option (global when `ubique`) |
| `@ operandus`  | see grammar above (including `vel` and `lista<T>`) | before `@ imperium` fn, or at top level with `ubique` | Positional (global when `ubique`) |

---

## Historical Implementation Notes (for Future Revival)

**2026-05 update**: The `ubique` modifier, unified `args` binding model, `vel`-based defaults, and the scoped CLI type support (primitives + `lista<textus>` + `lista<numerus>`, with everything else treated as `textus` for now) were added to this planned design during review against real-world CLIs such as `vivi`. These rules are not implemented in `radix-rs`; they are part of the preserved specification for future implementation.

The earlier reference implementation (`../faber-archivum/reference/faber-ts/codegen/cli/{detector.ts,resolver.ts}` and the incipit/incipiet generators) did the following:

- Built a full command tree supporting nested paths via `/` in `@ imperium` and `@ imperia`.
- Supported both **single-command mode** (pure `@ optio`/`@ operandus` on `incipit`) and **subcommand mode**.
- Resolved `@ imperia` mounts by following wildcard `importa * ut Alias` imports and recursively extracting commands (with cycle detection).
- Generated TypeScript argument parsing + formatted `--help` / `--version` output.
- Supported the `optiones <name>` modifier to receive a single options table.
- Distinguished `incipit` (sync) vs `incipiet` (async) entry points for CLI dispatch.

When reviving this in `radix-rs`, the prepared `OptioAnnotation` and `OperandusAnnotation` structs in the AST should be populated by enhancing `parse_annotation_kind` (or a post-parse CLI normalization pass) rather than keeping everything as generic `AnnotationStmt`.

The full historical CLI corpus is archive material now; `examples/exempla/cli` is only a small active illustration.

## CLI Modes

The annotation surface sketches two CLI patterns:

| Mode | Use case | Defined by |
|------|----------|------------|
| **Single-command** | Simple utilities (`echo`, `cat`, `true`) | `@ optio` / `@ operandus` annotations |
| **Subcommand** | Multi-command tools (`git`, `npm`, `docker`) | `@ imperium` annotations on functions |

The current compiler preserves these annotations as metadata; mode detection belongs to the eventual lowering contract.

---

## Single-Command Mode

For simple CLI programs that don't need subcommands. This example shows the intended annotation shape; it is not a shipped generation contract.

### Basic Example

```fab
@ cli "echo"
@ versio "0.1.0"
@ descriptio "Display a line of text"
@ optio bivalens n brevis "n" descriptio "Do not output trailing newline"
@ operandus ceteri textus strings

incipit argumenta args exitus code {
    si args.n {
        consolum.fundeTextum(args.strings.coniunge(" "))
    }
    secus {
        consolum.fundeLineam(args.strings.coniunge(" "))
    }
}
```

Illustrative usage:
```
echo hello world          # prints "hello world\n"
echo -n hello world       # prints "hello world" (no newline)
echo --help               # shows help text
```

### Options: @ optio

Declare command-line flags with `@ optio`:

```
@ optio <type> <binding> [brevis "<short>"] [longum "<long>"] [descriptio "..."]
```

| Part | Required | Description |
|------|----------|-------------|
| `<type>` | Yes | `bivalens` (flag), `textus` (string), `numerus` (integer) |
| `<binding>` | Yes | Internal binding name (identifier, accessed as `args.<binding>`) |
| `brevis "<short>"` | Conditional | Short flag, single char (e.g., `brevis "v"` -> `-v`) |
| `longum "<long>"` | Conditional | Long flag (e.g., `longum "verbose"` -> `--verbose`) |
| `descriptio "..."` | No | Help text for this option |

At least one of `brevis` or `longum` is required. The `brevis` value must be a single character.

Examples:

```fab
# Short only: -l
@ optio bivalens l brevis "l" descriptio "Long listing format"

# Long only: --color
@ optio textus color longum "color" descriptio "Colorize output"

# Both short and long: -v or --verbose
@ optio bivalens v brevis "v" longum "verbose" descriptio "Enable verbose output"

# Binding differs from flag (e.g., -1 flag)
@ optio bivalens singleColumn brevis "1" descriptio "One file per line"
```

### Operands: @ operandus

Declare positional arguments with `@ operandus`:

```
@ operandus [ceteri] <type> <binding> [descriptio "..."]
```

| Part | Required | Description |
|------|----------|-------------|
| `ceteri` | No | Makes this a rest/variadic argument (collects remaining args) |
| `<type>` | Yes | `textus`, `numerus`, etc. |
| `<binding>` | Yes | Internal binding name (identifier) |
| `descriptio "..."` | No | Help text |

Examples:

```fab
# Required positional argument
@ operandus textus input descriptio "Input file"

# Rest argument (collects all remaining positional args)
@ operandus ceteri textus files descriptio "Additional files"
```

Order matters: non-rest operands are matched first, then rest operand collects the remainder.

### Entry Point: incipit argumenta

Bind parsed arguments to a variable with `incipit argumenta <name>`:

```fab
incipit argumenta args {
    scribe args.verbose
    scribe args.input
}
```

The intended lowering would generate a typed `Argumenta` interface based on the declared annotations.

### Exit Codes: exitus

Control the program's exit code with the `exitus` modifier:

```fab
# Fixed exit code (always exits 0)
incipit argumenta args exitus 0 {
    # body
}

# Variable exit code (exits with value of 'code' at end)
incipit argumenta args exitus code {
    code = 1  # set non-zero on error
}
```

Without `exitus`, no explicit exit is part of the sketch.

### Complete Example

```fab
@ cli "copy"
@ versio "1.0.0"
@ descriptio "Copy files to a destination"
@ optio bivalens v brevis "v" longum "verbose" descriptio "Print files as they are copied"
@ optio bivalens f brevis "f" longum "force" descriptio "Overwrite existing files"
@ optio textus dest brevis "d" longum "dest" descriptio "Destination directory"
@ operandus textus source descriptio "Source file"
@ operandus ceteri textus additional descriptio "Additional source files"

incipit argumenta args exitus code {
    si args.v {
        scribe scriptum("Copying § to §", args.source, args.dest)
    }
    # ... copy logic
}
```

Illustrative help:
```
copy v1.0.0
Copy files to a destination

Usage: copy [options] <source> [additional...]

Options:
  -v, --verbose     Print files as they are copied
  -f, --force       Overwrite existing files
  -d, --dest        Destination directory

Arguments:
  source            Source file
  additional        Additional source files

  --help, -h        Show this help message
  --version, -v     Show version number
```

---

## Subcommand Mode

For multi-command CLIs where each command is a separate function. This is the more common pattern for complex tools.

### Basic Structure

```fab
@ cli "agent"
@ versio "0.1.0"
@ descriptio "CLI for spawning isolated AI agent runs"
incipit argumenta args {}

@ imperium "version"
@ alias "v"
functio version() -> vacuum {
    scribe "agent v0.1.0"
}
```

Illustrative usage:
```
agent version
agent v
agent --help
```

### Subcommands: @ imperium

The `@ imperium` annotation marks the intended command shape for a future lowering pass:

```fab
@ imperium "create"
@ descriptio "Create a new resource"
functio create(textus name) -> vacuum {
    scribe scriptum("Creating §", name)
}
```

### Command Aliases (Illustrative): @ alias

This shows the intended alias annotation shape for a future lowering pass. The current compiler preserves `@ alias` as annotation metadata only.

```fab
@ imperium "list"
@ alias "ls"
@ descriptio "List all jobs with status"
functio list() -> vacuum {
    # ...
}
```

Illustrative usage: `myapp list` or `myapp ls`

### Options on Commands: @ optio

Use `@ optio` annotations for flags with short forms and descriptions:

```fab
@ imperium "run"
@ descriptio "Spawn a new agent job"
@ optio textus repo brevis "r" longum "repo" descriptio "GitHub repository (owner/repo)"
@ optio numerus issue brevis "i" longum "issue" descriptio "GitHub issue number to work on"
@ optio textus model brevis "m" longum "model" descriptio "Model shortcut or full name"
@ optio textus persona longum "persona" descriptio "Persona to use for the agent"
@ optio bivalens pr longum "pr" descriptio "Create PR when work is complete"
functio run(
    textus target,
    si textus repo,
    si numerus issue,
    si textus model,
    si textus persona,
    si bivalens pr
) -> vacuum {
    # ...
}
```

The `@ optio` annotations sketch:
- Short flag via `brevis "r"` -> `-r`
- Long flag via `longum "repo"` -> `--repo`
- Help text via `descriptio "..."`

The function signature declares parameters with `si` for optionality. An illustrative `myapp run --help` could show:

```
Spawn a new agent job

Usage: agent run [options] <target>

Arguments:
  target

Options:
  -r, --repo        GitHub repository (owner/repo)
  -i, --issue       GitHub issue number to work on
  -m, --model       Model shortcut or full name
  --persona         Persona to use for the agent
  --pr              Create PR when work is complete

  --help, -h      Show this help message
```

Illustrative usage:
```
agent run "fix the bug" -r owner/repo -i 123 -m sonnet
agent run "implement feature" --repo owner/repo --model opus --pr
```

### Positional Arguments on Commands: @ operandus

Use `@ operandus` annotations on `@ imperium` functions to declare positional arguments with help text. Again, this is the intended shape of the annotation surface:

```fab
@ imperium "emit"
@ descriptio "Compile source files to target language"
@ operandus ceteri textus files descriptio "Source files to compile"
@ optio target brevis "t" longum "target" descriptio "Target language (ts, go)"
@ optio strict longum "strict" bivalens descriptio "Fail on errors"
functio emit(
    lista<textus> files,
    si textus target,
    si bivalens strict
) -> vacuum {
    # files contains all positional arguments
}
```

The `ceteri` modifier sketches a variadic argument that collects all remaining positional args:

```
rivus emit file1.fab file2.fab file3.fab -t go
rivus emit *.fab --strict
```

Constraints:
- Only one `ceteri` operand per function
- `ceteri` must be the last operand
- Type is always `textus` (collected into `lista<textus>`)

### Options Bundle: optiones

For commands with many options, use the `optiones` modifier to bundle all `@ optio` annotations into a `tabula<textus, textus>`:

```fab
@ imperium "emit"
@ operandus ceteri textus files descriptio "Files to compile"
@ optio target brevis "t" longum "target" descriptio "Target language"
@ optio strict longum "strict" bivalens descriptio "Fail on errors"
functio emit(lista<textus> files) optiones opts -> vacuum {
    fixum target = opts["target"] vel "ts"
    fixum strict = opts["strict"] ⇒ bivalens vel falsum
    # ...
}
```

The `optiones opts` modifier sketches:
- Bundles all `@ optio` annotations into a `Map<string, string>` named `opts`
- Positional arguments (`@ operandus`) stay explicit in the function signature
- Options are accessed via bracket notation: `opts["name"]`
- Use `vel` for defaults and type coercion (`⇒ bivalens`, `⇒ numerus`)

New `@ optio` syntax (preferred):
```fab
@ optio name [brevis "x"] [longum "xxx"] [bivalens] [descriptio "..."]
```

- Name comes first (no type prefix)
- `bivalens` is an optional modifier indicating a boolean flag
- Without `bivalens`, the option expects a value

---

## Implementation Artifacts (Historical)

For anyone reviving the feature, the following locations contain the most complete prior implementation:

- **Design + usage corpus**
  - Archive CLI examples under `../faber-archivum` — most complete historical examples
  - `examples/exempla/cli/main.fab` + `commands/greet.fab`

- **Old reference lowering** (faber-ts era)
  - `../faber-archivum/reference/faber-ts/codegen/cli/detector.ts` — command tree building, annotation extraction, module mounting via `@ imperia`
  - `../faber-archivum/reference/faber-ts/codegen/cli/resolver.ts` — module loading and recursive extraction
  - `../faber-archivum/reference/faber-ts/codegen/ts/statements/incipit.ts` and `incipiet.ts` — help generation and dispatch logic
  - `../faber-archivum/reference/faber-ts/codegen/ts/generator.ts` — `CliProgram`, `CliCommandNode`, `CliOption`, `CliOperand` types

- **Radix-rs AST preparation** (partial forward port)
  - `radix/crates/radix/src/syntax/ast.rs` — `AnnotationKind::Cli`, `OptioAnnotation`, `OperandusAnnotation`

- **Known limitation in current radix-rs**
  - Lowering for `incipit argumenta` and CLI dispatch was explicitly rejected (see commit `7e906c3b` "reject unsupported incipit argumenta lowering").

This planned grammar and the artifacts above should be sufficient to re-implement the full surface cleanly on top of the existing HIR and codegen pipeline when the time comes.

---

## Command Groups: @ imperia

Mount an entire module as a command group using `@ imperia`. This is the cleanest way to organize large CLIs.

### Main Entry Point

```fab
# main.fab
ex "./modules/jobs" importa * ut jobsModulum
ex "./modules/personas" importa * ut personaeModulum

@ cli "agent"
@ versio "0.1.0"
@ descriptio "CLI for spawning isolated AI agent runs"
@ imperia "jobs" ex jobsModulum
@ imperia "personas" ex personaeModulum
incipit argumenta args {}

@ imperium "version"
@ alias "v"
functio version() -> vacuum {
    scribe "agent v0.1.0"
}
```

### Module File

Modules are regular `.fab` files with `@ descriptio` on an empty `incipit`:

```fab
# modules/jobs.fab
@ descriptio "Manage agent jobs"
incipit {}

@ imperium "list"
@ alias "ls"
@ descriptio "List all jobs with status"
@ futura
functio list() -> vacuum {
    # ...
}

@ imperium "watch"
@ descriptio "Follow job output in real-time"
@ futura
functio watch(textus id) -> vacuum {
    # ...
}

@ imperium "kill"
@ descriptio "Stop a running job"
@ futura
functio kill(textus id) -> vacuum {
    # ...
}

@ imperium "clean"
@ descriptio "Remove old jobs"
@ optio bivalens all longum "all" descriptio "Remove all jobs"
@ optio textus olderThan longum "older-than" descriptio "Remove jobs older than (e.g., 7d, 24h)"
@ futura
functio clean(si bivalens all, si textus olderThan) -> vacuum {
    # ...
}
```

The `@ descriptio` on the module's `incipit` sketches the help text for the command group. Modules don't know their mount path - they're decoupled from where they're mounted.

### Generated Help

Root help (`agent --help`):
```
agent v0.1.0
CLI for spawning isolated AI agent runs

Usage: agent <command> [options]

Commands:
  version, v
  jobs ...        Manage agent jobs
  personas ...    Manage agent personas

Options:
  --help, -h     Show this help message
  --version, -v  Show version number
```

Group help (`agent jobs --help`):
```
Usage: agent jobs <command> [options]

Commands:
  list, ls      List all jobs with status
  watch         Follow job output in real-time
  kill          Stop a running job
  clean         Remove old jobs

Options:
  --help, -h     Show this help message
```

Command help (`agent jobs clean --help`):
```
Remove old jobs

Usage: agent jobs clean [options]

Options:
  --all           Remove all jobs
  --older-than    Remove jobs older than (e.g., 7d, 24h)

  --help, -h      Show this help message
```

---

## Parameter Conventions

Function signatures define CLI arguments. The compiler infers argument types from parameter patterns:

### Required Positional Arguments

Plain parameters become required positional arguments:

```fab
@ imperium "show"
@ descriptio "Show persona details"
functio show(textus name) -> vacuum {
    # ...
}
```

Illustrative usage: `myapp personas show reviewer`

Missing required arguments produce an error:
```
Missing required argument: name
```

### Optional Flags: si

The `si` prefix marks optional parameters:

```fab
@ imperium "build"
functio build(si textus output) -> vacuum { }
```

Illustrative usage: `myapp build --output dist/`

### Boolean Flags: si bivalens

Boolean flags don't take values - their presence sets them to `verum`:

```fab
@ imperium "run"
functio run(si bivalens verbose, si bivalens quiet) -> vacuum { }
```

Illustrative usage: `myapp run --verbose`

### Combining @ optio with si

Use `@ optio` for CLI metadata (short flags, descriptions) and `si` for type-system optionality:

```fab
@ imperium "compile"
@ optio textus output brevis "o" longum "output" descriptio "Output file"
@ optio bivalens verbose brevis "v" longum "verbose" descriptio "Verbose output"
functio compile(textus input, si textus output, si bivalens verbose) -> vacuum { }
```

This gives you:
- `-o` and `--output` flags with help text
- `-v` and `--verbose` flags with help text
- Type checking that `output` and `verbose` are optional

---

## Generated Features

The intended CLI framework would generate:

| Feature | Flags | Description |
|---------|-------|-------------|
| Help | `--help`, `-h` | Shows usage, commands, and options |
| Version | `--version`, `-v` | Shows version from `@ versio` |
| Error messages | - | Missing/invalid argument errors |
| Unknown command errors | - | Suggests running `--help` |

Illustrative help is contextual:
- `myapp --help` shows top-level commands
- `myapp jobs --help` shows that subgroup's commands
- `myapp jobs clean --help` shows that command's options

## Limitations

Not yet implemented:
- Typed `@ cli`, `@ optio`, and `@ operandus` parsing in `radix-rs`
- CLI argument lowering, generated help, generated version output, and dispatch
- Default values for options (`vel`)
- Global options and operands (`ubique`)
- Environment variable fallbacks
- Mutual exclusion constraints
- Negatable flags (`--no-verbose`)
- Command-alias lowering from `@ alias`
