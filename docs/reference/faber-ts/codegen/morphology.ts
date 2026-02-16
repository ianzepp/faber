/**
 * Morphology - Latin Verb Conjugation Parser
 *
 * Parses Latin verb endings to derive semantic flags for stdlib methods.
 * The morphology determines: mutation, async, allocation.
 *
 * WHY: Latin grammar as semantic machinery. The verb conjugation encodes
 *      behavior that would otherwise require separate method definitions.
 *
 * Ported from: fons/rivus/parser/morphologia.fab
 */

// =============================================================================
// TYPES
// =============================================================================

/** Semantic flags derived from verb morphology */
export interface MorphologyFlags {
    /** Modifies receiver in place */
    mutare: boolean;
    /** Returns Promise */
    async: boolean;
    /** Returns new collection (allocates) */
    reddeNovum: boolean;
    /** Needs allocator (Zig target) */
    allocatio: boolean;
}

/** Result of parsing a method name */
export interface ParsedMethod {
    /** Verb stem (e.g., "filtr", "add", "ordin") */
    stem: string;
    /** Canonical form name */
    form: MorphologyForm;
    /** Semantic flags */
    flags: MorphologyFlags;
}

/** Canonical morphology form names (matches @ radix annotations) */
export type MorphologyForm =
    | 'imperativus'           // -a/-e/-i: mutates, sync
    | 'perfectum'             // -ata/-ita/-ta/-sa: returns new, sync
    | 'futurum_indicativum'   // -abit/-ebit/-iet: mutates, async
    | 'futurum_activum'       // -atura/-itura: returns new, async
    | 'participium_praesens'; // -ans/-ens: generator/streaming

// =============================================================================
// FLAG CONSTANTS
// =============================================================================

/** Imperative: adde, filtra, ordina - mutates in place, sync */
const FLAGS_IMPERATIVUS: MorphologyFlags = {
    mutare: true,
    async: false,
    reddeNovum: false,
    allocatio: false,
};

/** Perfect passive participle: addita, filtrata, ordinata - returns new, sync */
const FLAGS_PERFECTUM: MorphologyFlags = {
    mutare: false,
    async: false,
    reddeNovum: true,
    allocatio: true,
};

/** Future active participle: additura, filtratura - returns new, async */
const FLAGS_FUTURUM_ACTIVUM: MorphologyFlags = {
    mutare: false,
    async: true,
    reddeNovum: true,
    allocatio: true,
};

/** Future indicative: addet, filtrabit - mutates, async */
const FLAGS_FUTURUM_INDICATIVUM: MorphologyFlags = {
    mutare: true,
    async: true,
    reddeNovum: false,
    allocatio: false,
};

/** Present participle: filtrans, legens - generator/streaming */
const FLAGS_PARTICIPIUM_PRAESENS: MorphologyFlags = {
    mutare: false,
    async: false,
    reddeNovum: false,
    allocatio: false,
};

// =============================================================================
// MORPHOLOGY PARSER
// =============================================================================

/**
 * Parse a method name to extract verb stem and semantic flags.
 *
 * Checks endings longest-first to avoid partial matches:
 *   -atura/-itura (5 chars) - future active participle
 *   -abit/-ebit (4 chars) - future indicative
 *   -ans/-ens (3 chars) - present participle (generator)
 *   -ata/-ita/-iet (3 chars) - perfect passive participle / future indicative
 *   -ta/-sa (2 chars) - some participles
 *   -a/-e/-i (1 char) - imperative
 *
 * @returns ParsedMethod if recognized morphology, undefined otherwise
 */
