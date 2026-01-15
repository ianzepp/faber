# CLI Framework

Faber provides a declarative CLI framework through annotations. The compiler generates argument parsing, help text, and command dispatch from function signatures and metadata annotations.

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

## Commands

### Single Commands: @ imperium

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

The current implementation supports **subcommand-based CLIs only**. Programs must have at least one `@ imperium` command. Single-command executables (like `echo` or `cat`) are not yet supported.

Not yet implemented:
- Default values for parameters (`vel`)
- Rest/variadic parameters (`ceteri`)
- Parameter help text and metadata (`@ arg`)
- Environment variable fallbacks
- Mutual exclusion constraints
