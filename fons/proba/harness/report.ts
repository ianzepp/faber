#!/usr/bin/env bun
/**
 * Feature matrix report generator.
 *
 * Reads test results from SQLite and generates a feature support report.
 *
 * USAGE
 *   bun run fons/proba/harness/report.ts                    Latest run, table format
 *   bun run fons/proba/harness/report.ts --run 5            Specific run ID
 *   bun run fons/proba/harness/report.ts --format markdown  Markdown table
 *   bun run fons/proba/harness/report.ts --format csv       CSV output
 *   bun run fons/proba/harness/report.ts --format json      JSON output
 *   bun run fons/proba/harness/report.ts --failures         Show only failures
 */

import { parseArgs } from 'util';
import { resolve } from 'path';
import { existsSync } from 'fs';

import { openDatabase, getLatestTestRun, getFeatureSummary, getResultsForRun, type FeatureSummary } from './schema';

type OutputFormat = 'table' | 'markdown' | 'csv' | 'json';

interface ReportOptions {
    dbPath: string;
    runId?: number;
    format: OutputFormat;
    showFailuresOnly: boolean;
}

function parseOptions(): ReportOptions {
    const { values } = parseArgs({
        args: Bun.argv.slice(2),
        options: {
            db: { type: 'string', default: 'opus/proba/results.db' },
            run: { type: 'string', default: '' },
            format: { type: 'string', default: 'table' },
            failures: { type: 'boolean', default: false },
        },
    });

    return {
        dbPath: resolve(values.db!),
        runId: values.run ? parseInt(values.run, 10) : undefined,
        format: values.format as OutputFormat,
        showFailuresOnly: values.failures!,
    };
}

function formatCell(passed: number, total: number): string {
    if (total === 0) return '-';
    if (passed === total) return '✓';
    if (passed === 0) return '✗';
    return `${passed}/${total}`;
}

function formatCellMarkdown(passed: number, total: number): string {
    if (total === 0) return '-';
    if (passed === total) return '✅';
    if (passed === 0) return '❌';
    return `${passed}/${total}`;
}

function printTable(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s => s.passed < s.total)
        : summaries;

    if (data.length === 0) {
        console.log('No data to display.');
        return;
    }

    // Calculate column widths
    const featureWidth = Math.max(7, ...data.map(s => s.feature.length));
    const colWidth = 8;

    // Header
    const header = [
        'Feature'.padEnd(featureWidth),
        'Result'.padStart(colWidth),
    ].join(' | ');

    const separator = [
        '-'.repeat(featureWidth),
        '-'.repeat(colWidth),
    ].join('-|-');

    console.log(header);
    console.log(separator);

    for (const s of data) {
        const row = [
            s.feature.padEnd(featureWidth),
            formatCell(s.passed, s.total).padStart(colWidth),
        ].join(' | ');
        console.log(row);
    }

    // Summary row
    console.log(separator);
    const totalPassed = data.reduce((sum, s) => sum + s.passed, 0);
    const totalTests = data.reduce((sum, s) => sum + s.total, 0);

    const totalRow = [
        'TOTAL'.padEnd(featureWidth),
        formatCell(totalPassed, totalTests).padStart(colWidth),
    ].join(' | ');
    console.log(totalRow);
}

function printMarkdown(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s => s.passed < s.total)
        : summaries;

    if (data.length === 0) {
        console.log('No data to display.');
        return;
    }

    console.log('| Feature | Result |');
    console.log('|---------|--------|');

    for (const s of data) {
        console.log(`| ${s.feature} | ${formatCellMarkdown(s.passed, s.total)} |`);
    }
}

function printCsv(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s => s.passed < s.total)
        : summaries;

    console.log('feature,passed,total');

    for (const s of data) {
        console.log(`${s.feature},${s.passed},${s.total}`);
    }
}

function printJson(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s => s.passed < s.total)
        : summaries;

    console.log(JSON.stringify(data, null, 2));
}

async function main() {
    const options = parseOptions();

    if (!existsSync(options.dbPath)) {
        console.error(`Database not found: ${options.dbPath}`);
        console.error('Run the harness first: bun run fons/proba/harness/runner.ts');
        process.exit(1);
    }

    const db = openDatabase(options.dbPath);

    // Get run ID
    let runId = options.runId;
    if (!runId) {
        const latest = getLatestTestRun(db);
        if (!latest) {
            console.error('No test runs found in database.');
            process.exit(1);
        }
        runId = latest.id;
        console.log(`Using latest run #${runId} (${latest.timestamp})\n`);
    }

    // Get feature summary
    const summaries = getFeatureSummary(db, runId);

    if (summaries.length === 0) {
        console.log('No results found for this run.');
        process.exit(0);
    }

    // Output in requested format
    switch (options.format) {
        case 'table':
            printTable(summaries, options.showFailuresOnly);
            break;
        case 'markdown':
            printMarkdown(summaries, options.showFailuresOnly);
            break;
        case 'csv':
            printCsv(summaries, options.showFailuresOnly);
            break;
        case 'json':
            printJson(summaries, options.showFailuresOnly);
            break;
    }

    // Show failed tests if format is table
    if (options.format === 'table') {
        const failures = getResultsForRun(db, runId).filter(r => !r.passed);
        if (failures.length > 0) {
            console.log('\n\nFailed Tests:\n');
            for (const f of failures) {
                console.log(`  ${f.feature} - ${f.test_name}`);
                if (f.error_msg) {
                    console.log(`    Error: ${f.error_msg}`);
                }
            }
        }
    }

    db.close();
}

main();