export function parseMethodum(name: string): ParsedMethod | undefined {
    const len = name.length;

    // Need at least 2 chars (1 stem + 1 ending)
    if (len < 2) {
        return undefined;
    }

    // Future active participle: -atura, -itura (async, returns new)
    if (len > 5) {
        const suffix5 = name.slice(-5);
        if (suffix5 === 'atura' || suffix5 === 'itura') {
            return {
                stem: name.slice(0, -5),
                form: 'futurum_activum',
                flags: FLAGS_FUTURUM_ACTIVUM,
            };
        }
    }

    // Future indicative: -abit, -ebit (async, mutates)
    if (len > 4) {
        const suffix4 = name.slice(-4);
        if (suffix4 === 'abit' || suffix4 === 'ebit') {
            return {
                stem: name.slice(0, -4),
                form: 'futurum_indicativum',
                flags: FLAGS_FUTURUM_INDICATIVUM,
            };
        }
    }

    // 3-char suffixes
    if (len > 3) {
        const suffix3 = name.slice(-3);

        // Present participle: -ans, -ens (generator/streaming)
        if (suffix3 === 'ans' || suffix3 === 'ens') {
            return {
                stem: name.slice(0, -3),
                form: 'participium_praesens',
                flags: FLAGS_PARTICIPIUM_PRAESENS,
            };
        }

        // Future indicative: -iet (3rd conjugation)
        if (suffix3 === 'iet') {
            return {
                stem: name.slice(0, -3),
                form: 'futurum_indicativum',
                flags: FLAGS_FUTURUM_INDICATIVUM,
            };
        }

        // Perfect passive participle: -ata, -ita (sync, returns new)
        if (suffix3 === 'ata' || suffix3 === 'ita') {
            return {
                stem: name.slice(0, -3),
                form: 'perfectum',
                flags: FLAGS_PERFECTUM,
            };
        }
    }

    // 2-char endings: -ta, -sa (some participles)
    // WHY: 3rd conjugation verbs like invertere -> inversus have -sa participle
    if (len > 2) {
        const suffix2 = name.slice(-2);
        if (suffix2 === 'ta' || suffix2 === 'sa') {
            return {
                stem: name.slice(0, -2),
                form: 'perfectum',
                flags: FLAGS_PERFECTUM,
            };
        }
    }

    // Imperative: -a, -e, -i (sync, mutates)
    const last = name.slice(-1);
    if (last === 'a' || last === 'e' || last === 'i') {
        return {
            stem: name.slice(0, -1),
            form: 'imperativus',
            flags: FLAGS_IMPERATIVUS,
        };
    }

    // No recognized morphology
    return undefined;
}

/**
 * Parse a method name with a known stem hint.
 *
 * WHY: The standard parser is greedy with suffixes (e.g., 'decapita' parses as
 *      'decap' + '-ita' instead of 'decapit' + '-a'). When we know the expected
 *      stem from @ radix, we can extract the correct form.
 *
 * @param name Method name to parse
 * @param stem Known verb stem
 * @returns ParsedMethod if stem matches and form is valid, undefined otherwise
 */
export function parseMethodumWithStem(name: string, stem: string): ParsedMethod | undefined {
    if (!name.startsWith(stem)) {
        return undefined;
    }

    const suffix = name.slice(stem.length);

    // Match suffix to form (ordered by length for clarity)
    switch (suffix) {
        case 'atura':
        case 'itura':
            return { stem, form: 'futurum_activum', flags: FLAGS_FUTURUM_ACTIVUM };
        case 'abit':
        case 'ebit':
            return { stem, form: 'futurum_indicativum', flags: FLAGS_FUTURUM_INDICATIVUM };
        case 'ans':
        case 'ens':
            return { stem, form: 'participium_praesens', flags: FLAGS_PARTICIPIUM_PRAESENS };
        case 'iet':
            return { stem, form: 'futurum_indicativum', flags: FLAGS_FUTURUM_INDICATIVUM };
        case 'ata':
        case 'ita':
            return { stem, form: 'perfectum', flags: FLAGS_PERFECTUM };
        case 'ta':
        case 'sa':
            return { stem, form: 'perfectum', flags: FLAGS_PERFECTUM };
        case 'a':
        case 'e':
        case 'i':
            return { stem, form: 'imperativus', flags: FLAGS_IMPERATIVUS };
        default:
            return undefined;
    }
}

/**
 * Get the canonical form name from morphology flags.
 */
export function formFromFlags(flags: MorphologyFlags): MorphologyForm | 'ignotum' {
    if (flags.mutare && !flags.async && !flags.reddeNovum) {
        return 'imperativus';
    }
    if (!flags.mutare && !flags.async && flags.reddeNovum) {
        return 'perfectum';
    }
    if (flags.mutare && flags.async && !flags.reddeNovum) {
        return 'futurum_indicativum';
    }
    if (!flags.mutare && flags.async && flags.reddeNovum) {
        return 'futurum_activum';
    }
    return 'ignotum';
}
