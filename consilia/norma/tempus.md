---
status: partial
targets: [ts]
note: Basic instant/sleep done for TS; datetime and formatting not yet implemented
updated: 2024-12
---

# Tempus - Time Operations

## Overview

Time operations: instants, datetimes, durations, sleeping, and formatting.

## Core Types

| Type       | Meaning                  | Use Case                       |
| ---------- | ------------------------ | ------------------------------ |
| `momentum` | Point in time (epoch ms) | Timestamps, elapsed time       |
| `tempus`   | Calendar datetime        | Dates, scheduling, display     |
| `duratio`  | Time span                | Delays, intervals, differences |

## Etymology

| Latin      | Meaning          | Maps To            |
| ---------- | ---------------- | ------------------ |
| `momentum` | instant, moment  | epoch milliseconds |
| `tempus`   | time, period     | Date/DateTime      |
| `duratio`  | duration, length | time span          |
| `nunc`     | now              | current instant    |
| `hodie`    | today            | current date       |
| `dormi`    | sleep!           | pause execution    |
| `forma`    | shape, form      | format to string   |
| `para`     | prepare, parse   | parse from string  |

---

## momentum (Instant)

A point in time as milliseconds since Unix epoch. Lightweight, no timezone.

```fab
ex "norma/tempus" importa momentum

fixum now = momentum.nunc()           # current instant (ms)
fixum later = momentum.nunc()
fixum elapsed = later - now           # duratio

# Comparison
si later > now { scribe "time passed" }

# Arithmetic with duratio
fixum future = now + duratio.secunda(5)
```

### momentum Methods

| Method              | Returns    | Meaning                           |
| ------------------- | ---------- | --------------------------------- |
| `momentum.nunc()`   | `momentum` | Current instant                   |
| `m.ad_tempus()`     | `tempus`   | Convert to datetime (local TZ)    |
| `m.ad_tempus(zona)` | `tempus`   | Convert to datetime (specific TZ) |
| `m - other`         | `duratio`  | Difference between instants       |
| `m + duratio`       | `momentum` | Add duration                      |
| `m - duratio`       | `momentum` | Subtract duration                 |

---

## tempus (DateTime)

Calendar datetime with year, month, day, hour, minute, second. Timezone-aware.

```fab
ex "norma/tempus" importa tempus

fixum now = tempus.nunc()             # current datetime (local TZ)
fixum today = tempus.hodie()          # current date at midnight
fixum utc = tempus.nunc_utc()         # current datetime (UTC)

# Access fields
scribe now.annus                      # 2025
scribe now.mensis                     # 12
scribe now.dies                       # 27
scribe now.hora                       # 14
scribe now.minutum                    # 30
scribe now.secundum                   # 45

# Day of week (1=Monday, 7=Sunday)
scribe now.dies_hebdomadis            # 6 (Saturday)
```

### tempus Fields

| Field             | Type      | Range | Meaning       |
| ----------------- | --------- | ----- | ------------- |
| `annus`           | `numerus` | any   | year          |
| `mensis`          | `numerus` | 1-12  | month         |
| `dies`            | `numerus` | 1-31  | day of month  |
| `hora`            | `numerus` | 0-23  | hour          |
| `minutum`         | `numerus` | 0-59  | minute        |
| `secundum`        | `numerus` | 0-59  | second        |
| `millisecundum`   | `numerus` | 0-999 | millisecond   |
| `dies_hebdomadis` | `numerus` | 1-7   | day of week   |
| `dies_anni`       | `numerus` | 1-366 | day of year   |
| `zona`            | `textus`  | IANA  | timezone name |

### tempus Constructors

```fab
# From fields
fixum t = tempus.ex(2025, 12, 25)                    # Dec 25, 2025 midnight
fixum t = tempus.ex(2025, 12, 25, 14, 30, 0)         # Dec 25, 2025 14:30:00

# From epoch
fixum t = tempus.ex_momento(momentum.nunc())

# From string
fixum t = tempus.para("2025-12-25", "YYYY-MM-DD")
fixum t = tempus.para("2025-12-25T14:30:00Z")        # ISO 8601 auto-detected
```

### tempus Methods

