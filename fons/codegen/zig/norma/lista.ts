/**
 * Lista Method Registry - Zig translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Zig translations for lista<T> (ArrayList) methods.
 * Zig's ArrayList requires an allocator for most operations. We use a
 * module-level arena allocator initialized in the preamble.
 *
 * ZIG SPECIFICS
 * =============
 * - lista<T> maps to std.ArrayList(T)
 * - Most mutating operations need allocator: list.append(alloc, x)
 * - Access via .items slice: list.items[0], list.items.len
 * - No functional methods (map, filter) - use ex...pro loops instead
 *
 * UNIMPLEMENTED METHODS
 * =====================
 * Functional methods (mappata, filtrata, reducta, etc.) are stubbed with
 * @compileError since Zig has no equivalent. Users should use explicit loops.
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
 * WHY: The curator parameter allows methods to use the correct allocator
 *      based on context (function parameter vs module-level arena).
 * WHY: The args parameter is a string[] (not a joined string) to preserve
 *      argument boundaries for multi-parameter lambdas.
 */
export type ZigGenerator = (obj: string, args: string[], curator: string) => string;

/**
 * Describes how to translate a Latin method to Zig.
 */
export interface ListaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the array in place */
    mutates: boolean;

    /**
     * Zig translation.
     * - string: simple property/method access
     * - function: custom code generation with allocator context
     */
    zig: string | ZigGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with Zig translations.
 *
 * WHY: Zig's ArrayList has different semantics than JS arrays:
 * - Requires allocator for growth operations
 * - Uses .items slice for element access
 * - No built-in functional methods
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    /** Add element to end (mutates, needs allocator) */
    adde: {
        latin: 'adde',
        mutates: true,
        // WHY: ArrayList.append() can fail on OOM, we catch and panic
        zig: (obj, args, curator) => `${obj}.append(${curator}, ${args[0]}) catch @panic("OOM")`,
    },

    /** Add element to end (returns new list) - NOT IMPLEMENTED */
    addita: {
        latin: 'addita',
        mutates: false,
        zig: () => `@compileError("addita (immutable append) not implemented for Zig - use adde or explicit loop")`,
    },

    /** Add element to start (mutates) */
    praepone: {
        latin: 'praepone',
        mutates: true,
        // WHY: ArrayList.insert() at index 0 = prepend
        zig: (obj, args, curator) => `${obj}.insert(${curator}, 0, ${args[0]}) catch @panic("OOM")`,
    },

    /** Add element to start (returns new list) - NOT IMPLEMENTED */
    praeposita: {
        latin: 'praeposita',
        mutates: false,
        zig: () => `@compileError("praeposita (immutable prepend) not implemented for Zig - use praepone or explicit loop")`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    /** Remove and return last element (mutates) */
    remove: {
        latin: 'remove',
        mutates: true,
        // WHY: pop() returns optional, we use .? to unwrap (panics if empty)
        zig: (obj, _args) => `${obj}.pop()`,
    },

    /** Remove last element (returns new list) - NOT IMPLEMENTED */
    remota: {
        latin: 'remota',
        mutates: false,
        zig: () => `@compileError("remota (immutable pop) not implemented for Zig - use remove or explicit loop")`,
    },

    /** Remove and return first element (mutates) */
    decapita: {
        latin: 'decapita',
        mutates: true,
        // WHY: orderedRemove(0) removes first element, preserving order
        zig: (obj, _args) => `${obj}.orderedRemove(0)`,
    },

    /** Remove first element (returns new list) - NOT IMPLEMENTED */
    decapitata: {
        latin: 'decapitata',
        mutates: false,
        zig: () => `@compileError("decapitata (immutable shift) not implemented for Zig - use decapita or explicit loop")`,
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        // WHY: clearRetainingCapacity keeps allocated memory for reuse
        zig: (obj, _args) => `${obj}.clearRetainingCapacity()`,
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    /** Get first element */
    primus: {
        latin: 'primus',
        mutates: false,
        // WHY: Direct slice access, returns optional-like behavior via bounds
        zig: (obj, _args) => `${obj}.items[0]`,
    },

    /** Get last element */
    ultimus: {
        latin: 'ultimus',
        mutates: false,
        // WHY: items.len - 1 gives last index
        zig: (obj, _args) => `${obj}.items[${obj}.items.len - 1]`,
    },

    /** Get element at index */
    accipe: {
        latin: 'accipe',
        mutates: false,
        zig: (obj, args) => `${obj}.items[${args[0]}]`,
    },

    // -------------------------------------------------------------------------
    // PROPERTIES
    // -------------------------------------------------------------------------

    /** Get length */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        zig: (obj, _args) => `${obj}.items.len`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        zig: (obj, _args) => `(${obj}.items.len == 0)`,
    },

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    /** Check if contains element - NOT FULLY IMPLEMENTED */
    continet: {
        latin: 'continet',
        mutates: false,
        // WHY: Zig has no built-in includes. Would need std.mem.indexOfScalar
        zig: (obj, args) => `(std.mem.indexOfScalar(@TypeOf(${obj}.items[0]), ${obj}.items, ${args[0]}) != null)`,
    },

    /** Find index of element */
    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        zig: (obj, args) => `std.mem.indexOfScalar(@TypeOf(${obj}.items[0]), ${obj}.items, ${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // FUNCTIONAL METHODS - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    /** Filter elements - NOT IMPLEMENTED */
    filtrata: {
        latin: 'filtrata',
        mutates: false,
        zig: () => `@compileError("filtrata not implemented for Zig - use 'ex list pro item { si ... }' loop")`,
    },

    /** Map elements - NOT IMPLEMENTED */
    mappata: {
        latin: 'mappata',
        mutates: false,
        zig: () => `@compileError("mappata not implemented for Zig - use 'ex list pro item { ... }' loop")`,
    },

    /** Reduce to single value - NOT IMPLEMENTED */
    reducta: {
        latin: 'reducta',
        mutates: false,
        zig: () => `@compileError("reducta not implemented for Zig - use explicit accumulator loop")`,
    },

    /** Reverse (returns new list) - NOT IMPLEMENTED */
    inversa: {
        latin: 'inversa',
        mutates: false,
        zig: () => `@compileError("inversa not implemented for Zig - use std.mem.reverse or explicit loop")`,
    },

    /** Sort (returns new list) - NOT IMPLEMENTED */
    ordinata: {
        latin: 'ordinata',
        mutates: false,
        zig: () => `@compileError("ordinata not implemented for Zig - use std.mem.sort on .items")`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate with callback - NOT IMPLEMENTED */
    perambula: {
        latin: 'perambula',
        mutates: false,
        zig: () => `@compileError("perambula not implemented for Zig - use 'ex list pro item { ... }' loop")`,
    },

    // -------------------------------------------------------------------------
    // AGGREGATION - NOT IMPLEMENTED
    // -------------------------------------------------------------------------

    summa: {
        latin: 'summa',
        mutates: false,
        zig: () => `@compileError("summa not implemented for Zig - use explicit accumulator loop")`,
    },

    minimus: {
        latin: 'minimus',
        mutates: false,
        zig: () => `@compileError("minimus not implemented for Zig - use std.mem.min on .items")`,
    },

    maximus: {
        latin: 'maximus',
        mutates: false,
        zig: () => `@compileError("maximus not implemented for Zig - use std.mem.max on .items")`,
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
