# Importa

Faber's module system lets programs organize code across files and incorporate external libraries. The design follows a simple principle: imports should read like Latin sentences. When you write `ex norma importa scribe`, you are saying "from the standard library, bring in scribe." The syntax mirrors how Latin expresses provenance and acquisition.

The verb `importa` is the imperative of *importare* (to bring in, to carry into). Its counterpart `exporta` comes from *exportare* (to carry out). These are not arbitrary keyword choices. They describe exactly what the operations do: bringing symbols into a scope, or carrying them out for others to use.

## Importing

### The ex...importa Pattern

Every import begins with `ex` followed by a source, then `importa` followed by the names you want:

```fab
ex norma importa scribe, lege
ex "lodash" importa map, filter
ex "./utils" importa helper
```

Imports may also use the sectional prefix `§`:

```fab
§ ex norma importa scribe, lege
§ ex "lodash" importa map, filter
```

The sectional form groups imports with other file-level directives like `§ dependentia` and `§ externa`. Both forms produce identical behavior; the `§` prefix is purely syntactic. Use whichever reads better in context.

This pattern reads naturally in Latin. The preposition `ex` means "from" or "out of," indicating origin or source. You are drawing bindings *out of* a module into your local scope.

The structure is consistent regardless of source type. Whether importing from the standard library, an external package, or a local file, the syntax remains the same. Only the source specifier changes.

### Why "ex"?

Latin prepositions carry semantic weight that English keywords lose through familiarity. When you see `ex items pro item` in a loop, the `ex` tells you where the data flows from. When you see `ex norma importa scribe`, the `ex` tells you where the symbol originates.

The preposition `ex` appears throughout Faber whenever something is drawn from a source:

- Imports draw symbols from modules: `ex module importa name`
- Iteration draws elements from collections: `ex items pro item { ... }`
- Destructuring draws fields from objects: `ex response fixum status, data`

This consistency is deliberate. By using positional grammar rather than distinct keywords for each context, Faber mirrors how Latin works. The preposition's meaning derives from its position in the sentence, not from memorizing what each keyword does in isolation.

### String Paths vs. Bare Identifiers

The source in an import can be either a quoted string or a bare identifier:

```fab
ex norma importa scribe           # bare identifier
ex "norma/tempus" importa dormi   # string path
ex "@hono/hono" importa Hono      # string with special characters
ex "./local" importa helper       # relative path
```

Bare identifiers work for simple module names that are valid identifiers. Use quoted strings when the path contains slashes, special characters, or needs to be a relative path. The `norma` standard library can be imported either way, but its submodules require string paths: `"norma/tempus"`, `"norma/crypto"`.

External packages from registries like npm use their published names as strings. Scoped packages retain their syntax: `"@scope/package"`. Relative imports use standard path notation: `"./sibling"`, `"../parent/child"`.

### Named Imports

Most imports specify exactly which symbols to bring in:

```fab
ex "hono" importa Hono, Context
ex "lodash" importa map, filter, reduce
ex norma importa scribe, lege, mone
```

This is explicit and intentional. Named imports make dependencies visible. A reader can see at a glance what a file uses from each module without hunting through the code.

Multiple symbols are comma-separated. There is no limit to how many you can import in a single statement, but consider readability. If you need a dozen symbols from one module, that module may be doing too much, or your file may be doing too much.

### Import Aliases

Sometimes you need to rename an import. Perhaps there is a naming conflict, or the original name is unclear in context, or you prefer a shorter form for frequently used symbols. The `ut` keyword provides aliasing:

```fab
ex "utils" importa helper ut h
ex "db" importa connect, query ut q, close
ex "lodash" importa map ut lodashMap
```

The `ut` preposition means "as" or "like." You are saying "import `helper` *as* `h`." The original name appears first, then `ut`, then the local name you want to use. This mirrors Latin's use of `ut` for comparison and equivalence.

Aliases work with any import, whether you are renaming one symbol or several:

```fab
ex "mod" importa foo ut f, bar ut b, baz ut z
```

Each renaming is independent. You can alias some imports while leaving others with their original names.

### Wildcard Imports

When you need everything a module exports, use the wildcard form with an alias:

```fab
ex "@std/crypto" importa * ut crypto
ex "lodash" importa * ut _
```

The asterisk `*` means "all exports." The alias is required because wildcard imports must be namespaced. You cannot dump all exports into the local scope without a container. This prevents name collisions and keeps dependencies traceable.

