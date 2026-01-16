# CLI Framework

Faber provides a declarative CLI framework through annotations. The compiler generates argument parsing, help text, and command dispatch from metadata annotations.

## File-Level Annotations

A CLI program is declared with file-level annotations on `incipit`:

```fab
@ cli "myapp"
@ versio "1.0.0"
@ descriptio "A command-line tool"
incipit {}
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
@ optio bivalens "n" descriptio "Do not output trailing newline"
@ operandus ceteri textus "args"

incipit optio opts exitus code {
    si opts.n {
        consolum.fundeTextum(opts.args.coniunge(" "))
    }
    secus {
        consolum.fundeLineam(opts.args.coniunge(" "))
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
@ optio <type> "<name>" [ut <internal>] [brevis "<short>"] [descriptio "..."]
```

| Part | Required | Description |
|------|----------|-------------|
| `<type>` | Yes | `bivalens` (flag), `textus` (string), `numerus` (integer) |
| `"<name>"` | Yes | External flag name as string literal (becomes `--name`) |
| `ut <internal>` | No | Internal binding name (defaults to `<name>` with hyphens removed) |
| `brevis "<short>"` | No | Short flag (e.g., `brevis "v"` → `-v`) |
| `descriptio "..."` | No | Help text for this option |

Examples:

```fab
# Boolean flag: --verbose or -v
@ optio bivalens "verbose" brevis "v" descriptio "Enable verbose output"

# String option: --output <path> or -o <path>
@ optio textus "output" brevis "o" descriptio "Output file path"

# With different internal name: --dry-run flag accessed as opts.dryRun
@ optio bivalens "dry-run" ut dryRun descriptio "Show what would happen"
```

### Operands: @ operandus

Declare positional arguments with `@ operandus`:

```
@ operandus [ceteri] <type> "<name>" [descriptio "..."]
```

| Part | Required | Description |
|------|----------|-------------|
| `ceteri` | No | Makes this a rest/variadic argument (collects remaining args) |
| `<type>` | Yes | `textus`, `numerus`, etc. |
| `"<name>"` | Yes | Binding name as string literal |
| `descriptio "..."` | No | Help text |

Examples:

```fab
# Required positional argument
@ operandus textus "input" descriptio "Input file"

# Rest argument (collects all remaining positional args)
@ operandus ceteri textus "files" descriptio "Additional files"
```

Order matters: non-rest operands are matched first, then rest operand collects the remainder.

### Entry Point: incipit optio

Bind parsed options to a variable with `incipit optio <name>`:

```fab
incipit optio opts {
    scribe opts.verbose
    scribe opts.input
}
```

The compiler generates a typed interface for the options object based on the declared annotations.

### Exit Codes: exitus

Control the program's exit code with the `exitus` modifier:

```fab
# Fixed exit code (always exits 0)
incipit optio opts exitus 0 {
    # body
}

# Variable exit code (exits with value of 'code' at end)
incipit optio opts exitus code {
    code = 1  # set non-zero on error
}
```

Without `exitus`, no explicit exit is generated.

### Complete Example

```fab
@ cli "copy"
@ versio "1.0.0"
@ descriptio "Copy files to a destination"
@ optio bivalens "verbose" ut v brevis "v" descriptio "Print files as they are copied"
@ optio bivalens "force" brevis "f" descriptio "Overwrite existing files"
@ optio textus "dest" brevis "d" descriptio "Destination directory"
@ operandus textus "source" descriptio "Source file"
@ operandus ceteri textus "additional" descriptio "Additional source files"

incipit optio opts exitus code {
    si opts.v {
        scribe scriptum("Copying § to §", opts.source, opts.dest)
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

For multi-command CLIs where each command is a separate function. Arguments are defined in function signatures.

### Subcommands: @ imperium

The `@ imperium` annotation marks a function as a CLI command. The function's parameters define the command's arguments and flags.

```fab
@ cli "backup"
@ versio "1.0.0"
incipit {}

