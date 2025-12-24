/**
 * Tabula Method Registry - TypeScript translations for Latin map methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (TypeScript target)
 *
 * ARCHITECTURE
 * ============
 * This module defines TypeScript translations for tabula<K,V> (Map) methods.
 * Each method specifies its Latin name, mutation semantics, and TS output.
 *
 * LATIN ETYMOLOGY
 * ===============
 * tabula: "board, tablet, table" - a writing surface with entries.
 * Feminine noun, so participle endings use -a (e.g., inversa, not inversus).
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: TypeScript code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Describes how to translate a Latin method to TypeScript.
 */
export interface TabulaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the map in place */
    mutates: boolean;

    /** True if method is async (future tense) */
    async: boolean;

    /**
     * TypeScript translation.
     * - string: simple method rename (obj.latin() -> obj.ts())
     * - function: custom code generation
     */
    ts: string | TsGenerator;
}

type TsGenerator = (obj: string, args: string) => string;

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin map methods with TypeScript translations.
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
        ts: 'set',
    },

    /** Get value by key */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        ts: 'get',
    },

    /** Check if key exists */
    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        ts: 'has',
    },

    /** Delete key (mutates) */
    dele: {
        latin: 'dele',
        mutates: true,
        async: false,
        ts: 'delete',
    },

    /** Get size */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.size`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.size === 0`,
    },

    /** Clear all entries (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        ts: 'clear',
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate keys */
    claves: {
        latin: 'claves',
        mutates: false,
        async: false,
        ts: 'keys',
    },

    /** Iterate values */
    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        ts: 'values',
    },

    /** Iterate entries as [key, value] pairs */
    paria: {
        latin: 'paria',
        mutates: false,
        async: false,
        ts: 'entries',
    },

    // -------------------------------------------------------------------------
    // LODASH-INSPIRED METHODS
    // -------------------------------------------------------------------------

    /** Get value or return default */
    accipeAut: {
        latin: 'accipeAut',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // args: "key, default"
            const match = args.match(/^(.+?),\s*(.+)$/);
            if (match) {
                return `(${obj}.get(${match[1]}) ?? ${match[2]})`;
            }
            return `${obj}.get(${args})`;
        },
    },

    /** Keep only specified keys (returns new map) */
    selige: {
        latin: 'selige',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // WHY: Filter to subset of keys. Args is spread of keys.
            return `new Map([...${obj}].filter(([k]) => [${args}].includes(k)))`;
        },
    },

    /** Remove specified keys (returns new map) */
    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            return `new Map([...${obj}].filter(([k]) => ![${args}].includes(k)))`;
        },
    },

    /** Merge with another map (returns new map) */
    confla: {
        latin: 'confla',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Map([...${obj}, ...${args}])`,
    },

    /** Swap keys and values (returns new map) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        ts: (obj, _args) => `new Map([...${obj}].map(([k, v]) => [v, k]))`,
    },

    /** Transform values (returns new map) */
    mappaValores: {
        latin: 'mappaValores',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Map([...${obj}].map(([k, v]) => [k, (${args})(v)]))`,
    },

    /** Transform keys (returns new map) */
    mappaClaves: {
        latin: 'mappaClaves',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Map([...${obj}].map(([k, v]) => [(${args})(k), v]))`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    /** Convert to lista of [key, value] pairs */
    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        ts: (obj, _args) => `[...${obj}]`,
    },

    /** Convert to object (string keys only) */
    inObjectum: {
        latin: 'inObjectum',
        mutates: false,
        async: false,
        ts: (obj, _args) => `Object.fromEntries(${obj})`,
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
