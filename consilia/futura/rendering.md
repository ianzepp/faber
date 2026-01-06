---
status: partial
targets: [ts, py]
note: `nexum` keyword and getter/setter codegen done; batching/diffing are library concerns
updated: 2024-12
---

# Rendering Model

Faber provides reactive component primitives. Rendering vocabulary is external.

## Implementation Status

| Feature                 | Status       | Notes                                  |
| ----------------------- | ------------ | -------------------------------------- |
| `nexum` keyword         | Done         | Parsed as field modifier               |
| Getter/setter codegen   | Done         | Private backing field + accessors      |
| `__invalidate?.()` hook | Done         | Called on setter, library handles rest |
| `pingo()` method        | Convention   | No special compiler treatment needed   |
| `deleo()` method        | Convention   | No special compiler treatment needed   |
| Batching                | **Not done** | Library concern                        |
| Diffing                 | **Not done** | Library concern                        |
| Async `pingo()`         | **Not done** | Open question                          |

## Design Philosophy

Faber is a compiler, like Svelte. But unlike Svelte, Faber doesn't prescribe what you render _to_. The same reactive component model works for:

- Web apps (DOM)
- Games (sprites, polygons, canvas)
- Terminal UIs (text, ANSI)
- Native apps (platform widgets)
- PDFs, SVGs, anything

**Faber provides:** Reactive state tracking and re-render triggers.

**Libraries provide:** Rendering primitives for specific targets.

## Core Additions

### `nexum` - Reactive Binding

A new keyword alongside `fixum` (immutable) and `varia` (mutable):

```
genus Counter {
    nexum count = 0        // Reactive - changes trigger pingo()
    varia cache = []       // Mutable but not reactive
    fixum id = "abc"       // Immutable
}
```

| Keyword | Mutable | Reactive | Use case                     |
| ------- | ------- | -------- | ---------------------------- |
| `fixum` | No      | No       | Constants, props from parent |
| `varia` | Yes     | No       | Internal state, caches       |
| `nexum` | Yes     | Yes      | UI-bound state               |

**Etymology:** `nexum` = "bound, connected" - the property is bound to the rendered output.

Assignment to a `nexum` field triggers a re-render:

```
functio increment() {
    count = count + 1  // Compiler generates: update, then call pingo()
}
```

### `pingo()` - Render Method

A conventional method name, like `creo()` for construction:

```
genus Card {
    nexum nomen = ""

    functio pingo() {
        // Return renderable content
    }
}
```

**Etymology:** `pingo, pingere` = "to paint, draw, depict" - root of "picture," "pigment."

The compiler recognizes: "this genus has `nexum` fields and a `pingo()` method - it's a reactive component."

## Render Targets

What `pingo()` returns depends on the target and libraries used.

### Web / DOM

```
ex dominus importa div, h1, span, button

genus Card {
    nexum nomen = ""
    nexum expanded = falsum

    functio toggle() {
        expanded = non expanded
    }

    functio pingo() {
        redde div(classis: "card") {
            h1 { ego.nomen }
            si expanded {
                span { "Details here" }
            }
            button(click: ego.toggle) { "Toggle" }
        }
    }
}
```

### Games / Canvas

```
ex ludus importa sprite, rect, text

genus Player {
    nexum x = 0
    nexum y = 0
    nexum health = 100

    functio pingo() {
        redde sprite(imago: "player.png", x: ego.x, y: ego.y) {
            rect(x: 0, y: -10, latitudo: ego.health, altitudo: 5, color: "green")
        }
    }
}
```

### Terminal UI

```
ex terminus importa box, text, list

genus Menu {
    nexum selected = 0
    nexum options = ["New", "Open", "Save", "Quit"]

    functio pingo() {
        redde box(titulus: "Menu") {
            list(items: ego.options, index: ego.selected)
        }
    }
}
```

## Lifecycle

Existing conventions apply:

| Method    | When called                             |
| --------- | --------------------------------------- |
| `creo()`  | After construction, before first render |
| `pingo()` | On mount and after any `nexum` change   |
| `deleo()` | On unmount/destruction                  |

## What Stays Outside Faber

These are library/framework concerns, not language features:

- **Event handling** - Libraries interpret function props as events (e.g., `click: ego.handler`)
- **Message passing** - If needed, a full subsystem or external library
- Routing
- Server-side rendering
- Hydration
- State management patterns
- Dev server / hot reload
- Build tooling

A "kit" (like SvelteKit) can be built _with_ Faber, not _in_ Faber.

## Compilation

The compiler generates minimal reactive hooks. Libraries handle scheduling and reconciliation.

```
// Faber source
genus Counter {
    nexum numerus count: 0

    functio increment() {
        ego.count = ego.count + 1
    }
}
```

```javascript
// Actual compiled output (TypeScript target)
class Counter {
    #count = 0;

    get count(): number { return this.#count; }
    set count(v: number) {
        this.#count = v;
        this.__invalidate?.('count');
    }

    constructor(overrides: { count?: number } = {}) {
        if (overrides.count !== undefined) this.count = overrides.count;
    }

    increment() {
        this.count = this.count + 1;
    }
}
```

Libraries (like `dominus` for DOM) attach the `__invalidate` handler to schedule updates:

```javascript
// Library attaches invalidation handler
counter.__invalidate = field => {
    if (!dirty) {
        dirty = true;
        queueMicrotask(() => {
            dirty = false;
            reconcile(counter.pingo());
        });
    }
};
```

## Open Questions

1. **Batching** - Multiple `nexum` changes in one function should batch into one `pingo()` call.
2. **Diffing strategy** - Full replace vs surgical updates? Library concern or compiler concern?
3. **Async rendering** - Should `pingo()` be allowed to be `futura`?
4. **Children/slots** - How do components compose? Pass children as props?
5. **Scoped styles** - Is `stilus` a genus-level block, or external CSS?
6. **Conditional `pingo()`** - What if a genus sometimes renders, sometimes doesn't?

## Summary

| Concept             | Faber provides       | Libraries provide     |
| ------------------- | -------------------- | --------------------- |
| Reactive state      | `nexum` keyword      | -                     |
| Render trigger      | `pingo()` convention | -                     |
| Lifecycle           | `creo()`, `deleo()`  | -                     |
| DOM primitives      | -                    | `dominus` or similar  |
| Game primitives     | -                    | `ludus` or similar    |
| Terminal primitives | -                    | `terminus` or similar |
| Routing, SSR, etc.  | -                    | "Kit" frameworks      |
