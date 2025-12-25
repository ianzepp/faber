# Gap Analysis: Checklist vs Consilia

Comparison of features documented in `consilia/` against `consilia/codegen/checklist.md`.

## Summary

| Category       | In Checklist | In Consilia Only | Total |
| -------------- | ------------ | ---------------- | ----- |
| Core Language  | 110+         | ~5               | ~115  |
| Stdlib Modules | ~25          | ~55              | ~80   |
| **Total**      | ~135         | ~60              | ~195  |

The checklist now covers core language features comprehensively. Stdlib is partially tracked.

---

## Features In Checklist But Not Consilia

These appear implemented/designed but lack dedicated consilia docs:

| Feature              | Status      | Notes                         |
| -------------------- | ----------- | ----------------------------- |
| `typus` (type alias) | Implemented | Could add to types.md         |
| `cum` (with/context) | Implemented | Brief mention in iteration.md |

---

## Features In Consilia But Missing From Checklist

### Core Language

Most core language features are now tracked in the checklist. Remaining gaps:

#### Participle Conjugation (collections.md)

| Pattern            | Meaning          | Example    | Status          |
| ------------------ | ---------------- | ---------- | --------------- |
| Imperative         | Mutate in place  | `adde`     | Implemented     |
| Perfect participle | Return new       | `addita`   | Implemented     |
| Future             | Async mutate     | `addet`    | Not implemented |
| Future participle  | Async return new | `additura` | Not implemented |

The async participle forms (`addet`, `additura`) are designed but not yet implemented.

---

## Stdlib Modules (Partially in Checklist)

The checklist now tracks high-priority stdlib items. These remain untracked:

### fasciculus.md — File I/O (Lower Priority)

| Faber             | Meaning                      | Priority |
| ----------------- | ---------------------------- | -------- |
| `aperi path`      | Open file descriptor         | Medium   |
| `claude fd`       | Close file descriptor        | Medium   |
| `quaere fd, pos`  | Seek in file                 | Low      |
| `formator<T>`     | Custom format interface      | Medium   |
| `verte`/`reverte` | Format serialize/deserialize | Medium   |

### solum.md — Local Filesystem (Lower Priority)

| Faber               | Meaning                     | Priority |
| ------------------- | --------------------------- | -------- |
| `duplica src, dest` | Copy file                   | Medium   |
| `move src, dest`    | Move/rename                 | Medium   |
| `inspice path`      | Get file info               | Medium   |
| `trunca path, size` | Truncate file               | Low      |
| `tange path`        | Touch (create/update mtime) | Low      |
| `ambula path`       | Walk directory tree         | Medium   |
| `vacua path`        | Remove empty directory      | Low      |
| `dele_arbor path`   | Recursive delete            | Medium   |
| `via.parse`         | Parse path                  | Medium   |
| `necte src, link`   | Create symlink              | Low      |
| `modus path`        | Get/set permissions         | Low      |
| `temporarium`       | Temp file with cura         | Medium   |

### caelum.md — Network I/O (Lower Priority)

| Faber                      | Meaning          | Priority |
| -------------------------- | ---------------- | -------- |
| `pone url, body`           | HTTP PUT         | Medium   |
| `dele url`                 | HTTP DELETE      | Medium   |
| `ws.aperi url`             | WebSocket client | Medium   |
| `socket proto, host, port` | TCP/UDP socket   | Low      |
| `servi proto, host, port`  | TCP/UDP server   | Low      |
| `resolve host`             | DNS lookup       | Low      |

### tempus.md — Time Operations (Lower Priority)

| Faber                      | Meaning             | Priority |
| -------------------------- | ------------------- | -------- |
| `tempus.nunc()`            | Current datetime    | High     |
| `hodie()`                  | Current date        | Medium   |
| `duratio.secunda(n)`       | Create duration     | Medium   |
| `post ms, fn`              | One-shot timer      | Medium   |
| `intervallum ms, fn`       | Repeating timer     | Medium   |
| `forma date, pattern`      | Format datetime     | Medium   |
| `lege_tempus str, pattern` | Parse datetime      | Medium   |
| `.in_zona(tz)`             | Timezone conversion | Low      |

