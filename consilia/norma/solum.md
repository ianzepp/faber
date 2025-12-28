---
status: planned
note: Stdlib design; not yet implemented
updated: 2024-12
---

# Solum - Local I/O

## Overview

Local filesystem operations for files and directories. The "ground" to `caelum`'s "sky" — operations on the local machine.

Note: Core read/write operations (`lege`, `inscribe`, `appone`, `aperi`, `claude`) are language statements. This module provides supplementary file/directory utilities.

## Etymology

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

---

## File Operations

### Check Existence

```
ex "norma/solum" importa { exstat }

si exstat("config.json") {
    fixum config = lege "config.json" ut json
}

si exstat("/tmp/cache") {
    scribe "Cache exists"
}
```

Returns `bivalens` — works for files and directories.

### Delete

```
ex "norma/solum" importa { dele }

dele("temp.txt")
dele("/tmp/old-cache")

// With existence check
si exstat("temp.txt") {
    dele("temp.txt")
}
```

Fails if file doesn't exist. For directories, must be empty (use `vacua`).

### Copy

```
ex "norma/solum" importa { duplica }

duplica("source.txt", "dest.txt")
duplica("config.json", "/backup/config.json")

// With options
duplica("source.txt", "dest.txt", {
    rescribe: verum    // overwrite if exists
})
```

**Etymology:** `duplica` — "duplicate, double" (imperative of duplicare)

### Move / Rename

```
ex "norma/solum" importa { move }

move("old-name.txt", "new-name.txt")
move("file.txt", "/other/dir/file.txt")

// Rename directory
move("src/old-module", "src/new-module")
```

### File Info

```
ex "norma/solum" importa { inspice }

fixum info = inspice("data.bin")

scribe info.magnitudo          // size in bytes
scribe info.creatum            // creation time
scribe info.mutatum            // modification time
scribe info.accessum           // access time
scribe info.genus              // "file" | "directory" | "symlink"
scribe info.modus              // permissions (octal)
```

### Truncate

```
ex "norma/solum" importa { trunca }

trunca("log.txt", 0)           // empty the file
trunca("data.bin", 1024)       // truncate to 1KB
```

### Touch

```
ex "norma/solum" importa { tange }

tange("file.txt")              // create if missing, update mtime if exists
tange("marker.lock")           // create empty marker file
```

---

## Directory Operations

### Create Directory

```
ex "norma/solum" importa { crea }

crea("new-folder")
crea("/tmp/deeply/nested/path")    // creates parents automatically

// With options
crea("secure-dir", { modus: 0o700 })
```

### List Directory

```
ex "norma/solum" importa { elenca }

fixum entries = elenca(".")
// ["file1.txt", "file2.txt", "subdir"]

pro entry in elenca("/home/user") {
    scribe entry
}

// With options
fixum files = elenca(".", {
    occulta: verum     // include hidden files
})
```

### Walk Directory Tree

```
ex "norma/solum" importa { ambula }

// Iterate all files recursively
pro path in ambula("src") {
    scribe path
}
// "src/main.fab"
// "src/lib/utils.fab"
// "src/lib/types.fab"

// With filter
pro path in ambula("src", { glob: "*.fab" }) {
    compile(path)
}

// With options
pro path in ambula(".", {
    glob: "*.md",
    sequere_symbola: falsum,    // don't follow symlinks
    max_altitudo: 3              // max depth
})
```

### Remove Empty Directory

```
ex "norma/solum" importa { vacua }

vacua("empty-dir")             // fails if not empty

// Remove directory and contents
ex "norma/solum" importa { dele_arbor }
dele_arbor("dir-with-contents")  // recursive delete
```

---

## Path Utilities

```
ex "norma/solum" importa { via }

// Join paths
fixum path = via.iunge("src", "lib", "utils.fab")
// "src/lib/utils.fab"

// Parse path
fixum p = via.parse("/home/user/file.txt")
scribe p.radix          // "/"
scribe p.dir            // "/home/user"
scribe p.basis          // "file.txt"
scribe p.nomen          // "file"
scribe p.extensio       // ".txt"

// Resolve relative paths
fixum abs = via.resolve("../sibling/file.txt")

// Normalize
fixum clean = via.norma("src/../lib/./utils.fab")
// "lib/utils.fab"

// Home directory
fixum home = via.domus()           // "/home/user"
fixum config = via.iunge(via.domus(), ".config", "app")
```