After a wildcard import, access symbols through the alias:

```fab
ex "lodash" importa * ut _
fixum doubled = _.map(numbers, pro x: x * 2)
```

Use wildcards sparingly. Named imports are clearer about dependencies and help tree-shaking in build systems. But wildcards have their place when you genuinely need most of what a module provides.

## Exporting

Modules expose their symbols through exports. Only exported symbols are visible to importers. Everything else remains internal to the module.

### Named Exports

To export existing declarations, list them after `exporta`:

```fab
fixum VERSION = "1.0.0"
functio greet(textus name) -> textus {
    redde scriptum("Salve, §!", name)
}

exporta VERSION, greet
```

The `exporta` statement names what leaves the module. This can appear anywhere in the file, but placing exports at the end (after definitions) or at the beginning (as a manifest) aids readability.

### Inline Exports

You can combine export with declaration for a more compact form:

```fab
exporta fixum VERSION = "1.0.0"

exporta functio greet(textus name) -> textus {
    redde scriptum("Salve, §!", name)
}

exporta genus User {
    textus nomen
    numerus aetas
}
```

When `exporta` precedes a declaration, that declaration is both defined and exported in one statement. This is convenient for modules where most definitions are public.

Choose between named exports and inline exports based on what makes your code clearer. A module with many internal helpers might prefer explicit `exporta` at the end. A module that is primarily a public API might prefer inline exports throughout.

### No Default Exports

Faber does not support default exports. Every export has a name. This is a deliberate choice. Named exports are explicit, searchable, and consistent. When you import `Hono` from a module, you know that is exactly what the module calls it. There is no ambiguity about whether you are importing the default or a named export.

If you are porting code from TypeScript or JavaScript that uses default exports, convert them to named exports with meaningful names.

## Dynamic Imports

### The importabit Verb

Static imports happen at module load time. Sometimes you need to import a module later, perhaps conditionally or to reduce initial load time. Faber provides dynamic imports using the future tense: `importabit`.

```fab
ex "./heavy" importabit modulus
scribe modulus.process(data)
```

The verb `importabit` is the future active indicative of *importare*: "it will bring in." This naming follows Faber's convention that future tense indicates asynchronous operations. Just as `fiet` (will become) signals an async result and `incipiet` (will begin) signals an async entry point, `importabit` signals that the import happens asynchronously.

Dynamic imports return a promise that resolves to the module. Use them within async contexts:

```fab
incipiet {
    ex pathVariable importabit modulus
    scribe modulus.result
}
```

The source can be an expression, not just a literal string. This enables computed imports based on configuration or runtime conditions:

```fab
varia textus modulePath = determineModule()
ex modulePath importabit loaded
```

Use dynamic imports judiciously. Static imports are analyzed at compile time, enabling better error checking and optimization. Dynamic imports defer this to runtime, trading analysis for flexibility.

## Module Organization

### The norma Standard Library

The identifier `norma` refers to Faber's standard library. Unlike external packages, `norma` modules are compiler-handled. The compiler recognizes `norma/*` paths, validates their exports, and generates appropriate target-language code without emitting import statements.

```fab
ex norma importa scribe, lege
ex "norma/tempus" importa nunc, dormi, SECUNDUM
```

This design means compiled output has no Faber-specific dependencies. The standard library is "batteries included" at compile time, not runtime. When you import `dormi` from `norma/tempus`, the compiler emits the appropriate sleep implementation for your target language directly.

### External Packages

External packages are imported by their published names:

```fab
ex "@hono/hono" importa Hono, Context
ex "pg" importa Pool
```

The compiler passes external package references through unchanged. This gives you full access to your target ecosystem's libraries while writing Faber syntax.

### Local Imports

Files within your project import from relative paths:

```fab
ex "./utils" importa helper, formatter
ex "../shared/types" importa User, Config
```

Local imports work like external packages for compilation purposes. The compiler rewrites the path extension (`.fab` to `.ts` or `.py` depending on target) but otherwise passes the import through.

For a project organized across multiple files, local imports let you compose modules while keeping each file focused. Export what others need; keep internals private by not exporting them.

---

The module system is how Faber programs scale beyond single files. The `ex...importa` pattern, once familiar, reads naturally and makes dependencies explicit. Exports mark boundaries. Dynamic imports handle runtime flexibility. And throughout, the Latin vocabulary reminds you what these operations actually do: bringing symbols in, carrying them out, drawing from sources.
