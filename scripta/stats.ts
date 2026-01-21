#!/usr/bin/env bun

/**
 * Project Statistics Generator
 *
 * Generates codebase statistics for the README.md Project Stats section.
 * Run with: bun run stats
 */

import { $ } from 'bun';

interface Stat {
    component: string;
    lines: number;
    files: number | string;
    description: string;
}

async function countLines(pattern: string): Promise<number> {
    try {
        const result = await $`find fons -name "${pattern}" -type f -exec wc -l {} + | tail -1`.text();
        const match = result.match(/(\d+)\s+total/);
        if (!match?.[1]) return 0;
        return parseInt(match[1], 10);
    } catch {
        return 0;
    }
}

async function countFiles(pattern: string): Promise<number> {
    try {
        const result = await $`find fons -type f -name "${pattern}" | wc -l`.text();
        return parseInt(result.trim(), 10);
    } catch {
        return 0;
    }
}

async function countTestCases(): Promise<number> {
    try {
        // Count passing tests from last test run output
        const result = await $`timeout 5 bun test fons/proba/faber.test.ts 2>&1 | grep 'pass' | head -1`.text();
        const match = result.match(/(\d+)\s+pass/);
        if (!match?.[1]) return 0;
        return parseInt(match[1], 10);
    } catch {
        return 0;
    }
}

async function gatherStats(): Promise<Stat[]> {
    console.error('Gathering statistics...\n');

    // Count TypeScript compiler source (excluding tests and generated files)
    const tsLines = await countLines('*.ts');
    const tsTestLines = await countLines('*.test.ts');
    const tsGenLines = await countLines('*.gen.ts');
    const compilerLines = tsLines - tsTestLines - tsGenLines;
    const compilerFiles = await countFiles('*.ts') - await countFiles('*.test.ts') - await countFiles('*.gen.ts');

    // Count Faber bootstrap compiler (excluding generated files)
    const fabLines = await countLines('*.fab');
    const fabGenLines = await countLines('*.gen.fab');
    const bootstrapLines = fabLines - fabGenLines;
    const bootstrapFiles = await countFiles('*.fab') - await countFiles('*.gen.fab');

    // Count test infrastructure
    const testLines = await countLines('*.test.ts');

    // Count test specifications (YAML)
    const testSpecLines = await countLines('*.yaml');
    const testCases = await countTestCases();

    // Count core compiler phases
    const tokenizerResult = await $`wc -l fons/faber-ts/tokenizer/index.ts`.text();
    const parserResult = await $`wc -l fons/faber-ts/parser/index.ts`.text();
    const semanticResult = await $`wc -l fons/faber-ts/semantic/index.ts`.text();
    const tokenizerLines = parseInt(tokenizerResult.match(/(\d+)/)?.[1] || '0', 10);
    const parserLines = parseInt(parserResult.match(/(\d+)/)?.[1] || '0', 10);
    const semanticLines = parseInt(semanticResult.match(/(\d+)/)?.[1] || '0', 10);
    const coreLines = tokenizerLines + parserLines + semanticLines;

    // Count codegen files and lines
    const codegenFiles = await countFiles('*.ts') - await countFiles('*.gen.ts');
    const codegenResult = await $`find fons/faber-ts/codegen -name "*.ts" ! -name "*.gen.ts" -exec wc -l {} + | tail -1`.text();
    const codegenLines = parseInt(codegenResult.match(/(\d+)/)?.[1] || '0', 10);

    // Count documentation
    const ebnfResult = await $`wc -l EBNF.md`.text();
    const ebnfLines = parseInt(ebnfResult.match(/(\d+)/)?.[1] || '0', 10);
    const grammarResult = await $`find fons/grammatica -name "*.md" -exec wc -l {} + | tail -1`.text();
    const grammarLines = parseInt(grammarResult.match(/(\d+)/)?.[1] || '0', 10);
    const docLines = ebnfLines + grammarLines;

    // Count examples
    const examplesResult = await $`find exempla -name "*.fab" -exec wc -l {} + 2>/dev/null | tail -1`.text();
    const examplesLines = parseInt(examplesResult.match(/(\d+)/)?.[1] || '0', 10);
    const examplesFiles = await $`find exempla -name "*.fab" | wc -l`.text();

    // Count targets
    const targetsResult = await $`find fons/faber-ts/codegen -maxdepth 1 -type d ! -name codegen | wc -l`.text();
    const numTargets = parseInt(targetsResult.trim(), 10);

    const stats: Stat[] = [
        {
            component: '**Compiler (faber)**',
            lines: compilerLines,
            files: compilerFiles,
            description: 'Reference compiler in TypeScript',
        },
        {
            component: '**Bootstrap (rivus)**',
            lines: bootstrapLines,
            files: bootstrapFiles,
            description: 'Self-hosting compiler in Faber',
        },
        {
            component: '**Tests**',
            lines: testLines,
            files: '—',
            description: 'Test infrastructure',
        },
        {
            component: '**Test Specs**',
            lines: testSpecLines,
            files: '—',
            description: `YAML test definitions (${testCases.toLocaleString()} passing)`,
        },
        {
            component: '**Core Phases**',
            lines: coreLines,
            files: 3,
            description: 'Tokenizer, parser, semantic analyzer',
        },
        {
            component: '**Codegen**',
            lines: codegenLines,
            files: codegenFiles,
            description: `Code generators for ${numTargets} targets`,
        },
        {
            component: '**Documentation**',
            lines: docLines,
            files: '—',
            description: 'Grammar spec + prose tutorials',
        },
        {
            component: '**Examples**',
            lines: examplesLines,
            files: parseInt(examplesFiles.trim(), 10),
            description: 'Sample Faber programs',
        },
    ];

    const totalLines = stats.reduce((sum, s) => sum + s.lines, 0);
    const totalFiles = stats
        .map(s => (typeof s.files === 'number' ? s.files : 0))
        .reduce((sum, f) => sum + f, 0);

    stats.push({
        component: '**Total**',
        lines: totalLines,
        files: totalFiles,
        description: 'Complete implementation',
    });

    return stats;
}

function formatTable(stats: Stat[]): string {
    const lines = [
        '## Project Stats',
        '',
        '| Component | Lines | Files | Description |',
        '|-----------|------:|------:|-------------|',
    ];

    for (const stat of stats) {
        const files = typeof stat.files === 'number' ? stat.files.toLocaleString() : stat.files;
        lines.push(
            `| ${stat.component} | ${stat.lines.toLocaleString()} | ${files} | ${stat.description} |`,
        );
    }

    lines.push('');

    // Add targets note
    const codegenStat = stats.find(s => s.component === '**Codegen**');
    if (codegenStat) {
        const numTargets = codegenStat.description.match(/(\d+) targets/)?.[1] || '6';
        lines.push(`**Compilation Targets:** TypeScript, Zig, Python, Rust, C++, Faber (round-trip)`);
    }

    return lines.join('\n');
}

// Main execution
const stats = await gatherStats();
const table = formatTable(stats);

console.log(table);
console.error('\n✓ Statistics generated successfully');
console.error('Copy the table above and replace the "Project Stats" section in README.md');
