/**
 * Lista Method Registry - Unified translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (all targets)
 *
 * ARCHITECTURE
 * ============
 * This module defines translations for lista<T> methods across all targets.
 * Single source of truth for method behavior, reducing N methods x M targets sprawl.
 *
 * LATIN VERB CONJUGATION
 * ======================
 * Latin verb forms encode mutability:
 *
 * |           | Mutates (in-place) | Returns New (copy) |
 * |-----------|--------------------|--------------------|
 * | Sync      | adde (imperative)  | addita (participle)|
 * | Async     | addet (future)     | additura (fut.part)|
 *
 * The feminine endings (-a, -ura) agree with lista/tabula/copia.
 */

// =============================================================================
// TYPES
// =============================================================================

/** Generator function for Zig - receives curator (allocator) */
export type ZigGenerator = (obj: string, args: string[], curator: string) => string;

/** Generator function for other targets */
export type TsGenerator = (obj: string, args: string[]) => string;
export type PyGenerator = (obj: string, args: string[]) => string;
export type RsGenerator = (obj: string, args: string[]) => string;
export type CppGenerator = (obj: string, args: string[]) => string;

/**
 * Describes how to translate a Latin method across all targets.
 */
export interface ListaMethod {
    /** True if method mutates the collection in place */
    mutates: boolean;

    /** True if method needs allocator (Zig: growing, returning new collection) */
    needsAlloc: boolean;

    /** TypeScript translation */
    ts?: string | TsGenerator;

    /** Python translation */
    py?: string | PyGenerator;

    /** Rust translation */
    rs?: string | RsGenerator;

    /** C++ translation */
    cpp?: string | CppGenerator;

    /** Zig translation */
    zig?: string | ZigGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with translations for all targets.
 *
 * Allocator categories (Zig):
 * - Construction: Yes (init, fromItems, clone)
 * - Destruction: Yes (deinit)
 * - Growing: Yes (adde, praepone)
 * - Shrinking: No (remove, decapita, purga)
 * - Reading: No (primus, longitudo, continet)
 * - Returns new collection: Yes (addita, filtrata, mappata, inversa)
 * - In-place (no resize): No (ordina, inverte)
 * - Aggregation: No (summa, reducta, minimus)
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // =========================================================================
    // ADDING ELEMENTS
    // =========================================================================

    /** Add element to end (mutates) */
    adde: {
        mutates: true,
        needsAlloc: true,
        ts: 'push',
        py: 'append',
        rs: 'push',
        cpp: 'push_back',
        zig: (obj, args, curator) => `${obj}.adde(${curator}, ${args[0]})`,
    },

