---
status: planned
targets: [ts, py]
note: Database query DSL design. Not yet implemented.
updated: 2024-12
---

# Tabularium - Database Query DSL

## Overview

Faber provides a unified syntax for database queries using the `de` preposition (read-only context) combined with query-specific constructs. Three forms are supported:

| Form                         | Use Case                      |
| ---------------------------- | ----------------------------- |
| `de db.table quaere...`      | Simple queries, dot notation  |
| `de db apud table quaere...` | Dynamic table names, explicit |
| `de db sql "..."`            | Raw SQL, complex queries      |

All forms use parameterized queries with `?` placeholders, rewritten per target backend.

---

## Preposition Semantics

| Preposition | Latin            | Role                           |
| ----------- | ---------------- | ------------------------------ |
| `de`        | from, concerning | Connection context (read-only) |
| `apud`      | at, on           | Table context                  |
| `ubi`       | where            | Filter clause                  |

The `de` preposition establishes read-only semantics — query results are borrowed, not owned. This contrasts with `ex` for in-memory collections where elements can be mutated.

---

## Query Forms

### Dot Notation (Simple)

```fab
de db.users quaere nomen ubi "status = ?" ("active") fiet user {
    scribe user.nomen
}
```

The table name is a property of the connection. Clean syntax for common cases.

### Explicit Table (apud)

```fab
de db apud users quaere nomen ubi "status = ?" ("active") fiet user {
    scribe user.nomen
}
```

Equivalent to dot notation, but allows dynamic table names:

```fab
fixum table = "users"
de db apud table quaere ubi "active = ?" (verum) fiet row { }
```

### Raw SQL (Escape Hatch)

```fab
de db sql "SELECT name FROM users WHERE status = ?" ("active") fiet row {
    scribe row.name
}
```

Full SQL power for complex queries:

```fab
de db sql "
    SELECT u.name, COUNT(o.id) as order_count
    FROM users u
    LEFT JOIN orders o ON o.user_id = u.id
    WHERE u.created_at > ?
    GROUP BY u.id
    HAVING COUNT(o.id) > ?
" (cutoffDate, minOrders) fiet row {
    scribe row.name, row.order_count
}
```

---

## Query Structure

### Full DSL Form

```
de <connection> [apud <table>] quaere [fields] [ubi "clause" (params)] [transforms] <verb> <binding> { body }
```

| Component       | Required | Description                               |
| --------------- | -------- | ----------------------------------------- |
| `de connection` | yes      | Database connection                       |
| `apud table`    | no       | Table name (alternative to dot notation)  |
| `quaere`        | yes      | Query keyword                             |
| `fields`        | no       | Column names (omit for SELECT \*)         |
| `ubi "clause"`  | no       | WHERE clause with ? placeholders          |
| `(params)`      | no       | Bound parameters                          |
| `transforms`    | no       | DSL transforms (prima, ultima, etc.)      |
| `verb`          | yes      | Iteration verb (pro/fit/fiet/fiunt/fient) |
| `binding`       | yes      | Row variable name                         |
| `{ body }`      | yes      | Loop body                                 |

### Examples

```fab
// Select all columns
de db.users quaere fiet user { }

// Select specific columns
de db.users quaere nomen, email fiet user { }

// With WHERE clause
de db.users quaere ubi "active = ?" (verum) fiet user { }

// With columns and WHERE
de db.users quaere nomen ubi "role = ?" ("admin") fiet user { }

// With DSL transforms
de db.logs quaere ubi "level = ?" ("error") prima 100 fiet log { }
```

---

## Parameterized Queries

All forms use `?` as the placeholder character. The compiler:

1. Counts `?` occurrences in the SQL string
2. Validates count matches argument list length
3. Rewrites placeholders for target backend

### Target Mappings

| Target     | Placeholder Style   |
| ---------- | ------------------- |
| PostgreSQL | `$1`, `$2`, `$3`    |
| MySQL      | `?`, `?`, `?`       |
| SQLite     | `?`, `?`, `?`       |
| SQL Server | `@p1`, `@p2`, `@p3` |

### Example

```fab
de db.users quaere ubi "email LIKE ? AND active = ?" (pattern, verum) fiet user { }
```

**PostgreSQL output:**

