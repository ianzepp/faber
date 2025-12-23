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
 * | Latin         | Rust                              |
 * |---------------|-----------------------------------|
 * | adde          | vec.push(x)                       |
 * | addita        | [vec.clone(), vec![x]].concat()   |
 * | filtrata      | vec.iter().filter(|x| ...).collect() |
 * | mappata       | vec.iter().map(|x| ...).collect() |
 * | reducta       | vec.iter().fold(init, |acc, x| ...)|
 * | primus        | vec.first()                       |
 * | ultimus       | vec.last()                        |
 * | longitudo     | vec.len()                         |
 * | continet      | vec.contains(&x)                  |
 * | inversa       | vec.iter().rev().collect()        |
 * | ordinata      | { let mut v = vec.clone(); v.sort(); v } |
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

type RsGenerator = (obj: string, args: string) => string;

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with Rust translations.
 *
 * TODO: Implement all methods from checklist.md
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        rs: 'push',
    },

    addita: {
        latin: 'addita',
        mutates: false,
        async: false,
        rs: (obj, args) => `{ let mut v = ${obj}.clone(); v.push(${args}); v }`,
    },

    // TODO: Implement remaining methods
    // See checklist.md for full list
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