    /** Add element to end (returns new list) */
    addita: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `[...${obj}, ${args.join(', ')}]`,
        py: (obj, args) => `[*${obj}, ${args.join(', ')}]`,
        rs: (obj, args) => `{ let mut v = ${obj}.clone(); v.push(${args[0]}); v }`,
        cpp: (obj, args) => `[&]{ auto v = ${obj}; v.push_back(${args.join(', ')}); return v; }()`,
        zig: (obj, args, curator) => `${obj}.addita(${curator}, ${args[0]})`,
    },

    /** Add element to start (mutates) */
    praepone: {
        mutates: true,
        needsAlloc: true,
        ts: 'unshift',
        py: (obj, args) => `${obj}.insert(0, ${args[0]})`,
        rs: (obj, args) => `${obj}.insert(0, ${args[0]})`,
        cpp: (obj, args) => `${obj}.insert(${obj}.begin(), ${args[0]})`,
        zig: (obj, args, curator) => `${obj}.praepone(${curator}, ${args[0]})`,
    },

    /** Add element to start (returns new list) */
    praeposita: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `[${args.join(', ')}, ...${obj}]`,
        py: (obj, args) => `[${args.join(', ')}, *${obj}]`,
        rs: (obj, args) => `{ let mut v = vec![${args[0]}]; v.extend(${obj}.iter().cloned()); v }`,
        cpp: (obj, args) => `[&]{ auto v = ${obj}; v.insert(v.begin(), ${args[0]}); return v; }()`,
        zig: (obj, args, curator) => `${obj}.praeposita(${curator}, ${args[0]})`,
    },

    // =========================================================================
    // REMOVING ELEMENTS
    // =========================================================================

    /** Remove and return last element (mutates) */
    remove: {
        mutates: true,
        needsAlloc: false,
        ts: 'pop',
        py: 'pop',
        rs: obj => `${obj}.pop()`,
        cpp: obj => `[&]{ auto v = ${obj}.back(); ${obj}.pop_back(); return v; }()`,
        zig: obj => `${obj}.remove()`,
    },

    /** Remove last element (returns new list) */
    remota: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `${obj}.slice(0, -1)`,
        py: obj => `${obj}[:-1]`,
        rs: obj => `${obj}[..${obj}.len().saturating_sub(1)].to_vec()`,
        cpp: obj => `std::vector(${obj}.begin(), ${obj}.end() - 1)`,
        zig: (obj, _args, curator) => `${obj}.remota(${curator})`,
    },

    /** Remove and return first element (mutates) */
    decapita: {
        mutates: true,
        needsAlloc: false,
        ts: 'shift',
        py: obj => `${obj}.pop(0)`,
        rs: obj => `${obj}.remove(0)`,
        cpp: obj => `[&]{ auto v = ${obj}.front(); ${obj}.erase(${obj}.begin()); return v; }()`,
        zig: obj => `${obj}.decapita()`,
    },

    /** Remove first element (returns new list) */
    decapitata: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `${obj}.slice(1)`,
        py: obj => `${obj}[1:]`,
        rs: obj => `${obj}[1..].to_vec()`,
        cpp: obj => `std::vector(${obj}.begin() + 1, ${obj}.end())`,

        zig: (obj, _args, curator) => `${obj}.decapitata(${curator})`,
    },

    /** Clear all elements (mutates) */
    purga: {
        mutates: true,
        needsAlloc: false,
        ts: obj => `${obj}.length = 0`,
        py: 'clear',
        rs: 'clear',
        cpp: 'clear',
        zig: obj => `${obj}.purga()`,
    },

    // =========================================================================
    // ACCESSING ELEMENTS
    // =========================================================================

    /** Get first element */
    primus: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}[0]`,
        py: obj => `${obj}[0]`,
        rs: obj => `${obj}.first()`,
        cpp: obj => `${obj}.front()`,
        zig: obj => `${obj}.primus()`,
    },

    /** Get last element */
    ultimus: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.at(-1)`,
        py: obj => `${obj}[-1]`,
        rs: obj => `${obj}.last()`,
        cpp: obj => `${obj}.back()`,
        zig: obj => `${obj}.ultimus()`,
    },

    /** Get element at index */
    accipe: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `${obj}[${args[0]}]`,
        py: (obj, args) => `${obj}[${args[0]}]`,
        rs: (obj, args) => `${obj}.get(${args[0]})`,
        cpp: (obj, args) => `${obj}.at(${args[0]})`,
        zig: (obj, args) => `${obj}.accipe(${args[0]})`,
    },

    /** Get raw slice for iteration (Zig-specific) */
    toSlice: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.toSlice()`,
    },

    // =========================================================================
    // PROPERTIES
    // =========================================================================

    /** Get length */
    longitudo: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.length`,
        py: obj => `len(${obj})`,
        rs: obj => `${obj}.len()`,
        cpp: obj => `${obj}.size()`,
        zig: obj => `${obj}.longitudo()`,
    },

    /** Check if empty */
    vacua: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.length === 0`,
        py: obj => `len(${obj}) == 0`,
        rs: obj => `${obj}.is_empty()`,
        cpp: obj => `${obj}.empty()`,
        zig: obj => `${obj}.vacua()`,
    },

    // =========================================================================
    // SEARCHING
    // =========================================================================

    /** Check if contains element */
    continet: {
        mutates: false,
        needsAlloc: false,
        ts: 'includes',
        py: (obj, args) => `(${args[0]} in ${obj})`,
        rs: (obj, args) => `${obj}.contains(&${args[0]})`,
        cpp: (obj, args) => `(std::find(${obj}.begin(), ${obj}.end(), ${args[0]}) != ${obj}.end())`,

        zig: (obj, args) => `${obj}.continet(${args[0]})`,
    },

    /** Find index of element */
    indiceDe: {
        mutates: false,
        needsAlloc: false,
        ts: 'indexOf',
        py: (obj, args) => `${obj}.index(${args[0]})`,
        rs: (obj, args) => `${obj}.iter().position(|e| e == &${args[0]})`,
        cpp: (obj, args) =>
            `[&]{ auto it = std::find(${obj}.begin(), ${obj}.end(), ${args[0]}); return it != ${obj}.end() ? std::distance(${obj}.begin(), it) : -1; }()`,

        zig: (obj, args) => `${obj}.indiceDe(${args[0]})`,
    },

    /** Find first element matching predicate */
    inveni: {
        mutates: false,
        needsAlloc: false,
        ts: 'find',
        py: (obj, args) => `next(filter(${args[0]}, ${obj}), None)`,
        rs: (obj, args) => `${obj}.iter().find(${args[0]})`,
        cpp: (obj, args) => `*std::find_if(${obj}.begin(), ${obj}.end(), ${args[0]})`,

        zig: (obj, args) => `${obj}.inveni(${args[0]})`,
    },

    /** Find index of first element matching predicate */
    inveniIndicem: {
        mutates: false,
        needsAlloc: false,
        ts: 'findIndex',
        py: (obj, args) => `next((i for i, x in enumerate(${obj}) if (${args[0]})(x)), -1)`,
        rs: (obj, args) => `${obj}.iter().position(${args[0]})`,
        cpp: (obj, args) =>
            `[&]{ auto it = std::find_if(${obj}.begin(), ${obj}.end(), ${args[0]}); return it != ${obj}.end() ? std::distance(${obj}.begin(), it) : -1; }()`,

        zig: (obj, args) => `${obj}.inveniIndicem(${args[0]})`,
    },

    // =========================================================================
    // PREDICATE METHODS
    // =========================================================================

    /** Check if all elements match predicate */
    omnes: {
        mutates: false,
        needsAlloc: false,
        ts: 'every',
        py: (obj, args) => `all(map(${args[0]}, ${obj}))`,
        rs: (obj, args) => `${obj}.iter().all(${args[0]})`,
        cpp: (obj, args) => `std::ranges::all_of(${obj}, ${args[0]})`,

        zig: (obj, args) => `${obj}.omnes(${args[0]})`,
    },

    /** Check if any element matches predicate */
    aliquis: {
        mutates: false,
        needsAlloc: false,
        ts: 'some',
        py: (obj, args) => `any(map(${args[0]}, ${obj}))`,
        rs: (obj, args) => `${obj}.iter().any(${args[0]})`,
        cpp: (obj, args) => `std::ranges::any_of(${obj}, ${args[0]})`,

        zig: (obj, args) => `${obj}.aliquis(${args[0]})`,
    },

    // =========================================================================
    // FUNCTIONAL METHODS (return new list)
    // =========================================================================

    /** Filter elements (returns new list) */
    filtrata: {
        mutates: false,
        needsAlloc: true,
        ts: 'filter',
        py: (obj, args) => `list(filter(${args[0]}, ${obj}))`,
        rs: (obj, args) => `${obj}.iter().filter(${args[0]}).cloned().collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::filter(${args[0]}) | std::ranges::to<std::vector>())`,
        zig: (obj, args, curator) => `${obj}.filtrata(${curator}, ${args[0]})`,
    },

    /** Map elements (returns new list) */
    mappata: {
        mutates: false,
        needsAlloc: true,
        ts: 'map',
        py: (obj, args) => `list(map(${args[0]}, ${obj}))`,
        rs: (obj, args) => `${obj}.iter().map(${args[0]}).collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::transform(${args[0]}) | std::ranges::to<std::vector>())`,
        zig: (obj, args, curator) => `${obj}.mappata(${curator}, ${args[0]})`,
    },

    /** Reduce to single value */
    reducta: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.reduce(${args[0]}, ${args[1]})`;
            }
            return `${obj}.reduce(${args[0]})`;
        },
        py: (obj, args) => {
            if (args.length >= 2) {
                return `functools.reduce(${args[0]}, ${obj}, ${args[1]})`;
            }
            return `functools.reduce(${args[0]}, ${obj})`;
        },
        rs: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.iter().fold(${args[1]}, ${args[0]})`;
            }
            return `${obj}.iter().fold(Default::default(), ${args[0]})`;
        },
        cpp: (obj, args) => {
            if (args.length >= 2) {
                return `std::ranges::fold_left(${obj}, ${args[0]}, ${args[1]})`;
            }
            return `std::ranges::fold_left(${obj}, ${args[0]})`;
        },
        zig: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.reducta(${args[0]}, ${args[1]})`;
            }
            return `${obj}.reducta(${args[0]}, 0)`;
        },
    },

    /** Flat map (map + flatten one level) */
    explanata: {
        mutates: false,
        needsAlloc: true,
        ts: 'flatMap',
        py: (obj, args) => `[y for x in ${obj} for y in (${args[0]})(x)]`,
        rs: (obj, args) => `${obj}.iter().flat_map(${args[0]}).collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::transform(${args[0]}) | std::views::join | std::ranges::to<std::vector>())`,
    },

    /** Flatten one level */
    plana: {
        mutates: false,
        needsAlloc: true,
        ts: 'flat',
        py: obj => `[y for x in ${obj} for y in x]`,
        rs: obj => `${obj}.iter().flatten().cloned().collect::<Vec<_>>()`,
        cpp: obj => `(${obj} | std::views::join | std::ranges::to<std::vector>())`,
    },

    /** Reverse (returns new list) */
    inversa: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `[...${obj}].reverse()`,
        py: obj => `${obj}[::-1]`,
        rs: obj => `${obj}.iter().rev().cloned().collect::<Vec<_>>()`,
        cpp: obj => `(${obj} | std::views::reverse | std::ranges::to<std::vector>())`,
        zig: (obj, _args, curator) => `${obj}.inversa(${curator})`,
    },

    /** Sort (returns new list) */
    ordinata: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => (args.length > 0 ? `[...${obj}].sort(${args[0]})` : `[...${obj}].sort()`),
        py: (obj, args) => (args.length > 0 ? `sorted(${obj}, key=${args[0]})` : `sorted(${obj})`),
        rs: (obj, args) =>
            args.length > 0 ? `{ let mut v = ${obj}.clone(); v.sort_by(${args[0]}); v }` : `{ let mut v = ${obj}.clone(); v.sort(); v }`,
        cpp: (obj, args) =>
            args.length > 0
                ? `[&]{ auto v = ${obj}; std::ranges::sort(v, ${args[0]}); return v; }()`
                : `[&]{ auto v = ${obj}; std::ranges::sort(v); return v; }()`,
        zig: (obj, _args, curator) => `${obj}.ordinata(${curator})`,
    },

    /** Slice - take elements from start to end */
    sectio: {
        mutates: false,
        needsAlloc: true,
        ts: 'slice',
        py: (obj, args) => (args.length >= 2 ? `${obj}[${args[0]}:${args[1]}]` : `${obj}[${args[0]}:]`),
        rs: (obj, args) => (args.length >= 2 ? `${obj}[${args[0]}..${args[1]}].to_vec()` : `${obj}[${args[0]}..].to_vec()`),
        cpp: (obj, args) =>
            args.length >= 2
                ? `std::vector(${obj}.begin() + ${args[0]}, ${obj}.begin() + ${args[1]})`
                : `std::vector(${obj}.begin() + ${args[0]}, ${obj}.end())`,
        zig: (obj, args, curator) =>
            args.length >= 2 ? `${obj}.sectio(${curator}, ${args[0]}, ${args[1]})` : `${obj}.sectio(${curator}, ${args[0]}, ${obj}.longitudo())`,
    },

    /** Take first n elements */
    prima: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `${obj}.slice(0, ${args[0]})`,
        py: (obj, args) => `${obj}[:${args[0]}]`,
        rs: (obj, args) => `${obj}.iter().take(${args[0]}).cloned().collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::take(${args[0]}) | std::ranges::to<std::vector>())`,
        zig: (obj, args, curator) => `${obj}.prima(${curator}, ${args[0]})`,
    },

    /** Take last n elements */
    ultima: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `${obj}.slice(-${args[0]})`,
        py: (obj, args) => `${obj}[-${args[0]}:]`,
        rs: (obj, args) => `${obj}.iter().rev().take(${args[0]}).cloned().collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::drop(${obj}.size() - ${args[0]}) | std::ranges::to<std::vector>())`,
        zig: (obj, args, curator) => `${obj}.ultima(${curator}, ${args[0]})`,
    },

    /** Skip first n elements */
    omitte: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `${obj}.slice(${args[0]})`,
        py: (obj, args) => `${obj}[${args[0]}:]`,
        rs: (obj, args) => `${obj}.iter().skip(${args[0]}).cloned().collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::drop(${args[0]}) | std::ranges::to<std::vector>())`,
        zig: (obj, args, curator) => `${obj}.omitte(${curator}, ${args[0]})`,
    },

    // =========================================================================
    // MUTATING OPERATIONS
    // =========================================================================

    /** Filter in place (mutates) */
    filtra: {
        mutates: true,
        needsAlloc: false,
        ts: (obj, args) => `(() => { for (let i = ${obj}.length - 1; i >= 0; i--) { if (!(${args[0]})(${obj}[i])) ${obj}.splice(i, 1); } })()`,
        py: (obj, args) => `${obj}[:] = [x for x in ${obj} if (${args[0]})(x)]`,
        cpp: (obj, args) => `${obj}.erase(std::remove_if(${obj}.begin(), ${obj}.end(), [&](auto& x) { return !(${args[0]})(x); }), ${obj}.end())`,
    },

    /** Sort in place (mutates) */
    ordina: {
        mutates: true,
        needsAlloc: false,
        ts: 'sort',
        py: (obj, args) => (args.length > 0 ? `${obj}.sort(key=${args[0]})` : `${obj}.sort()`),
        rs: (obj, args) => (args.length > 0 ? `${obj}.sort_by(${args[0]})` : `${obj}.sort()`),
        cpp: (obj, args) => (args.length > 0 ? `std::ranges::sort(${obj}, ${args[0]})` : `std::ranges::sort(${obj})`),
        zig: obj => `${obj}.ordina()`,
    },

    /** Reverse in place (mutates) */
    inverte: {
        mutates: true,
        needsAlloc: false,
        ts: 'reverse',
        py: obj => `${obj}.reverse()`,
        rs: obj => `${obj}.reverse()`,
        cpp: obj => `std::ranges::reverse(${obj})`,
        zig: obj => `${obj}.inverte()`,
    },

    // =========================================================================
    // ITERATION
    // =========================================================================

    /** Iterate with callback */
    perambula: {
        mutates: false,
        needsAlloc: false,
        ts: 'forEach',
        py: (obj, args) => `[(${args[0]})(x) for x in ${obj}]`,
        rs: (obj, args) => `${obj}.iter().for_each(${args[0]})`,
        cpp: (obj, args) => `std::ranges::for_each(${obj}, ${args[0]})`,
        zig: (obj, args) => `${obj}.perambula(${args[0]})`,
    },

    /** Join elements to string */
    coniunge: {
        mutates: false,
        needsAlloc: true,
        ts: 'join',
        py: (obj, args) => (args.length > 0 ? `${args[0]}.join(${obj})` : `"".join(${obj})`),
        rs: (obj, args) => `${obj}.join(${args[0]})`,
        cpp: (obj, args) => {
            const sep = args.length > 0 ? args[0] : '""';
            return `[&]{ std::string r; for (size_t i = 0; i < ${obj}.size(); ++i) { if (i > 0) r += ${sep}; r += ${obj}[i]; } return r; }()`;
        },
        zig: () => `@compileError("coniunge not implemented for Zig - string joining requires allocator and format")`,
    },

    // =========================================================================
    // AGGREGATION
    // =========================================================================

    /** Sum of numeric elements */
    summa: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.reduce((a, b) => a + b, 0)`,
        py: obj => `sum(${obj})`,
        rs: obj => `${obj}.iter().sum::<i64>()`,
        cpp: obj => `std::accumulate(${obj}.begin(), ${obj}.end(), decltype(${obj}[0]){})`,
        zig: obj => `${obj}.summa()`,
    },

    /** Average of numeric elements */
    medium: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `(${obj}.reduce((a, b) => a + b, 0) / ${obj}.length)`,
        py: obj => `(sum(${obj}) / len(${obj}))`,
        rs: obj => `(${obj}.iter().sum::<i64>() as f64 / ${obj}.len() as f64)`,
        cpp: obj => `(std::accumulate(${obj}.begin(), ${obj}.end(), 0.0) / ${obj}.size())`,
        zig: obj => `${obj}.medium()`,
    },

    /** Minimum value */
    minimus: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `Math.min(...${obj})`,
        py: obj => `min(${obj})`,
        rs: obj => `${obj}.iter().min()`,
        cpp: obj => `*std::ranges::min_element(${obj})`,
        zig: obj => `${obj}.minimus()`,
    },

    /** Maximum value */
    maximus: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `Math.max(...${obj})`,
        py: obj => `max(${obj})`,
        rs: obj => `${obj}.iter().max()`,
        cpp: obj => `*std::ranges::max_element(${obj})`,
        zig: obj => `${obj}.maximus()`,
    },

    /** Minimum by key function */
    minimusPer: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `${obj}.reduce((min, x) => (${args[0]})(x) < (${args[0]})(min) ? x : min)`,
        rs: (obj, args) => `${obj}.iter().min_by_key(${args[0]})`,
        cpp: (obj, args) => `*std::ranges::min_element(${obj}, [&](auto& a, auto& b) { return (${args[0]})(a) < (${args[0]})(b); })`,
    },

    /** Maximum by key function */
    maximusPer: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `${obj}.reduce((max, x) => (${args[0]})(x) > (${args[0]})(max) ? x : max)`,
        rs: (obj, args) => `${obj}.iter().max_by_key(${args[0]})`,
        cpp: (obj, args) => `*std::ranges::max_element(${obj}, [&](auto& a, auto& b) { return (${args[0]})(a) < (${args[0]})(b); })`,
    },

    /** Count elements (optionally matching predicate) */
    numera: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `${obj}.filter(${args[0]}).length`,
        py: (obj, args) => `sum(1 for x in ${obj} if (${args[0]})(x))`,
        rs: (obj, args) => `${obj}.iter().filter(${args[0]}).count()`,
        cpp: (obj, args) => `std::ranges::count_if(${obj}, ${args[0]})`,
        zig: (obj, args) => (args.length > 0 ? `${obj}.numera(${args[0]})` : `${obj}.numera(null)`),
    },

    // =========================================================================
    // LODASH-INSPIRED METHODS
    // =========================================================================

    /** Group by key function */
    congrega: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `Object.groupBy(${obj}, ${args[0]})`,
        py: (obj, args) => `{k: list(g) for k, g in itertools.groupby(sorted(${obj}, key=${args[0]}), key=${args[0]})}`,
        cpp: (obj, args) =>
            `[&]{ std::unordered_map<decltype((${args[0]})(${obj}[0])), std::vector<decltype(${obj}[0])>> m; for (auto& x : ${obj}) m[(${args[0]})(x)].push_back(x); return m; }()`,
    },

    /** Remove duplicates */
    unica: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `[...new Set(${obj})]`,
        py: obj => `list(dict.fromkeys(${obj}))`,
        rs: obj =>
            `{ let mut seen = std::collections::HashSet::new(); ${obj}.iter().filter(|x| seen.insert((*x).clone())).cloned().collect::<Vec<_>>() }`,
        cpp: obj => `[&]{ std::unordered_set<decltype(${obj}[0])> s(${obj}.begin(), ${obj}.end()); return std::vector(s.begin(), s.end()); }()`,
    },

    /** Flatten all levels */
    planaOmnia: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `${obj}.flat(Infinity)`,
        cpp: obj => `(${obj} | std::views::join | std::ranges::to<std::vector>())`,
    },

    /** Split into chunks of size n */
    fragmenta: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) =>
            `Array.from({ length: Math.ceil(${obj}.length / ${args[0]}) }, (_, i) => ${obj}.slice(i * ${args[0]}, i * ${args[0]} + ${args[0]}))`,
        py: (obj, args) => `[${obj}[i:i+${args[0]}] for i in range(0, len(${obj}), ${args[0]})]`,
        rs: (obj, args) => `${obj}.chunks(${args[0]}).map(|c| c.to_vec()).collect::<Vec<_>>()`,
        cpp: (obj, args) => `(${obj} | std::views::chunk(${args[0]}) | std::ranges::to<std::vector<std::vector<decltype(${obj}[0])>>>())`,
    },

    /** Remove falsy values */
    densa: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `${obj}.filter(Boolean)`,
        py: obj => `[x for x in ${obj} if x]`,
        cpp: obj => `(${obj} | std::views::filter([](auto& x) { return static_cast<bool>(x); }) | std::ranges::to<std::vector>())`,
    },

    /** Partition by predicate -> [truthy, falsy] */
    partire: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `${obj}.reduce(([t, f], x) => (${args[0]})(x) ? [[...t, x], f] : [t, [...f, x]], [[], []])`,
        py: (obj, args) => `[[x for x in ${obj} if (${args[0]})(x)], [x for x in ${obj} if not (${args[0]})(x)]]`,
        rs: (obj, args) => `${obj}.iter().cloned().partition::<Vec<_>, _>(${args[0]})`,
        cpp: (obj, args) =>
            `[&]{ std::vector<decltype(${obj}[0])> t, f; for (auto& x : ${obj}) ((${args[0]})(x) ? t : f).push_back(x); return std::make_pair(t, f); }()`,
    },

    /** Shuffle (returns new list) */
    misce: {
        mutates: false,
        needsAlloc: true,
        ts: obj =>
            `(() => { const a = [...${obj}]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; })()`,
        py: obj => `random.shuffle(${obj})`,
        cpp: obj => `[&]{ auto v = ${obj}; std::random_device rd; std::mt19937 g(rd()); std::ranges::shuffle(v, g); return v; }()`,
    },

    /** Random element */
    specimen: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}[Math.floor(Math.random() * ${obj}.length)]`,
        py: obj => `random.choice(${obj})`,
        cpp: obj => `${obj}[std::random_device{}() % ${obj}.size()]`,
    },

    /** Random n elements */
    specimina: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) =>
            `(() => { const a = [...${obj}]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a.slice(0, ${args[0]}); })()`,
        py: (obj, args) => `random.sample(${obj}, ${args[0]})`,
        cpp: (obj, args) =>
            `[&]{ auto v = ${obj}; std::random_device rd; std::mt19937 g(rd()); std::ranges::shuffle(v, g); v.resize(std::min(${args[0]}, v.size())); return v; }()`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Look up a Latin method name and return its definition.
 */
export function getListaMethod(name: string): ListaMethod | undefined {
    return LISTA_METHODS[name];
}

/**
 * Check if a method name is a known lista method.
 */
export function isListaMethod(name: string): boolean {
    return name in LISTA_METHODS;
}
