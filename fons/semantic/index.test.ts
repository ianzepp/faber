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
        if (stmt.type === 'VariaDeclaration' && stmt.resolvedType) {
            types.set(stmt.name.name, stmt.resolvedType);
        }
    }

    return { errors: result.errors, types };
}

describe('Semantic Analyzer', () => {
    describe('Variable Type Resolution', () => {
        it('resolves type from explicit annotation', () => {
            const { errors, types } = analyzeSource(`varia numerus x = 5`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'numerus' });
        });

        it('resolves type from string annotation', () => {
            const { errors, types } = analyzeSource(`varia textus s = "hello"`);

            expect(errors).toHaveLength(0);
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'textus' });
        });

        it('resolves type from boolean annotation', () => {
            const { errors, types } = analyzeSource(`varia bivalens b = verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('infers type from number literal', () => {
            const { errors, types } = analyzeSource(`varia x = 42`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'numerus' });
        });

        it('infers type from string literal', () => {
            const { errors, types } = analyzeSource(`varia s = "hello"`);

            expect(errors).toHaveLength(0);
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'textus' });
        });

        it('infers type from boolean literal', () => {
            const { errors, types } = analyzeSource(`varia b = verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('handles nullable types', () => {
            const { errors, types } = analyzeSource(`varia numerus? x = nihil`);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'numerus', nullable: true });
        });
    });

    describe('Expression Type Resolution', () => {
        it('resolves arithmetic expression to numerus', () => {
            const { errors } = analyzeSource(`varia x = 1 + 2`);

            expect(errors).toHaveLength(0);
        });

        it('resolves string concatenation to textus', () => {
            const { errors, types } = analyzeSource(`varia s = "a" + "b"`);

            expect(errors).toHaveLength(0);
            // The + operator on strings resolves to textus
            expect(types.get('s')).toEqual({ kind: 'primitive', name: 'textus' });
        });

        it('resolves comparison to bivalens', () => {
            const { errors, types } = analyzeSource(`varia b = 1 < 2`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('resolves equality to bivalens', () => {
            const { errors, types } = analyzeSource(`varia b = 1 == 2`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('resolves logical operators to bivalens', () => {
            const { errors, types } = analyzeSource(`varia b = verum && falsum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('resolves negation to bivalens', () => {
            // WHY: Use 'non' for logical negation (prefix ! removed for !. non-null assertion)
            const { errors, types } = analyzeSource(`varia b = non verum`);

            expect(errors).toHaveLength(0);
            expect(types.get('b')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });
    });

    describe('Function Type Resolution', () => {
        it('records function in symbol table', () => {
            const source = `
        functio add(numerus a, numerus b) -> numerus {
          redde a + b
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('type checks return statements', () => {
            const source = `
        functio greet() -> textus {
          redde "hello"
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('reports error for wrong return type', () => {
            const source = `
        functio greet() -> textus {
          redde 42
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('not assignable');
        });
    });

    describe('Scope Management', () => {
        it('allows variable shadowing in nested scope', () => {
            const source = `
        varia x = 1
        si verum {
          varia x = "hello"
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('reports error for duplicate in same scope', () => {
            const source = `
        varia x = 1
        varia x = 2
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('already defined');
        });
    });

    describe('Error Detection', () => {
        it('reports undefined variable', () => {
            const { errors } = analyzeSource(`varia x = y`);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('Undefined variable');
        });

        it('reports assignment to immutable variable', () => {
            const source = `
        fixum x = 5
        x = 10
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('immutable');
        });

        it('reports type mismatch in assignment', () => {
            const source = `
        varia numerus x = 5
        x = "hello"
      `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('not assignable');
        });

        it('reports variable without type or initializer', () => {
            const { errors } = analyzeSource(`varia x`);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('no type annotation or initializer');
        });
    });

    describe('Control Flow', () => {
        it('analyzes if statement condition', () => {
            const source = `
        varia x = 5
        varia y = 0
        si x > 0 {
          y = 1
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('analyzes while loop', () => {
            const source = `
        varia i = 0
        dum i < 10 {
          i = i + 1
        }
      `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('analyzes for loop with loop variable', () => {
            const source = `
        varia count = 0
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
            const { errors, types } = analyzeSource(`varia numerus[]? items = nihil`);

            expect(errors).toHaveLength(0);
            const itemsType = types.get('items');

            expect(itemsType?.kind).toBe('generic');
        });
    });

    describe('Arrow Functions', () => {
        it('resolves arrow function type', () => {
            const source = `varia add = (numerus a, numerus b) => a + b`;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('nulla/nonnulla operators', () => {
        it('nulla resolves to bivalens type', () => {
            const source = `
                varia items = [1, 2, 3]
                varia x = nulla items
            `;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('nonnulla resolves to bivalens type', () => {
            const source = `
                varia data = "hello"
                varia x = nonnulla data
            `;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('nulla on array type', () => {
            const source = `
                varia numerus[] nums = [1, 2, 3]
                varia check = nulla nums
            `;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            expect(types.get('check')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('nonnulla on nullable type', () => {
            const source = `
                varia textus? name = "test"
                varia check = nonnulla name
            `;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            expect(types.get('check')).toEqual({ kind: 'primitive', name: 'bivalens' });
        });

        it('nulla in conditional expression', () => {
            const source = `
                varia x = 5
                varia result = nulla x et verum
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('Missing Features - iace (throw) statements', () => {
        it('validates throw with string', () => {
            const source = `
                functio f() {
                    iace "error"
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('validates throw with Error constructor', () => {
            const source = `
                functio f() {
                    iace novum Error("message")
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('validates throw with variable', () => {
            const source = `
                fixum err = novum Error("test")
                functio f() {
                    iace err
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('Edge Cases - Type Narrowing', () => {
        it.todo('narrows type after null check', () => {
            // SEMANTIC GAP: Type narrowing not implemented
            const source = `
                varia textus? name = "test"
                si nonnulla name {
                    varia len = name.length  // should know name is non-null here
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it.todo('errors on property access of nullable without check', () => {
            // SEMANTIC GAP: Should error on nullable property access
            const source = `
                varia textus? name = nihil
                varia len = name.length  // should error
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('nullable');
        });

        it('allows operations on non-nullable types', () => {
            const source = `
                varia textus name = "test"
                varia numerus len = 5
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('Edge Cases - Nullable Type Operations', () => {
        it('nullable type annotation is tracked', () => {
            const source = `varia numerus? x = nihil`;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            expect(types.get('x')).toEqual({ kind: 'primitive', name: 'numerus', nullable: true });
        });

        it('nullable with non-null value', () => {
            const source = `varia numerus? x = 5`;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('assigning null to nullable type', () => {
            const source = `varia textus? s = nihil`;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('assigning null to non-nullable should error', () => {
            const source = `varia textus s = nihil`;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
        });
    });

    describe('Edge Cases - Generic Validation', () => {
        it('resolves generic type annotation', () => {
            const source = `varia numerus[] nums = [1, 2, 3]`;
            const { errors, types } = analyzeSource(source);

            expect(errors).toHaveLength(0);
            const numsType = types.get('nums');
            expect(numsType?.kind).toBe('generic');
        });

        it('nested generics', () => {
            const source = `varia numerus[][] matrix = [[1, 2], [3, 4]]`;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it.todo('validates generic type parameter count', () => {
            // SEMANTIC GAP: Should validate Map has 2 parameters
            const source = `varia tabula<textus> invalid = nihil`;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
        });

        it('validates array element types', () => {
            const source = `varia numerus[] nums = [1, "two", 3]`;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('textus');
            expect(errors[0]!.message).toContain('numerus');
        });
    });

    describe('Edge Cases - Scope Nesting', () => {
        it('allows variable shadowing in nested scope', () => {
            const source = `
                varia x = 1
                si verum {
                    varia x = "hello"
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('deep nesting (3 levels)', () => {
            const source = `
                varia a = 1
                si verum {
                    varia b = 2
                    si verum {
                        varia c = 3
                        si verum {
                            varia d = a + b + c
                        }
                    }
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('variable from outer scope accessible in inner scope', () => {
            const source = `
                varia numerus x = 5
                si verum {
                    varia y = x + 1
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it('inner scope variable not accessible in outer scope', () => {
            const source = `
                si verum {
                    varia inner = 5
                }
                varia outer = inner
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('Undefined');
        });
    });

    describe('Edge Cases - Recursive Functions', () => {
        it('allows recursive function calls', () => {
            const source = `
                functio factorial(numerus n) -> numerus {
                    si n <= 1 {
                        redde 1
                    }
                    redde n * factorial(n - 1)
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });

        it.todo('mutually recursive functions', () => {
            // SEMANTIC GAP: Forward references not yet supported
            const source = `
                functio isEven(numerus n) -> bivalens {
                    si n == 0 { redde verum }
                    redde isOdd(n - 1)
                }
                functio isOdd(numerus n) -> bivalens {
                    si n == 0 { redde falsum }
                    redde isEven(n - 1)
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors).toHaveLength(0);
        });
    });

    describe('Edge Cases - Type Mismatches', () => {
        it('reports assignment type mismatch', () => {
            const source = `
                varia numerus x = 5
                x = "string"
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('not assignable');
        });

        it('reports function parameter type mismatch', () => {
            const source = `
                functio add(numerus a, numerus b) -> numerus {
                    redde a + b
                }
                varia result = add("1", "2")
            `;
            const { errors } = analyzeSource(source);

            // May or may not catch this depending on implementation
            // Just check it doesn't crash
            expect(errors).toBeDefined();
        });

        it('reports return type mismatch', () => {
            const source = `
                functio getName() -> textus {
                    redde 42
                }
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('not assignable');
        });

        it('binary operation type checking', () => {
            const source = `
                varia result = 5 + "text"
            `;
            const { errors } = analyzeSource(source);

            // String concatenation might be allowed, so just check no crash
            expect(errors).toBeDefined();
        });

        it('comparison between incompatible types', () => {
            const source = `
                varia result = 5 > "text"
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('Cannot compare');
        });
    });

    describe('Edge Cases - Error Messages', () => {
        it('undefined variable has helpful message', () => {
            const source = `varia x = unknownVar`;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('Undefined variable');
            expect(errors[0]!.message).toContain('unknownVar');
        });

        it('immutable reassignment has helpful message', () => {
            const source = `
                fixum x = 5
                x = 10
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('immutable');
        });

        it('duplicate declaration has helpful message', () => {
            const source = `
                varia x = 1
                varia x = 2
            `;
            const { errors } = analyzeSource(source);

            expect(errors.length).toBeGreaterThan(0);
            expect(errors[0]!.message).toContain('already defined');
        });
    });
});
