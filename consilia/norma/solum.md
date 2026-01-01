---
status: planned
note: Stdlib design; not yet implemented
updated: 2024-12
---

# Solum - Local I/O

## Overview

Local filesystem operations for files and directories. The "ground" to `caelum`'s "sky" — operations on the local machine.

Faber provides two tiers:

- **Core statements**: `lege`, `inscribe`, `appone`, `aperi`, `claude` — language keywords for reading/writing
- **Utilities**: `exstat`, `dele`, `duplica`, `move`, etc. — stdlib functions for file/directory management

## Etymology

### Core Statements

- `lege` - "read!" (imperative of legere)
- `inscribe` - "write into!" (imperative of inscribere)
- `appone` - "add to!" (imperative of apponere)
- `aperi` - "open!" (imperative of aperire)
- `claude` - "close!" (imperative of claudere)
- `quaere` - "seek!" (imperative of quaerere)

### Utility Functions

- `solum` - "ground, floor, foundation" — local storage
- `exstat` - "it exists" — check existence
- `dele` - "destroy!" — delete
- `duplica` - "duplicate!" — copy file
- `move` - "move!" — relocate
- `inspice` - "inspect!" — get info
- `trunca` - "cut short!" — truncate
- `tange` - "touch!" — create/update mtime
- `crea` - "create!" — make directory
- `elenca` - "list!" — directory contents
- `ambula` - "walk!" — traverse tree
- `vacua` - "empty!" — remove empty dir

### Format/Encoding

- `ut` - "as" (conjunction) — format specifier
- `octeti` - "octets" (from octo, eight) — raw 8-bit bytes
- `lineae` - "lines" (plural of linea) — text split by newlines

---

## Core I/O Statements

### Reading Files

**Syntax:**

```
lege <path>
lege <path> ut <format>
```

**Examples:**

```fab
# Read as text (default)
fixum readme = lege "README.md"

# Read with format parsing
fixum config = lege "config.json" ut json
fixum settings = lege "settings.toml" ut toml
fixum records = lege "data.csv" ut csv

# Read as raw bytes
fixum raw = lege "image.png" ut octeti

# Read as lines
fixum lines = lege "data.txt" ut lineae
ex lines pro linea {
    scribe linea
}
```

**Supported formats:**

| Format   | Description  | Returns                    |
| -------- | ------------ | -------------------------- |
| (none)   | Raw text     | `textus`                   |
| `json`   | JSON parsing | object/array               |
| `toml`   | TOML parsing | object                     |
| `csv`    | CSV parsing  | `textus[][]` or `object[]` |
| `octeti` | Raw bytes    | `octeti` (Uint8Array)      |
| `lineae` | Lines        | `textus[]`                 |

### Writing Files

**Syntax:**

```
inscribe <path>, <data>
inscribe <path> ut <format>, <data>
```

**Examples:**

```fab
# Write text
inscribe "output.txt", "Hello, world!"

# Write with format serialization
inscribe "config.json" ut json, { version: 1, debug: verum }
inscribe "data.csv" ut csv, rows

# Write lines
inscribe "output.txt" ut lineae, ["first", "second", "third"]
```

### Appending

**Syntax:**

```
appone <path>, <data>
```

**Examples:**

```fab
appone "log.txt", "[INFO] Server started\n"
appone "events.jsonl", json.verte(event) + "\n"
```

---

## Low-Level File API

For systems programming: file descriptors, byte buffers, seeking.

### Opening Files

**Syntax:**

```
aperi <path>
aperi <path>, <options>
```

**Options:**

```fab
{
    modus: "r" | "w" | "rw" | "a",  # read/write/readwrite/append
    crea: bivalens,                  # create if missing
    trunca: bivalens                 # truncate existing
}
```

**Examples:**

```fab
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

```fab
fixum fd = aperi "data.bin"
varia buffer: octeti = nova octeti(1024)
fixum n = lege fd, buffer

# Or read into new buffer
fixum data = lege fd, 1024
```

### Writing to Descriptor

**Syntax:**

```
inscribe <fd>, <buffer>
```

**Returns:** Number of bytes written

**Examples:**

```fab
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

- `incipit` - from start (SEEK_SET)
- `hic` - from current position (SEEK_CUR)
- `finis` - from end (SEEK_END)

**Examples:**

```fab
quaere fd, 0                    # seek to start
quaere fd, 100                  # seek to position 100
quaere fd, -10, finis           # 10 bytes before end
```

### Closing

**Syntax:**

```
claude <fd>
```

### Resource Management

Use `cura` for automatic cleanup:

