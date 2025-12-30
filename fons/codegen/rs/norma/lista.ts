/**
 * Lista Method Registry - Rust translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Rust target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Rust translations for lista<T> (Vec<T>) methods.
 * Rust uses iterator combinators for functional-style operations.
 *
 * RUST IDIOMS
 * ===========
 * | Latin         | Rust                                      |
 * |---------------|-------------------------------------------|
 * | adde          | vec.push(x)                               |
 * | addita        | { let mut v = vec.clone(); v.push(x); v } |
 * | praepone      | vec.insert(0, x)                          |
 * | praeposita    | { let mut v = vec![x]; v.extend(...); v } |
 * | remove        | vec.pop()                                 |
 * | decapita      | vec.remove(0)                             |
 * | purga         | vec.clear()                               |
 * | primus        | vec.first()                               |
 * | ultimus       | vec.last()                                |
 * | accipe        | vec.get(i) or vec[i]                      |
 * | longitudo     | vec.len()                                 |
 * | vacua         | vec.is_empty()                            |
 * | continet      | vec.contains(&x)                          |
 * | indiceDe      | vec.iter().position(|e| e == &x)          |
 * | filtrata      | vec.iter().filter(...).cloned().collect() |
 * | mappata       | vec.iter().map(...).collect()             |
 * | reducta       | vec.iter().fold(init, fn)                 |
 * | inversa       | vec.iter().rev().cloned().collect()       |
 * | ordinata      | { let mut v = vec.clone(); v.sort(); v }  |
 * | prima         | vec[..n].to_vec()                         |
 * | ultima        | vec[vec.len()-n..].to_vec()               |
 * | omitte        | vec[n..].to_vec()                         |
 * | omnes         | vec.iter().all(fn)                        |
 * | aliquis       | vec.iter().any(fn)                        |
 * | ordina        | vec.sort()                                |
 * | inverte       | vec.reverse()                             |
 * | summa         | vec.iter().sum()                          |
 * | minimus       | vec.iter().min()                          |
 * | maximus       | vec.iter().max()                          |
 * | perambula     | vec.iter().for_each(fn)                   |
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

export interface ListaMethod {
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
 * Registry of Latin array methods with Rust translations.
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
        rs: 'push',
    },

    /** Add element to end (returns new vec) */
    addita: {
        latin: 'addita',
        mutates: false,
        async: false,
        rs: (obj, args) => `{ let mut v = ${obj}.clone(); v.push(${args[0]}); v }`,
    },

    /** Add element to start (mutates) */
    praepone: {
        latin: 'praepone',
        mutates: true,
        async: false,
        rs: (obj, args) => `${obj}.insert(0, ${args[0]})`,
    },

    /** Add element to start (returns new vec) */
    praeposita: {
        latin: 'praeposita',
        mutates: false,
        async: false,
        rs: (obj, args) => `{ let mut v = vec![${args[0]}]; v.extend(${obj}.iter().cloned()); v }`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    /** Remove last element (mutates, returns Option<T>) */
    remove: {
        latin: 'remove',
        mutates: true,
        async: false,
        rs: (obj, _args) => `${obj}.pop()`,
    },

    /** Remove last element (returns new vec without last) */
    remota: {
        latin: 'remota',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}[..${obj}.len().saturating_sub(1)].to_vec()`,
    },

    /** Remove first element (mutates, returns T) */
    decapita: {
        latin: 'decapita',
        mutates: true,
        async: false,
        rs: (obj, _args) => `${obj}.remove(0)`,
    },

    /** Remove first element (returns new vec without first) */
    decapitata: {
        latin: 'decapitata',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}[1..].to_vec()`,
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        rs: 'clear',
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    /** Get first element */
    primus: {
        latin: 'primus',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.first()`,
    },

    /** Get last element */
    ultimus: {
        latin: 'ultimus',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.last()`,
    },

    /** Get element at index */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.get(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // PROPERTIES
    // -------------------------------------------------------------------------

    /** Get length */
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

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    /** Check if contains element */
    continet: {
        latin: 'continet',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.contains(&${args[0]})`,
    },

    /** Find index of element */
    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().position(|e| e == &${args[0]})`,
    },

    /** Find first element matching predicate */
    inveni: {
        latin: 'inveni',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().find(${args[0]})`,
    },

    /** Find index of first element matching predicate */
    inveniIndicem: {
        latin: 'inveniIndicem',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().position(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // TRANSFORMATIONS (return new vecs)
    // -------------------------------------------------------------------------

    /** Filter elements (returns new vec) */
    filtrata: {
        latin: 'filtrata',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().filter(${args[0]}).cloned().collect::<Vec<_>>()`,
    },

    /** Map elements (returns new vec) */
    mappata: {
        latin: 'mappata',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().map(${args[0]}).collect::<Vec<_>>()`,
    },

    /** Reduce to single value */
    reducta: {
        latin: 'reducta',
        mutates: false,
        async: false,
        rs: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}.iter().fold(${args[1]}, ${args[0]})`;
            }
            return `${obj}.iter().fold(Default::default(), ${args[0]})`;
        },
    },

    /** Flat map */
    explanata: {
        latin: 'explanata',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().flat_map(${args[0]}).collect::<Vec<_>>()`,
    },

    /** Flatten one level */
    plana: {
        latin: 'plana',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().flatten().cloned().collect::<Vec<_>>()`,
    },

    /** Reverse (returns new vec) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().rev().cloned().collect::<Vec<_>>()`,
    },

    /** Sort (returns new vec) */
    ordinata: {
        latin: 'ordinata',
        mutates: false,
        async: false,
        rs: (obj, args) => {
            if (args.length > 0) {
                return `{ let mut v = ${obj}.clone(); v.sort_by(${args[0]}); v }`;
            }
            return `{ let mut v = ${obj}.clone(); v.sort(); v }`;
        },
    },

    /** Slice */
    sectio: {
        latin: 'sectio',
        mutates: false,
        async: false,
        rs: (obj, args) => {
            if (args.length >= 2) {
                return `${obj}[${args[0]}..${args[1]}].to_vec()`;
            }
            return `${obj}[${args[0]}..].to_vec()`;
        },
    },

    /** Take first n elements */
    prima: {
        latin: 'prima',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().take(${args[0]}).cloned().collect::<Vec<_>>()`,
    },

    /** Take last n elements */
    ultima: {
        latin: 'ultima',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().rev().take(${args[0]}).cloned().collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>()`,
    },

    /** Skip first n elements */
    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().skip(${args[0]}).cloned().collect::<Vec<_>>()`,
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    /** Check if all elements match predicate */
    omnes: {
        latin: 'omnes',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().all(${args[0]})`,
    },

    /** Check if any element matches predicate */
    aliquis: {
        latin: 'aliquis',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().any(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // AGGREGATION
    // -------------------------------------------------------------------------

    /** Join elements to string */
    coniunge: {
        latin: 'coniunge',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.join(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate with callback */
    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().for_each(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // MUTATING OPERATIONS
    // -------------------------------------------------------------------------

    /** Sort in place (mutates) */
    ordina: {
        latin: 'ordina',
        mutates: true,
        async: false,
        rs: (obj, args) => {
            if (args.length > 0) {
                return `${obj}.sort_by(${args[0]})`;
            }
            return `${obj}.sort()`;
        },
    },

    /** Reverse in place (mutates) */
    inverte: {
        latin: 'inverte',
        mutates: true,
        async: false,
        rs: (obj, _args) => `${obj}.reverse()`,
    },

    // -------------------------------------------------------------------------
    // NUMERIC AGGREGATION
    // -------------------------------------------------------------------------

    /** Sum of numbers */
    summa: {
        latin: 'summa',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().sum::<i64>()`,
    },

    /** Average of numbers */
    medium: {
        latin: 'medium',
        mutates: false,
        async: false,
        rs: (obj, _args) => `(${obj}.iter().sum::<i64>() as f64 / ${obj}.len() as f64)`,
    },

    /** Minimum value */
    minimus: {
        latin: 'minimus',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().min()`,
    },

    /** Maximum value */
    maximus: {
        latin: 'maximus',
        mutates: false,
        async: false,
        rs: (obj, _args) => `${obj}.iter().max()`,
    },

    /** Minimum by key function */
    minimusPer: {
        latin: 'minimusPer',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().min_by_key(${args[0]})`,
    },

    /** Maximum by key function */
    maximusPer: {
        latin: 'maximusPer',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().max_by_key(${args[0]})`,
    },

    /** Count elements matching predicate */
    numera: {
        latin: 'numera',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().filter(${args[0]}).count()`,
    },

    // -------------------------------------------------------------------------
    // LODASH-INSPIRED METHODS
    // -------------------------------------------------------------------------

    /** Remove duplicates */
    unica: {
        latin: 'unica',
        mutates: false,
        async: false,
        rs: (obj, _args) =>
            `{ let mut seen = std::collections::HashSet::new(); ${obj}.iter().filter(|x| seen.insert((*x).clone())).cloned().collect::<Vec<_>>() }`,
    },

    /** Split into chunks of size n */
    fragmenta: {
        latin: 'fragmenta',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.chunks(${args[0]}).map(|c| c.to_vec()).collect::<Vec<_>>()`,
    },

    /** Partition by predicate -> (truthy, falsy) */
    partire: {
        latin: 'partire',
        mutates: false,
        async: false,
        rs: (obj, args) => `${obj}.iter().cloned().partition::<Vec<_>, _>(${args[0]})`,
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
