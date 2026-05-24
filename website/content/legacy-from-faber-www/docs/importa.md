---
title: Importa
section: docs
order: 16
---

# Importa

Faber's module system lets programs organize code across files and incorporate external libraries. The design follows a simple principle: imports should read like Latin sentences. When you write `importa ex "norma:scribe" privata scribe`, you are saying "bring in scribe from the standard library, privately." The syntax mirrors how Latin expresses provenance and acquisition.

The verb `importa` is the imperative of *importare* (to bring in, to carry into). The preposition `ex` means "from" or "out of," indicating origin or source. You are drawing bindings *out of* a module into your local scope.

## Importing

### The importa ex Pattern

Every import begins with `importa ex`, followed by a source path, visibility (`privata` or `publica`), and the symbol to import:

```fab
importa ex "norma:scribe" privata scribe
importa ex "lodash" privata map
importa ex "./utils" privata helper
```

Each import is one line, one symbol. This makes imports grep-friendly and diff-friendly. No comma-separated lists to parse visually.

### Visibility: privata vs publica

Every import requires explicit visibility:

- **privata** — The symbol is available only within this file
- **publica** — The symbol is re-exported from this module

```fab
importa ex "./types" privata User           # internal use only
importa ex "./types" publica Config         # re-exported to importers
```

Re-exports with `publica` let you create barrel modules that aggregate exports from multiple files:

```fab
# index.fab - barrel module
importa ex "./user" publica User
importa ex "./config" publica Config
importa ex "./utils" publica format
```

Importers of this module see `User`, `Config`, and `format` as if they were defined here.

### String Paths

The source in an import is always a quoted string:

```fab
importa ex "norma:hal/solum" privata solum      # standard library
importa ex "@hono/hono" privata Hono            # scoped package
importa ex "./local" privata helper             # relative path
importa ex "../shared/types" privata User       # parent directory
```

External packages from registries like npm use their published names. Scoped packages retain their syntax: `"@scope/package"`. Relative imports use standard path notation: `"./sibling"`, `"../parent/child"`.

The standard library uses the `norma:` prefix: `"norma:hal/solum"`, `"norma:hal/consolum"`.

### Import Aliases

Sometimes you need to rename an import. Perhaps there is a naming conflict, or the original name is unclear in context, or you prefer a shorter form. The `ut` keyword provides aliasing:

```fab
importa ex "utils" privata helper ut h
importa ex "lodash" privata map ut lodashMap
importa ex "database" privata Connection ut DbConn
```

The `ut` preposition means "as" or "like." You are saying "import helper *as* h." The original name appears first, then `ut`, then the local name you want to use.

### Wildcard Imports

When you need everything a module exports as a namespace, use the wildcard form:

```fab
importa ex "lodash" privata * ut _
importa ex "@std/crypto" privata * ut crypto
```

The asterisk `*` means "all exports." The alias is required—wildcard imports must be namespaced. You cannot dump all exports into the local scope without a container. This prevents name collisions and keeps dependencies traceable.

After a wildcard import, access symbols through the alias:

```fab
importa ex "lodash" privata * ut _
fixum doubled = _.map(numbers, clausura x: x * 2)
```

Use wildcards sparingly. Named imports are clearer about dependencies and help tree-shaking in build systems.

## Exporting

Modules expose their symbols through exports. Only exported symbols are visible to importers. Everything else remains internal to the module.

### Inline Exports

The `@ publica` annotation marks a declaration as exported:

```fab
@ publica
fixum VERSION = "1.0.0"

@ publica
functio greet(textus name) -> textus {
    redde scriptum("Salve, §!", name)
}

@ publica
genus User {
    textus nomen
    numerus aetas
}
```

### Re-exports

Use `importa` with `publica` visibility to re-export symbols:

```fab
importa ex "./internal" publica helper          # re-export as-is
importa ex "./types" publica User ut UserType   # re-export with rename
```

### No Default Exports

Faber does not support default exports. Every export has a name. This is a deliberate choice. Named exports are explicit, searchable, and consistent. When you import `Hono` from a module, you know that is exactly what the module calls it.

## Module Organization

### The norma Standard Library

The `norma:` prefix refers to Faber's standard library. Unlike external packages, `norma` modules are compiler-handled. The compiler recognizes `norma:*` paths, validates their exports, and generates appropriate target-language code.

```fab
importa ex "norma:hal/solum" privata solum
importa ex "norma:hal/consolum" privata consolum
importa ex "norma:hal/tempus" privata nunc
importa ex "norma:hal/tempus" privata dormi
```

This design means compiled output has no Faber-specific dependencies. The standard library is "batteries included" at compile time, not runtime.

### External Packages

External packages are imported by their published names:

```fab
importa ex "@hono/hono" privata Hono
importa ex "@hono/hono" privata Context
importa ex "pg" privata Pool
```

The compiler passes external package references through unchanged. This gives you full access to your target ecosystem's libraries while writing Faber syntax.

### Local Imports

Files within your project import from relative paths:

```fab
importa ex "./utils" privata helper
importa ex "./utils" privata formatter
importa ex "../shared/types" privata User
importa ex "../shared/types" privata Config
```

Local imports work like external packages for compilation purposes. The compiler rewrites the path extension (`.fab` to `.ts` or `.py` depending on target) but otherwise passes the import through.

---

The module system is how Faber programs scale beyond single files. The `importa ex` pattern reads naturally and makes dependencies explicit. The one-import-per-line rule keeps diffs clean. Visibility markers (`privata`/`publica`) make re-export intent clear. And throughout, the Latin vocabulary reminds you what these operations actually do: bringing symbols in from sources.

