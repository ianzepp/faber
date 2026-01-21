/**
 * TypeScript Code Generator - ProbaStatement
 *
 * TRANSFORMS (Legacy Mode - inProbaStandalone = false):
 *   proba "parses integers" { adfirma parse("42") est 42 }
 *   -> test("parses integers", () => { ... });
 *
 *   proba omitte "blocked by #42" { ... }
 *   -> test.skip("blocked by #42", () => { ... });
 *
 *   proba futurum "needs feature" { ... }
 *   -> test.todo("needs feature");
 *
 * TRANSFORMS (Standalone Mode - inProbaStandalone = true):
 *   proba "parses integers" { ... }
 *   -> function __proba_suite_parses_integers(): string | null { ... }
 *   + registry entry
 *
 * WHY: Maps to test()/test.skip()/test.todo() for test runners,
 *      or to standalone functions with try/catch for harness.
 */

import type { ProbaStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

/**
 * Sanitize name for use as identifier
 */
function sanitizeName(name: string): string {
    return name
        .replace(/ /g, '_')
        .replace(/-/g, '_')
        .replace(/\./g, '_')
        .replace(/\//g, '_')
        .replace(/'/g, '')
        .replace(/"/g, '');
}

export function genProbaStatement(node: ProbaStatement, g: TsGenerator, semi: boolean): string {
    // Legacy mode: generate test()
    if (!g.inProbaStandalone) {
        if (node.modifier === 'omitte') {
            // Skip: test.skip("reason: name", () => { ... })
            const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
            const body = genBlockStatement(node.body, g);
            return `${g.ind()}test.skip("${reason}${node.name}", () => ${body})${semi ? ';' : ''}`;
        }

        if (node.modifier === 'futurum') {
            // Todo: test.todo("reason: name")
            const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
            return `${g.ind()}test.todo("${reason}${node.name}")${semi ? ';' : ''}`;
        }

        // Regular test
        const body = genBlockStatement(node.body, g);
        return `${g.ind()}test("${node.name}", () => ${body})${semi ? ';' : ''}`;
    }

    // Standalone mode: generate function + registry entry
    const suitePath = g.probaSuiteStack.join('_');
    const sanitizedName = sanitizeName(node.name);
    const funcName = suitePath ? `__proba_${suitePath}_${sanitizedName}` : `__proba_${sanitizedName}`;
    const suiteDisplay = g.probaSuiteStack.join(' > ');

    // Build registry entry
    const entry: typeof g.probaRegistry[0] = {
        suite: suiteDisplay,
        name: node.name,
        funcName,
    };

    if (node.modifier === 'omitte') {
        entry.skip = true;
        if (node.modifierReason) {
            entry.skipReason = node.modifierReason;
        }
    } else if (node.modifier === 'futurum') {
        entry.todo = true;
        if (node.modifierReason) {
            entry.todoReason = node.modifierReason;
        }
    }

    if (node.solum) {
        entry.only = true;
    }
    if (node.tags && node.tags.length > 0) {
        entry.tags = node.tags;
    }
    if (node.temporis !== undefined) {
        entry.timeout = node.temporis;
    }
    if (node.metior) {
        entry.benchmark = true;
    }
    if (node.repete !== undefined) {
        entry.repeat = node.repete;
    }
    if (node.fragilis !== undefined) {
        entry.retries = node.fragilis;
    }
    if (node.requirit) {
        entry.requireEnv = node.requirit;
    }
    if (node.solumIn) {
        entry.platformOnly = node.solumIn;
    }

    g.probaRegistry.push(entry);

    // Generate standalone function
    const lines: string[] = [];
    lines.push(`${g.ind()}function ${funcName}(): string | null {`);
    g.depth++;
    lines.push(`${g.ind()}try {`);
    g.depth++;

    // Generate body
    for (const stmt of node.body.body) {
        lines.push(g.genStatement(stmt));
    }

    lines.push(`${g.ind()}return null;`);
    g.depth--;
    lines.push(`${g.ind()}} catch (e) {`);
    g.depth++;
    lines.push(`${g.ind()}return String(e);`);
    g.depth--;
    lines.push(`${g.ind()}}`);
    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate the test harness (registry + runner).
 * Call this after all statements have been generated.
 */
export function genProbaHarness(g: TsGenerator): string {
    if (g.probaRegistry.length === 0) {
        return '';
    }

    const lines: string[] = [];
    lines.push('');
    lines.push('// =============================================================================');
    lines.push('// TEST HARNESS');
    lines.push('// =============================================================================');
    lines.push('');

    // Generate registry type
    lines.push('interface __TestEntry {');
    lines.push('  suite: string;');
    lines.push('  name: string;');
    lines.push('  fn: () => string | null;');
    lines.push('  skip?: boolean;');
    lines.push('  skipReason?: string;');
    lines.push('  todo?: boolean;');
    lines.push('  todoReason?: string;');
    lines.push('  only?: boolean;');
    lines.push('  tags?: string[];');
    lines.push('  timeout?: number;');
    lines.push('  benchmark?: boolean;');
    lines.push('  repeat?: number;');
    lines.push('  retries?: number;');
    lines.push('  requireEnv?: string;');
    lines.push('  platformOnly?: string;');
    lines.push('}');
    lines.push('');

    // Generate registry
    lines.push('const __tests: __TestEntry[] = [');
    for (const entry of g.probaRegistry) {
        const parts: string[] = [];
        parts.push(`suite: "${entry.suite}"`);
        parts.push(`name: "${entry.name}"`);
        parts.push(`fn: ${entry.funcName}`);

        if (entry.skip) {
            parts.push('skip: true');
            if (entry.skipReason) {
                parts.push(`skipReason: "${entry.skipReason}"`);
            }
        }
        if (entry.todo) {
            parts.push('todo: true');
            if (entry.todoReason) {
                parts.push(`todoReason: "${entry.todoReason}"`);
            }
        }
        if (entry.only) {
            parts.push('only: true');
        }
        if (entry.tags && entry.tags.length > 0) {
            parts.push(`tags: [${entry.tags.map(t => `"${t}"`).join(', ')}]`);
        }
        if (entry.timeout !== undefined) {
            parts.push(`timeout: ${entry.timeout}`);
        }
        if (entry.benchmark) {
            parts.push('benchmark: true');
        }
        if (entry.repeat !== undefined) {
            parts.push(`repeat: ${entry.repeat}`);
        }
        if (entry.retries !== undefined) {
            parts.push(`retries: ${entry.retries}`);
        }
        if (entry.requireEnv) {
            parts.push(`requireEnv: "${entry.requireEnv}"`);
        }
        if (entry.platformOnly) {
            parts.push(`platformOnly: "${entry.platformOnly}"`);
        }

        lines.push(`  { ${parts.join(', ')} },`);
    }
    lines.push('];');
    lines.push('');

    // Generate runner
    lines.push('function __runTests(options?: { tag?: string; exclude?: string; only?: boolean }): number {');
    lines.push('  const opts = options ?? {};');
    lines.push('  let testsToRun = __tests;');
    lines.push('');
    lines.push('  // Filter by \'only\' if any test has it');
    lines.push('  if (opts.only || testsToRun.some(t => t.only)) {');
    lines.push('    testsToRun = testsToRun.filter(t => t.only);');
    lines.push('  }');
    lines.push('');
    lines.push('  // Filter by tag');
    lines.push('  if (opts.tag) {');
    lines.push('    testsToRun = testsToRun.filter(t => t.tags?.includes(opts.tag!));');
    lines.push('  }');
    lines.push('');
    lines.push('  // Exclude by tag');
    lines.push('  if (opts.exclude) {');
    lines.push('    testsToRun = testsToRun.filter(t => !t.tags?.includes(opts.exclude!));');
    lines.push('  }');
    lines.push('');
    lines.push('  let passed = 0, failed = 0, skipped = 0, todo = 0;');
    lines.push('');
    lines.push('  for (const t of testsToRun) {');
    lines.push('    const label = t.suite ? `${t.suite} > ${t.name}` : t.name;');
    lines.push('');
    lines.push('    // Check platform filter');
    lines.push('    if (t.platformOnly && process.platform !== t.platformOnly) {');
    lines.push('      console.log(`SKIP ${label} (platform: ${t.platformOnly})`);');
    lines.push('      skipped++;');
    lines.push('      continue;');
    lines.push('    }');
    lines.push('');
    lines.push('    // Check required env');
    lines.push('    if (t.requireEnv && !process.env[t.requireEnv]) {');
    lines.push('      console.log(`SKIP ${label} (missing env: ${t.requireEnv})`);');
    lines.push('      skipped++;');
    lines.push('      continue;');
    lines.push('    }');
    lines.push('');
    lines.push('    // Skip');
    lines.push('    if (t.skip) {');
    lines.push('      const reason = t.skipReason ? `: ${t.skipReason}` : \'\';');
    lines.push('      console.log(`SKIP ${label}${reason}`);');
    lines.push('      skipped++;');
    lines.push('      continue;');
    lines.push('    }');
    lines.push('');
    lines.push('    // Todo');
    lines.push('    if (t.todo) {');
    lines.push('      const reason = t.todoReason ? `: ${t.todoReason}` : \'\';');
    lines.push('      console.log(`TODO ${label}${reason}`);');
    lines.push('      todo++;');
    lines.push('      continue;');
    lines.push('    }');
    lines.push('');
    lines.push('    // Run with retries');
    lines.push('    const maxAttempts = t.retries ?? 1;');
    lines.push('    const repeatCount = t.repeat ?? 1;');
    lines.push('    let lastErr: string | null = null;');
    lines.push('');
    lines.push('    for (let rep = 0; rep < repeatCount; rep++) {');
    lines.push('      for (let attempt = 0; attempt < maxAttempts; attempt++) {');
    lines.push('        lastErr = t.fn();');
    lines.push('        if (!lastErr) break;');
    lines.push('      }');
    lines.push('      if (lastErr) break;');
    lines.push('    }');
    lines.push('');
    lines.push('    if (lastErr) {');
    lines.push('      console.log(`FAIL ${label}: ${lastErr}`);');
    lines.push('      failed++;');
    lines.push('    } else {');
    lines.push('      console.log(`PASS ${label}`);');
    lines.push('      passed++;');
    lines.push('    }');
    lines.push('  }');
    lines.push('');
    lines.push('  console.log(`\\n${passed} passed, ${failed} failed, ${skipped} skipped, ${todo} todo`);');
    lines.push('  return failed > 0 ? 1 : 0;');
    lines.push('}');
    lines.push('');

    // Generate entry point check
    lines.push('// Run tests if executed directly');
    lines.push('if (typeof require !== \'undefined\' && require.main === module) {');
    lines.push('  process.exit(__runTests());');
    lines.push('}');

    return lines.join('\n');
}