### crypto.md — Cryptography (Lower Priority)

| Faber                     | Meaning                   | Priority |
| ------------------------- | ------------------------- | -------- |
| `hmac msg, key, algo`     | HMAC                      | Medium   |
| `cifra data, key, algo`   | Encrypt (AES-GCM)         | Medium   |
| `decifra data, key, algo` | Decrypt                   | Medium   |
| `fortuita_uuid()`         | UUID v4                   | Medium   |
| `deriva pass, salt, opts` | Key derivation (argon2id) | Medium   |

### comprimo.md — Compression

| Faber                 | Meaning               | Priority |
| --------------------- | --------------------- | -------- |
| `comprimo data, algo` | Compress (gzip, zstd) | Medium   |
| `laxo data, algo`     | Decompress            | Medium   |
| `compressor algo`     | Streaming compress    | Low      |
| `laxator algo`        | Streaming decompress  | Low      |

### codex.md — Encoding

| Faber                  | Meaning                    | Priority |
| ---------------------- | -------------------------- | -------- |
| `coda data, encoding`  | Encode (base64, hex)       | Medium   |
| `decoda str, encoding` | Decode                     | Medium   |
| URL encoding           | `"url"`, `"url_component"` | Medium   |
| HTML entities          | `"html"`                   | Low      |

### eventus.md — Events (Lower Priority)

| Faber           | Meaning               | Priority |
| --------------- | --------------------- | -------- |
| `audi name, fn` | Callback subscription | Low      |

---

## Implementation Priority Recommendation

### Phase 1: Core Language Gaps (Remaining)

Core language features for TypeScript are complete. Remaining work is stdlib and advanced features.

### Phase 2: Basic Stdlib

1. **File I/O**: `lege`, `inscribe`, `exstat`, `dele`, `crea`, `elenca`
2. **Time**: `nunc`, `dormi`, duration constants
3. **HTTP**: `pete` (fetch)

### Phase 3: Extended Stdlib

1. Crypto: `digere`, `fortuita`
2. Encoding: `coda`, `decoda`
3. Compression: `comprimo`, `laxo`

### Phase 4: Advanced Features

1. `nexum` (reactive fields in genus) — **Implemented for TypeScript and Python**
2. Collection DSL — `ex items filtra ubi...`
3. Async participles — `addet`, `additura`

---

## Recently Added to Checklist

The following items were added to the checklist and are now tracked:

### Control Flow

- `sin` (else if poetic) — TS: implemented
- `secus` (else/ternary alt) — TS: implemented
- `fac` (do/block) — TS: implemented
- `ergo` (then one-liner) — TS: implemented
- `rumpe` (break) — TS: implemented
- `perge` (continue) — TS: implemented
- `elige` — TS: emits if/else chains (not switch)

### Declarations

- `ordo` (enum) — TS: implemented
- `figendum` (async immutable) — Implemented for TypeScript and Python
- `variandum` (async mutable) — Implemented for TypeScript and Python
- `nexum` (reactive field) — Implemented for TypeScript and Python (within genus)

### Operators

- `est` (strict equality) — TS: implemented
- `sic`/`secus` ternary — TS: implemented
- `aut` (logical or) — TS: implemented

### Lifecycle

- `deleo` (destructor) — Not implemented
- `pingo` (render) — Not implemented

### Error Handling

- `mori` (panic) — TS: implemented (throws Error with [PANIC] prefix)

### Collection DSL

- All items now tracked as "Future"

### Stdlib Sections Added

- File I/O (fasciculus) — High-priority items
- Filesystem (solum) — High-priority items
- Network (caelum) — High-priority items
- Time (tempus) — High-priority items
- Crypto — High-priority items
- Resource Management (cura)

### Infrastructure Added

- Preamble / Prologue system — Feature-dependent imports/includes (see `consilia/codegen/preamble.md`)
