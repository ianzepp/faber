/**
 * Lista Method Registry - TypeScript translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (TypeScript target)
 *
 * ARCHITECTURE
 * ============
 * This module defines TypeScript translations for lista<T> (array) methods.
 * Each method specifies its Latin name, mutation semantics, and TS output.
 *
 * LATIN VERB CONJUGATION
 * ======================
 * Latin verb forms encode mutability:
 *
 * |           | Mutates (in-place) | Returns New (copy) |
 * |-----------|--------------------|--------------------|
 * | Sync      | adde (imperative)  | addita (participle)|
 * | Async     | addet (future)     | additura (fut.part)|
 *
 * The feminine endings (-a, -ura) agree with lista/tabula/copia.
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
 * Generator function type for TypeScript collection methods.
 *
 * WHY: The args parameter is a string[] (not a joined string) to preserve
 *      argument boundaries. This allows methods like reducta to correctly
 *      handle multi-parameter lambdas that contain commas.
 *
 * @param obj - The collection expression (e.g., "myList")
 * @param args - Array of argument strings, each already generated
 * @returns TypeScript code string
 */
export type TsGenerator = (obj: string, args: string[]) => string;

/**
 * Describes how to translate a Latin method to TypeScript.
 */
export interface ListaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the array in place */
    mutates: boolean;

    /** True if method is async (future tense) */
    async: boolean;

    /**
     * TypeScript translation.
     * - string: simple method rename (obj.latin() -> obj.ts())
     * - function: custom code generation with structured args
     */
    ts: string | TsGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with TypeScript translations.
 *
 * Organized by category for clarity. All methods are keyed by Latin name.
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    /** Add element to end (mutates) */
    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        ts: 'push',
    },

    /** Add element to end (returns new array) */
    addita: {
        latin: 'addita',
        mutates: false,
        async: false,
        ts: (obj, args) => `[...${obj}, ${args.join(', ')}]`,
    },

    /** Add element to start (mutates) */
    praepone: {
        latin: 'praepone',
        mutates: true,
        async: false,
        ts: 'unshift',
    },

    /** Add element to start (returns new array) */
    praeposita: {
        latin: 'praeposita',
        mutates: false,
        async: false,
        ts: (obj, args) => `[${args.join(', ')}, ...${obj}]`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    /** Remove last element (mutates, returns removed) */
    remove: {
        latin: 'remove',
        mutates: true,
        async: false,
        ts: 'pop',
    },

    /** Remove last element (returns new array without last) */
    remota: {
        latin: 'remota',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.slice(0, -1)`,
    },

    /** Remove first element (mutates, returns removed) */
    decapita: {
        latin: 'decapita',
        mutates: true,
        async: false,
        ts: 'shift',
    },

    /** Remove first element (returns new array without first) */
    decapitata: {
        latin: 'decapitata',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.slice(1)`,
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        ts: (obj, _args) => `${obj}.length = 0`,
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    /** Get first element */
    primus: {
        latin: 'primus',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}[0]`,
    },

    /** Get last element */
    ultimus: {
        latin: 'ultimus',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.at(-1)`,
    },

    /** Get element at index */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        ts: (obj, args) => `${obj}[${args[0]}]`,
    },

    // -------------------------------------------------------------------------
    // PROPERTIES (as methods for consistency)
    // -------------------------------------------------------------------------

    /** Get length */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.length`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.length === 0`,
    },

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    /** Check if contains element */
    continet: {
        latin: 'continet',
        mutates: false,
        async: false,
        ts: 'includes',
    },

    /** Find index of element (-1 if not found) */
    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        async: false,
        ts: 'indexOf',
    },

    /** Find first element matching predicate */
    inveni: {
        latin: 'inveni',
        mutates: false,
        async: false,
        ts: 'find',
    },

    /** Find index of first element matching predicate */
    inveniIndicem: {
        latin: 'inveniIndicem',
        mutates: false,
        async: false,
        ts: 'findIndex',
    },

    // -------------------------------------------------------------------------
    // TRANSFORMATIONS (return new arrays)
    // -------------------------------------------------------------------------

    /** Filter elements (returns new array) */
    filtrata: {
        latin: 'filtrata',
        mutates: false,
        async: false,
        ts: 'filter',
    },

    /** Map elements (returns new array) */
    mappata: {
        latin: 'mappata',
        mutates: false,
        async: false,
        ts: 'map',
    },

    /** Reduce to single value - Faber uses (fn, init), JS uses (fn, init) */
    reducta: {
        latin: 'reducta',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // WHY: Now that args is a proper array, we can directly access
            //      fn and init without string parsing. No more comma ambiguity.
            if (args.length >= 2) {
                const fn = args[0];
                const init = args[1];
                return `${obj}.reduce(${fn}, ${init})`;
            }
            // Just function, no initial value
            return `${obj}.reduce(${args[0]})`;
        },
    },

    /** Flat map (map + flatten one level) */
    explanata: {
        latin: 'explanata',
        mutates: false,
        async: false,
        ts: 'flatMap',
    },

    /** Flatten one level */
    plana: {
        latin: 'plana',
        mutates: false,
        async: false,
        ts: 'flat',
    },

    /** Reverse (returns new array) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        ts: (obj, _args) => `[...${obj}].reverse()`,
    },

    /** Sort (returns new array) */
    ordinata: {
        latin: 'ordinata',
        mutates: false,
        async: false,
        ts: (obj, args) => (args.length > 0 ? `[...${obj}].sort(${args[0]})` : `[...${obj}].sort()`),
    },

    /** Slice (returns new array) */
    sectio: {
        latin: 'sectio',
        mutates: false,
        async: false,
        ts: 'slice',
    },

    /** Take first n elements */
    prima: {
        latin: 'prima',
        mutates: false,
        async: false,
        ts: (obj, args) => `${obj}.slice(0, ${args[0]})`,
    },

    /** Take last n elements */
    ultima: {
        latin: 'ultima',
        mutates: false,
        async: false,
        ts: (obj, args) => `${obj}.slice(-${args[0]})`,
    },

    /** Skip first n elements */
    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        ts: (obj, args) => `${obj}.slice(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    /** Check if all elements match predicate */
    omnes: {
        latin: 'omnes',
        mutates: false,
        async: false,
        ts: 'every',
    },

    /** Check if any element matches predicate */
    aliquis: {
        latin: 'aliquis',
        mutates: false,
        async: false,
        ts: 'some',
    },

    // -------------------------------------------------------------------------
    // AGGREGATION
    // -------------------------------------------------------------------------

    /** Join elements to string */
    coniunge: {
        latin: 'coniunge',
        mutates: false,
        async: false,
        ts: 'join',
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate with callback (no return value) */
    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        ts: 'forEach',
    },

    // -------------------------------------------------------------------------
    // MUTATING VARIANTS (in-place operations)
    // -------------------------------------------------------------------------

    /** Filter in place (mutates) */
    filtra: {
        latin: 'filtra',
        mutates: true,
        async: false,
        ts: (obj, args) => {
            // WHY: JS has no in-place filter. Splice out non-matching elements.
            // This is expensive but semantically correct for mutation.
            const fn = args[0];
            return `(() => { for (let i = ${obj}.length - 1; i >= 0; i--) { if (!(${fn})(${obj}[i])) ${obj}.splice(i, 1); } })()`;
        },
    },

    /** Sort in place (mutates) */
    ordina: {
        latin: 'ordina',
        mutates: true,
        async: false,
        ts: 'sort',
    },

    /** Reverse in place (mutates) */
    inverte: {
        latin: 'inverte',
        mutates: true,
        async: false,
        ts: 'reverse',
    },

    // -------------------------------------------------------------------------
    // LODASH-INSPIRED METHODS
    // -------------------------------------------------------------------------

    /** Group by key function -> tabula<K, lista<T>> */
    congrega: {
        latin: 'congrega',
        mutates: false,
        async: false,
        ts: (obj, args) => `Object.groupBy(${obj}, ${args[0]})`,
    },

    /** Remove duplicates */
    unica: {
        latin: 'unica',
        mutates: false,
        async: false,
        ts: (obj, _args) => `[...new Set(${obj})]`,
    },

    /** Flatten all levels */
    planaOmnia: {
        latin: 'planaOmnia',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.flat(Infinity)`,
    },

    /** Split into chunks of size n */
    fragmenta: {
        latin: 'fragmenta',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // WHY: No native chunk. Build inline for simple cases.
            const n = args[0];
            return `Array.from({ length: Math.ceil(${obj}.length / ${n}) }, (_, i) => ${obj}.slice(i * ${n}, i * ${n} + ${n}))`;
        },
    },

    /** Remove falsy values */
    densa: {
        latin: 'densa',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.filter(Boolean)`,
    },

    /** Partition by predicate -> [truthy, falsy] */
    partire: {
        latin: 'partire',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            const fn = args[0];
            return `${obj}.reduce(([t, f], x) => (${fn})(x) ? [[...t, x], f] : [t, [...f, x]], [[], []])`;
        },
    },

    /** Shuffle (Fisher-Yates) */
    misce: {
        latin: 'misce',
        mutates: false,
        async: false,
        ts: (obj, _args) => {
            // WHY: Returns new shuffled array. Uses Fisher-Yates in IIFE.
            return `(() => { const a = [...${obj}]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a; })()`;
        },
    },

    /** Random element */
    specimen: {
        latin: 'specimen',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}[Math.floor(Math.random() * ${obj}.length)]`,
    },

    /** Random n elements */
    specimina: {
        latin: 'specimina',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            // WHY: Shuffle then take first n. Not most efficient but correct.
            const n = args[0];
            return `(() => { const a = [...${obj}]; for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; } return a.slice(0, ${n}); })()`;
        },
    },

    // -------------------------------------------------------------------------
    // AGGREGATION (numeric operations)
    // -------------------------------------------------------------------------

    /** Sum of numbers */
    summa: {
        latin: 'summa',
        mutates: false,
        async: false,
        ts: (obj, _args) => `${obj}.reduce((a, b) => a + b, 0)`,
    },

    /** Average of numbers */
    medium: {
        latin: 'medium',
        mutates: false,
        async: false,
        ts: (obj, _args) => `(${obj}.reduce((a, b) => a + b, 0) / ${obj}.length)`,
    },

    /** Minimum value */
    minimus: {
        latin: 'minimus',
        mutates: false,
        async: false,
        ts: (obj, _args) => `Math.min(...${obj})`,
    },

    /** Maximum value */
    maximus: {
        latin: 'maximus',
        mutates: false,
        async: false,
        ts: (obj, _args) => `Math.max(...${obj})`,
    },

    /** Minimum by key function */
    minimusPer: {
        latin: 'minimusPer',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            const fn = args[0];
            return `${obj}.reduce((min, x) => (${fn})(x) < (${fn})(min) ? x : min)`;
        },
    },

    /** Maximum by key function */
    maximusPer: {
        latin: 'maximusPer',
        mutates: false,
        async: false,
        ts: (obj, args) => {
            const fn = args[0];
            return `${obj}.reduce((max, x) => (${fn})(x) > (${fn})(max) ? x : max)`;
        },
    },

    /** Count elements matching predicate */
    numera: {
        latin: 'numera',
        mutates: false,
        async: false,
        ts: (obj, args) => `${obj}.filter(${args[0]}).length`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Look up a Latin method name and return its definition.
 *
 * @param name - Latin method name
 * @returns Method definition or undefined if not found
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
