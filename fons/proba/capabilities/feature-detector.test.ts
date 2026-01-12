/**
 * Tests for FeatureDetector - AST traversal and feature identification
 *
 * Uses small inline AST fixtures to test detection of each feature type.
 */

import { describe, test, expect } from 'bun:test';
import { FeatureDetector } from '../../faber/codegen/feature-detector';
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

describe('FeatureDetector', () => {
    test('detects async function', () => {
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('fetch'),
            params: [],
            async: true,
            generator: false,
            position: pos,
        };

        const detector = new FeatureDetector();
        const features = detector.detect(program([func]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('controlFlow.asyncFunction');
        expect(features[0].context).toBe('function fetch');
    });

    test('detects generator function', () => {
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('range'),
            params: [],
            async: false,
            generator: true,
            position: pos,
        };

        const detector = new FeatureDetector();
        const features = detector.detect(program([func]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('controlFlow.generatorFunction');
        expect(features[0].context).toBe('function range');
    });

    test('detects try-catch', () => {
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

        const detector = new FeatureDetector();
        const features = detector.detect(program([stmt]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('errors.tryCatch');
        expect(features[0].context).toBe('try-catch block');
    });

    test('detects throw statement', () => {
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

        const detector = new FeatureDetector();
        const features = detector.detect(program([stmt]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('errors.throw');
        expect(features[0].context).toBe('throw statement');
    });

    test('detects object pattern binding', () => {
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

        const detector = new FeatureDetector();
        const features = detector.detect(program([stmt]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('binding.pattern.object');
        expect(features[0].context).toBe('destructure declaration');
    });

    test('detects default parameter values', () => {
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

        const detector = new FeatureDetector();
        const features = detector.detect(program([func]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('params.defaultValues');
        expect(features[0].context).toBe('function greet');
    });

    test('deduplicates features', () => {
        // Two async functions, should only report once
        const func1: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('fetch'),
            params: [],
            async: true,
            generator: false,
            position: pos,
        };

        const func2: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('load'),
            params: [],
            async: true,
            generator: false,
            position: pos,
        };

        const detector = new FeatureDetector();
        const features = detector.detect(program([func1, func2]));

        expect(features).toHaveLength(1);
        expect(features[0].key).toBe('controlFlow.asyncFunction');
        expect(features[0].context).toBe('function fetch'); // First occurrence
    });

    test('detects multiple different features', () => {
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

        const detector = new FeatureDetector();
        const features = detector.detect(program([asyncFunc, throwStmt]));

        expect(features).toHaveLength(2);
        const keys = features.map(f => f.key);
        expect(keys).toContain('controlFlow.asyncFunction');
        expect(keys).toContain('errors.throw');
    });

    test('handles empty program', () => {
        const detector = new FeatureDetector();
        const features = detector.detect(program([]));

        expect(features).toHaveLength(0);
    });

    test('handles program with no detected features', () => {
        // Simple sync function with no special features
        const func: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: id('add'),
            params: [
                {
                    type: 'Parameter',
                    name: id('a'),
                    position: pos,
                },
                {
                    type: 'Parameter',
                    name: id('b'),
                    position: pos,
                },
            ],
            async: false,
            generator: false,
            position: pos,
        };

        const detector = new FeatureDetector();
        const features = detector.detect(program([func]));

        expect(features).toHaveLength(0);
    });
});
