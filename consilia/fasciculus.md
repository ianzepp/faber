# Fasciculus - File I/O Statements

## Overview

This document covers **core file I/O statements** — language-level keywords for reading and writing files.

Faber provides two tiers:
- **High-level**: `lege`, `inscribe`, `appone` — convenience operations with optional format parsing
- **Low-level**: `aperi`, `claude`, `quaere` — file descriptors for systems programming

For file/directory utilities (copy, delete, etc.), see `solum.md`.

## Etymology

- `fasciculus` - "small bundle, packet" (diminutive of fascis) - a file as a bundle of data
- `lege` - "read!" (imperative of legere)
- `inscribe` - "write into!" (imperative of inscribere)
- `appone` - "add to!" (imperative of apponere)
- `aperi` - "open!" (imperative of aperire)
- `claude` - "close!" (imperative of claudere)
- `ut` - "as" (conjunction) - format specifier
- `octeti` - "octets" (from octo, eight) - raw 8-bit bytes, distinct from textus
- `lineae` - "lines" (plural of linea) - text split by newlines

---

## High-Level API

### Reading Files

**Syntax:**
```
lege <path>
lege <path> ut <format>
```

**Examples:**
```
// Read as text (default)
fixum readme = lege "README.md"

// Read with format parsing
fixum config = lege "config.json" ut json
fixum settings = lege "settings.toml" ut toml
fixum records = lege "data.csv" ut csv

// Destructure after reading
fixum config = lege "~/.config/app.json" ut json
ex config fixum { routes, parsers, database }
```

**Supported formats:**
| Format | Description | Returns |
|--------|-------------|---------|
| (none) | Raw text | `textus` |
| `json` | JSON parsing | object/array |
| `toml` | TOML parsing | object |
| `csv` | CSV parsing | `lista<lista<textus>>` or `lista<object>` |
| `octeti` | Raw bytes | `octeti` (Uint8Array / []u8) |

### Formatter Interface

Formats are types implementing the `formator` interface. All formatters work on octets (raw bytes) - text is just UTF-8 encoded octets.

```
pactum formator<T> {
    functio verte(T data) -> octeti       // turn data into octets
    functio reverte(octeti input) -> T    // turn octets back into data
}
```

**Etymology:**
- `verte` - "turn!" (imperative of vertere) - turn data into octets
- `reverte` - "turn back!" - turn octets back into data
- `octeti` - "octets" (8-bit bytes) - raw binary data without encoding assumptions

**Standard formatters:**

```
// Encoding formatters (primitive conversions)
genus Textus implet formator<textus> {
    functio verte(textus data) -> octeti     // UTF-8 encode
    functio reverte(octeti input) -> textus  // UTF-8 decode
}

genus Octeti implet formator<octeti> {
    functio verte(octeti data) -> octeti     // identity (passthrough)
    functio reverte(octeti input) -> octeti  // identity (passthrough)
}

genus Lineae implet formator<lista<textus>> {
    functio verte(lista<textus> lines) -> octeti   // join with \n, encode
    functio reverte(octeti input) -> lista<textus> // decode, split on \n
}

// Structured text formats
genus JSON implet formator<object> { }
genus TOML implet formator<object> { }
genus CSV implet formator<lista<object>> { }

// Binary formats
genus MessagePack implet formator<object> { }
genus PNG implet formator<Image> { }
genus ZIP implet formator<Archivum> { }
```

**Usage:**
```
// Read file as raw bytes
fixum raw = lege "image.png" ut octeti

// Read bytes, decode as UTF-8 text
fixum text = lege "readme.txt" ut textus

// Read file as lines
fixum lines = lege "data.txt" ut lineae
pro linea in lines {
    scribe linea
}

// Write lines to file
inscribe "output.txt" ut lineae, ["first", "second", "third"]

// Encode text to bytes for binary protocol
fixum encoded = textus.verte("hello")  // -> octeti

// Decode bytes from network
fixum decoded = textus.reverte(buffer)  // -> textus
```

**How `ut` expands:**