```ts
for await (const user of db.query('SELECT * FROM users WHERE email LIKE $1 AND active = $2', [pattern, true])) {
}
```

**SQLite output:**

```python
for user in db.execute(
    "SELECT * FROM users WHERE email LIKE ? AND active = ?",
    (pattern, True)
):
```

### Validation

Mismatch between placeholders and arguments is a compile-time error:

```fab
de db.users quaere ubi "a = ? AND b = ?" (onlyOne)
// Error: 2 placeholders, 1 argument
```

---

## Iteration Verbs

The verb conjugation determines sync/async and batch/streaming behavior:

| Verb    | Sync/Async | Behavior           | Use Case                |
| ------- | ---------- | ------------------ | ----------------------- |
| `pro`   | sync       | Batch, all at once | In-memory after fetch   |
| `fit`   | sync       | Same as pro        | Explicit sync           |
| `fiet`  | async      | Awaited iteration  | Standard async query    |
| `fiunt` | sync       | Generator, yields  | Streaming (sync driver) |
| `fient` | async      | Async generator    | Streaming over network  |

### Batch (fiet)

Fetches all rows, then iterates:

```fab
de db.users quaere ubi "active = ?" (verum) fiet user {
    scribe user.nomen
}
```

### Streaming (fient)

Yields rows as they arrive, enables backpressure:

```fab
de db.logs quaere ubi "level = ?" ("error") fient log {
    scribe log.message
    cede
}
```

The `cede` (yield) inside the body allows the generator to pause between rows.

---

## DSL Transforms

Collection DSL transforms can follow the query:

| Transform  | SQL Equivalent | Notes                    |
| ---------- | -------------- | ------------------------ |
| `prima n`  | `LIMIT n`      | First n rows             |
| `ultima n` | —              | Client-side, last n rows |
| `omitte n` | `OFFSET n`     | Skip first n rows        |
| `summa`    | —              | Client-side aggregation  |

```fab
// Server-side LIMIT
de db.logs quaere ubi "level = ?" ("error") prima 100 fiet log { }

// Pagination
de db.users quaere omitte 20, prima 10 fiet user { }
```

Note: `ultima` and `summa` require fetching rows first, so they execute client-side.

---

## Read-Only Semantics

The `de` preposition enforces read-only access. Query results are borrowed, not owned:

```fab
de db.users quaere fiet user {
    scribe user.nomen      // OK: reading
    user.nomen = "new"     // Error: cannot mutate borrowed value
}
```

For mutations, use the `in` preposition (see Mutations section below).

---

## Mutations (`in`)

The `in` preposition provides mutable database operations. This mirrors in-memory semantics where `de` is read-only and `in` allows modification.

### Preposition Contrast

| Preposition | Semantics        | Operations             |
| ----------- | ---------------- | ---------------------- |
| `de`        | Read-only, query | SELECT                 |
| `in`        | Mutable, modify  | INSERT, UPDATE, DELETE |

### Update (muta)

```fab
// Update with WHERE clause
in db.users muta "status = ?" ("active") ubi "id = ?" (userId)

// Update multiple fields
in db.users muta "status = ?, updated_at = ?" ("active", now) ubi "role = ?" ("pending")

// Raw SQL update
in db sql "UPDATE users SET status = ? WHERE id = ?" ("active", userId)
```

The `muta` verb (Latin: "change") generates UPDATE statements.

### Insert (adde)

```fab
// Insert with field values
in db.users adde (nomen: "Marcus", email: "marcus@roma.it")

// Insert multiple rows
in db.users adde [
    (nomen: "Marcus", email: "marcus@roma.it"),
    (nomen: "Julia", email: "julia@roma.it")
]

// Raw SQL insert
in db sql "INSERT INTO users (name, email) VALUES (?, ?)" ("Marcus", "marcus@roma.it")
```

The `adde` verb (Latin: "add") generates INSERT statements. Matches the collection method for adding elements.

### Delete (dele)

```fab
// Delete with WHERE clause
in db.users dele ubi "status = ?" ("inactive")

// Delete by ID
in db.users dele ubi "id = ?" (userId)

// Raw SQL delete
in db sql "DELETE FROM users WHERE id = ?" (userId)
```

The `dele` verb (Latin: "remove") generates DELETE statements. Matches the collection method for removing elements.

### Mutation Verbs Summary

