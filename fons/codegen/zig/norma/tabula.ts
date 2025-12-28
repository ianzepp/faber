/**
 * Tabula Method Registry - Zig translations for Latin map methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Zig translations for tabula<K,V> (HashMap) methods.
 * Zig's HashMap requires an allocator for most operations.
 *
 * ZIG SPECIFICS
 * =============
 * - tabula<textus, V> maps to std.StringHashMap(V)
 * - tabula<K, V> maps to std.AutoHashMap(K, V)
 * - Operations need allocator: map.put(alloc, key, value)
 * - get() returns optional: map.get(key) orelse default
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: Zig code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Generator function type for Zig collection methods.
 *
 * WHY: The curator parameter allows methods to use the correct allocator.
 * WHY: The args parameter is a string[] (not a joined string) to preserve
 *      argument boundaries for multi-parameter lambdas.
 */
export type ZigGenerator = (obj: string, args: string[], curator: string) => string;

export interface TabulaMethod {
    latin: string;
    mutates: boolean;
    zig: string | ZigGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

export const TABULA_METHODS: Record<string, TabulaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    /** Set key-value pair (mutates, needs allocator) */
    pone: {
        latin: 'pone',
        mutates: true,
        // WHY: put() can fail on OOM
        zig: (obj, args, curator) => {
            // WHY: args is now an array, so direct access
            if (args.length >= 2) {
                return `${obj}.put(${curator}, ${args[0]}, ${args[1]}) catch @panic("OOM")`;
            }
            return `@compileError("pone requires two arguments: key, value")`;
        },
    },

    /** Get value by key (returns optional) */
    accipe: {
        latin: 'accipe',
        mutates: false,
        zig: (obj, args) => `${obj}.get(${args[0]})`,
    },

    /** Check if key exists */
    habet: {
        latin: 'habet',
        mutates: false,
        zig: (obj, args) => `${obj}.contains(${args[0]})`,
    },

    /** Delete key (mutates) */
    dele: {
        latin: 'dele',
        mutates: true,
        // WHY: remove() returns bool indicating if key existed
        zig: (obj, args) => `_ = ${obj}.remove(${args[0]})`,
    },

    /** Get size */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        zig: (obj, _args) => `${obj}.count()`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        zig: (obj, _args) => `(${obj}.count() == 0)`,
    },

    /** Clear all entries (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        zig: (obj, _args) => `${obj}.clearRetainingCapacity()`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Get keys iterator */
    claves: {
        latin: 'claves',
        mutates: false,
        // WHY: keyIterator() returns iterator over keys
        zig: (obj, _args) => `${obj}.keyIterator()`,
    },

    /** Get values iterator */
    valores: {
        latin: 'valores',
        mutates: false,
        zig: (obj, _args) => `${obj}.valueIterator()`,
    },

    /** Get entries iterator */
    paria: {
        latin: 'paria',
        mutates: false,
        // WHY: iterator() returns key-value pairs
        zig: (obj, _args) => `${obj}.iterator()`,
    },

    // -------------------------------------------------------------------------
    // EXTENDED OPERATIONS - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    /** Get value or return default */
    accipeAut: {
        latin: 'accipeAut',
        mutates: false,
        // WHY: Use orelse for default value
        zig: (obj, args) => {
            // WHY: args is now an array
            if (args.length >= 2) {
                return `(${obj}.get(${args[0]}) orelse ${args[1]})`;
            }
            return `${obj}.get(${args[0]})`;
        },
    },

    /** Keep only specified keys - NOT IMPLEMENTED */
    selige: {
        latin: 'selige',
        mutates: false,
        zig: () => `@compileError("selige not implemented for Zig - use explicit loop")`,
    },

    /** Remove specified keys - NOT IMPLEMENTED */
    omitte: {
        latin: 'omitte',
        mutates: false,
        zig: () => `@compileError("omitte not implemented for Zig - use explicit loop")`,
    },

    /** Merge maps - NOT IMPLEMENTED */
    confla: {
        latin: 'confla',
        mutates: false,
        zig: () => `@compileError("confla not implemented for Zig - use explicit loop")`,
    },

    /** Swap keys and values - NOT IMPLEMENTED */
    inversa: {
        latin: 'inversa',
        mutates: false,
        zig: () => `@compileError("inversa not implemented for Zig - use explicit loop")`,
    },

    /** Transform values - NOT IMPLEMENTED */
    mappaValores: {
        latin: 'mappaValores',
        mutates: false,
        zig: () => `@compileError("mappaValores not implemented for Zig - use explicit loop")`,
    },

    /** Transform keys - NOT IMPLEMENTED */
    mappaClaves: {
        latin: 'mappaClaves',
        mutates: false,
        zig: () => `@compileError("mappaClaves not implemented for Zig - use explicit loop")`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    /** Convert to list of entries */
    inLista: {
        latin: 'inLista',
        mutates: false,
        zig: () => `@compileError("inLista not implemented for Zig - iterate with ex...pro")`,
    },

    /** Convert to object */
    inObjectum: {
        latin: 'inObjectum',
        mutates: false,
        zig: () => `@compileError("inObjectum not implemented for Zig - Zig has no object type")`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getTabulaMethod(name: string): TabulaMethod | undefined {
    return TABULA_METHODS[name];
}

export function isTabulaMethod(name: string): boolean {
    return name in TABULA_METHODS;
}
