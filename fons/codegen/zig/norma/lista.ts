/**
 * Lista Method Registry - Zig translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * STATUS
 * ======
 * Placeholder - Zig ArrayList translations not yet implemented.
 *
 * Zig uses std.ArrayList(T) which has different semantics:
 * - Explicit allocator required
 * - append() instead of push()
 * - items field for slice access
 * - Memory must be manually managed
 */

// TODO: Implement Zig-specific lista method translations
// See ts/norma/lista.ts for structure reference

export interface ListaMethod {
    latin: string;
    mutates: boolean;
    async: boolean;
    zig: string | ((obj: string, args: string) => string);
}

export const LISTA_METHODS: Record<string, ListaMethod> = {
    // Placeholder - add methods as Zig backend matures
};

export function getListaMethod(name: string): ListaMethod | undefined {
    return LISTA_METHODS[name];
}

export function isListaMethod(name: string): boolean {
    return name in LISTA_METHODS;
}
