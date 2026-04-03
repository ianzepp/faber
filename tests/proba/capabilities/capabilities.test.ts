/**
 * Tests for TARGET_SUPPORT capability definitions
 *
 * Verifies that capability matrix includes all required targets
 * and has complete feature coverage per target.
 */

import { describe, test, expect } from 'bun:test';
import { TARGET_SUPPORT } from '../../faber/codegen/capabilities';

describe('TARGET_SUPPORT', () => {
    test('has entries for all 5 targets', () => {
        const targets = Object.keys(TARGET_SUPPORT);
        expect(targets).toContain('ts');
        expect(targets).toContain('py');
        expect(targets).toContain('rs');
        expect(targets).toContain('zig');
        expect(targets).toContain('cpp');
    });

    test('TypeScript supports all features', () => {
        const ts = TARGET_SUPPORT.ts;
        expect(ts.controlFlow.asyncFunction).toBe('supported');
        expect(ts.controlFlow.generatorFunction).toBe('supported');
        expect(ts.errors.tryCatch).toBe('supported');
        expect(ts.errors.throw).toBe('supported');
        expect(ts.binding.pattern.object).toBe('supported');
        expect(ts.params.defaultValues).toBe('supported');
    });

    test('Python supports most features with emulated destructuring', () => {
        const py = TARGET_SUPPORT.py;
        expect(py.controlFlow.asyncFunction).toBe('supported');
        expect(py.controlFlow.generatorFunction).toBe('supported');
        expect(py.errors.tryCatch).toBe('supported');
        expect(py.errors.throw).toBe('supported');
        expect(py.binding.pattern.object).toBe('emulated'); // field-by-field extraction
        expect(py.params.defaultValues).toBe('supported');
    });

    test('Rust has limited support', () => {
        const rs = TARGET_SUPPORT.rs;
        expect(rs.controlFlow.asyncFunction).toBe('supported');
        expect(rs.controlFlow.generatorFunction).toBe('unsupported');
        expect(rs.errors.tryCatch).toBe('emulated');
        expect(rs.errors.throw).toBe('emulated');
        expect(rs.binding.pattern.object).toBe('emulated');
        expect(rs.params.defaultValues).toBe('unsupported');
    });

    test('Zig has minimal support', () => {
        const zig = TARGET_SUPPORT.zig;
        expect(zig.controlFlow.asyncFunction).toBe('unsupported');
        expect(zig.controlFlow.generatorFunction).toBe('unsupported');
        expect(zig.errors.tryCatch).toBe('emulated');
        expect(zig.errors.throw).toBe('emulated');
        expect(zig.binding.pattern.object).toBe('emulated');
        expect(zig.params.defaultValues).toBe('unsupported');
    });

    test('C++ supports exceptions and defaults', () => {
        const cpp = TARGET_SUPPORT.cpp;
        expect(cpp.controlFlow.asyncFunction).toBe('unsupported');
        expect(cpp.controlFlow.generatorFunction).toBe('unsupported');
        expect(cpp.errors.tryCatch).toBe('supported');
        expect(cpp.errors.throw).toBe('supported');
        expect(cpp.binding.pattern.object).toBe('emulated');
        expect(cpp.params.defaultValues).toBe('supported');
    });

    test('Faber round-trip supports all features', () => {
        const fab = TARGET_SUPPORT.fab;
        expect(fab.controlFlow.asyncFunction).toBe('supported');
        expect(fab.controlFlow.generatorFunction).toBe('supported');
        expect(fab.errors.tryCatch).toBe('supported');
        expect(fab.errors.throw).toBe('supported');
        expect(fab.binding.pattern.object).toBe('supported');
        expect(fab.params.defaultValues).toBe('supported');
    });
});
