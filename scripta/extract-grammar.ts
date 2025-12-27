/**
 * Extract GRAMMAR blocks from parser source and generate grammatica/*.md files.
 *
 * Usage: bun run scripta/extract-grammar.ts
 *
 * Reads fons/parser/index.ts, extracts GRAMMAR: blocks with their WHY: explanations,
 * and generates consolidated markdown files in grammatica/.
 */

import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

// ---------------------------------------------------------------------------
// Category Mapping
// ---------------------------------------------------------------------------

// Map function names to grammatica categories
const CATEGORY_MAP: Record<string, string> = {
    // fundamenta - variables, constants, literals
    parseVariableDeclaration: 'fundamenta',
    parseObjectPattern: 'fundamenta',

    // typi - type system
    parseTypeAnnotation: 'typi',
    parseTypeAliasDeclaration: 'typi',
    parseEnumDeclaration: 'typi',

    // operatores - operators
    parseAssignment: 'operatores',
    parseTernary: 'operatores',
    parseOr: 'operatores',
    parseAnd: 'operatores',
    parseEquality: 'operatores',
    parseComparison: 'operatores',
    parseRange: 'operatores',
    parseAdditive: 'operatores',
    parseMultiplicative: 'operatores',
    parseUnary: 'operatores',

    // functiones - function declarations
    parseFunctionDeclaration: 'functiones',
    parseParameterList: 'functiones',
    parseParameter: 'functiones',
    parseArrowFunction: 'functiones',
    parseProExpression: 'functiones',

    // regimen - control flow
    parse: 'regimen', // program structure
    parseProgram: 'regimen',
    parseStatement: 'regimen',
    parseIfStatement: 'regimen',
    parseWhileStatement: 'regimen',
    parseExStatement: 'regimen',
    parseDeStatement: 'regimen',
    parseInStatement: 'regimen',
    parseSwitchStatement: 'regimen',
    parseGuardStatement: 'regimen',
    parseAssertStatement: 'regimen',
    parseReturnStatement: 'regimen',
    parseBreakStatement: 'regimen',
    parseContinueStatement: 'regimen',
    parseBlockStatement: 'regimen',
    parseExpressionStatement: 'regimen',
    parseExpression: 'regimen',

    // errores - error handling
    parseThrowStatement: 'errores',
    parseTryStatement: 'errores',
    parseCatchClause: 'errores',
    parseFacBlockStatement: 'errores',

    // structurae - data structures
    parseGenusDeclaration: 'structurae',
    parseGenusMember: 'structurae',
    parsePactumDeclaration: 'structurae',
    parsePactumMethod: 'structurae',
    parseNewExpression: 'structurae',
    parseCall: 'structurae',
    parseArgumentList: 'structurae',
    parsePrimary: 'structurae',
    parseIdentifier: 'structurae',

    // importa - module system
    parseImportDeclaration: 'importa',

    // Output statements go to fundamenta (basic I/O)
    parseOutputStatement: 'fundamenta',
};