```fab
cura aperi "data.bin" fit fd {
    fixum data = lege fd, 1024
    process(data)
}
# fd automatically closed

# With error handling
cura aperi "data.bin" fit fd {
    inscribe fd, data
} cape err {
    mone "Write failed:", err
}
# fd still closed even on error
```

---

## Formatter Interface

Formats are types implementing the `formator` interface:

```fab
pactum formator<T> {
    functio verte(T data) -> octeti       # turn data into octets
    functio reverte(octeti input) -> T    # turn octets back into data
}
```

**Etymology:**

- `verte` - "turn!" (imperative of vertere) — turn data into octets
- `reverte` - "turn back!" — turn octets back into data

**Standard formatters:**

```fab
# Encoding formatters
genus Textus implet formator<textus> { }   # UTF-8 encode/decode
genus Octeti implet formator<octeti> { }   # identity (passthrough)
genus Lineae implet formator<textus[]> { } # join/split on \n

# Structured formats
genus JSON implet formator<object> { }
genus TOML implet formator<object> { }
genus CSV implet formator<object[]> { }
```

**Custom formatters:**

```fab
genus MyFormat implet formator<MyData> {
    functio verte(MyData data) -> octeti {
        # serialize to octets
    }

    functio reverte(octeti input) -> MyData {
        # parse from octets
    }
}

fixum formatter = novum MyFormat()
fixum data = lege "data.custom" ut formatter
```

**How `ut` expands:**

```fab
# lege "config.json" ut json
# expands to:
fixum raw = lege "config.json" ut octeti
fixum config = json.reverte(raw)

# inscribe "out.json" ut json, data
# expands to:
fixum serialized = json.verte(data)
inscribe "out.json" ut octeti, serialized
```

---

## File Utilities

These are stdlib functions, imported from `norma/solum`.

### Check Existence

```fab
ex "norma/solum" importa { exstat }

si exstat("config.json") {
    fixum config = lege "config.json" ut json
}
```

Returns `bivalens` — works for files and directories.

### Delete

```fab
ex "norma/solum" importa { dele }

dele("temp.txt")
dele("/tmp/old-cache")
```

Fails if file doesn't exist. For directories, must be empty (use `vacua`).

### Copy

```fab
ex "norma/solum" importa { duplica }

duplica("source.txt", "dest.txt")
duplica("config.json", "/backup/config.json")

# With options
duplica("source.txt", "dest.txt", {
    rescribe: verum    # overwrite if exists
})
```

### Move / Rename

```fab
ex "norma/solum" importa { move }

move("old-name.txt", "new-name.txt")
move("file.txt", "/other/dir/file.txt")
move("src/old-module", "src/new-module")
```

### File Info

```fab
ex "norma/solum" importa { inspice }

fixum info = inspice("data.bin")

scribe info.magnitudo          # size in bytes
scribe info.creatum            # creation time
scribe info.mutatum            # modification time
scribe info.accessum           # access time
scribe info.genus              # "file" | "directory" | "symlink"
scribe info.modus              # permissions (octal)
```

### Truncate

```fab
ex "norma/solum" importa { trunca }

trunca("log.txt", 0)           # empty the file
trunca("data.bin", 1024)       # truncate to 1KB
```

### Touch

```fab
ex "norma/solum" importa { tange }

tange("file.txt")              # create if missing, update mtime if exists
tange("marker.lock")           # create empty marker file
```

---

## Directory Operations

### Create Directory

```fab
ex "norma/solum" importa { crea }

crea("new-folder")
crea("/tmp/deeply/nested/path")    # creates parents automatically

# With options
crea("secure-dir", { modus: 0o700 })
```

### List Directory

```fab
ex "norma/solum" importa { elenca }

fixum entries = elenca(".")
# ["file1.txt", "file2.txt", "subdir"]

ex elenca("/home/user") pro entry {
    scribe entry
}

# With options
fixum files = elenca(".", {
    occulta: verum     # include hidden files
})
```

### Walk Directory Tree

```fab
ex "norma/solum" importa { ambula }

# Iterate all files recursively
ex ambula("src") pro path {
    scribe path
}
# "src/main.fab"
# "src/lib/utils.fab"

# With filter
ex ambula("src", { glob: "*.fab" }) pro path {
    compile(path)
}

# With options
ex ambula(".", {
    glob: "*.md",
    sequere_symbola: falsum,    # don't follow symlinks
    max_altitudo: 3              # max depth
}) pro path {
    process(path)
}
```

### Remove Empty Directory

