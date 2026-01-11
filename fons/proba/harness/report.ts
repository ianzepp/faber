#!/usr/bin/env bun
/**
 * Feature matrix report generator.
 *
 * Reads test results from SQLite and generates a feature support matrix.
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

import { openDatabase, getLatestTestRun, getFeatureSummary, getResultsForRun, type FeatureSummary, type TestResult } from './schema';

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
    // Filter if needed
    const data = showFailuresOnly
        ? summaries.filter(s =>
            s.ts_passed < s.ts_total ||
            s.py_passed < s.py_total ||
            s.cpp_passed < s.cpp_total ||
            s.rs_passed < s.rs_total ||
            s.zig_passed < s.zig_total)
        : summaries;

    if (data.length === 0) {
        console.log('No data to display.');
        return;
    }

    // Calculate column widths
    const featureWidth = Math.max(7, ...data.map(s => s.feature.length));
    const colWidth = 6;

    // Header
    const header = [
        'Feature'.padEnd(featureWidth),
        'ts'.padStart(colWidth),
        'py'.padStart(colWidth),
        'cpp'.padStart(colWidth),
        'rs'.padStart(colWidth),
        'zig'.padStart(colWidth),
    ].join(' | ');

    const separator = [
        '-'.repeat(featureWidth),
        '-'.repeat(colWidth),
        '-'.repeat(colWidth),
        '-'.repeat(colWidth),
        '-'.repeat(colWidth),
        '-'.repeat(colWidth),
    ].join('-|-');

    console.log(header);
    console.log(separator);

    for (const s of data) {
        const row = [
            s.feature.padEnd(featureWidth),
            formatCell(s.ts_passed, s.ts_total).padStart(colWidth),
            formatCell(s.py_passed, s.py_total).padStart(colWidth),
            formatCell(s.cpp_passed, s.cpp_total).padStart(colWidth),
            formatCell(s.rs_passed, s.rs_total).padStart(colWidth),
            formatCell(s.zig_passed, s.zig_total).padStart(colWidth),
        ].join(' | ');
        console.log(row);
    }

    // Summary row
    console.log(separator);
    const totals = {
        ts_passed: data.reduce((sum, s) => sum + s.ts_passed, 0),
        ts_total: data.reduce((sum, s) => sum + s.ts_total, 0),
        py_passed: data.reduce((sum, s) => sum + s.py_passed, 0),
        py_total: data.reduce((sum, s) => sum + s.py_total, 0),
        cpp_passed: data.reduce((sum, s) => sum + s.cpp_passed, 0),
        cpp_total: data.reduce((sum, s) => sum + s.cpp_total, 0),
        rs_passed: data.reduce((sum, s) => sum + s.rs_passed, 0),
        rs_total: data.reduce((sum, s) => sum + s.rs_total, 0),
        zig_passed: data.reduce((sum, s) => sum + s.zig_passed, 0),
        zig_total: data.reduce((sum, s) => sum + s.zig_total, 0),
    };

    const totalRow = [
        'TOTAL'.padEnd(featureWidth),
        formatCell(totals.ts_passed, totals.ts_total).padStart(colWidth),
        formatCell(totals.py_passed, totals.py_total).padStart(colWidth),
        formatCell(totals.cpp_passed, totals.cpp_total).padStart(colWidth),
        formatCell(totals.rs_passed, totals.rs_total).padStart(colWidth),
        formatCell(totals.zig_passed, totals.zig_total).padStart(colWidth),
    ].join(' | ');
    console.log(totalRow);
}

function printMarkdown(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s =>
            s.ts_passed < s.ts_total ||
            s.py_passed < s.py_total ||
            s.cpp_passed < s.cpp_total ||
            s.rs_passed < s.rs_total ||
            s.zig_passed < s.zig_total)
        : summaries;

    if (data.length === 0) {
        console.log('No data to display.');
        return;
    }

    console.log('| Feature | ts | py | cpp | rs | zig |');
    console.log('|---------|----|----|-----|----|----|');

    for (const s of data) {
        console.log(`| ${s.feature} | ${formatCellMarkdown(s.ts_passed, s.ts_total)} | ${formatCellMarkdown(s.py_passed, s.py_total)} | ${formatCellMarkdown(s.cpp_passed, s.cpp_total)} | ${formatCellMarkdown(s.rs_passed, s.rs_total)} | ${formatCellMarkdown(s.zig_passed, s.zig_total)} |`);
    }
}

function printCsv(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s =>
            s.ts_passed < s.ts_total ||
            s.py_passed < s.py_total ||
            s.cpp_passed < s.cpp_total ||
            s.rs_passed < s.rs_total ||
            s.zig_passed < s.zig_total)
        : summaries;

    console.log('feature,ts_passed,ts_total,py_passed,py_total,cpp_passed,cpp_total,rs_passed,rs_total,zig_passed,zig_total');

    for (const s of data) {
        console.log(`${s.feature},${s.ts_passed},${s.ts_total},${s.py_passed},${s.py_total},${s.cpp_passed},${s.cpp_total},${s.rs_passed},${s.rs_total},${s.zig_passed},${s.zig_total}`);
    }
}

function printJson(summaries: FeatureSummary[], showFailuresOnly: boolean): void {
    const data = showFailuresOnly
        ? summaries.filter(s =>
            s.ts_passed < s.ts_total ||
            s.py_passed < s.py_total ||
            s.cpp_passed < s.cpp_total ||
            s.rs_passed < s.rs_total ||
            s.zig_passed < s.zig_total)
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
                console.log(`  ${f.feature} [${f.target}] - ${f.test_name}`);
                if (f.error_msg) {
                    console.log(`    Error: ${f.error_msg}`);
                }
            }
        }
    }

    db.close();
}

main();
