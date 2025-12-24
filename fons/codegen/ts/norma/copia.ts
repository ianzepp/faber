/**
 * Copia Method Registry - TypeScript translations for Latin set methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (TypeScript target)
 *
 * ARCHITECTURE
 * ============
 * This module defines TypeScript translations for copia<T> (Set) methods.
 * Each method specifies its Latin name, mutation semantics, and TS output.
 *
 * LATIN ETYMOLOGY
 * ===============
 * copia: "abundance, supply" - a collection of resources.
 * Feminine noun, so participle endings use -a (e.g., unita, not unitus).
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
export interface CopiaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the set in place */
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
 * Registry of Latin set methods with TypeScript translations.
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
        ts: 'add',
    },

    /** Check if element exists */
    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        ts: 'has',
    },

    /** Delete element (mutates) */
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

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        ts: 'clear',
    },

    // -------------------------------------------------------------------------
    // SET OPERATIONS (return new sets)
    // -------------------------------------------------------------------------

    /** Union: A U B */
    unio: {
        latin: 'unio',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Set([...${obj}, ...${args}])`,
    },

    /** Intersection: A n B */
    intersectio: {
        latin: 'intersectio',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Set([...${obj}].filter(x => ${args}.has(x)))`,
    },

    /** Difference: A - B */
    differentia: {
        latin: 'differentia',
        mutates: false,
        async: false,
        ts: (obj, args) => `new Set([...${obj}].filter(x => !${args}.has(x)))`,
    },

    /** Symmetric difference: (A - B) U (B - A) */
    symmetrica: {
        latin: 'symmetrica',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // WHY: XOR requires checking both directions
            return `new Set([...[...${obj}].filter(x => !${args}.has(x)), ...[...${args}].filter(x => !${obj}.has(x))])`;
        },
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    /** Is subset of other */
    subcopia: {
        latin: 'subcopia',
        mutates: false,
        async: false,
        ts: (obj, args) => `[...${obj}].every(x => ${args}.has(x))`,
    },

    /** Is superset of other */
    supercopia: {
        latin: 'supercopia',
        mutates: false,
        async: false,
        ts: (obj, args) => `[...${args}].every(x => ${obj}.has(x))`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    /** Convert to lista */
    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        ts: (obj, _args) => `[...${obj}]`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate values */
    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        ts: 'values',
    },

    /** Iterate with callback (no return value) */
    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        ts: 'forEach',
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