```fab
ex "norma/solum" importa { vacua }

vacua("empty-dir")             # fails if not empty

# Remove directory and contents
ex "norma/solum" importa { dele_arbor }
dele_arbor("dir-with-contents")  # recursive delete
```

---

## Path Utilities

```fab
ex "norma/solum" importa { via }

# Join paths
fixum path = via.iunge("src", "lib", "utils.fab")
# "src/lib/utils.fab"

# Parse path
fixum p = via.parse("/home/user/file.txt")
scribe p.radix          # "/"
scribe p.dir            # "/home/user"
scribe p.basis          # "file.txt"
scribe p.nomen          # "file"
scribe p.extensio       # ".txt"

# Resolve relative paths
fixum abs = via.resolve("../sibling/file.txt")

# Normalize
fixum clean = via.norma("src/../lib/./utils.fab")
# "lib/utils.fab"

# Home directory
fixum home = via.domus()           # "/home/user"
fixum config = via.iunge(via.domus(), ".config", "app")
```

**Home directory expansion:**

`~` expands to user home directory in paths:

```fab
fixum config = lege "~/.config/app.json" ut json
```

---

## Symbolic Links

```fab
ex "norma/solum" importa { necte, lege_nexum }

# Create symlink
necte("actual-file.txt", "link.txt")

# Read symlink target
fixum target = lege_nexum("link.txt")
# "actual-file.txt"

# Check if symlink
fixum info = inspice("link.txt")
si info.genus == "symlink" {
    scribe "It's a link to:", lege_nexum("link.txt")
}
```

---

## Permissions

```fab
ex "norma/solum" importa { modus }

# Get permissions
fixum m = modus("script.sh")      # 0o755

# Set permissions
modus("script.sh", 0o755)         # rwxr-xr-x
modus("secret.key", 0o600)        # rw-------
```

---

## Temporary Files

```fab
ex "norma/solum" importa { temporarium }

# Create temp file
cura temporarium() fit tmp {
    inscribe tmp.path, "temporary data"
    fixum data = lege tmp.path
}
# tmp file automatically deleted

# With prefix
cura temporarium({ praefixum: "cache-" }) fit tmp {
    # tmp.path = "/tmp/cache-abc123"
}

# Temp directory
cura temporarium({ genus: "dir" }) fit tmp_dir {
    crea(via.iunge(tmp_dir.path, "subdir"))
}
```

---

## Async Considerations

### High-Level

High-level operations are async by default on async-capable targets:

```fab
# Using explicit cede
futura functio loadConfig() -> object {
    fixum config = cede lege "config.json" ut json
    redde config
}

# Using figendum (implicit await)
futura functio loadConfig() -> object {
    figendum config = lege "config.json" ut json
    redde config
}
```

### Low-Level

Low-level operations are sync by default:

```fab
fixum fd = aperi "file.bin"    # sync open
fixum data = lege fd, 1024     # sync read
claude fd                       # sync close
```

---

## Target Mappings

### TypeScript (Node.js)

```typescript
import * as fs from 'fs/promises';
import * as path from 'path';

// lege "config.json" ut json
JSON.parse(await fs.promises.readFile('config.json', 'utf-8'));

// inscribe "out.json" ut json, data
await fs.promises.writeFile('out.json', JSON.stringify(data));

// aperi "file.bin"
await fs.promises.open('file.bin', 'r');

// exstat(path)
await fs
    .access(path)
    .then(() => true)
    .catch(() => false);

// dele(path)
await fs.unlink(path);

// duplica(src, dest)
await fs.copyFile(src, dest);

// move(src, dest)
await fs.rename(src, dest);

// inspice(path)
await fs.stat(path);

// crea(path)
await fs.mkdir(path, { recursive: true });

// elenca(path)
await fs.readdir(path);
```

### Python

```python
import os
import shutil
import json
from pathlib import Path

# lege "config.json" ut json
with open("config.json") as f:
    json.load(f)

# inscribe "out.json" ut json, data
with open("out.json", "w") as f:
    json.dump(data, f)

# exstat(path)
os.path.exists(path)

# dele(path)
os.remove(path)

# duplica(src, dest)
shutil.copy2(src, dest)

# move(src, dest)
shutil.move(src, dest)

# inspice(path)
os.stat(path)

# crea(path)
os.makedirs(path, exist_ok=True)

# elenca(path)
os.listdir(path)

# ambula(path)
for root, dirs, files in os.walk(path):
    ...
```

### Rust

