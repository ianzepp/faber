# Faber Romanus Grammar

Complete syntax reference for the Faber Romanus programming language.

## For LLMs

This document is designed for both human readers and LLM code generation. When generating Faber code:

**Style preferences:**

- PREFER Latin keywords over symbols: `et` over `&&`, `aut` over `||`, `non` over `!`
- PREFER `si`/`sin`/`secus` (poetic) or `si`/`aliter si`/`aliter` (literal) consistently
- PREFER `pro x: expr` for short lambdas, `pro x redde expr` when clarity helps
- ALWAYS use type-first syntax: `textus nomen` not `nomen: textus`
- NEVER use JavaScript/TypeScript/Python syntax where Faber has its own

**Common mistakes to avoid:**

- `return` instead of `redde`
- `const`/`let` instead of `fixum`/`varia`
- `if`/`else` instead of `si`/`aliter`
- `for...of` instead of `ex...pro`
- `string`/`number`/`boolean` instead of `textus`/`numerus`/`bivalens`
- `null` instead of `nihil`
- `this` instead of `ego`
- `new` instead of `novum`
- `await` instead of `cede`
- `async function` instead of `futura functio`

---

## Quick Reference

### Types

| Faber      | TypeScript   | Python     | Zig          | Meaning        |
| ---------- | ------------ | ---------- | ------------ | -------------- |
| `textus`   | `string`     | `str`      | `[]const u8` | text/string    |
| `numerus`  | `number`     | `int`      | `i64`        | integer        |
| `fractus`  | `number`     | `float`    | `f64`        | floating point |
| `decimus`  | `number`     | `Decimal`  | -            | decimal        |
| `magnus`   | `bigint`     | `int`      | `i128`       | big integer    |
| `bivalens` | `boolean`    | `bool`     | `bool`       | boolean        |
| `nihil`    | `null`       | `None`     | `null`       | null           |
| `vacuum`   | `void`       | `None`     | `void`       | void           |
| `numquam`  | `never`      | `NoReturn` | `noreturn`   | never          |
| `ignotum`  | `unknown`    | `Any`      | -            | unknown        |
| `octeti`   | `Uint8Array` | `bytes`    | `[]u8`       | bytes          |
| `objectum` | `object`     | `object`   | -            | object         |

### Generic Collections

| Faber          | TypeScript    | Python         | Meaning        |
| -------------- | ------------- | -------------- | -------------- |
| `lista<T>`     | `T[]`         | `list[T]`      | array/list     |
| `tabula<K,V>`  | `Map<K,V>`    | `dict[K,V]`    | map/dictionary |
| `copia<T>`     | `Set<T>`      | `set[T]`       | set            |
| `promissum<T>` | `Promise<T>`  | `Awaitable[T]` | promise/future |
| `cursor<T>`    | `Iterator<T>` | `Iterator[T]`  | iterator       |
| `unio<A,B>`    | `A \| B`      | `A \| B`       | union type     |

### Literals

| Faber    | Meaning   |
| -------- | --------- |
| `verum`  | true      |
| `falsum` | false     |
| `nihil`  | null      |
| `ego`    | this/self |

### Keywords by Category

**Declarations:**

- `fixum` — immutable binding (const)
- `varia` — mutable binding (let)
- `functio` — function
- `genus` — class/struct
- `pactum` — interface/protocol
- `typus` — type alias
- `ordo` — enum
- `discretio` — tagged union

**Control flow:**

- `si` / `sin` / `aliter` / `secus` — if / else-if / else
- `dum` — while
- `ex...pro` — for-of (iteration)
- `de...pro` — for-in (keys)
- `elige` — switch/match
- `custodi` — guard clauses
- `rumpe` — break
- `perge` — continue

**Functions:**

- `redde` — return
- `futura` — async modifier
- `cede` — await
- `cursor` — generator modifier
- `pro x: expr` — lambda expression

**Error handling:**

- `tempta` — try
- `cape` — catch
- `demum` — finally
- `iace` — throw (recoverable)
- `mori` — panic (fatal)
- `adfirma` — assert

**Output:**

- `scribe` — console.log
- `vide` — console.debug
- `mone` — console.warn

**Operators:**

- `et` — logical and (&&)
- `aut` — logical or (||)
- `non` — logical not (!)
- `vel` — nullish coalescing (??)
- `est` — instanceof/typeof check
- `qua` — type cast (as)

### Collection Methods (lista)

Common array methods (see README for complete list):

| Latin               | JavaScript          | Mutates? |
| ------------------- | ------------------- | -------- |
| `adde(x)`           | `push(x)`           | yes      |
| `remove()`          | `pop()`             | yes      |
| `primus`            | `[0]`               | no       |
| `ultimus`           | `[arr.length-1]`    | no       |
| `longitudo`         | `.length`           | no       |
| `mappata(fn)`       | `.map(fn)`          | no       |
| `filtrata(fn)`      | `.filter(fn)`       | no       |
| `reducta(fn, init)` | `.reduce(fn, init)` | no       |
| `inveni(fn)`        | `.find(fn)`         | no       |
| `continet(x)`       | `.includes(x)`      | no       |
| `coniunge(sep)`     | `.join(sep)`        | no       |

### Collection Methods (tabula)

| Latin        | JavaScript   | Mutates? |
| ------------ | ------------ | -------- |
| `pone(k, v)` | `.set(k, v)` | yes      |
| `accipe(k)`  | `.get(k)`    | no       |
| `habet(k)`   | `.has(k)`    | no       |
| `dele(k)`    | `.delete(k)` | yes      |
| `claves()`   | `.keys()`    | no       |
| `valores()`  | `.values()`  | no       |

### Collection Methods (copia)

| Latin                | JavaScript       | Mutates? |
| -------------------- | ---------------- | -------- |
| `adde(x)`            | `.add(x)`        | yes      |
| `habet(x)`           | `.has(x)`        | no       |
| `dele(x)`            | `.delete(x)`     | yes      |
| `unio(other)`        | set union        | no       |
| `intersectio(other)` | set intersection | no       |

---

## Complete Program Example

```fab
// A simple API handler demonstrating multiple features
ex hono importa Hono, Context

genus UserService {
    privatus textus baseUrl

    functio creo(textus url) {
        ego.baseUrl = url
    }

    futura functio fetch(numerus id) fiet User? {
        fixum response = cede ego.client.get(`${ego.baseUrl}/users/${id}`)

        custodi {
            si response.status !== 200 { redde nihil }
        }

        redde response.json() qua User
    }

    futura functio fetchAll() fiet lista<User> {
        fixum response = cede ego.client.get(`${ego.baseUrl}/users`)
        fixum users = cede response.json() qua User[]

        redde users.filtrata(pro u: u.active)
    }
}

fixum app = novum Hono()

app.get("/users/:id", futura functio(Context ctx) {
    fixum id = ctx.param("id") qua numerus
    fixum service = novum UserService("https://api.example.com")
    fixum user = cede service.fetch(id)

    si user === nihil {
        redde ctx.json({ error: "Not found" }, 404)
    }

    redde ctx.json(user)
})
```

---
