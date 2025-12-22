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

import type { NounEntry } from './types';

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
    jsType: string;
    category: 'primitive' | 'collection' | 'structural' | 'iteration';
    generic?: boolean;
    // WHY: 3rd declension neuters like tempus/temporis have nominatives that
    //      differ from their stems. This field allows parseType to recognize them.
    nominative?: string;
}

// =============================================================================
// BUILT-IN TYPE DEFINITIONS
// =============================================================================

/**
 * Complete vocabulary of Latin type names.
 *
 * WHY: Organized by category (primitives, collections, etc.) for clarity.
 *      Each type's gender and declension chosen for linguistic appropriateness.
 *
 * CASE: Stems are lowercase (canonical form). Lookup is case-insensitive,
 *       so "textus", "Textus", and "TEXTUS" all resolve to the same type.
 */
export const builtinTypes: TypeEntry[] = [
    // ---------------------------------------------------------------------------
    // Primitives
    // ---------------------------------------------------------------------------

    // WHY: textus (4th decl masculine) - "texture, fabric, text"
    {
        stem: 'text',
        declension: 4,
        gender: 'masculine',
        meaning: 'text/string',
        jsType: 'string',
        category: 'primitive',
    },
    // WHY: numerus (2nd decl masculine) - "number, count"
    {
        stem: 'numer',
        declension: 2,
        gender: 'masculine',
        meaning: 'number',
        jsType: 'number',
        category: 'primitive',
    },
    // WHY: bivalens (3rd decl masculine) - "two-valued, having two values"
    {
        stem: 'bivalen',
        declension: 3,
        gender: 'masculine',
        meaning: 'boolean',
        jsType: 'boolean',
        category: 'primitive',
    },
    // WHY: fractus (2nd decl masculine) - "broken, fractional" (floating point)
    {
        stem: 'fract',
        declension: 2,
        gender: 'masculine',
        meaning: 'floating point',
        jsType: 'number',
        category: 'primitive',
    },
    // WHY: decimus (2nd decl masculine) - "tenth, decimal"
    {
        stem: 'decim',
        declension: 2,
        gender: 'masculine',
        meaning: 'decimal',
        jsType: 'Decimal',
        category: 'primitive',
    },
    // WHY: signum (2nd decl neuter) - "sign, mark, token"
    {
        stem: 'sign',
        declension: 2,
        gender: 'neuter',
        meaning: 'symbol',
        jsType: 'symbol',
        category: 'primitive',
    },
    // WHY: incertum (2nd decl neuter) - "uncertain, undefined"
    {
        stem: 'incert',
        declension: 2,
        gender: 'neuter',
        meaning: 'undefined',
        jsType: 'undefined',
        category: 'primitive',
    },
    // WHY: nihil (irregular neuter) - "nothing, null"
    {
        stem: 'nihil',
        declension: 3,
        gender: 'neuter',
        meaning: 'null',
        jsType: 'null',
        category: 'primitive',
    },

    // ---------------------------------------------------------------------------
    // Collections (Generic)
    // ---------------------------------------------------------------------------

    // WHY: lista (1st decl feminine) - "list, edge, border" (feminine for containers)
    {
        stem: 'list',
        declension: 1,
        gender: 'feminine',
        meaning: 'list/array',
        jsType: 'Array',
        category: 'collection',
        generic: true,
    },
    // WHY: tabula (1st decl feminine) - "board, tablet, table"
    {
        stem: 'tabul',
        declension: 1,
        gender: 'feminine',
        meaning: 'table/map',
        jsType: 'Map',
        category: 'collection',
        generic: true,
    },
    // WHY: copia (1st decl feminine) - "abundance, supply, collection"
    {
        stem: 'copi',
        declension: 1,
        gender: 'feminine',
        meaning: 'set/collection',
        jsType: 'Set',
        category: 'collection',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Structural Types
    // ---------------------------------------------------------------------------

    // WHY: promissum (2nd decl neuter) - "promise, guarantee" (neuter for abstract concepts)
    {
        stem: 'promiss',
        declension: 2,
        gender: 'neuter',
        meaning: 'promise',
        jsType: 'Promise',
        category: 'structural',
        generic: true,
    },
    // WHY: forsitan (adverb as type) - "perhaps, maybe" (optional type)
    {
        stem: 'forsitan',
        declension: 3,
        gender: 'neuter',
        meaning: 'optional',
        jsType: 'T | null',
        category: 'structural',
        generic: true,
    },
    // WHY: fors (3rd decl feminine) - "chance, fortune" (result type)
    {
        stem: 'for',
        declension: 3,
        gender: 'feminine',
        meaning: 'result',
        jsType: 'Result',
        category: 'structural',
        generic: true,
    },
    // WHY: tempus (3rd decl neuter) - "time, period"
    //      Nominative differs from stem (tempus vs tempor-)
    {
        stem: 'tempor',
        nominative: 'tempus',
        declension: 3,
        gender: 'neuter',
        meaning: 'time/date',
        jsType: 'Date',
        category: 'structural',
    },
    // WHY: erratum (2nd decl neuter) - "error, mistake" (neuter participle)
    {
        stem: 'errat',
        declension: 2,
        gender: 'neuter',
        meaning: 'error',
        jsType: 'Error',
        category: 'structural',
    },
    // WHY: vacuum (2nd decl neuter) - "empty space, void"
    {
        stem: 'vacu',
        declension: 2,
        gender: 'neuter',
        meaning: 'void/empty',
        jsType: 'void',
        category: 'structural',
    },
    // WHY: quodlibet (3rd decl neuter) - "whatever you please, anything"
    {
        stem: 'quodlibet',
        declension: 3,
        gender: 'neuter',
        meaning: 'any',
        jsType: 'any',
        category: 'structural',
    },
    // WHY: ignotum (2nd decl neuter) - "unknown, unfamiliar"
    {
        stem: 'ignot',
        declension: 2,
        gender: 'neuter',
        meaning: 'unknown',
        jsType: 'unknown',
        category: 'structural',
    },

    // ---------------------------------------------------------------------------
    // Iteration & Streaming
    // ---------------------------------------------------------------------------

    // WHY: cursor (3rd decl masculine) - "runner, iterator" (agent noun, masculine)
    {
        stem: 'cursor',
        declension: 3,
        gender: 'masculine',
        meaning: 'cursor/iterator',
        jsType: 'Iterator',
        category: 'iteration',
        generic: true,
    },
    // WHY: fluxus (4th decl masculine) - "flow, stream, flux"
    {
        stem: 'flux',
        declension: 4,
        gender: 'masculine',
        meaning: 'flow/stream',
        jsType: 'AsyncIterable',
        category: 'iteration',
        generic: true,
    },
    // WHY: futuracursor (compound) - "future iterator" (async iterator)
    {
        stem: 'futuracursor',
        declension: 3,
        gender: 'masculine',
        meaning: 'async iterator',
        jsType: 'AsyncIterator',
        category: 'iteration',
        generic: true,
    },
    // WHY: futurusfluxus (compound) - "future stream" (async stream)
    {
        stem: 'futurusflux',
        declension: 4,
        gender: 'masculine',
        meaning: 'async stream',
        jsType: 'AsyncIterable',
        category: 'iteration',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Systems Types (Zig/WASM targets)
    // ---------------------------------------------------------------------------

    // WHY: indicium (2nd decl neuter) - "indication, pointer"
    {
        stem: 'indici',
        declension: 2,
        gender: 'neuter',
        meaning: 'pointer',
        jsType: 'pointer',
        category: 'structural',
        generic: true,
    },
    // WHY: refera (1st decl feminine) - "reference" (from refero - to refer back)
    {
        stem: 'refer',
        declension: 1,
        gender: 'feminine',
        meaning: 'reference',
        jsType: 'reference',
        category: 'structural',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Utility Types (TypeScript only)
    // ---------------------------------------------------------------------------

    // WHY: pars (3rd decl feminine) - "part, portion" (Partial<T>)
    {
        stem: 'par',
        declension: 3,
        gender: 'feminine',
        meaning: 'partial',
        jsType: 'Partial',
        category: 'structural',
        generic: true,
    },
    // WHY: totum (2nd decl neuter) - "whole, total" (Required<T>)
    {
        stem: 'tot',
        declension: 2,
        gender: 'neuter',
        meaning: 'required',
        jsType: 'Required',
        category: 'structural',
        generic: true,
    },
    // WHY: lectum (2nd decl neuter) - "read, chosen" (Readonly<T>)
    {
        stem: 'lect',
        declension: 2,
        gender: 'neuter',
        meaning: 'readonly',
        jsType: 'Readonly',
        category: 'structural',
        generic: true,
    },
    // WHY: registrum (2nd decl neuter) - "record, registry" (Record<K, V>)
    {
        stem: 'registr',
        declension: 2,
        gender: 'neuter',
        meaning: 'record',
        jsType: 'Record',
        category: 'structural',
        generic: true,
    },
    // WHY: selectum (2nd decl neuter) - "selected, picked" (Pick<T, K>)
    {
        stem: 'select',
        declension: 2,
        gender: 'neuter',
        meaning: 'pick',
        jsType: 'Pick',
        category: 'structural',
        generic: true,
    },
    // WHY: omissum (2nd decl neuter) - "omitted, left out" (Omit<T, K>)
    {
        stem: 'omiss',
        declension: 2,
        gender: 'neuter',
        meaning: 'omit',
        jsType: 'Omit',
        category: 'structural',
        generic: true,
    },
    // WHY: extractum (2nd decl neuter) - "extracted, drawn out" (Extract<T, U>)
    {
        stem: 'extract',
        declension: 2,
        gender: 'neuter',
        meaning: 'extract',
        jsType: 'Extract',
        category: 'structural',
        generic: true,
    },
    // WHY: exclusum (2nd decl neuter) - "excluded, shut out" (Exclude<T, U>)
    {
        stem: 'exclus',
        declension: 2,
        gender: 'neuter',
        meaning: 'exclude',
        jsType: 'Exclude',
        category: 'structural',
        generic: true,
    },
    // WHY: nonnihil (compound) - "not null" (NonNullable<T>)
    {
        stem: 'nonnihil',
        declension: 3,
        gender: 'neuter',
        meaning: 'non-nullable',
        jsType: 'NonNullable',
        category: 'structural',
        generic: true,
    },
    // WHY: reditus (4th decl masculine) - "return, going back" (ReturnType<T>)
    {
        stem: 'redit',
        declension: 4,
        gender: 'masculine',
        meaning: 'return type',
        jsType: 'ReturnType',
        category: 'structural',
        generic: true,
    },
    // WHY: parametra (1st decl feminine plural) - "parameters" (Parameters<T>)
    {
        stem: 'parametr',
        declension: 1,
        gender: 'feminine',
        meaning: 'parameters',
        jsType: 'Parameters',
        category: 'structural',
        generic: true,
    },
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
 * CASE: Modifiers are lowercase (canonical form). Lookup is case-insensitive.
 *
 * USAGE:
 *   - numerus<32, naturalis>  (unsigned 32-bit integer)
 *   - textus<proprius>        (owned string)
 *   - textus<alienus>         (borrowed string reference)
 *   - indicium<T, mutabilis>  (mutable pointer)
 */
export const typeModifiers = {
    // WHY: naturalis (3rd decl adj) - "natural" (unsigned numbers in systems programming)
    naturalis: 'unsigned',

    // WHY: proprius (2nd decl adj) - "one's own" (owned/move semantics)
    proprius: 'owned',

    // WHY: alienus (2nd decl adj) - "belonging to another" (borrowed/borrow semantics)
    alienus: 'borrowed',

    // WHY: mutabilis (3rd decl adj) - "changeable, mutable"
    mutabilis: 'mutable',
} as const;

/**
 * Type for valid modifier names.
 */
export type TypeModifier = keyof typeof typeModifiers;

/**
 * Check if a name is a valid type modifier.
 *
 * WHY: Case-insensitive matching follows Latin convention.
 *
 * @param name - The name to check
 * @returns true if name is a valid type modifier
 */
export function isTypeModifier(name: string): name is TypeModifier {
    return name.toLowerCase() in typeModifiers;
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
