/**
 * Tabula Method Registry - Rust translations for Latin map methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Rust target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Rust translations for tabula<K,V> (HashMap<K,V>) methods.
 * Rust's HashMap provides efficient key-value storage.
 *
 * RUST IDIOMS
 * ===========
 * | Latin       | Rust                                      |
 * |-------------|-------------------------------------------|
 * | pone        | map.insert(k, v)                          |
 * | accipe      | map.get(&k)                               |
 * | habet       | map.contains_key(&k)                      |
 * | dele        | map.remove(&k)                            |
 * | longitudo   | map.len()                                 |
 * | vacua       | map.is_empty()                            |
 * | purga       | map.clear()                               |
 * | claves      | map.keys()                                |
 * | valores     | map.values()                              |
 * | paria       | map.iter()                                |
 * | accipeAut   | map.get(&k).cloned().unwrap_or(default)   |
 * | confla      | map.extend(other)                         |
 * | inLista     | map.iter().collect::<Vec<_>>()            |
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: Rust code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

export interface TabulaMethod {
    latin: string;
    mutates: boolean;
    async: boolean;
    rs: string | RsGenerator;
}

type RsGenerator = (obj: string, args: string[]) => string;

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin map methods with Rust translations.
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
        rs: 'insert',
    },

    /** Get value by key */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.get(&${args[0]})`,
    },

    /** Check if key exists */
    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.contains_key(&${args[0]})`,
    },

    /** Delete key (mutates) */
    dele: {
        latin: 'dele',
        mutates: true,
        async: false,
        rs: (obj, args) => `${obj}.remove(&${args[0]})`,
    },

    /** Get size */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.len()`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.is_empty()`,
    },

    /** Clear all entries (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        rs: 'clear',
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate keys */
    claves: {
        latin: 'claves',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.keys()`,
    },

    /** Iterate values */
    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.values()`,
    },

    /** Iterate entries as (key, value) pairs */
    paria: {
        latin: 'paria',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter()`,
    },

    // -------------------------------------------------------------------------
    // EXTENDED METHODS
    // -------------------------------------------------------------------------

    /** Get value or return default */
    accipeAut: {
        latin: 'accipeAut',
        mutates: false,
        async: false,
        rs: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.get(&${args[0]}).cloned().unwrap_or(${args[1]})`;
            }
            return `${obj}.get(&${args[0]})`;
        },
    },

    /** Merge with another map (mutates, extends in place) */
    confla: {
        latin: 'confla',
        mutates: true,
        async: false,
        rs: (obj, args) => `${obj}.extend(${args[0]}.iter().map(|(k, v)| (k.clone(), v.clone())))`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    /** Convert to lista of (key, value) tuples */
    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()`,
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
