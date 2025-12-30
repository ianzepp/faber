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
 * Phase 2: Zig only. Other targets added in Phase 3.
 *
 * WHY UNIFIED REGISTRY
 * ====================
 * - Single source of truth for method behavior across targets
 * - needsAlloc flag determines if codegen passes curator (allocator)
 * - Reduces N methods x M targets file sprawl
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Generator function type for Zig collection methods.
 * WHY: curator parameter is the current allocator from curatorStack.
 */
export type ZigGenerator = (obj: string, args: string[], curator: string) => string;

/**
 * Describes how to translate a Latin method.
 */
export interface TabulaMethod {
    /** True if method mutates the collection in place */
    mutates: boolean;

    /** True if method needs allocator (growing, returning new collection) */
    needsAlloc: boolean;

    /**
     * Zig translation.
     * - string: method name to delegate to stdlib
     * - function: custom code generation
     */
    zig?: string | ZigGenerator;

    // Future phases will add: ts, py, rs, cpp
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin map methods with translations.
 *
 * Allocator categories:
 * - Growing (pone): Yes - may need to resize
 * - Reading (accipe, habet, longitudo): No
 * - Shrinking (dele, purga): No - doesn't allocate
 * - Iteration (claves, valores, paria): No
 */
export const TABULA_METHODS: Record<string, TabulaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    /** Set key-value pair (mutates, needs allocator for growth) */
    pone: {
        mutates: true,
        needsAlloc: true,
        zig: (obj, args, curator) => {
            if (args.length >= 2) {
                return `${obj}.pone(${curator}, ${args[0]}, ${args[1]})`;
            }
            return `@compileError("pone requires two arguments: key, value")`;
        },
    },

    /** Get value by key (returns optional) */
    accipe: {
        mutates: false,
        needsAlloc: false,
        zig: (obj, args) => `${obj}.accipe(${args[0]})`,
    },

    /** Get value or return default */
    accipeAut: {
        mutates: false,
        needsAlloc: false,
        zig: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.accipeAut(${args[0]}, ${args[1]})`;
            }
            return `${obj}.accipe(${args[0]})`;
        },
    },

    /** Check if key exists */
    habet: {
        mutates: false,
        needsAlloc: false,
        zig: (obj, args) => `${obj}.habet(${args[0]})`,
    },

    /** Delete key (mutates) */
    dele: {
        mutates: true,
        needsAlloc: false,
        zig: (obj, args) => `_ = ${obj}.dele(${args[0]})`,
    },

    /** Get size */
    longitudo: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.longitudo()`,
    },

    /** Check if empty */
    vacua: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.vacua()`,
    },

    /** Clear all entries (mutates) */
    purga: {
        mutates: true,
        needsAlloc: false,
        zig: obj => `${obj}.purga()`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Get keys iterator */
    claves: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.claves()`,
    },

    /** Get values iterator */
    valores: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.valores()`,
    },

    /** Get entries iterator */
    paria: {
        mutates: false,
        needsAlloc: false,
        zig: obj => `${obj}.paria()`,
    },

    // -------------------------------------------------------------------------
    // NOT IMPLEMENTED - complex operations that need explicit loops in Zig
    // -------------------------------------------------------------------------

    /** Keep only specified keys */
    selige: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("selige not implemented for Zig - use explicit loop")`,
    },

    /** Remove specified keys */
    omitte: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("omitte not implemented for Zig - use explicit loop")`,
    },

    /** Merge maps */
    confla: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("confla not implemented for Zig - use explicit loop")`,
    },

    /** Swap keys and values */
    inversa: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("inversa not implemented for Zig - use explicit loop")`,
    },

    /** Transform values */
    mappaValores: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("mappaValores not implemented for Zig - use explicit loop")`,
    },

    /** Transform keys */
    mappaClaves: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("mappaClaves not implemented for Zig - use explicit loop")`,
    },

    /** Convert to list of entries */
    inLista: {
        mutates: false,
        needsAlloc: true,
        zig: () => `@compileError("inLista not implemented for Zig - iterate with ex...pro")`,
    },

    /** Convert to object */
    inObjectum: {
        mutates: false,
        needsAlloc: false,
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
