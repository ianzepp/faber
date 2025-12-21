/**
 * Built-in Types - Latin Type System Vocabulary
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (type vocabulary)
 *
 * ARCHITECTURE
 * ============
 * This module defines the built-in Latin type names that map to JavaScript/
 * TypeScript primitives and standard library types. Types follow Latin noun
 * declension patterns but use TitleCase to distinguish them from user-defined
 * identifiers.
 *
 * TYPE CATEGORIES:
 * - primitive: Basic scalar types (Textus, Numerus, Bivalens)
 * - collection: Container types (Lista, Tabula, Copia)
 * - structural: Complex types (Functio, Promissum, Erratum)
 * - iteration: Iterator/stream types (Cursor, Fluxus)
 *
 * LINGUISTIC DESIGN:
 * Latin noun genders and declensions are chosen for semantic clarity:
 * - Masculine: Active entities (Numerus, Cursor, Fluxus)
 * - Feminine: Collections (Lista, Tabula, Copia)
 * - Neuter: Abstract concepts (Datum, Erratum, Signum)
 *
 * TARGET MAPPING:
 * Each type maps to a JavaScript/TypeScript type for code generation.
 * Generic types (Array, Map, Set, Promise) support type parameters.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Type stem (string) to lookup
 * OUTPUT: TypeEntry with declension info and target language mapping
 * ERRORS: N/A - lookup operations do not fail
 *
 * INVARIANTS
 * ==========
 * INV-1: Type stems are TitleCase (Textus not textus)
 * INV-2: All types follow valid Latin declension patterns
 * INV-3: Generic types are marked with generic flag
 * INV-4: Each type has a unique stem
 *
 * @module lexicon/types-builtin
 */

import type { NounEntry } from "./types";

// =============================================================================
// TYPES
// =============================================================================

/**
 * Extended noun entry with target language mapping.
 *
 * DESIGN: Extends NounEntry because types decline like nouns. Adds target
 *         language information for code generation.
 */
export interface TypeEntry extends NounEntry {
    jsType: string
    category: "primitive" | "collection" | "structural" | "iteration"
    generic?: boolean
}

// =============================================================================
// BUILT-IN TYPE DEFINITIONS
// =============================================================================

/**
 * Complete vocabulary of Latin type names.
 *
 * WHY: Organized by category (primitives, collections, etc.) for clarity.
 *      Each type's gender and declension chosen for linguistic appropriateness.
 */
