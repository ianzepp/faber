# Gap Analysis: Checklist vs Consilia

Comparison of features documented in `consilia/` against `fons/codegen/checklist.md`.

## Summary

| Category | In Checklist | In Consilia Only | Total |
|----------|--------------|------------------|-------|
| Core Language | 95+ | ~15 | ~110 |
| Stdlib Modules | 0 | ~80 | ~80 |
| **Total** | ~95 | ~95 | ~190 |

The checklist covers core language features well but omits the entire stdlib design.

---

## Features In Checklist But Not Consilia

These appear implemented/designed but lack dedicated consilia docs:

| Feature | Status | Notes |
|---------|--------|-------|
| `typus` (type alias) | Implemented | Could add to types.md |
| `cum` (with/context) | Implemented | Brief mention in iteration.md |

---

## Features In Consilia But Missing From Checklist

### Core Language (should be added to checklist)

#### Control Flow

| Faber | Meaning | Source | Status |
|-------|---------|--------|--------|
| `sin` | else if (poetic) | keywords.ts | Not in checklist |
| `secus` | else/ternary alternate | keywords.ts | Not in checklist |
| `fac` | do/block | keywords.ts | Not in checklist |
| `ergo` | then (one-liner) | iteration.md | Not in checklist |
| `quando` | case (for elige) | keywords.ts | Not in checklist |
| `rumpe` | break | iteration.md | Designed, not impl |
| `perge` | continue | iteration.md | Designed, not impl |

#### Declarations

| Faber | Meaning | Source | Status |
|-------|---------|--------|--------|
| `ordo` | enum | keywords.ts | Not in checklist |
| `figendum` | async immutable binding | vincula.md | Designed, not impl |
| `variandum` | async mutable binding | vincula.md | Designed, not impl |
| `nexum` | reactive binding | rendering.md | Designed, not impl |

#### Operators / Verb Forms

| Faber | Meaning | Source | Status |
|-------|---------|--------|--------|
| `est` | === (strict equality) | keywords.ts | Not in checklist |
| `sic` | ? (ternary condition) | keywords.ts | Not in checklist |
| `fit` | -> (sync single) | keywords.ts | Partial (arrow fns) |
| `fiet` | async -> | keywords.ts | Partial |
| `fiunt` | yields -> (generator) | keywords.ts | Partial |
| `fient` | async yields -> | keywords.ts | Partial |

#### Lifecycle / Methods

| Faber | Meaning | Source | Status |
|-------|---------|--------|--------|
| `creo` | constructor hook | types.md | Implemented |
| `pingo` | render method | rendering.md | Designed, not impl |
| `deleo` | destructor | rendering.md | Designed, not impl |

#### Error Handling

| Faber | Meaning | Source | Status |
|-------|---------|--------|--------|
| `mori` | panic/fatal error | keywords.ts | Not in checklist |

### Collection DSL (consilia/collections.md)

| Feature | Description | Status |
|---------|-------------|--------|
| `ex items filtra ubi...` | DSL pipeline | Designed, not impl |
| `cum property` | Property shorthand | Designed, not impl |
| `{ .property }` | Implicit subject | Designed, not impl |
| `cum aetas descendens` | Sort direction | Designed, not impl |

### Participle Conjugation (collections.md)

| Pattern | Meaning | Example |
|---------|---------|---------|
| Imperative | Mutate in place | `adde` |
| Perfect participle | Return new | `addita` |
| Future | Async mutate | `addet` |
| Future participle | Async return new | `additura` |

---

## Stdlib Modules (Entirely Missing From Checklist)

These are designed in consilia but not tracked in the codegen checklist.

### fasciculus.md — File I/O (Core Statements)

| Faber | Meaning | Priority |
|-------|---------|----------|
| `lege path` | Read file as text | High |
| `lege path ut format` | Read with parsing (json, toml, csv) | High |
| `inscribe path, data` | Write file | High |
| `appone path, data` | Append to file | Medium |
| `aperi path` | Open file descriptor | Medium |
| `claude fd` | Close file descriptor | Medium |
| `quaere fd, pos` | Seek in file | Low |
| `formator<T>` | Custom format interface | Medium |
| `verte`/`reverte` | Format serialize/deserialize | Medium |

### solum.md — Local Filesystem

| Faber | Meaning | Priority |
|-------|---------|----------|
| `exstat path` | Check existence | High |
| `dele path` | Delete file | High |
| `duplica src, dest` | Copy file | Medium |
| `move src, dest` | Move/rename | Medium |
| `inspice path` | Get file info | Medium |
| `trunca path, size` | Truncate file | Low |
| `tange path` | Touch (create/update mtime) | Low |
| `crea path` | Create directory | High |
| `elenca path` | List directory | High |
| `ambula path` | Walk directory tree | Medium |
| `vacua path` | Remove empty directory | Low |
| `dele_arbor path` | Recursive delete | Medium |
| `via.iunge` | Path join | High |
| `via.parse` | Parse path | Medium |
| `necte src, link` | Create symlink | Low |
| `modus path` | Get/set permissions | Low |
| `temporarium` | Temp file with cura | Medium |

### caelum.md — Network I/O