| Method              | Returns    | Meaning                   |
| ------------------- | ---------- | ------------------------- |
| `tempus.nunc()`     | `tempus`   | Current local datetime    |
| `tempus.nunc_utc()` | `tempus`   | Current UTC datetime      |
| `tempus.hodie()`    | `tempus`   | Today at midnight (local) |
| `tempus.ex(...)`    | `tempus`   | From fields               |
| `tempus.para(str)`  | `tempus?`  | Parse from string         |
| `t.ad_momentum()`   | `momentum` | Convert to instant        |
| `t.in_zona(zona)`   | `tempus`   | Convert to timezone       |
| `t.forma(pattern)`  | `textus`   | Format to string          |
| `t + duratio`       | `tempus`   | Add duration              |
| `t - duratio`       | `tempus`   | Subtract duration         |
| `t - other`         | `duratio`  | Difference                |

---

## duratio (Duration)

A length of time, independent of calendar. Used for arithmetic and delays.

```fab
ex "norma/tempus" importa duratio

# Named constructors
fixum d = duratio.milli(500)          # 500 milliseconds
fixum d = duratio.secunda(5)          # 5 seconds
fixum d = duratio.minuta(2)           # 2 minutes
fixum d = duratio.hora(1)             # 1 hour
fixum d = duratio.dies(7)             # 7 days

# Arithmetic
fixum total = duratio.hora(1) + duratio.minuta(30)   # 1h 30m

# Access components
scribe total.in_secundis              # 5400
scribe total.in_minutis               # 90
scribe total.in_horis                 # 1.5
```

### duratio Constructors

| Constructor          | Meaning        |
| -------------------- | -------------- |
| `duratio.milli(n)`   | n milliseconds |
| `duratio.secunda(n)` | n seconds      |
| `duratio.minuta(n)`  | n minutes      |
| `duratio.hora(n)`    | n hours        |
| `duratio.dies(n)`    | n days         |

### duratio Accessors

| Accessor        | Returns   | Meaning            |
| --------------- | --------- | ------------------ |
| `d.in_millis`   | `numerus` | total milliseconds |
| `d.in_secundis` | `numerus` | total seconds      |
| `d.in_minutis`  | `fractus` | total minutes      |
| `d.in_horis`    | `fractus` | total hours        |
| `d.in_diebus`   | `fractus` | total days         |

### duratio Arithmetic

```fab
fixum a = duratio.secunda(30)
fixum b = duratio.secunda(20)

fixum sum = a + b                     # 50 seconds
fixum diff = a - b                    # 10 seconds
fixum doubled = a * 2                 # 60 seconds
fixum halved = a / 2                  # 15 seconds
```

---

## Duration Constants

Convenience constants for common durations:

```fab
ex "norma/tempus" importa {
    MILLISECUNDUM,    # duratio.milli(1)
    SECUNDUM,         # duratio.secunda(1)
    MINUTUM,          # duratio.minuta(1)
    HORA,             # duratio.hora(1)
    DIES              # duratio.dies(1)
}

cede dormi(5 * SECUNDUM)
cede dormi(HORA + 30 * MINUTUM)
```

---

## Sleeping

```fab
ex "norma/tempus" importa dormi, SECUNDUM

# Sleep for duration
cede dormi(duratio.secunda(5))
cede dormi(5 * SECUNDUM)              # same thing

# Sleep until instant
cede dormi_usque(target)
```

| Function                | Meaning             |
| ----------------------- | ------------------- |
| `dormi(duratio)`        | Sleep for duration  |
| `dormi_usque(momentum)` | Sleep until instant |

---

## Formatting

```fab
ex "norma/tempus" importa tempus

fixum now = tempus.nunc()

scribe now.forma("YYYY-MM-DD")            # "2025-12-27"
scribe now.forma("HH:mm:ss")              # "14:30:45"
scribe now.forma("YYYY-MM-DD HH:mm:ss")   # "2025-12-27 14:30:45"
scribe now.forma()                        # ISO 8601 default
```

### Format Tokens

| Token  | Meaning          | Example |
| ------ | ---------------- | ------- |
| `YYYY` | 4-digit year     | 2025    |
| `YY`   | 2-digit year     | 25      |
| `MM`   | Month (01-12)    | 12      |
| `DD`   | Day (01-31)      | 27      |
| `HH`   | Hour 24h (00-23) | 14      |
| `hh`   | Hour 12h (01-12) | 02      |
| `mm`   | Minutes (00-59)  | 30      |
| `ss`   | Seconds (00-59)  | 45      |
| `SSS`  | Milliseconds     | 123     |
| `a`    | am/pm            | pm      |
| `A`    | AM/PM            | PM      |
| `Z`    | Timezone offset  | +05:00  |
| `ZZ`   | Timezone offset  | +0500   |

---

## Parsing

