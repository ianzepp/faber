/**
 * Target Compatibility Validation - TypeScript feature validation
 *
 * COMPILER PHASE
 * ==============
 * codegen (pre-validation)
 *
 * ARCHITECTURE
 * ============
 * Faber targets TypeScript only, and TypeScript supports all Faber features
 * natively. This module is kept for API compatibility but always returns
 * no errors.
 *
 * For multi-target validation with feature detection, see Rivus.
 * See consilia/compiler-roles.md for compiler separation rationale.
 *
 * @module codegen/validator
 */

import type { Program } from '../parser/ast';
import type { Position } from '../tokenizer/types';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Validation error for an unsupported feature.
 *
 * WHY: Kept for API compatibility. TypeScript supports all features,
 *      so this is never instantiated in practice.
 */
export interface ValidationError {
    feature: string;
    message: string;
    position?: Position;
    context?: string;
    suggestion?: string;
}

// =============================================================================
// VALIDATION
// =============================================================================

/**
 * Validate target compatibility for a program.
 *
 * WHY: TypeScript supports all Faber features natively, so this always
 *      returns an empty array. Kept for API compatibility.
 *
 * @param _program - The AST to validate (unused)
 * @returns Empty array (TypeScript supports everything)
 */
export function validateTargetCompatibility(_program: Program): ValidationError[] {
    // TypeScript supports all Faber features natively
    return [];
}

// =============================================================================
// ERROR CLASS
// =============================================================================

/**
 * Custom error class for target compatibility errors.
 *
 * WHY: Kept for API compatibility. Never thrown for TypeScript target.
 */
export class TargetCompatibilityError extends Error {
    constructor(
        public errors: ValidationError[],
    ) {
        super(`Target compatibility errors:\n\n${formatValidationErrors(errors)}`);
        this.name = 'TargetCompatibilityError';
    }
}

/**
 * Format validation errors for display.
 */
function formatValidationErrors(errors: ValidationError[]): string {
    return errors
        .map(err => {
            const pos = err.position ? `${err.position.line}:${err.position.column}` : 'unknown';
            const location = `  --> ${pos}`;
            const context = err.context ? ` (in ${err.context})` : '';
            const msg = `error: ${err.message}${context}`;
            const suggestion = err.suggestion ? `  = help: ${err.suggestion}` : '';

            return [location, msg, suggestion].filter(Boolean).join('\n');
        })
        .join('\n\n');
}
