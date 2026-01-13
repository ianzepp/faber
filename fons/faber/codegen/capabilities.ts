/**
 * Target Capability Definitions - TypeScript feature support
 *
 * COMPILER PHASE
 * ==============
 * codegen (pre-validation)
 *
 * ARCHITECTURE
 * ============
 * Faber is the TypeScript-only reference compiler. TypeScript supports all
 * Faber language features natively, so this module is simplified.
 *
 * For multi-target capability matrices, see Rivus.
 * See consilia/compiler-roles.md for compiler separation rationale.
 *
 * @module codegen/capabilities
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Support level for a language feature.
 *
 * TypeScript supports all Faber features natively, so this is simplified
 * from the multi-target version which included 'emulated' and 'unsupported'.
 */
export type SupportLevel = 'supported';

/**
 * TypeScript capability matrix.
 *
 * WHY: Kept for API compatibility with validator and feature-detector.
 *      All features are 'supported' since TS handles everything natively.
 */
export interface TargetSupport {
    controlFlow: {
        asyncFunction: SupportLevel; // futura functio
        generatorFunction: SupportLevel; // cursor functio
    };

    errors: {
        tryCatch: SupportLevel; // tempta...cape
        throw: SupportLevel; // iace
    };

    binding: {
        pattern: {
            object: SupportLevel; // ex obj fixum {x, y}
        };
    };

    params: {
        defaultValues: SupportLevel; // functio f(numerus x vel 0)
    };
}

// =============================================================================
// TYPESCRIPT SUPPORT
// =============================================================================

/**
 * TypeScript feature support.
 *
 * All features are natively supported:
 * - Full async/await, generators, async generators
 * - Exception handling with try/catch/throw
 * - Object destructuring
 * - Default parameters
 */
export const TARGET_SUPPORT: TargetSupport = {
    controlFlow: {
        asyncFunction: 'supported',
        generatorFunction: 'supported',
    },
    errors: {
        tryCatch: 'supported',
        throw: 'supported',
    },
    binding: {
        pattern: {
            object: 'supported',
        },
    },
    params: {
        defaultValues: 'supported',
    },
};