```
// lege "config.json" ut json
// expands to:
fixum raw = lege "config.json" ut octeti
fixum config = json.reverte(raw)

// inscribe "out.json" ut json, data
// expands to:
fixum serialized = json.verte(data)
inscribe "out.json" ut octeti, serialized
```

**Custom formatters:**

```
genus MyFormat implet formator<MyData> {
    functio verte(MyData data) -> octeti {
        // serialize to octets
    }

    functio reverte(octeti input) -> MyData {
        // parse from octets
    }
}

fixum formatter = novum MyFormat()
fixum data = lege "data.custom" ut formatter
```

### Writing Files

**Syntax:**
```
inscribe <path>, <data>
inscribe <path> ut <format>, <data>
```

**Examples:**
```
// Write text
inscribe "output.txt", "Hello, world!"

// Write with format serialization
inscribe "config.json" ut json, { version: 1, debug: verum }
inscribe "data.csv" ut csv, rows

// Variable content
fixum report = generateReport()
inscribe "report.md", report
```

### Appending

**Syntax:**
```
appone <path>, <data>
```

**Examples:**
```
appone "log.txt", "[INFO] Server started\n"
appone "events.jsonl", JSON.stringify(event) + "\n"
```

---

## Low-Level API

For systems programming: file descriptors, byte buffers, seeking.

### Opening Files

**Syntax:**
```
aperi <path>
aperi <path>, <options>
```

**Options:**
```
{
    modus: "r" | "w" | "rw" | "a",  // read/write/readwrite/append
    crea: bivalens,                  // create if missing
    trunca: bivalens                 // truncate existing
}
```

**Examples:**
```
fixum fd = aperi "data.bin"
fixum fd = aperi "output.bin", { modus: "w", crea: verum }
```

### Reading from Descriptor

**Syntax:**
```
lege <fd>, <buffer>
lege <fd>, <count>
```

**Returns:** Number of bytes read

**Examples:**
```
fixum fd = aperi "data.bin"
faber buffer: octeti = nova octeti(1024)
fixum n = lege fd, buffer

// Or read into new buffer
fixum data = lege fd, 1024
```

### Writing to Descriptor

**Syntax:**
```
inscribe <fd>, <buffer>
```

**Returns:** Number of bytes written

**Examples:**
```
fixum fd = aperi "output.bin", { modus: "w" }
inscribe fd, buffer
claude fd
```

### Seeking

**Syntax:**
```
quaere <fd>, <position>
quaere <fd>, <offset>, <origin>
```

**Origins:**
- `initium` - from start (SEEK_SET)
- `hic` - from current position (SEEK_CUR)
- `finis` - from end (SEEK_END)

**Examples:**
```
quaere fd, 0                    // seek to start
quaere fd, 100                  // seek to position 100
quaere fd, -10, finis           // 10 bytes before end
```

### Closing

**Syntax:**
```
claude <fd>
```

**Examples:**
```
fixum fd = aperi "file.bin"
// ... operations ...
claude fd
```

### Resource Management

Use `cura` (care for) for automatic cleanup:

```
cura aperi "data.bin" fit fd {
    fixum data = lege fd, 1024
    process(data)
}
// fd automatically closed
```

**With error handling:**
```
cura aperi "data.bin" fit fd {
    inscribe fd, data
} cape err {
    mone "Write failed:", err
}
// fd still closed even on error
```

See `consilia/cura.md` for the full resource management design.

---

## Async Considerations

### High-Level

High-level operations are async by default on async-capable targets:

```
// Using explicit cede
futura functio loadConfig() -> object {
    fixum config = cede lege "config.json" ut json
    redde config
}

// Using figendum (implicit await, more natural Latin)
futura functio loadConfig() -> object {
    figendum config = lege "config.json" ut json
    redde config
}
```

Both forms are valid. The `figendum` gerundive ("that which will be fixed") signals async intent without explicit `cede`. See `vincula.md` for details.

### Low-Level

Low-level operations are sync by default:

```
fixum fd = aperi "file.bin"    // sync open
fixum data = lege fd, 1024     // sync read
claude fd                       // sync close
```

Async low-level via explicit wrapping (future consideration).

---

