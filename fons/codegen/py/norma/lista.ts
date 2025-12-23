/**
 * Lista Method Registry - Python translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Python target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Python translations for lista<T> (array) methods.
 * Python uses list comprehensions and built-in functions rather than
 * method chaining for many operations.
 *
 * PYTHON IDIOMS
 * =============
 * | Latin         | Python                              |
 * |---------------|-------------------------------------|
 * | adde          | list.append(x)                      |
 * | addita        | [*list, x]                          |
 * | filtrata      | [x for x in list if pred(x)]        |
 * | mappata       | [f(x) for x in list]                |
 * | reducta       | functools.reduce(fn, list, init)    |
 * | primus        | list[0]                             |
 * | ultimus       | list[-1]                            |
 * | longitudo     | len(list)                           |
 * | continet      | x in list                           |
 * | inversa       | list[::-1]                          |
 * | ordinata      | sorted(list)                        |
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

export interface ListaMethod {
    latin: string;
    mutates: boolean;
    async: boolean;
    py: string | PyGenerator;
}

type PyGenerator = (obj: string, args: string) => string;

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with Python translations.
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        py: 'append',
    },

    addita: {
        latin: 'addita',
        mutates: false,
        async: false,
        py: (obj, args) => `[*${obj}, ${args}]`,
    },

    praepone: {
        latin: 'praepone',
        mutates: true,
        async: false,
        py: (obj, args) => `${obj}.insert(0, ${args})`,
    },

    praeposita: {
        latin: 'praeposita',
        mutates: false,
        async: false,
        py: (obj, args) => `[${args}, *${obj}]`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    remove: {
        latin: 'remove',
        mutates: true,
        async: false,
        py: 'pop',
    },

    remota: {
        latin: 'remota',
        mutates: false,
        async: false,
        py: (obj) => `${obj}[:-1]`,
    },

    decapita: {
        latin: 'decapita',
        mutates: true,
        async: false,
        py: (obj) => `${obj}.pop(0)`,
    },

    decapitata: {
        latin: 'decapitata',
        mutates: false,
        async: false,
        py: (obj) => `${obj}[1:]`,
    },

    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        py: 'clear',
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    primus: {
        latin: 'primus',
        mutates: false,
        async: false,
        py: (obj) => `${obj}[0]`,
    },

    ultimus: {
        latin: 'ultimus',
        mutates: false,
        async: false,
        py: (obj) => `${obj}[-1]`,
    },

    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        py: (obj, args) => `${obj}[${args}]`,
    },

    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        py: (obj) => `len(${obj})`,
    },

    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        py: (obj) => `len(${obj}) == 0`,
    },

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    continet: {
        latin: 'continet',
        mutates: false,
        async: false,
        py: (obj, args) => `(${args} in ${obj})`,
    },

    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        async: false,
        py: (obj, args) => `${obj}.index(${args})`,
    },

    inveni: {
        latin: 'inveni',
        mutates: false,
        async: false,
        py: (obj, args) => `next((x for x in ${obj} if ${args}(x)), None)`,
    },

    inveniIndicem: {
        latin: 'inveniIndicem',
        mutates: false,
        async: false,
        py: (obj, args) => `next((i for i, x in enumerate(${obj}) if ${args}(x)), -1)`,
    },

    // -------------------------------------------------------------------------
    // TRANSFORMATIONS (return new list)
    // -------------------------------------------------------------------------

    filtrata: {
        latin: 'filtrata',
        mutates: false,
        async: false,
        py: (obj, args) => `[x for x in ${obj} if ${args}(x)]`,
    },

    mappata: {
        latin: 'mappata',
        mutates: false,
        async: false,
        py: (obj, args) => `[${args}(x) for x in ${obj}]`,
    },

    reducta: {
        latin: 'reducta',
        mutates: false,
        async: false,
        py: (obj, args) => {
            // args is "fn, init" or just "fn"
            const parts = args.split(',').map(s => s.trim());
            if (parts.length >= 2) {
                return `functools.reduce(${parts[0]}, ${obj}, ${parts[1]})`;
            }
            return `functools.reduce(${args}, ${obj})`;
        },
    },

    explanata: {
        latin: 'explanata',
        mutates: false,
        async: false,
        py: (obj, args) => `[y for x in ${obj} for y in ${args}(x)]`,
    },

    plana: {
        latin: 'plana',
        mutates: false,
        async: false,
        py: (obj) => `[y for x in ${obj} for y in x]`,
    },

    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        py: (obj) => `${obj}[::-1]`,
    },

    ordinata: {
        latin: 'ordinata',
        mutates: false,
        async: false,
        py: (obj, args) => {
            if (args) {
                return `sorted(${obj}, key=${args})`;
            }
            return `sorted(${obj})`;
        },
    },

    // -------------------------------------------------------------------------
    // SLICING
    // -------------------------------------------------------------------------

    sectio: {
        latin: 'sectio',
        mutates: false,
        async: false,
        py: (obj, args) => {
            // args is "start, end" or "start"
            const parts = args.split(',').map(s => s.trim());
            if (parts.length >= 2) {
                return `${obj}[${parts[0]}:${parts[1]}]`;
            }
            return `${obj}[${args}:]`;
        },
    },

    prima: {
        latin: 'prima',
        mutates: false,
        async: false,
        py: (obj, args) => `${obj}[:${args}]`,
    },

    ultima: {
        latin: 'ultima',
        mutates: false,
        async: false,
        py: (obj, args) => `${obj}[-${args}:]`,
    },

    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        py: (obj, args) => `${obj}[${args}:]`,
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    omnes: {
        latin: 'omnes',
        mutates: false,
        async: false,
        py: (obj, args) => `all(${args}(x) for x in ${obj})`,
    },

    aliquis: {
        latin: 'aliquis',
        mutates: false,
        async: false,
        py: (obj, args) => `any(${args}(x) for x in ${obj})`,
    },

    // -------------------------------------------------------------------------
    // STRING OPERATIONS
    // -------------------------------------------------------------------------

    coniunge: {
        latin: 'coniunge',
        mutates: false,
        async: false,
        py: (obj, args) => {
            if (args) {
                return `${args}.join(${obj})`;
            }
            return `"".join(${obj})`;
        },
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        // forEach doesn't exist in Python; we use a for loop or list comprehension
        // For side effects, we use: [fn(x) for x in list] and discard result
        // Or generate a for loop if in statement context
        py: (obj, args) => `[${args}(x) for x in ${obj}]`,
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

export function getListaMethod(name: string): ListaMethod | undefined {
    return LISTA_METHODS[name];
}

export function isListaMethod(name: string): boolean {
    return name in LISTA_METHODS;
}
