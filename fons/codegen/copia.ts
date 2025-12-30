/**
 * Copia Method Registry - Unified translations for Latin set methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (all targets)
 *
 * ARCHITECTURE
 * ============
 * This module defines translations for copia<T> (HashSet) methods across all targets.
 * Single source of truth for method behavior, reducing N methods x M targets sprawl.
 *
 * LATIN ETYMOLOGY
 * ===============
 * copia: "abundance, supply" - a collection of resources.
 * Feminine noun, so participle endings use -a (e.g., unita, not unitus).
 *
 * ZIG SPECIFICS
 * =============
 * Zig has no native Set type. We implement copia as HashMap(T, void).
 * - copia<textus> maps to CopiaTextus (StringHashMap wrapper)
 * - copia<T> maps to CopiaAuto(T) (AutoHashMap wrapper)
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
export interface CopiaMethod {
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
 * Registry of Latin set methods with translations for all targets.
 *
 * Allocator categories (Zig):
 * - Adding (adde): Yes - may need to resize
 * - Reading (habet, longitudo, vacua): No
 * - Removing (dele, purga): No - doesn't allocate
 * - Iteration (valores): No
 * - Set operations (unio, intersectio, etc.): Yes - return new sets
 */
export const COPIA_METHODS: Record<string, CopiaMethod> = {
    // =========================================================================
    // CORE OPERATIONS
    // =========================================================================

    /** Add element (mutates) */
    adde: {
        mutates: true,
        needsAlloc: true,
        ts: 'add',
        py: 'add',
        rs: 'insert',
        cpp: 'insert',
        zig: (obj, args, curator) => `${obj}.adde(${curator}, ${args[0]})`,
    },

    /** Check if element exists */
    habet: {
        mutates: false,
        needsAlloc: false,
        ts: 'has',
        py: (obj, args) => `(${args[0]} in ${obj})`,
        rs: (obj, args) => `${obj}.contains(&${args[0]})`,
        cpp: (obj, args) => `${obj}.contains(${args[0]})`,
        zig: (obj, args) => `${obj}.habet(${args[0]})`,
    },

    /** Delete element (mutates) */
    dele: {
        mutates: true,
        needsAlloc: false,
        ts: 'delete',
        py: 'discard',
        rs: (obj, args) => `${obj}.remove(&${args[0]})`,
        cpp: (obj, args) => `${obj}.erase(${args[0]})`,
        zig: (obj, args) => `_ = ${obj}.dele(${args[0]})`,
    },

    /** Get size */
    longitudo: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.size`,
        py: obj => `len(${obj})`,
        rs: obj => `${obj}.len()`,
        cpp: obj => `${obj}.size()`,
        zig: obj => `${obj}.longitudo()`,
    },

    /** Check if empty */
    vacua: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `${obj}.size === 0`,
        py: obj => `len(${obj}) == 0`,
        rs: obj => `${obj}.is_empty()`,
        cpp: obj => `${obj}.empty()`,
        zig: obj => `${obj}.vacua()`,
    },

    /** Clear all elements (mutates) */
    purga: {
        mutates: true,
        needsAlloc: false,
        ts: 'clear',
        py: 'clear',
        rs: 'clear',
        cpp: 'clear',
        zig: obj => `${obj}.purga()`,
    },

    // =========================================================================
    // ITERATION
    // =========================================================================

    /** Get values iterator (for sets, iterates elements) */
    valores: {
        mutates: false,
        needsAlloc: false,
        ts: 'values',
        py: obj => `iter(${obj})`,
        rs: obj => `${obj}.iter()`,
        cpp: obj => `${obj}`,
        zig: obj => `${obj}.valores()`,
    },

    /** Iterate with callback */
    perambula: {
        mutates: false,
        needsAlloc: false,
        ts: 'forEach',
        py: (obj, args) => `[(${args[0]})(x) for x in ${obj}]`,
        rs: (obj, args) => `${obj}.iter().for_each(${args[0]})`,
        cpp: (obj, args) => `std::ranges::for_each(${obj}, ${args[0]})`,
        zig: () => `@compileError("perambula not implemented for Zig - use 'ex set.valores() pro item { ... }' loop")`,
    },

    // =========================================================================
    // SET OPERATIONS
    // =========================================================================

    /** Union: A U B (returns new set) */
    unio: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Set([...${obj}, ...${args[0]}])`,
        py: (obj, args) => `${obj} | ${args[0]}`,
        rs: (obj, args) => `${obj}.union(&${args[0]}).cloned().collect::<HashSet<_>>()`,
        cpp: (obj, args) => `[&]{ auto s = ${obj}; s.insert(${args[0]}.begin(), ${args[0]}.end()); return s; }()`,
        zig: () => `@compileError("unio not implemented for Zig - use explicit loop to merge sets")`,
    },

    /** Intersection: A n B (returns new set) */
    intersectio: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Set([...${obj}].filter(x => ${args[0]}.has(x)))`,
        py: (obj, args) => `${obj} & ${args[0]}`,
        rs: (obj, args) => `${obj}.intersection(&${args[0]}).cloned().collect::<HashSet<_>>()`,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (${args[0]}.contains(x)) r.insert(x); return r; }()`,
        zig: () => `@compileError("intersectio not implemented for Zig - use explicit loop")`,
    },

    /** Difference: A - B (returns new set) */
    differentia: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Set([...${obj}].filter(x => !${args[0]}.has(x)))`,
        py: (obj, args) => `${obj} - ${args[0]}`,
        rs: (obj, args) => `${obj}.difference(&${args[0]}).cloned().collect::<HashSet<_>>()`,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (!${args[0]}.contains(x)) r.insert(x); return r; }()`,
        zig: () => `@compileError("differentia not implemented for Zig - use explicit loop")`,
    },

    /** Symmetric difference: (A - B) U (B - A) (returns new set) */
    symmetrica: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Set([...[...${obj}].filter(x => !${args[0]}.has(x)), ...[...${args[0]}].filter(x => !${obj}.has(x))])`,
        py: (obj, args) => `${obj} ^ ${args[0]}`,
        rs: (obj, args) => `${obj}.symmetric_difference(&${args[0]}).cloned().collect::<HashSet<_>>()`,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (!${args[0]}.contains(x)) r.insert(x); for (auto& x : ${args[0]}) if (!${obj}.contains(x)) r.insert(x); return r; }()`,
        zig: () => `@compileError("symmetrica not implemented for Zig - use explicit loop")`,
    },

    // =========================================================================
    // PREDICATES
    // =========================================================================

    /** Is subset of other */
    subcopia: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `[...${obj}].every(x => ${args[0]}.has(x))`,
        py: (obj, args) => `${obj} <= ${args[0]}`,
        rs: (obj, args) => `${obj}.is_subset(&${args[0]})`,
        cpp: (obj, args) => `std::ranges::all_of(${obj}, [&](auto& x) { return ${args[0]}.contains(x); })`,
        zig: () => `@compileError("subcopia not implemented for Zig - use explicit loop")`,
    },

    /** Is superset of other */
    supercopia: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => `[...${args[0]}].every(x => ${obj}.has(x))`,
        py: (obj, args) => `${obj} >= ${args[0]}`,
        rs: (obj, args) => `${obj}.is_superset(&${args[0]})`,
        cpp: (obj, args) => `std::ranges::all_of(${args[0]}, [&](auto& x) { return ${obj}.contains(x); })`,
        zig: () => `@compileError("supercopia not implemented for Zig - use explicit loop")`,
    },

    // =========================================================================
    // CONVERSIONS
    // =========================================================================

    /** Convert to list */
    inLista: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `[...${obj}]`,
        py: obj => `list(${obj})`,
        rs: obj => `${obj}.iter().cloned().collect::<Vec<_>>()`,
        cpp: obj => `std::vector(${obj}.begin(), ${obj}.end())`,
        zig: () => `@compileError("inLista not implemented for Zig - iterate with ex...pro into ArrayList")`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Look up a Latin method name and return its definition.
 */
export function getCopiaMethod(name: string): CopiaMethod | undefined {
    return COPIA_METHODS[name];
}

/**
 * Check if a method name is a known copia method.
 */
export function isCopiaMethod(name: string): boolean {
    return name in COPIA_METHODS;
}
