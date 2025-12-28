/**
 * Copia Method Registry - Zig translations for Latin set methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Zig translations for copia<T> (HashSet) methods.
 * Zig doesn't have a native Set type - we use AutoHashMap(T, void).
 *
 * ZIG SPECIFICS
 * =============
 * - copia<textus> maps to std.StringHashMap(void)
 * - copia<T> maps to std.AutoHashMap(T, void)
 * - Add: map.put(alloc, value, {})
 * - Has: map.contains(value)
 * - Delete: map.remove(value)
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

export interface CopiaMethod {
    latin: string;
    mutates: boolean;
    zig: string | ZigGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

export const COPIA_METHODS: Record<string, CopiaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    /** Add element (mutates, needs allocator) */
    adde: {
        latin: 'adde',
        mutates: true,
        // WHY: Set is implemented as HashMap with void value
        zig: (obj, args, curator) => `${obj}.put(${curator}, ${args[0]}, {}) catch @panic("OOM")`,
    },

    /** Check if element exists */
    habet: {
        latin: 'habet',
        mutates: false,
        zig: (obj, args) => `${obj}.contains(${args[0]})`,
    },

    /** Delete element (mutates) */
    dele: {
        latin: 'dele',
        mutates: true,
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

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        zig: (obj, _args) => `${obj}.clearRetainingCapacity()`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Get values (keys) iterator */
    valores: {
        latin: 'valores',
        mutates: false,
        // WHY: For a set implemented as map(T, void), keys are the values
        zig: (obj, _args) => `${obj}.keyIterator()`,
    },

    /** Iterate with callback - NOT IMPLEMENTED */
    perambula: {
        latin: 'perambula',
        mutates: false,
        zig: () => `@compileError("perambula not implemented for Zig - use 'ex set.keyIterator() pro item { ... }' loop")`,
    },

    // -------------------------------------------------------------------------
    // SET OPERATIONS - NOT IMPLEMENTED
    // WHY: These require creating new sets with allocator, complex to generate
    // -------------------------------------------------------------------------

    /** Union: A U B */
    unio: {
        latin: 'unio',
        mutates: false,
        zig: () => `@compileError("unio not implemented for Zig - use explicit loop to merge sets")`,
    },

    /** Intersection: A n B */
    intersectio: {
        latin: 'intersectio',
        mutates: false,
        zig: () => `@compileError("intersectio not implemented for Zig - use explicit loop")`,
    },

    /** Difference: A - B */
    differentia: {
        latin: 'differentia',
        mutates: false,
        zig: () => `@compileError("differentia not implemented for Zig - use explicit loop")`,
    },

    /** Symmetric difference */
    symmetrica: {
        latin: 'symmetrica',
        mutates: false,
        zig: () => `@compileError("symmetrica not implemented for Zig - use explicit loop")`,
    },

    // -------------------------------------------------------------------------
    // PREDICATES - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    /** Is subset of other */
    subcopia: {
        latin: 'subcopia',
        mutates: false,
        zig: () => `@compileError("subcopia not implemented for Zig - use explicit loop")`,
    },

    /** Is superset of other */
    supercopia: {
        latin: 'supercopia',
        mutates: false,
        zig: () => `@compileError("supercopia not implemented for Zig - use explicit loop")`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    /** Convert to list */
    inLista: {
        latin: 'inLista',
        mutates: false,
        zig: () => `@compileError("inLista not implemented for Zig - iterate with ex...pro into ArrayList")`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getCopiaMethod(name: string): CopiaMethod | undefined {
    return COPIA_METHODS[name];
}

export function isCopiaMethod(name: string): boolean {
    return name in COPIA_METHODS;
}
