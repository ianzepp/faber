/**
 * Norma Registry - Loads stdlib definitions from fons/norma/index.json
 *
 * Data is generated at build time by: bun run build:norma
 * See consilia/futura/norma-faber.md for the full design.
 *
 * Key structure:
 *   "lista"           → { innatum: { ts: "Array", ... } }
 *   "lista:adde"      → { radixForms: ["add", "imperativus", "perfectum"] }
 *   "lista:adde:ts"   → { method: "push" } or { template: "...", params: [...] }
 */

import registryData from '../../../fons/norma/index.json';
import { parseMethodum, parseMethodumWithStem } from './morphology';

// =============================================================================
// TYPES
// =============================================================================

/** Entry in the flat JSON registry */
interface RegistryEntry {
    // Collection-level (1 part key)
    innatum?: Record<string, string>;
    // Method-level (2 part key)
    radixForms?: string[];
    // Translation-level (3 part key)
    method?: string;
    template?: string;
    params?: string[];
}

const registry = registryData as Record<string, RegistryEntry>;

/** Translation for a single target */
export interface VerteTranslation {
    /** Simple method name (e.g., 'push') */
    method?: string;
    /** Template string with § placeholders */
    template?: string;
    /** Parameter names for template form */
    params?: string[];
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
    const key = `${collection}:${method}:${target}`;
    const entry = registry[key];
    if (!entry) return undefined;

    // Only return if it's a translation entry (has method or template)
    if (entry.method || entry.template) {
        return {
            method: entry.method,
            template: entry.template,
            params: entry.params,
        };
    }
    return undefined;
}

/**
 * Apply a norma template translation.
 *
 * @param template Template string with § placeholders
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
export function hasNormaMethod(target: string, collection: string, method: string): boolean {
    return getNormaTranslation(target, collection, method) !== undefined;
}

/**
 * Get all collections defined in norma files.
 */
export function getNormaCollections(): string[] {
    const collections: string[] = [];
    for (const key of Object.keys(registry)) {
        // Collection keys have no colons
        const entry = registry[key];
        if (!key.includes(':') && entry?.innatum) {
            collections.push(key);
        }
    }
    return collections;
}

/**
 * Find receiver-type collections that define a given method for a target.
 *
 * WHY: When the semantic analyzer cannot resolve a receiver type (UNKNOWN),
 *      TS codegen must not guess. This helper lets codegen detect "this looks
 *      like a stdlib method" and emit a compiler error prompting a type fix.
 */
export function getNormaReceiverCollectionsForMethod(target: string, method: string): string[] {
    // Restrict to receiver types (not module-like entries such as mathesis/solum).
    // Keep this list small and explicit; expand as receiver stdlib grows.
    const receiverCollections = ['lista', 'tabula', 'copia', 'textus'];

    const matches: string[] = [];
    for (const collection of receiverCollections) {
        const key = `${collection}:${method}:${target}`;
        if (registry[key]) {
            matches.push(collection);
        }
    }
    return matches;
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
    const key = `${collection}:${method}`;
    const entry = registry[key];
    return entry?.radixForms;
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
    // Check if collection exists
    if (!registry[collection]) return undefined;

    // Check if method has a translation for any target (method exists)
    const hasMethod = Object.keys(registry).some(k => k.startsWith(`${collection}:${methodName}:`));
    if (!hasMethod) return undefined;

    // Get radixForms if available
    const radixForms = getNormaRadixForms(collection, methodName);

    // If method has radixForms, use stem-guided parsing
    if (radixForms && radixForms.length > 0) {
        const stem = radixForms[0]!;
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
 * Get all methods with radixForms for a collection.
 */
function getMethodsWithRadix(collection: string): Array<{ method: string; radixForms: string[] }> {
    const methods: Array<{ method: string; radixForms: string[] }> = [];
    const prefix = `${collection}:`;

    for (const key of Object.keys(registry)) {
        if (key.startsWith(prefix) && !key.includes(':', prefix.length)) {
            // This is a method-level key (collection:method, not collection:method:target)
            const entry = registry[key];
            if (entry?.radixForms) {
                methods.push({
                    method: key.slice(prefix.length),
                    radixForms: entry.radixForms,
                });
            }
        }
    }
    return methods;
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
    // Check if collection exists
    if (!registry[collection]) {
        // Not a stdlib collection - skip validation
        return { valid: true };
    }

    // Get radixForms for this method directly
    const directRadix = getNormaRadixForms(collection, methodName);

    if (directRadix) {
        // Method exists with radixForms - validate the morphology
        const declaredStem = directRadix[0]!;
        const declaredForms = directRadix.slice(1);

        // WHY: Use stem-guided parsing first. The greedy parser can misparse
        // words like 'decapita' as 'decap' + '-ita' instead of 'decapit' + '-a'.
        const stemParsed = parseMethodumWithStem(methodName, declaredStem);
        const greedyParsed = parseMethodum(methodName);

        // WHY: Prefer stem-guided parser, but if it returns a form that's not declared,
        // check if the greedy parser gives a declared form. This handles cases like
        // 'selecta' with stem 'select': stem-guided gives 'a' -> imperativus (wrong),
        // but greedy gives 'ta' -> perfectum (correct with stem 'selec').
        let parsed = stemParsed;
        if (stemParsed && !declaredForms.includes(stemParsed.form)) {
            if (greedyParsed && declaredForms.includes(greedyParsed.form)) {
                // Greedy parser gives a declared form - prefer it over stem-guided
                // We override the stem to match the declared stem for validation
                parsed = { ...greedyParsed, stem: declaredStem };
            }
        }

        // Fallback to whichever parser succeeded
        if (!parsed) {
            parsed = stemParsed || greedyParsed;
        }

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

        // Method exists and passes validation
        return { valid: true };
    }

    // Check if method exists (has any translation)
    const hasMethod = Object.keys(registry).some(k => k.startsWith(`${collection}:${methodName}:`));
    if (hasMethod) {
        // Method exists but no radixForms - skip validation
        return { valid: true };
    }

    // Method doesn't exist directly - try parsing morphology to find related stem
    // WHY: Check all methods with radixForms to see if this could be an undeclared
    // form of a known verb stem. Use stem-guided parsing to handle ambiguous cases.
    for (const { radixForms } of getMethodsWithRadix(collection)) {
        const declaredStem = radixForms[0]!;
        const declaredForms = radixForms.slice(1);

        // Try both parsers with disambiguation logic (same as above)
        const stemParsed = parseMethodumWithStem(methodName, declaredStem);
        const greedyParsed = parseMethodum(methodName);

        let parsed = stemParsed;
        if (stemParsed && !declaredForms.includes(stemParsed.form)) {
            if (greedyParsed && declaredForms.includes(greedyParsed.form)) {
                parsed = { ...greedyParsed, stem: declaredStem };
            }
        }

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
