/**
 * Semantic Analyzer Tests
 *
 * Tests for type resolution, symbol table management, and error detection.
 */

import { describe, expect, it } from 'bun:test';
import { tokenize } from '../tokenizer';
import { parse } from '../parser';
import { analyze, type SemanticError } from './index';
import type { SemanticType } from './types';

/**
 * Helper to parse and analyze source code.
 */
function analyzeSource(source: string): {
    errors: SemanticError[];
    types: Map<string, SemanticType>;
} {
    const { tokens } = tokenize(source);
    const { program } = parse(tokens);

    if (!program) {
        throw new Error('Parse failed');
    }

    const result = analyze(program);

    // Collect resolved types from variable declarations for inspection
    const types = new Map<string, SemanticType>();

    for (const stmt of program.body) {
        if (stmt.type === 'VariableDeclaration' && stmt.resolvedType) {
            types.set(stmt.name.name, stmt.resolvedType);
        }
    }

    return { errors: result.errors, types };
}

describe('Semantic Analyzer', () => {
    describe('Variable Type Resolution', () => {
        it('resolves type from explicit annotation', () => {
            const { errors, types } = analyzeSource(`esto Numerus x = 5`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'Numerus' });
        });

        it('resolves type from string annotation', () => {
            const { errors, types } = analyzeSource(`esto Textus s = "hello"`);

            expect(errors).toHaveLength(0);
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'Textus' });
        });

        it('resolves type from boolean annotation', () => {
            const { errors, types } = analyzeSource(`esto Bivalens b = verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });

        it('infers type from number literal', () => {
            const { errors, types } = analyzeSource(`esto x = 42`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'Numerus' });
        });

        it('infers type from string literal', () => {
            const { errors, types } = analyzeSource(`esto s = "hello"`);

            expect(errors).toHaveLength(0);
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'Textus' });
        });

        it('infers type from boolean literal', () => {
            const { errors, types } = analyzeSource(`esto b = verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });

        it('handles nullable types', () => {
            const { errors, types } = analyzeSource(`esto Numerus? x = nihil`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'Numerus', nullable: true });
        });
    });

    describe('Expression Type Resolution', () => {
        it('resolves arithmetic expression to Numerus', () => {
            const { errors } = analyzeSource(`esto x = 1 + 2`);

            expect(errors).toHaveLength(0);
        });

        it('resolves string concatenation to Textus', () => {
            const { errors, types } = analyzeSource(`esto s = "a" + "b"`);

            expect(errors).toHaveLength(0);
            // The + operator on strings resolves to Textus
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'Textus' });
        });

        it('resolves comparison to Bivalens', () => {
            const { errors, types } = analyzeSource(`esto b = 1 < 2`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });

        it('resolves equality to Bivalens', () => {
            const { errors, types } = analyzeSource(`esto b = 1 == 2`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });

        it('resolves logical operators to Bivalens', () => {
            const { errors, types } = analyzeSource(`esto b = verum && falsum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });

        it('resolves negation to Bivalens', () => {
            const { errors, types } = analyzeSource(`esto b = !verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'Bivalens' });
        });
    });

    describe('Function Type Resolution', () => {
        it('records function in symbol table', () => {
            const source = `
        functio add(Numerus a, Numerus b) -> Numerus {
          redde a + b
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('type checks return statements', () => {
            const source = `
        functio greet() -> Textus {
          redde "hello"
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('reports error for wrong return type', () => {
            const source = `
        functio greet() -> Textus {
          redde 42
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('not assignable');
        });
    });

    describe('Scope Management', () => {
        it('allows variable shadowing in nested scope', () => {
            const source = `
        esto x = 1
        si verum {
          esto x = "hello"
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('reports error for duplicate in same scope', () => {
            const source = `
        esto x = 1
        esto x = 2
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('already defined');
        });
    });

    describe('Error Detection', () => {
        it('reports undefined variable', () => {
            const { errors } = analyzeSource(`esto x = y`);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('Undefined variable');
        });

        it('reports assignment to immutable variable', () => {
            const source = `
        fixum x = 5
        x = 10
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('immutable');
        });

        it('reports type mismatch in assignment', () => {
            const source = `
        esto Numerus x = 5
        x = "hello"
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('not assignable');
        });

        it('reports variable without type or initializer', () => {
            const { errors } = analyzeSource(`esto x`);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0].message).toContain('no type annotation or initializer');
        });
    });

    describe('Control Flow', () => {
        it('analyzes if statement condition', () => {
            const source = `
        esto x = 5
        esto y = 0
        si x > 0 {
          y = 1
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('analyzes while loop', () => {
            const source = `
        esto i = 0
        dum i < 10 {
          i = i + 1
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('analyzes for loop with loop variable', () => {
            const source = `
        esto count = 0
        ex "test" pro item {
          count = count + 1
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('Generic Types', () => {
        it('resolves generic type annotation', () => {
            const { errors, types } = analyzeSource(`esto Lista<Numerus>? items = nihil`);

            expect(errors).toHaveLength(0);
            const itemsType = types.get('items');

            expect(itemsType?.kind).toBe('generic');
        });
    });

    describe('Arrow Functions', () => {
        it('resolves arrow function type', () => {
            const source = `esto add = (Numerus a, Numerus b) => a + b`;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });
});
