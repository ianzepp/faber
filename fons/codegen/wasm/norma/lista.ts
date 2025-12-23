/**
 * Lista Method Registry - WASM translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (WASM target)
 *
 * STATUS
 * ======
 * Placeholder - WASM array translations not yet implemented.
 *
 * WASM has no native array type. Implementation options:
 * - Linear memory with manual management
 * - Import array operations from host (JavaScript)
 * - Use WASM GC proposal (future)
 */

// TODO: Implement WASM-specific lista method translations
// See ts/norma/lista.ts for structure reference

export interface ListaMethod {
    latin: string;
    mutates: boolean;
    async: boolean;
    wasm: string | ((obj: string, args: string) => string);
}

export const LISTA_METHODS: Record<string, ListaMethod> = {
    // Placeholder - add methods as WASM backend matures
};

export function getListaMethod(name: string): ListaMethod | undefined {
    return LISTA_METHODS[name];
}

export function isListaMethod(name: string): boolean {
    return name in LISTA_METHODS;
}