export const builtinTypes: TypeEntry[] = [
    // ---------------------------------------------------------------------------
    // Primitives
    // ---------------------------------------------------------------------------

    // WHY: Textus (4th decl masculine) - "texture, fabric, text"
    { stem: "Text", declension: 4, gender: "masculine", meaning: "text/string", jsType: "string", category: "primitive" },
    // WHY: Numerus (2nd decl masculine) - "number, count"
    { stem: "Numer", declension: 2, gender: "masculine", meaning: "number", jsType: "number", category: "primitive" },
    // WHY: Bivalens (3rd decl masculine) - "two-valued, having two values"
    { stem: "Bivalen", declension: 3, gender: "masculine", meaning: "boolean", jsType: "boolean", category: "primitive" },
    // WHY: Fractus (2nd decl masculine) - "broken, fractional" (floating point)
    { stem: "Fract", declension: 2, gender: "masculine", meaning: "floating point", jsType: "number", category: "primitive" },
    // WHY: Decimus (2nd decl masculine) - "tenth, decimal"
    { stem: "Decim", declension: 2, gender: "masculine", meaning: "decimal", jsType: "Decimal", category: "primitive" },
    // WHY: Signum (2nd decl neuter) - "sign, mark, token"
    { stem: "Sign", declension: 2, gender: "neuter", meaning: "symbol", jsType: "symbol", category: "primitive" },
    // WHY: Incertum (2nd decl neuter) - "uncertain, undefined"
    { stem: "Incert", declension: 2, gender: "neuter", meaning: "undefined", jsType: "undefined", category: "primitive" },
    // WHY: Nihil (irregular neuter) - "nothing, null"
    { stem: "Nihil", declension: 3, gender: "neuter", meaning: "null", jsType: "null", category: "primitive" },

    // ---------------------------------------------------------------------------
    // Collections (Generic)
    // ---------------------------------------------------------------------------

    // WHY: Lista (1st decl feminine) - "list, edge, border" (feminine for containers)
    { stem: "List", declension: 1, gender: "feminine", meaning: "list/array", jsType: "Array", category: "collection", generic: true },
    // WHY: Tabula (1st decl feminine) - "board, tablet, table"
    { stem: "Tabul", declension: 1, gender: "feminine", meaning: "table/map", jsType: "Map", category: "collection", generic: true },
    // WHY: Copia (1st decl feminine) - "abundance, supply, collection"
    { stem: "Copi", declension: 1, gender: "feminine", meaning: "set/collection", jsType: "Set", category: "collection", generic: true },

    // ---------------------------------------------------------------------------
    // Structural Types
    // ---------------------------------------------------------------------------

    // WHY: Res (3rd decl feminine) - "thing, object, matter" (irregular but fundamental)
    { stem: "R", declension: 3, gender: "feminine", meaning: "thing/object", jsType: "object", category: "structural" },
    // WHY: Functio (3rd decl feminine) - "performance, execution, function"
    { stem: "Function", declension: 3, gender: "feminine", meaning: "function", jsType: "Function", category: "structural" },
    // WHY: Promissum (2nd decl neuter) - "promise, guarantee" (neuter for abstract concepts)
    { stem: "Promiss", declension: 2, gender: "neuter", meaning: "promise", jsType: "Promise", category: "structural", generic: true },
    // WHY: Forsitan (adverb as type) - "perhaps, maybe" (optional type)
    { stem: "Forsitan", declension: 3, gender: "neuter", meaning: "optional", jsType: "T | null", category: "structural", generic: true },
    // WHY: Fors (3rd decl feminine) - "chance, fortune" (result type)
    { stem: "For", declension: 3, gender: "feminine", meaning: "result", jsType: "Result", category: "structural", generic: true },
    // WHY: Tempus (3rd decl neuter) - "time, period"
    { stem: "Tempor", declension: 3, gender: "neuter", meaning: "time/date", jsType: "Date", category: "structural" },
    // WHY: Erratum (2nd decl neuter) - "error, mistake" (neuter participle)
    { stem: "Errat", declension: 2, gender: "neuter", meaning: "error", jsType: "Error", category: "structural" },
    // WHY: Vacuum (2nd decl neuter) - "empty space, void"
    { stem: "Vacu", declension: 2, gender: "neuter", meaning: "void/empty", jsType: "void", category: "structural" },
    // WHY: Quodlibet (3rd decl neuter) - "whatever you please, anything"
    { stem: "Quodlibet", declension: 3, gender: "neuter", meaning: "any", jsType: "any", category: "structural" },
    // WHY: Ignotum (2nd decl neuter) - "unknown, unfamiliar"
    { stem: "Ignot", declension: 2, gender: "neuter", meaning: "unknown", jsType: "unknown", category: "structural" },

    // ---------------------------------------------------------------------------
    // Iteration & Streaming
    // ---------------------------------------------------------------------------

    // WHY: Cursor (3rd decl masculine) - "runner, iterator" (agent noun, masculine)
    { stem: "Cursor", declension: 3, gender: "masculine", meaning: "cursor/iterator", jsType: "Iterator", category: "iteration", generic: true },
    // WHY: Fluxus (4th decl masculine) - "flow, stream, flux"
    { stem: "Flux", declension: 4, gender: "masculine", meaning: "flow/stream", jsType: "AsyncIterable", category: "iteration", generic: true },
    // WHY: FuturaCursor (compound) - "future iterator" (async iterator)
    { stem: "FuturaCursor", declension: 3, gender: "masculine", meaning: "async iterator", jsType: "AsyncIterator", category: "iteration", generic: true },
    // WHY: FuturusFluxus (compound) - "future stream" (async stream)
    { stem: "FuturusFlux", declension: 4, gender: "masculine", meaning: "async stream", jsType: "AsyncIterable", category: "iteration", generic: true },

    // ---------------------------------------------------------------------------
    // Systems Types (Zig/WASM targets)
    // ---------------------------------------------------------------------------

    // WHY: Indicium (2nd decl neuter) - "indication, pointer"
    { stem: "Indici", declension: 2, gender: "neuter", meaning: "pointer", jsType: "pointer", category: "structural", generic: true },
    // WHY: Refera (1st decl feminine) - "reference" (from refero - to refer back)
    { stem: "Refer", declension: 1, gender: "feminine", meaning: "reference", jsType: "reference", category: "structural", generic: true },

    // ---------------------------------------------------------------------------
    // Utility Types (TypeScript only)
    // ---------------------------------------------------------------------------

    // WHY: Pars (3rd decl feminine) - "part, portion" (Partial<T>)
    { stem: "Par", declension: 3, gender: "feminine", meaning: "partial", jsType: "Partial", category: "structural", generic: true },
    // WHY: Totum (2nd decl neuter) - "whole, total" (Required<T>)
    { stem: "Tot", declension: 2, gender: "neuter", meaning: "required", jsType: "Required", category: "structural", generic: true },
    // WHY: Lectum (2nd decl neuter) - "read, chosen" (Readonly<T>)
    { stem: "Lect", declension: 2, gender: "neuter", meaning: "readonly", jsType: "Readonly", category: "structural", generic: true },
    // WHY: Registrum (2nd decl neuter) - "record, registry" (Record<K, V>)
    { stem: "Registr", declension: 2, gender: "neuter", meaning: "record", jsType: "Record", category: "structural", generic: true },
    // WHY: Selectum (2nd decl neuter) - "selected, picked" (Pick<T, K>)
    { stem: "Select", declension: 2, gender: "neuter", meaning: "pick", jsType: "Pick", category: "structural", generic: true },
    // WHY: Omissum (2nd decl neuter) - "omitted, left out" (Omit<T, K>)
    { stem: "Omiss", declension: 2, gender: "neuter", meaning: "omit", jsType: "Omit", category: "structural", generic: true },
    // WHY: Extractum (2nd decl neuter) - "extracted, drawn out" (Extract<T, U>)
    { stem: "Extract", declension: 2, gender: "neuter", meaning: "extract", jsType: "Extract", category: "structural", generic: true },
    // WHY: Exclusum (2nd decl neuter) - "excluded, shut out" (Exclude<T, U>)
    { stem: "Exclus", declension: 2, gender: "neuter", meaning: "exclude", jsType: "Exclude", category: "structural", generic: true },
    // WHY: NonNihil (compound) - "not null" (NonNullable<T>)
    { stem: "NonNihil", declension: 3, gender: "neuter", meaning: "non-nullable", jsType: "NonNullable", category: "structural", generic: true },
    // WHY: Reditus (4th decl masculine) - "return, going back" (ReturnType<T>)
    { stem: "Redit", declension: 4, gender: "masculine", meaning: "return type", jsType: "ReturnType", category: "structural", generic: true },
    // WHY: Parametra (1st decl feminine plural) - "parameters" (Parameters<T>)
    { stem: "Parametr", declension: 1, gender: "feminine", meaning: "parameters", jsType: "Parameters", category: "structural", generic: true },
];