| Verb   | Latin  | SQL    | Collection Equivalent  |
| ------ | ------ | ------ | ---------------------- |
| `muta` | change | UPDATE | (no direct equivalent) |
| `adde` | add    | INSERT | `lista.adde(x)`        |
| `dele` | remove | DELETE | `lista.dele(x)`        |

### Return Values

Mutations can return affected row count or inserted IDs:

```fab
// Get affected count
fixum count = in db.users muta "status = ?" ("active") ubi "role = ?" ("pending")

// Get inserted ID (if supported by driver)
fixum id = in db.users adde (nomen: "Marcus", email: "marcus@roma.it")
```

### Mutation Structure

```
in <connection>.<table> <verb> <values/set-clause> [ubi "where" (params)]
in <connection> sql "raw SQL" (params)
```

| Component       | Required  | Description                                        |
| --------------- | --------- | -------------------------------------------------- |
| `in connection` | yes       | Database connection (mutable context)              |
| `.table`        | yes (DSL) | Table name                                         |
| `verb`          | yes       | `muta`, `adde`, or `dele`                          |
| `values/clause` | yes       | SET clause or field values                         |
| `ubi "where"`   | no        | WHERE clause (required for muta/dele in safe mode) |
| `(params)`      | no        | Bound parameters                                   |

### Safety: WHERE Required

To prevent accidental mass updates/deletes, `muta` and `dele` require `ubi` clause by default:

```fab
// Error: WHERE clause required
in db.users dele  // Compile error: dele requires ubi clause

// OK: explicit "all rows" intent
in db.users dele ubi "1 = 1" ()

// OK: with condition
in db.users dele ubi "status = ?" ("inactive")
```

---

## Connection Handling

Connection management is outside the query DSL scope. Assumed patterns:

```fab
// Connection passed in or configured
fixum db = connecta("postgres://...")

// Queries use the connection
de db.users quaere fiet user { }

// Mutations use the connection
in db.users adde (nomen: "Marcus", email: "marcus@roma.it")
```

### Transactions

Transaction blocks use `cura ... fiet` to acquire a transaction handle:

```fab
cura db.transactio fiet tx {
    // Read within transaction
    de tx.accounts quaere ubi "id = ?" (fromId) fiet from {
        fixum balance = from.balance
    }

    // Write within transaction
    in tx.accounts muta "balance = balance - ?" (amount) ubi "id = ?" (fromId)
    in tx.accounts muta "balance = balance + ?" (amount) ubi "id = ?" (toId)

    // Insert audit log
    in tx.transfers adde (fromId: fromId, toId: toId, amount: amount)
}
```

The `fiet` signals async acquisition. The `tx` binding is the transaction handle — all operations inside use it explicitly. The `cura` block ensures:

- All operations use the same transaction
- Automatic commit on success
- Automatic rollback on error

---

## Comparison: DSL vs Raw

| Aspect         | DSL (`apud...quaere`)         | Raw (`sql`)                |
| -------------- | ----------------------------- | -------------------------- |
| Readability    | High for simple queries       | Familiar SQL               |
| Validation     | Table/field checking possible | No compile-time validation |
| Joins          | Not supported                 | Full support               |
| Subqueries     | Not supported                 | Full support               |
| Aggregations   | Limited                       | Full support               |
| Learning curve | New syntax                    | Known SQL                  |

**Recommendation:** Use DSL for simple single-table queries. Use raw SQL for anything involving joins, subqueries, or complex aggregations.

---

## Future Considerations

### Schema Integration

If genus definitions match database tables:

```fab
genus User {
    textus nomen
    textus email
    bivalens active
}

// Compiler validates field names
de db.users quaere nomen, email ubi "active = ?" (verum) fiet User user { }
```

### Additional DSL Verbs

| Verb           | Clause   | Example                           |
| -------------- | -------- | --------------------------------- |
| `ordina per`   | ORDER BY | `ordina per createdAt descendens` |
| `congrega per` | GROUP BY | `congrega per status`             |
| `iunge`        | JOIN     | TBD — may be too complex for DSL  |

### Prepared Statements

```fab
fixum stmt = db.para("SELECT * FROM users WHERE status = ?")

de stmt (status) fiet user { }
de stmt (anotherStatus) fiet user { }
```

Reusable prepared statements for repeated queries.
