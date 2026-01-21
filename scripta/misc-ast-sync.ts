#!/usr/bin/env bun
/**
 * Check AST Node Sync
 *
 * Verifies that prettier printer and tree-sitter grammar handle all AST node types.
 * Reports any missing cases.
 */

import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

const ROOT = join(import.meta.dir, '..');

// =============================================================================
// Extract AST node types from ast.ts
// =============================================================================

function extractAstTypes(): Set<string> {
    const astPath = join(ROOT, 'fons', 'faber-ts', 'parser', 'ast.ts');
    const content = readFileSync(astPath, 'utf-8');

    const types = new Set<string>();

    // Match: type: 'NodeName'
    const typeMatches = content.matchAll(/type:\s*['"](\w+)['"]/g);
    for (const match of typeMatches) {
        types.add(match[1]!);
    }

    // Also match interface names that extend BaseNode
    const interfaceMatches = content.matchAll(/interface\s+(\w+)\s+extends\s+BaseNode/g);
    for (const match of interfaceMatches) {
        types.add(match[1]!);
    }

    return types;
}

// =============================================================================
// Check prettier printer coverage
// =============================================================================

function checkPrettierCoverage(astTypes: Set<string>): string[] {
    const printerPath = join(ROOT, 'fons', 'faber-ts', 'prettier', 'printer.ts');

    if (!existsSync(printerPath)) {
        return ['prettier/printer.ts not found'];
    }

    const content = readFileSync(printerPath, 'utf-8');
    const missing: string[] = [];

    for (const nodeType of astTypes) {
        // Check for case 'NodeType': or case "NodeType":
        const pattern = new RegExp(`case\\s+['"]${nodeType}['"]`);
        if (!pattern.test(content)) {
            missing.push(nodeType);
        }
    }

    return missing;
}

// =============================================================================
// Check tree-sitter grammar coverage
// =============================================================================

function checkTreeSitterCoverage(astTypes: Set<string>): string[] {
    const grammarPath = join(ROOT, 'editors/zed/grammars/faber_romanus/grammar.js');

    if (!existsSync(grammarPath)) {
        return ['tree-sitter grammar.js not found'];
    }

    const content = readFileSync(grammarPath, 'utf-8');
    const missing: string[] = [];

    // Map AST types to likely tree-sitter rule names (snake_case)
    const toSnakeCase = (s: string) => s.replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase();

    for (const nodeType of astTypes) {
        const snakeName = toSnakeCase(nodeType);
        // Check for rule definition: rule_name: $ => or rule_name: ($) =>
        const pattern = new RegExp(`${snakeName}:\\s*\\(?\\$\\)?\\s*=>`);
        if (!pattern.test(content)) {
            missing.push(`${nodeType} (expected: ${snakeName})`);
        }
    }

    return missing;
}

// =============================================================================
// Main
// =============================================================================

console.log('Extracting AST node types from fons/faber-ts/parser/ast.ts...\n');

const astTypes = extractAstTypes();
console.log(`Found ${astTypes.size} node types:\n`);

const sortedTypes = [...astTypes].sort();
for (const t of sortedTypes) {
    console.log(`  ${t}`);
}

console.log('\n' + '='.repeat(60) + '\n');

// Check prettier
console.log('Checking fons/faber-ts/prettier/printer.ts...\n');
const prettierMissing = checkPrettierCoverage(astTypes);

if (prettierMissing.length === 0) {
    console.log('  ✓ All node types covered\n');
} else {
    console.log(`  ✗ Missing ${prettierMissing.length} node types:\n`);
    for (const m of prettierMissing) {
        console.log(`    - ${m}`);
    }
    console.log();
}

// Check tree-sitter
console.log('Checking editors/zed/grammars/faber_romanus/grammar.js...\n');
const treeSitterMissing = checkTreeSitterCoverage(astTypes);

if (treeSitterMissing.length === 0) {
    console.log('  ✓ All node types covered\n');
} else {
    console.log(`  ✗ Missing ${treeSitterMissing.length} node types:\n`);
    for (const m of treeSitterMissing) {
        console.log(`    - ${m}`);
    }
    console.log();
}

// Exit code
const totalMissing = prettierMissing.length + treeSitterMissing.length;
if (totalMissing > 0) {
    process.exit(1);
}
