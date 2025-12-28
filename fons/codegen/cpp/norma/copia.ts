/**
 * Copia Method Registry - C++23 translations for Latin set methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (C++23 target)
 *
 * ARCHITECTURE
 * ============
 * This module defines C++23 translations for copia<T> (std::unordered_set) methods.
 *
 * C++23 IDIOMS
 * ============
 * | Latin         | C++23                                    |
 * |---------------|------------------------------------------|
 * | adde          | set.insert(x)                            |
 * | habet         | set.contains(x)                          |
 * | dele          | set.erase(x)                             |
 * | unio          | set_union via ranges                     |
 * | intersectio   | set_intersection via ranges              |
 *
 * LATIN ETYMOLOGY
 * ===============
 * copia: "abundance, supply" - a collection of resources.
 * Feminine noun, so participle endings use -a (e.g., unita).
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: C++23 code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

export interface CopiaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the set in place */
    mutates: boolean;

    /** True if method is async (not applicable for C++) */
    async: boolean;

    /**
     * C++23 translation.
     * - string: simple method rename
     * - function: custom code generation
     */
    cpp: string | CppGenerator;

    /** Required headers for this method */
    headers?: string[];
}

type CppGenerator = (obj: string, args: string[]) => string;

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin set methods with C++23 translations.
 */
export const COPIA_METHODS: Record<string, CopiaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    /** Add element (mutates) */
    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        cpp: 'insert',
    },

    /** Check if element exists */
    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        // WHY: C++20 contains() is cleaner than find() != end()
        cpp: (obj, args) => `${obj}.contains(${args[0]})`,
    },

    /** Delete element (mutates) */
    dele: {
        latin: 'dele',
        mutates: true,
        async: false,
        cpp: 'erase',
    },

    /** Get size */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.size()`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.empty()`,
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        cpp: 'clear',
    },

    // -------------------------------------------------------------------------
    // SET OPERATIONS (return new sets)
    // -------------------------------------------------------------------------

    /** Union: A U B */
    unio: {
        latin: 'unio',
        mutates: false,
        async: false,
        // WHY: Create new set from both inputs
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r(${obj}.begin(), ${obj}.end()); r.insert(${args[0]}.begin(), ${args[0]}.end()); return r; }()`,
        headers: ['<unordered_set>'],
    },

    /** Intersection: A n B */
    intersectio: {
        latin: 'intersectio',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (${args[0]}.contains(x)) r.insert(x); return r; }()`,
        headers: ['<unordered_set>'],
    },

    /** Difference: A - B */
    differentia: {
        latin: 'differentia',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (!${args[0]}.contains(x)) r.insert(x); return r; }()`,
        headers: ['<unordered_set>'],
    },

    /** Symmetric difference: (A - B) U (B - A) */
    symmetrica: {
        latin: 'symmetrica',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::unordered_set<typename std::decay_t<decltype(${obj})>::value_type> r; for (auto& x : ${obj}) if (!${args[0]}.contains(x)) r.insert(x); for (auto& x : ${args[0]}) if (!${obj}.contains(x)) r.insert(x); return r; }()`,
        headers: ['<unordered_set>'],
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    /** Is subset of other */
    subcopia: {
        latin: 'subcopia',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::all_of(${obj}, [&](auto& x) { return ${args[0]}.contains(x); })`,
        headers: ['<algorithm>'],
    },

    /** Is superset of other */
    supercopia: {
        latin: 'supercopia',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::all_of(${args[0]}, [&](auto& x) { return ${obj}.contains(x); })`,
        headers: ['<algorithm>'],
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    /** Convert to vector */
    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `std::vector(${obj}.begin(), ${obj}.end())`,
        headers: ['<vector>'],
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate values (set is already iterable) */
    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        // WHY: unordered_set is already iterable
        cpp: (obj, _args) => obj,
    },

    /** Iterate with callback (no return value) */
    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::for_each(${obj}, ${args[0]})`,
        headers: ['<algorithm>'],
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

/**
 * Get all headers required by a method.
 */
export function getCopiaHeaders(name: string): string[] {
    return COPIA_METHODS[name]?.headers ?? [];
}