@ imperium "create"
functio create(textus source, textus dest) -> vacuum {
    scribe scriptum("Backing up § to §", source, dest)
}

@ imperium "restore"
functio restore(textus archive) -> vacuum {
    scribe scriptum("Restoring §", archive)
}
```

Usage:
```
backup create ./data ./backup
backup restore ./backup/archive.tar
```

### Command Aliases: @ alias

Add short aliases to commands:

```fab
@ imperium "version"
@ alias "v"
functio version() -> vacuum {
    scribe "v1.0.0"
}
```

Usage: `myapp version` or `myapp v`

### Nested Commands: Path Syntax

Use `/` in the command name for nested subcommands:

```fab
@ imperium "config/set"
functio configSet(textus key, textus value) -> vacuum { }

@ imperium "config/get"
functio configGet(textus key) -> vacuum { }
```

Usage:
```
myapp config set theme dark
myapp config get theme
myapp config --help    # lists set, get
```

Intermediate nodes (like `config`) automatically show help listing their children.

## Command Groups: @ imperia

Mount an entire module as a command group using `@ imperia`:

```fab
# main.fab
ex "./commands/remote" importa * ut remoteModule

@ cli "git"
@ versio "1.0.0"
@ imperia "remote" ex remoteModule
incipit {}

@ imperium "status"
functio status() -> vacuum { }
```

```fab
# commands/remote.fab
@ descriptio "Manage remote repositories"
incipit {}

@ imperium "add"
functio add(textus name, textus url) -> vacuum { }

@ imperium "list"
functio list() -> vacuum { }
```

Usage:
```
git status
git remote add origin https://...
git remote list
git remote --help    # shows "Manage remote repositories"
```

The `@ descriptio` on the submodule's `incipit` provides help text for the command group. Submodules are decoupled — they don't know their mount path.

## Parameter Conventions

Function signatures define CLI arguments. The compiler infers argument types from parameter patterns:

### Required Positional Arguments

Plain parameters become required positional arguments:

```fab
@ imperium "greet"
functio greet(textus name) -> vacuum { }
```

Usage: `myapp greet Marcus`

Missing required arguments produce an error:
```
Missing required argument: name
```

### Optional Flags: si

The `si` prefix marks optional parameters, generating `--flagname`:

```fab
@ imperium "build"
functio build(si textus output) -> vacuum { }
```

Usage: `myapp build --output dist/`

### Short Forms: ut

Add short flag aliases with `ut`:

```fab
@ imperium "build"
functio build(si textus output ut o) -> vacuum { }
```

Usage: `myapp build -o dist/` or `myapp build --output dist/`

### Boolean Flags: si bivalens

Boolean flags don't take values — their presence sets them to `verum`:

```fab
@ imperium "run"
functio run(si bivalens verbose ut v, si bivalens quiet ut q) -> vacuum { }
```

Usage: `myapp run -v` or `myapp run --verbose`

### Combined Example

```fab
@ imperium "process"
functio process(
    textus input,
    si textus output ut o,
    si bivalens verbose ut v,
    si bivalens dry ut n
) -> vacuum { }
```

Usage:
```
myapp process data.txt                     # required only
myapp process data.txt -o out.txt          # with output
myapp process data.txt -v -n               # with flags
myapp process data.txt --output out.txt --verbose --dry
```

## Generated Features

The CLI framework automatically generates:

| Feature | Flags | Description |
|---------|-------|-------------|
| Help | `--help`, `-h` | Shows usage, commands, and options |
| Version | `--version`, `-v` | Shows version from `@ versio` |
| Error messages | — | Missing/invalid argument errors |
| Unknown command errors | — | Suggests running `--help` |

Help is contextual — running `myapp --help` shows top-level commands, while `myapp remote --help` shows that subgroup's commands.

## Limitations

Not yet implemented:
- Default values for options (`vel`)
- Environment variable fallbacks
- Mutual exclusion constraints
- Negatable flags (`--no-verbose`)