```rust
use std::fs;
use std::path::Path;

// exstat(path)
Path::new(path).exists()

// dele(path)
fs::remove_file(path)?

// duplica(src, dest)
fs::copy(src, dest)?

// move(src, dest)
fs::rename(src, dest)?

// inspice(path)
fs::metadata(path)?

// crea(path)
fs::create_dir_all(path)?

// elenca(path)
fs::read_dir(path)?

// ambula(path)
walkdir::WalkDir::new(path)
```

### Zig

```zig
const std = @import("std");
const fs = std.fs;

// lege "file.txt"
const file = try std.fs.cwd().openFile("file.txt", .{});
defer file.close();
const content = try file.readToEndAlloc(allocator, max_size);

// inscribe "file.txt", data
const file = try std.fs.cwd().createFile("file.txt", .{});
defer file.close();
try file.writeAll(data);

// exstat(path)
fs.cwd().access(path, .{}) catch |err| false;

// dele(path)
try fs.cwd().deleteFile(path);

// duplica(src, dest)
try fs.cwd().copyFile(src, dest, .{});

// move(src, dest)
try fs.cwd().rename(src, dest);

// inspice(path)
try fs.cwd().statFile(path);

// crea(path)
try fs.cwd().makePath(path);

// elenca(path)
var dir = try fs.cwd().openDir(path, .{ .iterate = true });
defer dir.close();
```

---

## Implementation Status

### Core Statements

| Feature              | Status   | Notes                 |
| -------------------- | -------- | --------------------- |
| `lege` (text)        | Not Done | High-level read       |
| `lege` ut format     | Not Done | With parsing          |
| `inscribe`           | Not Done | High-level write      |
| `appone`             | Not Done | Append                |
| `aperi`/`claude`     | Not Done | Low-level open/close  |
| Low-level `lege`     | Not Done | Read to buffer        |
| Low-level `inscribe` | Not Done | Write buffer          |
| `quaere`             | Not Done | Seek                  |
| `~` expansion        | Not Done | Home directory        |
| `formator` interface | Not Done | Custom format parsing |

### Utility Functions

| Feature         | Status   | Notes            |
| --------------- | -------- | ---------------- |
| `exstat`        | Not Done | Check existence  |
| `dele`          | Not Done | Delete file      |
| `duplica`       | Not Done | Copy file        |
| `move`          | Not Done | Move/rename      |
| `inspice`       | Not Done | File info        |
| `trunca`        | Not Done | Truncate         |
| `tange`         | Not Done | Touch            |
| `crea`          | Not Done | Create directory |
| `elenca`        | Not Done | List directory   |
| `ambula`        | Not Done | Walk tree        |
| `vacua`         | Not Done | Remove empty dir |
| `dele_arbor`    | Not Done | Recursive delete |
| `via` utilities | Not Done | Path operations  |
| `necte`         | Not Done | Symlinks         |
| `modus`         | Not Done | Permissions      |
| `temporarium`   | Not Done | Temp files       |

### Target Codegen

| Target     | Status   |
| ---------- | -------- |
| TypeScript | Not Done |
| Python     | Not Done |
| Rust       | Not Done |
| Zig        | Not Done |
| C++        | Not Done |

---

## Design Decisions

### Why keywords for core I/O?

Core file I/O (read/write/open/close) is fundamental. Making these keyword-based:

1. Keeps the most common operations concise
2. Enables target-specific optimization
3. Feels natural in Latin syntax

### Why stdlib for utilities?

Secondary operations (exists, delete, copy, move, etc.) are less frequent and benefit from explicit imports that signal intent.

### Why `ut` for format?

- Already exists as "as" preposition
- Natural reading: "read file.json as json"
- Consistent with destructuring renames
- Formatters are first-class via `pactum formator<T>`

### Why `via` for paths?

`via` means "way, path" — perfect for filesystem paths. Short and distinct from file operations.

### Why `ambula` returns iterator?

Walking large directory trees shouldn't load all paths into memory. An iterator (cursor) allows lazy traversal and early exit.

### Why `temporarium` uses `cura`?

Temp files must be cleaned up. Using `cura` ensures automatic deletion on scope exit, preventing temp file leaks.

### Permissions model?

Unix-style octal permissions (0o755) are the common denominator. Windows permission mapping is target-specific and may lose fidelity.

---

## Future Considerations

1. **Streaming**: Read large files in chunks

    ```fab
    ex ausculta lege "large.csv" fiet chunk {
        processChunk(chunk)
    }
    ```

2. **Glob patterns**: Read multiple files

    ```fab
    fixum files = lege "src/**/*.fab"
    ```

3. **Watching**: File change notifications

    ```fab
    ex vigila "config.json" fiet change {
        reloadConfig()
    }
    ```