## Target Mappings

### TypeScript (Node.js)

```typescript
// lege "config.json" ut json
JSON.parse(await fs.promises.readFile("config.json", "utf-8"))

// inscribe "out.json" ut json, data
await fs.promises.writeFile("out.json", JSON.stringify(data))

// aperi "file.bin"
await fs.promises.open("file.bin", "r")
```

### Python

```python
# lege "config.json" ut json
import json
with open("config.json") as f:
    json.load(f)

# inscribe "out.json" ut json, data
with open("out.json", "w") as f:
    json.dump(data, f)
```

### Zig

```zig
// lege "file.txt"
const file = try std.fs.cwd().openFile("file.txt", .{});
defer file.close();
const content = try file.readToEndAlloc(allocator, max_size);

// inscribe "file.txt", data
const file = try std.fs.cwd().createFile("file.txt", .{});
defer file.close();
try file.writeAll(data);
```

---

## Path Handling

### Home Directory

`~` expands to user home directory:

```
fixum config = lege "~/.config/app.json" ut json
```

### Path Joining

Use `/` operator or stdlib:

```
fixum path = baseDir + "/" + filename
// or
fixum path = Via.iunge(baseDir, filename)
```

### Path Type

Future consideration: `via` (path/way) as a dedicated path type:

```
fixum configPath: via = "~/.config/app.json"
fixum config = lege configPath ut json
```

---

## Related Documentation

This document covers core file I/O **statements** (`lege`, `inscribe`, `aperi`, `claude`).

For other functionality, see:

| Doc | Module | Content |
|-----|--------|---------|
| `cura.md` | — | Resource management (`cura` keyword) |
| `solum.md` | `norma/solum` | File/dir utilities (exstat, dele, copia, etc.) |
| `caelum.md` | `norma/caelum` | Network I/O (HTTP, sockets) |
| `tempus.md` | `norma/tempus` | Time operations (nunc, dormi) |
| `crypto.md` | `norma/crypto` | Cryptography (digere, cifra, fortuita) |
| `comprimo.md` | `norma/comprimo` | Compression (comprimo, laxo) |
| `codex.md` | `norma/codex` | Encoding (coda, decoda) |

---

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `lege` (text) | Not Done | High-level read |
| `lege` ut format | Not Done | With parsing |
| `inscribe` | Not Done | High-level write |
| `appone` | Not Done | Append |
| `aperi`/`claude` | Not Done | Low-level open/close |
| Low-level `lege` | Not Done | Read to buffer |
| Low-level `inscribe` | Not Done | Write buffer |
| `quaere` | Not Done | Seek |
| `~` expansion | Not Done | Home directory |
| `formator` interface | Not Done | Custom format parsing |

---

## Design Decisions

### Why keywords over stdlib?

Core file I/O (read/write/open/close) is fundamental. Making these keyword-based:
1. Keeps the most common operations concise
2. Enables target-specific optimization
3. Feels natural in Latin syntax

Secondary operations (exists, delete, copy, move, etc.) go to stdlib — they're less frequent and benefit from explicit imports that signal intent.

### Why `ut` for format?

- Already exists as "as" preposition
- Natural reading: "read file.json as json"
- Consistent with destructuring renames
- Formatters are first-class: `pactum formator<T>` with `verte`/`reverte`
- Users can define custom formatters, not limited to builtins

### Why separate high/low level?

- High-level for 90% of use cases (scripts, apps)
- Low-level for systems work (OS, VM, binary formats)
- Different users, different needs

### Sync vs Async?

- High-level: Target decides (async for Node, sync for Zig)
- Low-level: Sync by default (explicit control)
- User can force async with `cede` where needed

---

## Future Considerations

1. **Streaming**: Read large files in chunks via `ausculta`?
   ```
   ex ausculta lege "large.csv" fiet chunk {
       processChunk(chunk)
   }
   ```

2. **Glob patterns**: Read multiple files
   ```
   fixum files = lege "src/**/*.fab"
   ```

3. **Watching**: File change notifications
   ```
   ex vigila "config.json" fiet change {
       reloadConfig()
   }
   ```
