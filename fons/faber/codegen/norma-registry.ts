/**
 * Norma Registry - Loads stdlib definitions from fons/norma/*.fab
 *
 * Data is generated at build time by: bun run build:norma
 * See consilia/futura/norma-faber.md for the full design.
 */

// WHY: Import generated registry data (avoids circular dependency with parser)
import { registry } from './norma-registry.gen';
import { parseMethodum, parseMethodumWithStem } from './morphology';

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
export function getNormaTranslation(target: string, collection: string, method: string): VerteTranslation | undefined {
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
export function applyNormaTemplate(template: string, params: string[], obj: string, args: string[]): string {
    // Build value map: ego -> obj, other params -> args
    // WHY: Zig codegen passes curator as last arg when template has 'alloc' param
    const values: string[] = [];
    for (const param of params) {
        if (param === 'ego') {
            values.push(obj);
        } else {
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
export function hasNormaMethod(target: string, collection: string, method: string): boolean {
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
export function applyNormaModuleCall(target: string, module: string, func: string, args: string[]): string | undefined {
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

// =============================================================================
// MORPHOLOGY VALIDATION
// =============================================================================

/** Result of morphology validation */
export interface MorphologyValidation {
    valid: boolean;
    /** Error message if invalid */
    error?: string;
    /** Parsed stem (if morphology recognized) */
    stem?: string;
    /** Parsed form (if morphology recognized) */
    form?: string;
}

/**
 * Get the radixForms for a method in a collection.
 *
 * @returns radixForms array if method has @ radix, undefined otherwise
 */
export function getNormaRadixForms(collection: string, method: string): string[] | undefined {
    const coll = registry.get(collection);
    if (!coll) return undefined;

    const m = coll.methods.get(method);
    if (!m) return undefined;

    return m.radixForms;
}

/** Receiver ownership for method calls */
export type ReceiverOwnership = 'de' | 'in' | undefined;

/**
 * Get the implied receiver ownership for a stdlib method.
 *
 * WHY: Latin morphology encodes mutation semantics:
 *   - imperativus (adde, filtra) → mutates receiver → 'in' (&mut self)
 *   - perfectum (addita, filtrata) → returns new → 'de' (&self)
 *   - futurum_indicativum (addebit) → async mutates → 'in'
 *   - futurum_activum (additura) → async returns new → 'de'
 *   - participium_praesens (addens) → streaming → 'de'
 *
 * @param collection Collection name (lista, tabula, copia, etc.)
 * @param methodName Method being called
 * @returns 'in' for mutating, 'de' for non-mutating, undefined if unknown
 */
export function getReceiverOwnership(collection: string, methodName: string): ReceiverOwnership {
    const coll = registry.get(collection);
    if (!coll) return undefined;

    const method = coll.methods.get(methodName);
    if (!method) return undefined;

    // If method has radixForms, use stem-guided parsing
    if (method.radixForms && method.radixForms.length > 0) {
        const stem = method.radixForms[0]!;
        const parsed = parseMethodumWithStem(methodName, stem);
        if (parsed) {
            // mutare=true forms need mutable receiver
            if (parsed.flags.mutare) {
                return 'in';
            }
            // mutare=false forms only need immutable receiver
            return 'de';
        }
    }

    // Fallback: try parsing without stem hint
    const parsed = parseMethodum(methodName);
    if (parsed) {
        return parsed.flags.mutare ? 'in' : 'de';
    }

    // Unknown morphology - no ownership inference
    return undefined;
}

/**
 * Validate morphology for a method call on a stdlib collection.
 *
 * Validation only runs when:
 * 1. The collection is in the stdlib (norma registry)
 * 2. The method (or a related stem) has @ radix defined
 *
 * @param collection Collection name (lista, tabula, copia, etc.)
 * @param methodName Method being called (filtrata, adde, etc.)
 * @returns Validation result with error message if invalid
 */
export function validateMorphology(collection: string, methodName: string): MorphologyValidation {
    const coll = registry.get(collection);
    if (!coll) {
        // Not a stdlib collection - skip validation
        return { valid: true };
    }

    // Check if method exists directly
    const directMethod = coll.methods.get(methodName);
    if (directMethod) {
        // Method exists - if it has radixForms, validate the morphology
        if (directMethod.radixForms) {
            const declaredStem = directMethod.radixForms[0]!;
            const declaredForms = directMethod.radixForms.slice(1);

            // WHY: Use stem-guided parsing first. The greedy parser can misparse
            // words like 'decapita' as 'decap' + '-ita' instead of 'decapit' + '-a'.
            const parsed = parseMethodumWithStem(methodName, declaredStem) || parseMethodum(methodName);
            if (parsed) {
                // Check stem matches (should always match with stem-guided parsing)
                if (parsed.stem !== declaredStem) {
                    return {
                        valid: false,
                        error: `Morphology mismatch: '${methodName}' has stem '${parsed.stem}', expected '${declaredStem}'`,
                        stem: parsed.stem,
                        form: parsed.form,
                    };
                }

                // Check form is declared
                if (!declaredForms.includes(parsed.form)) {
                    return {
                        valid: false,
                        error: `Morphology form '${parsed.form}' not declared for stem '${declaredStem}'. Valid forms: ${declaredForms.join(', ')}`,
                        stem: parsed.stem,
                        form: parsed.form,
                    };
                }
            }
        }
        // Method exists and passes validation (or has no radixForms)
        return { valid: true };
    }

    // Method doesn't exist directly - try parsing morphology to find related stem
    // WHY: Check all methods with radixForms to see if this could be an undeclared
    // form of a known verb stem. Use stem-guided parsing to handle ambiguous cases.
    for (const method of coll.methods.values()) {
        if (!method.radixForms) continue;

        const declaredStem = method.radixForms[0]!;
        const declaredForms = method.radixForms.slice(1);

        // Try stem-guided parsing
        const parsed = parseMethodumWithStem(methodName, declaredStem);
        if (parsed) {
            // Found a stem match - check if form is declared
            if (!declaredForms.includes(parsed.form)) {
                return {
                    valid: false,
                    error: `Unknown method '${methodName}': stem '${parsed.stem}' exists but form '${parsed.form}' is not declared. Valid forms: ${declaredForms.join(', ')}`,
                    stem: parsed.stem,
                    form: parsed.form,
                };
            }
            // Form is declared but method not registered - should not happen
            // (if form is declared, the method should exist). Pass through.
            break;
        }
    }

    // Method doesn't exist and no morphology match - let it pass through
    // WHY: Could be a user-defined method or extension. Codegen will handle.
    return { valid: true };
}