// =============================================================================
// TYPE MODIFIERS
// =============================================================================

/**
 * Type modifier constants for parameterized types.
 *
 * WHY: These Latin modifiers express ownership, mutability, and signedness
 *      semantics that are critical for systems programming (Zig target) but
 *      can be ignored for TypeScript target.
 *
 * USAGE:
 *   - Numerus<32, Naturalis>  (unsigned 32-bit integer)
 *   - Textus<Proprius>        (owned string)
 *   - Textus<Alienus>         (borrowed string reference)
 *   - Indicium<T, Mutabilis>  (mutable pointer)
 */
export const typeModifiers = {
    // WHY: Naturalis (3rd decl adj) - "natural" (unsigned numbers in systems programming)
    Naturalis: "unsigned",

    // WHY: Proprius (2nd decl adj) - "one's own" (owned/move semantics)
    Proprius: "owned",

    // WHY: Alienus (2nd decl adj) - "belonging to another" (borrowed/borrow semantics)
    Alienus: "borrowed",

    // WHY: Mutabilis (3rd decl adj) - "changeable, mutable"
    Mutabilis: "mutable",
} as const;

/**
 * Type for valid modifier names.
 */
export type TypeModifier = keyof typeof typeModifiers;

/**
 * Check if a name is a valid type modifier.
 *
 * @param name - The name to check
 * @returns true if name is a valid type modifier
 */
export function isTypeModifier(name: string): name is TypeModifier {
    return name in typeModifiers;
}

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Type lookup map for O(1) access.
 *
 * PERF: Pre-computed Map is faster than linear array search.
 *
 * WHY: Lowercase keys allow case-insensitive stem lookup while preserving
 *      TitleCase in the actual type entries.
 */
const typeMap = new Map(builtinTypes.map(t => [t.stem.toLowerCase(), t]));

/**
 * Check if a stem is a built-in type.
 *
 * @param stem - The type stem to check (e.g., "Text", "Numer")
 * @returns true if stem is a built-in type, false otherwise
 */
export function isBuiltinType(stem: string): boolean {
    return typeMap.has(stem.toLowerCase());
}

/**
 * Get type metadata for a stem.
 *
 * @param stem - The type stem to lookup
 * @returns TypeEntry if stem is a built-in type, undefined otherwise
 */
export function getBuiltinType(stem: string): TypeEntry | undefined {
    return typeMap.get(stem.toLowerCase());
}
