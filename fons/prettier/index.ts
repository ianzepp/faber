/**
 * Prettier Plugin Entry Point
 *
 * ARCHITECTURE
 * ============
 * This module exports the Prettier plugin interface for formatting .fab files.
 * It registers the Faber Romanus language, parser, and printer with Prettier.
 *
 * @module prettier
 */

import type { Plugin, SupportLanguage, Parser, Printer, SupportOption } from 'prettier';
import { faberParse, locStart, locEnd } from './parser.ts';
import { faberPrint } from './printer.ts';
import { printComment, canAttachComment, isBlockComment } from './comments.ts';

// =============================================================================
// LANGUAGE DEFINITION
// =============================================================================

const languages: SupportLanguage[] = [
    {
        name: 'Faber Romanus',
        parsers: ['faber'],
        extensions: ['.fab'],
        vscodeLanguageIds: ['faber-romanus'],
    },
];

// =============================================================================
// PARSER DEFINITION
// =============================================================================

const parsers: Record<string, Parser> = {
    faber: {
        parse: faberParse,
        astFormat: 'faber-ast',
        locStart,
        locEnd,
    },
};

// =============================================================================
// PRINTER DEFINITION
// =============================================================================

const printers: Record<string, Printer> = {
    'faber-ast': {
        print: faberPrint,
        printComment,
        canAttachComment,
        isBlockComment,
    },
};

// =============================================================================
// CUSTOM OPTIONS
// =============================================================================

const options: Record<string, SupportOption> = {
    faberBreakThreshold: {
        category: 'Faber',
        type: 'int',
        default: 3,
        description: 'Break parameter/element lists to one-per-line when count >= this value',
    },
};

// =============================================================================
// PLUGIN EXPORT
// =============================================================================

const plugin: Plugin = {
    languages,
    parsers,
    printers,
    options,
};

export default plugin;
export { languages, parsers, printers, options };
