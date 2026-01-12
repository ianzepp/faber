/**
 * Target Compatibility Validation - Validates features against target support
 *
 * COMPILER PHASE
 * ==============
 * codegen (pre-validation)
 *
 * ARCHITECTURE
 * ============
 * Validates that all language features used in a program are supported by
 * the target. Runs after feature detection, before codegen.
 *
 * Returns structured errors with context and suggestions to help users
 * refactor their code for the target.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST, target name
 * OUTPUT: Array of ValidationError (empty if compatible)
 * ERRORS: Never throws - returns validation errors as data
 *
 * @module codegen/validator
 */

import type { Program } from '../parser/ast';
import type { Position } from '../tokenizer/types';
import type { CodegenTarget } from './types';
import { FeatureDetector } from './feature-detector';
import type { FeatureKey } from './feature-detector';
import { TARGET_SUPPORT } from './capabilities';
import type { SupportLevel, TargetSupport } from './capabilities';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Validation error for an unsupported feature.
 *
 * WHY: Structured format enables rich error reporting.
 *      Position and context help user locate the issue.
 *      Suggestion provides actionable guidance.
 */
export interface ValidationError {
    feature: FeatureKey;
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
 * WHY: Single entry point for validation.
 *      Returns all errors at once (not fail-fast).
 *
 * @param program - The AST to validate
 * @param target - The target language
 * @returns Array of validation errors (empty if compatible)
 */
export function validateTargetCompatibility(program: Program, target: CodegenTarget): ValidationError[] {
    const detector = new FeatureDetector();
    const usedFeatures = detector.detect(program);
    const support = TARGET_SUPPORT[target];
    const errors: ValidationError[] = [];

    for (const used of usedFeatures) {
        const level = getSupportLevel(used.key, support);

        // ARCHITECTURE: Allow both 'supported' and 'emulated' features.
        // Only 'unsupported' features generate errors.
        if (level === 'unsupported') {
            errors.push({
                feature: used.key,
                message: formatFeatureError(used.key, target),
                position: used.node.position,
                context: used.context,
                suggestion: getFeatureSuggestion(used.key, target),
            });
        }
    }

    return errors;
}

/**
 * Get support level for a feature key.
 *
 * WHY: Navigates hierarchical structure using dot notation.
 *      Returns 'unsupported' for unknown keys (fail-safe).
 *
 * @param featureKey - Hierarchical feature key (e.g., "errors.tryCatch")
 * @param support - Target support matrix
 * @returns Support level for the feature
 */
function getSupportLevel(featureKey: FeatureKey, support: TargetSupport): SupportLevel {
    const parts = featureKey.split('.');
    let current: any = support;

    for (const part of parts) {
        if (current?.[part] === undefined) {
            return 'unsupported';
        }
        current = current[part];
    }

    return current as SupportLevel;
}

/**
 * Format error message for an unsupported feature.
 *
 * WHY: Provides clear, consistent error messages.
 *      Mentions Faber keyword/construct in message.
 *
 * @param feature - Feature key
 * @param target - Target language
 * @returns Formatted error message
 */
function formatFeatureError(feature: FeatureKey, target: CodegenTarget): string {
    const messages: Record<FeatureKey, string> = {
        'controlFlow.asyncFunction': `Target '${target}' does not support async functions (futura)`,
        'controlFlow.generatorFunction': `Target '${target}' does not support generator functions (cursor)`,
        'errors.tryCatch': `Target '${target}' does not support exception handling (tempta...cape)`,
        'errors.throw': `Target '${target}' does not support throw statements (iace)`,
        'binding.pattern.object': `Target '${target}' does not support object pattern binding`,
        'params.defaultValues': `Target '${target}' does not support default parameters`,
    };

    return messages[feature] || `Target '${target}' does not support feature '${feature}'`;
}

/**
 * Get suggestion for refactoring unsupported feature.
 *
 * WHY: Provides actionable guidance for fixing the issue.
 *      Target-specific advice when available.
 *
 * @param feature - Feature key
 * @param target - Target language
 * @returns Suggestion text or generic fallback
 */
function getFeatureSuggestion(feature: FeatureKey, target: CodegenTarget): string {
    const suggestions: Partial<Record<FeatureKey, Partial<Record<CodegenTarget, string>>>> = {
        'controlFlow.asyncFunction': {
            zig: 'Refactor to synchronous code; consider explicit callbacks or event loop',
            cpp: 'Refactor to synchronous code or adopt an async runtime',
            rs: 'Use async fn (supported in stable Rust)',
        },
        'controlFlow.generatorFunction': {
            rs: 'Use iterators and iterator adapters instead',
            zig: 'Use a while loop or explicit iterator type',
            cpp: 'Use manual iteration or ranges library',
        },
        'errors.tryCatch': {
            rs: 'Use Result<T, E> and propagate errors explicitly',
            zig: 'Use error unions (!T) and handle errors explicitly',
        },
        'errors.throw': {
            rs: 'Return Result<T, E> instead of throwing',
            zig: 'Return error union (!T) instead of throwing',
        },
        'binding.pattern.object': {
            py: 'Use explicit field or dict access',
            zig: 'Use explicit field access',
            cpp: 'Use explicit member access',
        },
        'params.defaultValues': {
            rs: 'Use Option<T> params or provide helper overloads',
            zig: 'Use optional params (?T) and handle null',
        },
    };

    const targetSuggestions = suggestions[feature];
    if (targetSuggestions?.[target]) {
        return targetSuggestions[target]!;
    }

    return 'Refactor to avoid this construct';
}

// =============================================================================
// ERROR CLASS
// =============================================================================

/**
 * Custom error class for target compatibility errors.
 *
 * WHY: Distinguishes validation errors from other compiler errors.
 *      Provides formatted output for CLI.
 */
export class TargetCompatibilityError extends Error {
    constructor(
        public errors: ValidationError[],
        public target: CodegenTarget,
    ) {
        const formatted = formatValidationErrors(errors, target);
        super(`Target compatibility errors for '${target}':\n\n${formatted}`);
        this.name = 'TargetCompatibilityError';
    }
}

/**
 * Format validation errors for display.
 *
 * WHY: Rust-style error format with position, message, and suggestion.
 *
 * @param errors - Validation errors
 * @param target - Target language (unused, kept for consistency)
 * @returns Formatted error output
 */
function formatValidationErrors(errors: ValidationError[], target: CodegenTarget): string {
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
