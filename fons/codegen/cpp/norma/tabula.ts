/**
 * Tabula Method Registry - C++23 translations for Latin map methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (C++23 target)
 *
 * ARCHITECTURE
 * ============
 * This module defines C++23 translations for tabula<K,V> (std::unordered_map) methods.
 *
 * C++23 IDIOMS
 * ============
 * | Latin         | C++23                                    |
 * |---------------|------------------------------------------|
 * | pone          | map[k] = v or map.insert_or_assign(k,v)  |
 * | accipe        | map.at(k) or map[k]                      |
 * | habet         | map.contains(k)                          |
 * | dele          | map.erase(k)                             |
 * | claves        | views::keys                              |
 * | valores       | views::values                            |
 *
 * LATIN ETYMOLOGY
 * ===============
 * tabula: "board, tablet, table" - a writing surface with entries.
 * Feminine noun, so participle endings use -a (e.g., inversa).
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

export interface TabulaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the map in place */
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
 * Registry of Latin map methods with C++23 translations.
 */
export const TABULA_METHODS: Record<string, TabulaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    /** Set key-value pair (mutates) */
    pone: {
        latin: 'pone',
        mutates: true,
        async: false,
        // WHY: insert_or_assign handles both insert and update
        cpp: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.insert_or_assign(${args[0]}, ${args.slice(1).join(', ')})`;
            }
            return `${obj}.insert_or_assign(${args[0]})`;
        },
        headers: ['<unordered_map>'],
    },

    /** Get value by key */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        // WHY: at() throws if key missing (safer than operator[])
        cpp: (obj, args) => `${obj}.at(${args[0]})`,
    },

    /** Check if key exists */
    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        // WHY: C++20 contains() is cleaner than find() != end()
        cpp: (obj, args) => `${obj}.contains(${args[0]})`,
    },

    /** Delete key (mutates) */
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

    /** Clear all entries (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        cpp: 'clear',
    },

    // -------------------------------------------------------------------------
    // ITERATION (C++20/23 ranges)
    // -------------------------------------------------------------------------

    /** Iterate keys */
    claves: {
        latin: 'claves',
        mutates: false,
        async: false,
        // WHY: C++20 views::keys provides lazy key iteration
        cpp: (obj, _args) => `(${obj} | std::views::keys)`,
        headers: ['<ranges>'],
    },

    /** Iterate values */
    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `(${obj} | std::views::values)`,
        headers: ['<ranges>'],
    },

    /** Iterate entries as pairs */
    paria: {
        latin: 'paria',
        mutates: false,
        async: false,
        // WHY: unordered_map is already iterable as pairs
        cpp: (obj, _args) => obj,
    },

    // -------------------------------------------------------------------------
    // EXTENDED METHODS
    // -------------------------------------------------------------------------

    /** Get value or return default */
    accipeAut: {
        latin: 'accipeAut',
        mutates: false,
        async: false,
        cpp: (obj, args) => {
            if (args.length >= 2) {
                const key = args[0];
                const defaultVal = args.slice(1).join(', ');
                return `(${obj}.contains(${key}) ? ${obj}.at(${key}) : ${defaultVal})`;
            }
            return `${obj}.at(${args[0]})`;
        },
    },

    /** Keep only specified keys (returns new map) */
    selige: {
        latin: 'selige',
        mutates: false,
        async: false,
        // WHY: Filter to subset of keys
        cpp: (obj, args) => {
            return `[&]{ std::unordered_map<decltype(${obj})::key_type, decltype(${obj})::mapped_type> r; std::initializer_list<decltype(${obj})::key_type> keys = {${args.join(', ')}}; for (auto& k : keys) if (${obj}.contains(k)) r[k] = ${obj}.at(k); return r; }()`;
        },
        headers: ['<unordered_map>'],
    },

    /** Remove specified keys (returns new map) */
    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        cpp: (obj, args) => {
            return `[&]{ auto r = ${obj}; std::initializer_list<decltype(${obj})::key_type> keys = {${args.join(', ')}}; for (auto& k : keys) r.erase(k); return r; }()`;
        },
        headers: ['<unordered_map>'],
    },

    /** Merge with another map (returns new map) */
    confla: {
        latin: 'confla',
        mutates: false,
        async: false,
        cpp: (obj, args) => `[&]{ auto r = ${obj}; for (auto& [k, v] : ${args[0]}) r[k] = v; return r; }()`,
        headers: ['<unordered_map>'],
    },

    /** Swap keys and values (returns new map) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        cpp: (obj, _args) =>
            `[&]{ std::unordered_map<decltype(${obj})::mapped_type, decltype(${obj})::key_type> r; for (auto& [k, v] : ${obj}) r[v] = k; return r; }()`,
        headers: ['<unordered_map>'],
    },

    /** Transform values (returns new map) */
    mappaValores: {
        latin: 'mappaValores',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::unordered_map<decltype(${obj})::key_type, decltype((${args[0]})(${obj}.begin()->second))> r; for (auto& [k, v] : ${obj}) r[k] = (${args[0]})(v); return r; }()`,
        headers: ['<unordered_map>'],
    },

    /** Transform keys (returns new map) */
    mappaClaves: {
        latin: 'mappaClaves',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::unordered_map<decltype((${args[0]})(${obj}.begin()->first)), decltype(${obj})::mapped_type> r; for (auto& [k, v] : ${obj}) r[(${args[0]})(k)] = v; return r; }()`,
        headers: ['<unordered_map>'],
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    /** Convert to vector of pairs */
    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `std::vector(${obj}.begin(), ${obj}.end())`,
        headers: ['<vector>'],
    },

    /** Convert to object - not directly applicable in C++ */
    inObjectum: {
        latin: 'inObjectum',
        mutates: false,
        async: false,
        // WHY: C++ has no dynamic object type; just return the map
        cpp: (obj, _args) => obj,
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

/**
 * Get all headers required by a method.
 */
export function getTabulaHeaders(name: string): string[] {
    return TABULA_METHODS[name]?.headers ?? [];
}