| Faber | Meaning | Priority |
|-------|---------|----------|
| `pete url` | HTTP GET | High |
| `mitte url, body` | HTTP POST | High |
| `pone url, body` | HTTP PUT | Medium |
| `dele url` | HTTP DELETE | Medium |
| Response methods | `.corpus()`, `.textus()`, `.json()` | High |
| `ws.aperi url` | WebSocket client | Medium |
| `socket proto, host, port` | TCP/UDP socket | Low |
| `servi proto, host, port` | TCP/UDP server | Low |
| `resolve host` | DNS lookup | Low |

### tempus.md — Time Operations

| Faber | Meaning | Priority |
|-------|---------|----------|
| `nunc()` | Current epoch ms | High |
| `tempus.nunc()` | Current datetime | High |
| `hodie()` | Current date | Medium |
| `dormi ms` | Async sleep | High |
| Duration constants | `SECUNDUM`, `MINUTUM`, `HORA`, `DIES` | High |
| `duratio.secunda(n)` | Create duration | Medium |
| `post ms, fn` | One-shot timer | Medium |
| `intervallum ms, fn` | Repeating timer | Medium |
| `forma date, pattern` | Format datetime | Medium |
| `lege_tempus str, pattern` | Parse datetime | Medium |
| `.in_zona(tz)` | Timezone conversion | Low |

### crypto.md — Cryptography

| Faber | Meaning | Priority |
|-------|---------|----------|
| `digere data, algo` | Hash (sha256, etc.) | High |
| `hmac msg, key, algo` | HMAC | Medium |
| `cifra data, key, algo` | Encrypt (AES-GCM) | Medium |
| `decifra data, key, algo` | Decrypt | Medium |
| `fortuita n` | Secure random bytes | High |
| `fortuita_uuid()` | UUID v4 | Medium |
| `deriva pass, salt, opts` | Key derivation (argon2id) | Medium |

### comprimo.md — Compression

| Faber | Meaning | Priority |
|-------|---------|----------|
| `comprimo data, algo` | Compress (gzip, zstd) | Medium |
| `laxo data, algo` | Decompress | Medium |
| `compressor algo` | Streaming compress | Low |
| `laxator algo` | Streaming decompress | Low |

### codex.md — Encoding

| Faber | Meaning | Priority |
|-------|---------|----------|
| `coda data, encoding` | Encode (base64, hex) | Medium |
| `decoda str, encoding` | Decode | Medium |
| URL encoding | `"url"`, `"url_component"` | Medium |
| HTML entities | `"html"` | Low |

### cura.md — Resource Management

| Faber | Meaning | Priority |
|-------|---------|----------|
| `cura acquire fit binding { }` | Scoped resource | High |
| `curator` interface | Resource with cleanup | High |
| `solve()` method | Cleanup action | High |

### eventus.md — Events

| Faber | Meaning | Priority |
|-------|---------|----------|
| `emitte name, data` | Emit event | Medium |
| `ausculta name` | Event stream | Medium |
| `audi name, fn` | Callback subscription | Low |

---

## Implementation Priority Recommendation

### Phase 1: Core Language Gaps

1. `ordo` (enum) — Common need
2. `rumpe`/`perge` (break/continue) — Expected loop control
3. `mori` (panic) — Error handling completeness
4. `fac`/`cape` for systems targets — Zig/Rust need this

### Phase 2: Basic Stdlib

1. **File I/O**: `lege`, `inscribe`, `exstat`, `dele`, `crea`, `elenca`
2. **Time**: `nunc`, `dormi`, duration constants
3. **HTTP**: `pete` (fetch)

### Phase 3: Extended Stdlib

1. Crypto: `digere`, `fortuita`
2. Encoding: `coda`, `decoda`
3. Compression: `comprimo`, `laxo`

### Phase 4: Advanced Features

1. `nexum`/`pingo` — Reactive rendering
2. Collection DSL — `ex items filtra ubi...`
3. `figendum`/`variandum` — Async bindings

---

## Checklist Update Suggestions

The codegen checklist should add sections for:

1. **Enum (`ordo`)** — Under declarations
2. **Break/Continue** — Under control flow
3. **Fatal Errors (`mori`)** — Under exception handling
4. **Async Bindings** — New section for `figendum`/`variandum`
5. **Reactive Bindings** — New section for `nexum`
6. **Stdlib Intrinsics** — Expand beyond `_scribe` to include file I/O, time, etc.

---

## Cross-Reference: Keywords Not In Checklist

From `fons/lexicon/keywords.ts`:

| Keyword | Category | In Checklist |
|---------|----------|--------------|
| `sin` | control | No |
| `secus` | control | No |
| `fac` | control | No |
| `ergo` | control | No |
| `quando` | control | No |
| `rumpe` | control | No |
| `perge` | control | No |
| `mori` | control | No |
| `ordo` | declaration | No |
| `futura` | modifier | Yes (as `futura functio`) |
| `cursor` | modifier | Yes |
| `est` | operator | No |
| `sic` | operator | No |
| `fit` | operator | Partial |
| `fiet` | operator | Partial |
| `fiunt` | operator | Partial |
| `fient` | operator | Partial |
| `vel` | operator | No |
