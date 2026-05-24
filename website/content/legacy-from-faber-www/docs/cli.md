---
title: CLI Framework
section: docs
order: 50
---

# CLI Framework

Faber provides a declarative CLI framework through annotations. The compiler generates argument parsing, help text, and command dispatch from metadata annotations.

## File-Level Annotations

A CLI program is declared with file-level annotations on `incipit`:

```fab
@ cli "myapp"
@ versio "1.0.0"
@ descriptio "A command-line tool"
incipit argumenta args {}
```

| Annotation | Purpose |
|------------|---------|
| `@ cli "name"` | Declares file as CLI program, sets executable name |
| `@ versio "x.y.z"` | Program version (shown with `--version`) |
| `@ descriptio "..."` | Program description (shown in help text) |

The `@ cli` annotation replaces the need for manual entry point logic. The compiler generates argument parsing and dispatch.

## CLI Modes

Faber supports two CLI patterns:

| Mode | Use case | Defined by |
|------|----------|------------|
| **Single-command** | Simple utilities (`echo`, `cat`, `true`) | `@ optio` / `@ operandus` annotations |
| **Subcommand** | Multi-command tools (`git`, `npm`, `docker`) | `@ imperium` annotations on functions |

The compiler detects the mode automatically from which annotations are present.

---

## Single-Command Mode

For simple CLI programs that don't need subcommands. Options and positional arguments are declared with annotations, then bound to a variable in `incipit`.

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

Usage:
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

The compiler generates a typed `Argumenta` interface based on the declared annotations.

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

Without `exitus`, no explicit exit is generated.

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

Generated help:
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

Usage:
```
agent version
agent v
agent --help
```

### Subcommands: @ imperium

The `@ imperium` annotation marks a function as a CLI command:

```fab
@ imperium "create"
@ descriptio "Create a new resource"
functio create(textus name) -> vacuum {
    scribe scriptum("Creating §", name)
}
```

### Command Aliases: @ alias

Add short aliases to commands:

```fab
@ imperium "list"
@ alias "ls"
@ descriptio "List all jobs with status"
functio list() -> vacuum {
    # ...
}
```

Usage: `myapp list` or `myapp ls`

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

The `@ optio` annotations provide:
- Short flag via `brevis "r"` -> `-r`
- Long flag via `longum "repo"` -> `--repo`
- Help text via `descriptio "..."`

The function signature declares parameters with `si` for optionality. Running `myapp run --help` shows:

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

Usage:
```
agent run "fix the bug" -r owner/repo -i 123 -m sonnet
agent run "implement feature" --repo owner/repo --model opus --pr
```

### Positional Arguments on Commands: @ operandus

Use `@ operandus` annotations on `@ imperium` functions to declare positional arguments with help text:

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

The `ceteri` modifier creates a variadic argument that collects all remaining positional args:

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
    fixum strict = opts["strict"] bivalentum vel falsum
    # ...
}
```

The `optiones opts` modifier:
- Bundles all `@ optio` annotations into a `Map<string, string>` named `opts`
- Positional arguments (`@ operandus`) stay explicit in the function signature
- Options are accessed via bracket notation: `opts["name"]`
- Use `vel` for defaults and type coercion (`bivalentum`, `numeratum`)

New `@ optio` syntax (preferred):
```fab
@ optio name [brevis "x"] [longum "xxx"] [bivalens] [descriptio "..."]
```

- Name comes first (no type prefix)
- `bivalens` is an optional modifier indicating a boolean flag
- Without `bivalens`, the option expects a value

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

The `@ descriptio` on the module's `incipit` provides help text for the command group. Modules don't know their mount path - they're decoupled from where they're mounted.

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

Usage: `myapp personas show reviewer`

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

Usage: `myapp build --output dist/`

### Boolean Flags: si bivalens

Boolean flags don't take values - their presence sets them to `verum`:

```fab
@ imperium "run"
functio run(si bivalens verbose, si bivalens quiet) -> vacuum { }
```

Usage: `myapp run --verbose`

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

The CLI framework automatically generates:

| Feature | Flags | Description |
|---------|-------|-------------|
| Help | `--help`, `-h` | Shows usage, commands, and options |
| Version | `--version`, `-v` | Shows version from `@ versio` |
| Error messages | - | Missing/invalid argument errors |
| Unknown command errors | - | Suggests running `--help` |

Help is contextual:
- `myapp --help` shows top-level commands
- `myapp jobs --help` shows that subgroup's commands
- `myapp jobs clean --help` shows that command's options

## Limitations

Not yet implemented:
- Default values for options (`vel`)
- Environment variable fallbacks
- Mutual exclusion constraints
- Negatable flags (`--no-verbose`)

