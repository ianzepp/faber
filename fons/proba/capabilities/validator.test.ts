/**
 * Tests for validateTargetCompatibility - Feature validation against targets
 *
 * Tests validation logic with inline AST fixtures and verifies error messages.
 */

import { describe, test, expect } from 'bun:test';
import { validateTargetCompatibility, TargetCompatibilityError } from '../../faber/codegen/validator';
import type { Program, FunctioDeclaration, Statement } from '../../faber/parser/ast';

// Helper to create minimal position
const pos = { line: 1, column: 1, offset: 0 };

// Helper to create minimal identifier
const id = (name: string) => ({ type: 'Identifier' as const, name, position: pos });

// Helper to create minimal program
const program = (body: Statement[]): Program => ({
    type: 'Program',
    body,
    position: pos,
});

describe('validateTargetCompatibility', () => {
    test('returns no errors for compatible program', () => {
        // Simple sync function - compatible with all targets
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('add'),
            params: [
                {
                    type: 'Parameter',
                    name: id('a'),
                    position: pos,
                },
            ],
            async: false,
            generator: false,
            position: pos,
        };

        const errors = validateTargetCompatibility(program([func]), 'zig');
        expect(errors).toHaveLength(0);
    });

    test('detects async function incompatibility with Zig', () => {
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('fetch'),
            params: [],
            async: true,
            generator: false,
            position: pos,
        };

        const errors = validateTargetCompatibility(program([func]), 'zig');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('controlFlow.asyncFunction');
        expect(errors[0].message).toContain("Target 'zig' does not support async functions");
        expect(errors[0].message).toContain('futura');
        expect(errors[0].context).toBe('function fetch');
        expect(errors[0].suggestion).toContain('synchronous');
    });

    test('detects generator incompatibility with Rust', () => {
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('range'),
            params: [],
            async: false,
            generator: true,
            position: pos,
        };

        const errors = validateTargetCompatibility(program([func]), 'rs');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('controlFlow.generatorFunction');
        expect(errors[0].message).toContain("Target 'rs' does not support generator functions");
        expect(errors[0].suggestion).toContain('iterators');
    });

    test('detects exception handling incompatibility with Zig', () => {
        const stmt: Statement = {
            type: 'TemptaStatement',
            body: {
                type: 'BlockStatement',
                body: [],
                position: pos,
            },
            handlers: [
                {
                    type: 'CapeClause',
                    param: id('err'),
                    body: {
                        type: 'BlockStatement',
                        body: [],
                        position: pos,
                    },
                    position: pos,
                },
            ],
            position: pos,
        };

        const errors = validateTargetCompatibility(program([stmt]), 'zig');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('errors.tryCatch');
        expect(errors[0].message).toContain("Target 'zig' does not support exception handling");
        expect(errors[0].message).toContain('tempta...cape');
        expect(errors[0].suggestion).toContain('error unions');
    });

    test('detects throw incompatibility with Rust', () => {
        const stmt: Statement = {
            type: 'IaceStatement',
            argument: {
                type: 'Literal',
                value: 'error',
                raw: '"error"',
                position: pos,
            },
            position: pos,
        };

        const errors = validateTargetCompatibility(program([stmt]), 'rs');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('errors.throw');
        expect(errors[0].message).toContain("Target 'rs' does not support throw statements");
        expect(errors[0].message).toContain('iace');
        expect(errors[0].suggestion).toContain('Result');
    });

    test('detects object destructuring incompatibility with Python', () => {
        const stmt: Statement = {
            type: 'DestructureDeclaration',
            source: id('obj'),
            kind: 'fixum',
            specifiers: [
                {
                    type: 'ImportSpecifier',
                    imported: id('x'),
                    local: id('x'),
                    position: pos,
                },
            ],
            position: pos,
        };

        const errors = validateTargetCompatibility(program([stmt]), 'py');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('binding.pattern.object');
        expect(errors[0].message).toContain("Target 'py' does not support object pattern binding");
        expect(errors[0].suggestion).toContain('explicit field');
    });

    test('detects default parameters incompatibility with Zig', () => {
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('greet'),
            params: [
                {
                    type: 'Parameter',
                    name: id('name'),
                    defaultValue: {
                        type: 'Literal',
                        value: 'World',
                        raw: '"World"',
                        position: pos,
                    },
                    position: pos,
                },
            ],
            async: false,
            generator: false,
            position: pos,
        };

        const errors = validateTargetCompatibility(program([func]), 'zig');

        expect(errors).toHaveLength(1);
        expect(errors[0].feature).toBe('params.defaultValues');
        expect(errors[0].message).toContain("Target 'zig' does not support default parameters");
        expect(errors[0].suggestion).toContain('optional');
    });

    test('returns multiple errors for multiple incompatibilities', () => {
        const asyncFunc: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('fetch'),
            params: [],
            async: true,
            generator: false,
            position: pos,
        };

        const throwStmt: Statement = {
            type: 'IaceStatement',
            argument: {
                type: 'Literal',
                value: 'error',
                raw: '"error"',
                position: pos,
            },
            position: pos,
        };

        const errors = validateTargetCompatibility(program([asyncFunc, throwStmt]), 'zig');

        expect(errors).toHaveLength(2);
        const features = errors.map(e => e.feature);
        expect(features).toContain('controlFlow.asyncFunction');
        expect(features).toContain('errors.throw');
    });

    test('TypeScript allows all features', () => {
        const asyncFunc: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('fetch'),
            params: [
                {
                    type: 'Parameter',
                    name: id('x'),
                    defaultValue: {
                        type: 'Literal',
                        value: 0,
                        raw: '0',
                        position: pos,
                    },
                    position: pos,
                },
            ],
            async: true,
            generator: false,
            position: pos,
        };

        const destructure: Statement = {
            type: 'DestructureDeclaration',
            source: id('obj'),
            kind: 'fixum',
            specifiers: [
                {
                    type: 'ImportSpecifier',
                    imported: id('x'),
                    local: id('x'),
                    position: pos,
                },
            ],
            position: pos,
        };

        const errors = validateTargetCompatibility(program([asyncFunc, destructure]), 'ts');

        expect(errors).toHaveLength(0);
    });
});

describe('TargetCompatibilityError', () => {
    test('formats error message with position and context', () => {
        const errors = [
            {
                feature: 'controlFlow.asyncFunction' as const,
                message: "Target 'zig' does not support async functions (futura)",
                position: { line: 5, column: 10, offset: 50 },
                context: 'function fetch',
                suggestion: 'Refactor to synchronous code',
            },
        ];

        const error = new TargetCompatibilityError(errors, 'zig');

        expect(error.message).toContain("Target compatibility errors for 'zig'");
        expect(error.message).toContain('5:10');
        expect(error.message).toContain('function fetch');
        expect(error.message).toContain('async functions (futura)');
        expect(error.message).toContain('help: Refactor to synchronous code');
    });

    test('formats multiple errors', () => {
        const errors = [
            {
                feature: 'controlFlow.asyncFunction' as const,
                message: "Target 'zig' does not support async functions",
                position: { line: 5, column: 1, offset: 50 },
            },
            {
                feature: 'errors.throw' as const,
                message: "Target 'zig' does not support throw statements",
                position: { line: 10, column: 5, offset: 100 },
            },
        ];

        const error = new TargetCompatibilityError(errors, 'zig');

        expect(error.message).toContain('5:1');
        expect(error.message).toContain('10:5');
        expect(error.message).toContain('async functions');
        expect(error.message).toContain('throw statements');
    });
});