---

## Symbolic Links

```
ex "norma/solum" importa { necte, lege_nexum }

// Create symlink
necte("actual-file.txt", "link.txt")

// Read symlink target
fixum target = lege_nexum("link.txt")
// "actual-file.txt"

// Check if symlink
fixum info = inspice("link.txt")
si info.genus == "symlink" {
    scribe "It's a link to:", lege_nexum("link.txt")
}
```

---

## Permissions

```
ex "norma/solum" importa { modus }

// Get permissions
fixum m = modus("script.sh")      // 0o755

// Set permissions
modus("script.sh", 0o755)         // rwxr-xr-x
modus("secret.key", 0o600)        // rw-------
```

---

## Temporary Files

```
ex "norma/solum" importa { temporarium }

// Create temp file
cura temporarium() fit tmp {
    inscribe tmp.path, "temporary data"
    fixum data = lege tmp.path
}
// tmp file automatically deleted

// With prefix
cura temporarium({ praefixum: "cache-" }) fit tmp {
    // tmp.path = "/tmp/cache-abc123"
}

// Temp directory
cura temporarium({ genus: "dir" }) fit tmp_dir {
    crea(via.iunge(tmp_dir.path, "subdir"))
}
```

---

## Target Mappings

### TypeScript (Node.js)

```typescript
import * as fs from 'fs/promises';
import * as path from 'path';

// exstat(path)
await fs
    .access(path)
    .then(() => true)
    .catch(() => false);

// dele(path)
await fs.unlink(path);

// copia(src, dest)
await fs.copyFile(src, dest);

// move(src, dest)
await fs.rename(src, dest);

// inspice(path)
await fs.stat(path);

// crea(path)
await fs.mkdir(path, { recursive: true });

// elenca(path)
await fs.readdir(path);

// ambula(path)
// Use fs.readdir with recursive option or glob library
```

### Python

```python
import os
import shutil
from pathlib import Path

# exstat(path)
os.path.exists(path)

# dele(path)
os.remove(path)

# copia(src, dest)
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

// copia(src, dest)
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

// exstat(path)
fs.cwd().access(path, .{}) catch |err| false;

// dele(path)
try fs.cwd().deleteFile(path);

// copia(src, dest)
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

| Feature            | Status   | Notes            |
| ------------------ | -------- | ---------------- |
| `exstat`           | Not Done | Check existence  |
| `dele`             | Not Done | Delete file      |
| `duplica`          | Not Done | Copy file        |
| `move`             | Not Done | Move/rename      |
| `inspice`          | Not Done | File info        |
| `trunca`           | Not Done | Truncate         |
| `tange`            | Not Done | Touch            |
| `crea`             | Not Done | Create directory |
| `elenca`           | Not Done | List directory   |
| `ambula`           | Not Done | Walk tree        |
| `vacua`            | Not Done | Remove empty dir |
| `dele_arbor`       | Not Done | Recursive delete |
| `via` utilities    | Not Done | Path operations  |
| `necte`            | Not Done | Symlinks         |
| `modus`            | Not Done | Permissions      |
| `temporarium`      | Not Done | Temp files       |
| TypeScript codegen | Not Done | fs/promises      |
| Python codegen     | Not Done | os, shutil       |
| Rust codegen       | Not Done | std::fs          |
| Zig codegen        | Not Done | std.fs           |

---

## Design Decisions

### Why separate from core I/O?

Core operations (`lege`, `inscribe`, `aperi`, `claude`) are language statements — fundamental and frequent. Utility operations (copy, move, delete) are less common and benefit from explicit imports.

### Why `via` for paths?

`via` means "way, path" — perfect for filesystem paths. It's also short and distinct from file operations.

### Why `ambula` returns iterator?

Walking large directory trees shouldn't load all paths into memory. An iterator (cursor) allows lazy traversal and early exit.

### Why `temporarium` uses `cura`?

Temp files must be cleaned up. Using `cura` ensures automatic deletion on scope exit, preventing temp file leaks.

### Permissions model?

Unix-style octal permissions (0o755) are the common denominator. Windows permission mapping is target-specific and may lose fidelity.
