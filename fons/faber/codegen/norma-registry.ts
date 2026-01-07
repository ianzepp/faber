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
    // WHY: Zig codegen passes curator as last arg when template has 'alloc' param
    const values: string[] = [];
    for (const param of params) {
        if (param === 'ego') {
            values.push(obj);
        }
        else {
            // Take next arg (includes 'alloc' - curator passed by Zig codegen)
            values.push(args.shift() || '');
        }
    }

    // Replace § placeholders - supports both positional and indexed
    // §  = next value (implicit positional)
    // §0, §1, etc. = explicit index into values array
    let result = template;
    let implicitIdx = 0;

    result = result.replace(/§(\d+)?/g, (_, indexStr) => {
        if (indexStr !== undefined) {
            // Explicit index: §0, §1, etc.
            const idx = parseInt(indexStr, 10);
            return values[idx] || '';
        }
        // Implicit positional: plain §
        return values[implicitIdx++] || '';
    });

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

/**
 * Apply a norma module function call (no receiver object).
 *
 * For module functions like mathesis.pavimentum(x) or solum.lege(path),
 * there's no 'ego' receiver - just direct function arguments.
 *
 * @param target Target language (ts, py, rs, cpp, zig)
 * @param module Module name (mathesis, solum, etc.)
 * @param func Function name (pavimentum, lege, etc.)
 * @param args The argument expressions
 * @returns Generated code string, or undefined if not found
 */
export function applyNormaModuleCall(
    target: string,
    module: string,
    func: string,
    args: string[],
): string | undefined {
    const translation = getNormaTranslation(target, module, func);
    if (!translation?.template || !translation?.params) {
        return undefined;
    }

    // For module functions, params map directly to args (no ego)
    const values = [...args];

    // Replace § placeholders
    let result = translation.template;
    let implicitIdx = 0;

    result = result.replace(/§(\d+)?/g, (_, indexStr) => {
        if (indexStr !== undefined) {
            const idx = parseInt(indexStr, 10);
            return values[idx] || '';
        }
        return values[implicitIdx++] || '';
    });

    return result;
}
