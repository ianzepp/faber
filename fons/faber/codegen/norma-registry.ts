/**
 * Norma Registry - Loads stdlib definitions from fons/norma/*.fab
 *
 * Data is generated at build time by: bun run build:norma
 * See consilia/futura/norma-faber.md for the full design.
 */

// WHY: Import generated registry data (avoids circular dependency with parser)
import { registry } from './norma-registry.gen';

// =============================================================================
// TYPES
// =============================================================================

/** Translation for a single target */
export interface VerteTranslation {
    /** Simple method name (e.g., 'push') */
    method?: string;
    /** Template string with S placeholders */
    template?: string;
    /** Parameter names for template form */
    params?: string[];
}

/** Method entry in the registry */
export interface NormaMethod {
    /** Method name (e.g., 'adde') */
    name: string;
    /** Translations per target */
    translations: Map<string, VerteTranslation>;
    /** Morphology forms from @ radix (if present) */
    radixForms?: string[];
}

/** Collection entry in the registry */
export interface NormaCollection {
    /** Collection name (e.g., 'lista') */
    name: string;
    /** Native type mappings from @ innatum */
    innatum: Map<string, string>;
    /** Methods defined for this collection */
    methods: Map<string, NormaMethod>;
}

// =============================================================================
// LOOKUP API
// =============================================================================

/**
 * Look up a norma translation for a method call.
 *
 * @param target Target language (ts, py, rs, cpp, zig)
 * @param collection Collection name (lista, tabula, copia)
 * @param method Method name (adde, filtrata, etc.)
 * @returns Translation if found, undefined otherwise
 */
export function getNormaTranslation(
    target: string,
    collection: string,
    method: string,
): VerteTranslation | undefined {
    const coll = registry.get(collection);
    if (!coll) return undefined;

    const m = coll.methods.get(method);
    if (!m) return undefined;

    return m.translations.get(target);
}

/**
 * Apply a norma template translation.
 *
 * @param template Template string with S placeholders
 * @param params Parameter names from @ verte
 * @param obj The object/receiver expression
 * @param args The argument expressions
 * @returns Generated code string
 */
export function applyNormaTemplate(
    template: string,
    params: string[],
    obj: string,
    args: string[],
): string {
    // Build value map: ego -> obj, other params -> args
    const values: string[] = [];
    for (const param of params) {
        if (param === 'ego') {
            values.push(obj);
        }
        else if (param === 'alloc') {
            // WHY: Allocator is handled specially by Zig codegen, skip for now
            values.push('allocator');
        }
        else {
            // Take next arg
            values.push(args.shift() || '');
        }
    }

    // Replace S placeholders positionally
    let result = template;
    let idx = 0;
    // WHY: Use regex to replace each S one at a time
    result = result.replace(/S/g, () => values[idx++] || '');

    return result;
}

/**
 * Check if a method has a norma definition for the given target.
 */
export function hasNormaMethod(
    target: string,
    collection: string,
    method: string,
): boolean {
    return getNormaTranslation(target, collection, method) !== undefined;
}

/**
 * Get all collections defined in norma files.
 */
export function getNormaCollections(): string[] {
    return Array.from(registry.keys());
}
