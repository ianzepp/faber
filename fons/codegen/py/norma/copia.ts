/**
 * Copia Method Registry - Python translations for Latin set methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Python target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Python translations for copia<T> (set) methods.
 * Python sets have native support for set operations.
 *
 * PYTHON IDIOMS
 * =============
 * | Latin         | Python                              |
 * |---------------|-------------------------------------|
 * | adde          | set.add(x)                          |
 * | habet         | x in set                            |
 * | dele          | set.discard(x)                      |
 * | unio          | set | other                         |
 * | intersectio   | set & other                         |
 * | differentia   | set - other                         |
 * | symmetrica    | set ^ other                         |
 * | subcopia      | set <= other                        |
 * | supercopia    | set >= other                        |
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: Python code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Generator function type for Python collection methods.
 */
export type PyGenerator = (obj: string, args: string[]) => string;

export interface CopiaMethod {
    latin: string;
    mutates: boolean;
    async: boolean;
    py: string | PyGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

export const COPIA_METHODS: Record<string, CopiaMethod> = {
    // -------------------------------------------------------------------------
    // CORE OPERATIONS
    // -------------------------------------------------------------------------

    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        py: 'add',
    },

    habet: {
        latin: 'habet',
        mutates: false,
        async: false,
        py: (obj, args) => `(${args[0]} in ${obj})`,
    },

    dele: {
        latin: 'dele',
        mutates: true,
        async: false,
        py: 'discard',
    },

    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        py: obj => `len(${obj})`,
    },

    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        py: obj => `len(${obj}) == 0`,
    },

    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        py: 'clear',
    },

    // -------------------------------------------------------------------------
    // SET OPERATIONS (return new sets)
    // -------------------------------------------------------------------------

    unio: {
        latin: 'unio',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} | ${args[0]})`,
    },

    intersectio: {
        latin: 'intersectio',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} & ${args[0]})`,
    },

    differentia: {
        latin: 'differentia',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} - ${args[0]})`,
    },

    symmetrica: {
        latin: 'symmetrica',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} ^ ${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    subcopia: {
        latin: 'subcopia',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} <= ${args[0]})`,
    },

    supercopia: {
        latin: 'supercopia',
        mutates: false,
        async: false,
        py: (obj, args) => `(${obj} >= ${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // CONVERSIONS
    // -------------------------------------------------------------------------

    inLista: {
        latin: 'inLista',
        mutates: false,
        async: false,
        py: obj => `list(${obj})`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    valores: {
        latin: 'valores',
        mutates: false,
        async: false,
        py: obj => `iter(${obj})`,
    },

    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        // forEach - use list comprehension for side effects
        py: (obj, args) => `[(${args[0]})(x) for x in ${obj}]`,
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
