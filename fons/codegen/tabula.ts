/**
 * Tabula Method Registry - Unified translations for Latin map methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (all targets)
 *
 * ARCHITECTURE
 * ============
 * This module defines translations for tabula<K,V> methods across all targets.
 * Single source of truth for method behavior, reducing N methods x M targets sprawl.
 *
 * LATIN ETYMOLOGY
 * ===============
 * tabula: "board, tablet, table" - a writing surface with entries.
 * Feminine noun, so participle endings use -a (e.g., inversa, not inversus).
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
export interface TabulaMethod {
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
 * Registry of Latin map methods with translations for all targets.
 *
 * Allocator categories (Zig):
 * - Growing (pone): Yes - may need to resize
 * - Reading (accipe, habet, longitudo): No
 * - Shrinking (dele, purga): No - doesn't allocate
 * - Iteration (claves, valores, paria): No
 */
export const TABULA_METHODS: Record<string, TabulaMethod> = {
    // =========================================================================
    // CORE OPERATIONS
    // =========================================================================

    /** Set key-value pair (mutates) */
    pone: {
        mutates: true,
        needsAlloc: true,
        ts: 'set',
        py: (obj, args) => `${obj}[${args[0]}] = ${args[1]}`,
        rs: (obj, args) => `${obj}.insert(${args[0]}, ${args[1]})`,
        cpp: (obj, args) =>
            args.length >= 2 ? `${obj}.insert_or_assign(${args[0]}, ${args.slice(1).join(', ')})` : `${obj}.insert_or_assign(${args[0]})`,
        zig: (obj, args, curator) => {
            if (args.length >= 2) {
                return `${obj}.pone(${curator}, ${args[0]}, ${args[1]})`;
            }
            return `@compileError("pone requires two arguments: key, value")`;
        },
    },

    /** Get value by key */
    accipe: {
        mutates: false,
        needsAlloc: false,
        ts: 'get',
        py: (obj, args) => `${obj}.get(${args[0]})`,
        rs: (obj, args) => `${obj}.get(&${args[0]})`,
        cpp: (obj, args) => `${obj}.at(${args[0]})`,
        zig: (obj, args) => `${obj}.accipe(${args[0]})`,
    },

    /** Get value or return default */
    accipeAut: {
        mutates: false,
        needsAlloc: false,
        ts: (obj, args) => (args.length >= 2 ? `(${obj}.get(${args[0]}) ?? ${args[1]})` : `${obj}.get(${args[0]})`),
        py: (obj, args) => (args.length >= 2 ? `${obj}.get(${args[0]}, ${args[1]})` : `${obj}.get(${args[0]})`),
        rs: (obj, args) => (args.length >= 2 ? `${obj}.get(&${args[0]}).cloned().unwrap_or(${args[1]})` : `${obj}.get(&${args[0]})`),
        cpp: (obj, args) => (args.length >= 2 ? `(${obj}.contains(${args[0]}) ? ${obj}.at(${args[0]}) : ${args[1]})` : `${obj}.at(${args[0]})`),
        zig: (obj, args) => (args.length >= 2 ? `${obj}.accipeAut(${args[0]}, ${args[1]})` : `${obj}.accipe(${args[0]})`),
    },

    /** Check if key exists */
    habet: {
        mutates: false,
        needsAlloc: false,
        ts: 'has',
        py: (obj, args) => `(${args[0]} in ${obj})`,
        rs: (obj, args) => `${obj}.contains_key(&${args[0]})`,
        cpp: (obj, args) => `${obj}.contains(${args[0]})`,
        zig: (obj, args) => `${obj}.habet(${args[0]})`,
    },

    /** Delete key (mutates) */
    dele: {
        mutates: true,
        needsAlloc: false,
        ts: 'delete',
        py: (obj, args) => `del ${obj}[${args[0]}]`,
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

    /** Clear all entries (mutates) */
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

    /** Get keys iterator */
    claves: {
        mutates: false,
        needsAlloc: false,
        ts: 'keys',
        py: 'keys',
        rs: obj => `${obj}.keys()`,
        cpp: obj => `${obj} | std::views::keys`,
        zig: obj => `${obj}.claves()`,
    },

    /** Get values iterator */
    valores: {
        mutates: false,
        needsAlloc: false,
        ts: 'values',
        py: 'values',
        rs: obj => `${obj}.values()`,
        cpp: obj => `${obj} | std::views::values`,
        zig: obj => `${obj}.valores()`,
    },

    /** Get entries iterator */
    paria: {
        mutates: false,
        needsAlloc: false,
        ts: 'entries',
        py: 'items',
        rs: obj => `${obj}.iter()`,
        cpp: obj => `${obj}`,
        zig: obj => `${obj}.paria()`,
    },

    // =========================================================================
    // ADVANCED OPERATIONS
    // =========================================================================

    /** Keep only specified keys (returns new map) */
    selige: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Map([...${obj}].filter(([k]) => [${args.join(', ')}].includes(k)))`,
        py: (obj, args) => `{k: v for k, v in ${obj}.items() if k in [${args.join(', ')}]}`,
        zig: () => `@compileError("selige not implemented for Zig - use explicit loop")`,
    },

    /** Remove specified keys (returns new map) */
    omitte: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Map([...${obj}].filter(([k]) => ![${args.join(', ')}].includes(k)))`,
        py: (obj, args) => `{k: v for k, v in ${obj}.items() if k not in [${args.join(', ')}]}`,
        zig: () => `@compileError("omitte not implemented for Zig - use explicit loop")`,
    },

    /** Merge maps (returns new map) */
    confla: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Map([...${obj}, ...${args[0]}])`,
        py: (obj, args) => `{**${obj}, **${args[0]}}`,
        rs: (obj, args) => `{ let mut m = ${obj}.clone(); m.extend(${args[0]}.iter().map(|(k, v)| (k.clone(), v.clone()))); m }`,
        cpp: (obj, args) => `[&]{ auto r = ${obj}; for (auto& [k, v] : ${args[0]}) r[k] = v; return r; }()`,
        zig: () => `@compileError("confla not implemented for Zig - use explicit loop")`,
    },

    /** Swap keys and values (returns new map) */
    inversa: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `new Map([...${obj}].map(([k, v]) => [v, k]))`,
        py: obj => `{v: k for k, v in ${obj}.items()}`,
        zig: () => `@compileError("inversa not implemented for Zig - use explicit loop")`,
    },

    /** Transform values (returns new map) */
    mappaValores: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Map([...${obj}].map(([k, v]) => [k, (${args[0]})(v)]))`,
        py: (obj, args) => `{k: (${args[0]})(v) for k, v in ${obj}.items()}`,
        zig: () => `@compileError("mappaValores not implemented for Zig - use explicit loop")`,
    },

    /** Transform keys (returns new map) */
    mappaClaves: {
        mutates: false,
        needsAlloc: true,
        ts: (obj, args) => `new Map([...${obj}].map(([k, v]) => [(${args[0]})(k), v]))`,
        py: (obj, args) => `{(${args[0]})(k): v for k, v in ${obj}.items()}`,
        zig: () => `@compileError("mappaClaves not implemented for Zig - use explicit loop")`,
    },

    // =========================================================================
    // CONVERSIONS
    // =========================================================================

    /** Convert to list of [key, value] pairs */
    inLista: {
        mutates: false,
        needsAlloc: true,
        ts: obj => `[...${obj}]`,
        py: obj => `list(${obj}.items())`,
        rs: obj => `${obj}.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()`,
        cpp: obj => `std::vector(${obj}.begin(), ${obj}.end())`,
        zig: () => `@compileError("inLista not implemented for Zig - iterate with ex...pro")`,
    },

    /** Convert to object */
    inObjectum: {
        mutates: false,
        needsAlloc: false,
        ts: obj => `Object.fromEntries(${obj})`,
        py: obj => `dict(${obj})`,
        zig: () => `@compileError("inObjectum not implemented for Zig - Zig has no object type")`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Look up a Latin method name and return its definition.
 */
export function getTabulaMethod(name: string): TabulaMethod | undefined {
    return TABULA_METHODS[name];
}

/**
 * Check if a method name is a known tabula method.
 */
export function isTabulaMethod(name: string): boolean {
    return name in TABULA_METHODS;
}