// Category metadata for generated files
const CATEGORY_META: Record<string, { title: string; description: string }> = {
    fundamenta: {
        title: 'Fundamenta',
        description: 'Basic language elements: variables, constants, literals, and output.',
    },
    typi: {
        title: 'Typi',
        description: 'Type system: type annotations, aliases, enums, nullable types, and collections.',
    },
    operatores: {
        title: 'Operatores',
        description: 'Operators: arithmetic, logical, comparison, ternary, nullish coalescing, and ranges.',
    },
    functiones: {
        title: 'Functiones',
        description: 'Function declarations: basic functions, typed parameters, async, generators, and lambdas.',
    },
    regimen: {
        title: 'Regimen',
        description: 'Control flow: conditionals, loops, guards, assertions, and program structure.',
    },
    errores: {
        title: 'Errores',
        description: 'Error handling: try/catch, throw, panic, and scoped error handling.',
    },
    structurae: {
        title: 'Structurae',
        description: 'Data structures: classes (genus), objects, member access, and instantiation.',
    },
    importa: {
        title: 'Importa',
        description: 'Module system: imports and exports.',
    },
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface GrammarBlock {
    functionName: string;
    grammar: string[];
    why: string[];
    examples: string[];
    category: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Remove common leading whitespace from lines.
 */
function dedent(lines: string[]): string[] {
    // Find minimum indent of non-empty lines
    let minIndent = Infinity;
    for (const line of lines) {
        if (line.trim() === '') continue;
        const match = line.match(/^(\s*)/);
        if (match?.[1]) {
            minIndent = Math.min(minIndent, match[1].length);
        }
    }
    if (minIndent === Infinity || minIndent === 0) return lines;

    // Strip the common indent
    return lines.map(line => (line.trim() === '' ? '' : line.slice(minIndent)));
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

function extractGrammarBlocks(source: string): GrammarBlock[] {
    const blocks: GrammarBlock[] = [];

    // Match JSDoc comments followed by function declarations
    const pattern = /\/\*\*[\s\S]*?\*\/\s*function\s+(\w+)/g;
    let match;

    while ((match = pattern.exec(source)) !== null) {
        const comment = match[0];
        const functionName = match[1];

        // Skip if no GRAMMAR: in comment
        if (!comment.includes('GRAMMAR:')) continue;

        const block = parseGrammarBlock(comment, functionName);
        if (block) blocks.push(block);
    }

    return blocks;
}

function parseGrammarBlock(comment: string, functionName: string | undefined): GrammarBlock | null {
    if (!functionName) return null;

    // Extract just the JSDoc comment (everything between /** and */)
    const jsdocMatch = comment.match(/\/\*\*([\s\S]*?)\*\//);
    if (!jsdocMatch?.[1]) return null;

    const jsdocContent = jsdocMatch[1];
    // WHY: Preserve leading whitespace for indented content (lists, code blocks)
    // Only strip the JSDoc line prefix (* ), not trailing content
    const lines: string[] = jsdocContent.split('\n').map(l => {
        // Remove leading whitespace + asterisk + optional single space
        const match = l.match(/^\s*\*( ?)(.*)/);
        if (match?.[2] !== undefined) {
            // match[2] is the content after the asterisk
            return match[2];
        }
        return l.trim();
    });

    const grammar: string[] = [];
    const why: string[] = [];
    const examples: string[] = [];

    let section: 'none' | 'grammar' | 'why' | 'examples' = 'none';

    for (const line of lines) {
        const trimmed = line.trim();

        // End of comment or start of new section terminates current section
        if (trimmed.startsWith('GRAMMAR:')) {
            section = 'grammar';
            continue;
        }
        if (trimmed.startsWith('WHY:')) {
            section = 'why';
            why.push(trimmed.substring(4).trim());
            continue;
        }
        if (trimmed.startsWith('Examples:') || trimmed.startsWith('Example:')) {
            section = 'examples';
            continue;
        }
        // These tags end the current section
        if (
            trimmed.startsWith('EDGE:') ||
            trimmed.startsWith('ERROR') ||
            trimmed.startsWith('PRECEDENCE:') ||
            trimmed.startsWith('INVARIANT:') ||
            trimmed.startsWith('@') ||
            trimmed.startsWith('/')
        ) {
            section = 'none';
            continue;
        }
        // Blank line ends grammar section (but not why/examples which can be multi-paragraph)
        if (trimmed === '') {
            if (section === 'grammar' && grammar.length > 0) section = 'none';
            // Preserve blank lines in why/examples sections
            if (section === 'why') why.push('');
            if (section === 'examples') examples.push('');
            continue;
        }

        switch (section) {
            case 'grammar':
                grammar.push(trimmed);
                break;
            case 'why':
                // WHY: Trim WHY content since it's rendered as blockquotes
                // JSDoc continuation indentation looks odd in markdown
                why.push(trimmed);
                break;
            case 'examples':
                // Stop at function declarations leaking in
                if (trimmed.startsWith('function ')) break;
                // WHY: Preserve indentation for code examples
                examples.push(line);
                break;
        }
    }

    if (grammar.length === 0) return null;

    const category = CATEGORY_MAP[functionName] || 'regimen';

    // WHY: Trim trailing blank lines from why/examples arrays
    while (why.length > 0 && why[why.length - 1] === '') why.pop();
    while (examples.length > 0 && examples[examples.length - 1] === '') examples.pop();

    // WHY: Dedent examples by removing common leading whitespace
    const dedentedExamples = dedent(examples);

    return { functionName, grammar, why, examples: dedentedExamples, category };
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

function groupByCategory(blocks: GrammarBlock[]): Map<string, GrammarBlock[]> {
    const groups = new Map<string, GrammarBlock[]>();

    for (const block of blocks) {
        const existing = groups.get(block.category) || [];
        existing.push(block);
        groups.set(block.category, existing);
    }

    return groups;
}

function generateMarkdown(category: string, blocks: GrammarBlock[]): string {
    const meta = CATEGORY_META[category] || { title: category, description: '' };
    const lines: string[] = [];

    lines.push(`# ${meta.title}`);
    lines.push('');
    lines.push(meta.description);
    lines.push('');
    lines.push('## Exempla');
    lines.push('');
    lines.push(`- \`exempla/${category}/\``);
    lines.push('');
    lines.push('---');
    lines.push('');
    lines.push('## Syntax');
    lines.push('');

    for (const block of blocks) {
        // Section header from function name
        const title = functionNameToTitle(block.functionName);
        lines.push(`### ${title}`);
        lines.push('');

        // EBNF grammar
        lines.push('```ebnf');
        for (const g of block.grammar) {
            lines.push(g);
        }
        lines.push('```');
        lines.push('');

        // WHY explanation
        if (block.why.length > 0) {
            for (const w of block.why) {
                lines.push(`> ${w}`);
            }
            lines.push('');
        }

        // Examples
        if (block.examples.length > 0) {
            lines.push('**Examples:**');
            lines.push('');
            lines.push('```fab');
            for (const e of block.examples) {
                lines.push(e);
            }
            lines.push('```');
            lines.push('');
        }
    }

    lines.push('---');
    lines.push('');
    lines.push('*Generated from `fons/parser/index.ts` — do not edit directly.*');

    return lines.join('\n');
}

function functionNameToTitle(name: string): string {
    // parseVariableDeclaration -> Variable Declaration
    return name
        .replace(/^parse/, '')
        .replace(/([A-Z])/g, ' $1')
        .trim();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const ROOT = join(import.meta.dir, '..');
const PARSER_PATH = join(ROOT, 'fons/parser/index.ts');
const GRAMMATICA_DIR = join(ROOT, 'grammatica');
const PREAMBLE_PATH = join(GRAMMATICA_DIR, 'preamble.md');

console.log('Extracting GRAMMAR blocks from parser...');

const source = readFileSync(PARSER_PATH, 'utf-8');
const blocks = extractGrammarBlocks(source);

console.log(`Found ${blocks.length} GRAMMAR blocks`);

const groups = groupByCategory(blocks);

// Category order for concatenated output
const CATEGORY_ORDER = ['fundamenta', 'typi', 'operatores', 'functiones', 'regimen', 'errores', 'structurae', 'importa'];

for (const [category, categoryBlocks] of groups) {
    const markdown = generateMarkdown(category, categoryBlocks);
    const outPath = join(GRAMMATICA_DIR, `${category}.md`);

    writeFileSync(outPath, markdown);
    console.log(`  ${category}.md (${categoryBlocks.length} rules)`);
}

// Generate concatenated GRAMMAR.md at project root
// Start with preamble (static content for LLM consumption)
let preamble = '';
try {
    preamble = readFileSync(PREAMBLE_PATH, 'utf-8');
    console.log('  Loaded preamble.md');
} catch {
    console.log('  No preamble.md found, using minimal header');
    preamble = '# Faber Romanus Grammar\n\nComplete syntax reference.\n\n';
}

const allSections: string[] = [];

// Add preamble content
allSections.push(preamble.trim());
allSections.push('');
allSections.push('## Table of Contents');
allSections.push('');

for (const category of CATEGORY_ORDER) {
    const meta = CATEGORY_META[category];
    if (meta) {
        const brief = meta.description.split(':')[0]?.toLowerCase() ?? '';
        allSections.push(`- [${meta.title}](#${category}) — ${brief}`);
    }
}

allSections.push('');
allSections.push('---');
allSections.push('');

for (const category of CATEGORY_ORDER) {
    const categoryBlocks = groups.get(category);
    if (!categoryBlocks) continue;

    const meta = CATEGORY_META[category] || { title: category, description: '' };

    // Use h2 for category headers in concatenated file
    allSections.push(`<a id="${category}"></a>`);
    allSections.push('');
    allSections.push(`## ${meta.title}`);
    allSections.push('');
    allSections.push(meta.description);
    allSections.push('');

    for (const block of categoryBlocks) {
        const title = functionNameToTitle(block.functionName);
        allSections.push(`### ${title}`);
        allSections.push('');
        allSections.push('```ebnf');
        for (const g of block.grammar) {
            allSections.push(g);
        }
        allSections.push('```');
        allSections.push('');

        if (block.why.length > 0) {
            for (const w of block.why) {
                allSections.push(`> ${w}`);
            }
            allSections.push('');
        }

        if (block.examples.length > 0) {
            allSections.push('**Examples:**');
            allSections.push('');
            allSections.push('```fab');
            for (const e of block.examples) {
                allSections.push(e);
            }
            allSections.push('```');
            allSections.push('');
        }
    }

    allSections.push('---');
    allSections.push('');
}

allSections.push('*Generated from `fons/parser/index.ts` — do not edit directly.*');

const grammarPath = join(ROOT, 'GRAMMAR.md');
writeFileSync(grammarPath, allSections.join('\n'));
console.log(`\n  GRAMMAR.md (${blocks.length} rules, concatenated)`);

console.log('\nDone. Opus perfectum est.');