```fab
ex "norma/tempus" importa tempus

# With explicit format
fixum t = tempus.para("2025-12-25", "YYYY-MM-DD")

# ISO 8601 auto-detected
fixum t = tempus.para("2025-12-25T14:30:00Z")
fixum t = tempus.para("2025-12-25T14:30:00+05:00")

# Returns nihil on failure
fixum t = tempus.para("invalid")      # nihil
```

---

## Timezones

```fab
ex "norma/tempus" importa tempus

fixum now = tempus.nunc()                     # local timezone
fixum tokyo = now.in_zona("Asia/Tokyo")
fixum utc = now.in_zona("UTC")

scribe now.zona                               # "America/New_York"
scribe tokyo.zona                             # "Asia/Tokyo"
scribe tokyo.hora                             # different hour
```

IANA timezone names are used (e.g., "America/New_York", "Europe/London", "Asia/Tokyo").

---

## Elapsed Time Pattern

```fab
ex "norma/tempus" importa momentum

fixum start = momentum.nunc()

# ... do work ...

fixum elapsed = momentum.nunc() - start

si elapsed > duratio.secunda(5) {
    mone "Operation took too long:", elapsed.in_secundis, "s"
}
```

---

## Target Mappings

### TypeScript

```typescript
// momentum.nunc()
Date.now();

// tempus.nunc()
new Date();

// tempus.ex(2025, 12, 25)
new Date(2025, 11, 25); // Note: JS months are 0-indexed

// duratio.secunda(5)
5000; // milliseconds

// dormi(duratio)
await new Promise(r => setTimeout(r, duratio));

// t.forma("YYYY-MM-DD")
// Requires Intl.DateTimeFormat or date-fns
```

### Python

```python
import time
import datetime

# momentum.nunc()
int(time.time() * 1000)

# tempus.nunc()
datetime.datetime.now()

# tempus.ex(2025, 12, 25)
datetime.datetime(2025, 12, 25)

# duratio.secunda(5)
datetime.timedelta(seconds=5)

# dormi(duratio)
await asyncio.sleep(duratio.total_seconds())

# t.forma("YYYY-MM-DD")
t.strftime("%Y-%m-%d")
```

### Zig

```zig
const std = @import("std");

// momentum.nunc()
std.time.milliTimestamp()

// duratio: just use i64 milliseconds

// dormi(duratio)
std.time.sleep(duratio * 1_000_000)  // ns

// tempus: no built-in, needs external lib
```

---

## Implementation Status

| Feature                | TS   | Py  | Zig | Notes                 |
| ---------------------- | ---- | --- | --- | --------------------- |
| `momentum.nunc()`      | Done | -   | -   | Epoch ms              |
| `momentum` arithmetic  | -    | -   | -   |                       |
| `tempus.nunc()`        | -    | -   | -   | Datetime object       |
| `tempus.hodie()`       | -    | -   | -   | Today at midnight     |
| `tempus` fields        | -    | -   | -   | annus, mensis, etc.   |
| `tempus.forma()`       | -    | -   | -   | Formatting            |
| `tempus.para()`        | -    | -   | -   | Parsing               |
| `duratio` constructors | -    | -   | -   | secunda, minuta, etc. |
| `duratio` arithmetic   | -    | -   | -   | +, -, \*, /           |
| Duration constants     | Done | -   | -   | SECUNDUM, etc.        |
| `dormi()`              | Done | -   | -   | Async sleep           |
| Timezone support       | -    | -   | -   | in_zona()             |

---

## Design Decisions

### Why three types?

- `momentum` — machine time, for performance measurement and timestamps
- `tempus` — human time, for display and scheduling
- `duratio` — intervals, for arithmetic and delays

Mixing these is a common bug source. Explicit types prevent it.

### Why `momentum` not `instantum`?

"Momentum" is already English (borrowed from Latin). "Instantum" is awkward Latin. `momentum` reads naturally in both languages.

### Why method syntax over free functions?

`tempus.nunc()` vs `nunc()`:

- Namespaced: no collisions with user code
- Discoverable: type tells you what operations exist
- Consistent: same pattern as collections

### Why `para` for parsing?

"Para" (prepare/make ready) implies transformation into a usable form. Alternatives considered:

- `lege` — but that means "read" (files)
- `ex_texto` — verbose
- `parse` — not Latin

### Why milliseconds for momentum?

JavaScript's `Date.now()` returns milliseconds. Most APIs use ms. Nanoseconds available via `momentum.nunc_nano()` where targets support it.
