/**
 * Target Capability Definitions - Feature support matrix per codegen target
 *
 * COMPILER PHASE
 * ==============
 * codegen (pre-validation)
 *
 * ARCHITECTURE
 * ============
 * Defines which Faber language features are supported by each compilation target.
 * Used by the validation phase to detect incompatibilities before codegen runs.
 *
 * Support levels:
 * - 'supported': Native, faithful implementation with correct semantics
 * - 'unsupported': Cannot be emitted or would break semantics
 *
 * NOTE: This is Phase 1 implementation. 'emulated' and 'mismatched' support
 *       levels are defined in the design doc but not implemented yet.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (constant definitions)
 * OUTPUT: TARGET_SUPPORT lookup table for validation
 * ERRORS: N/A (compile-time type checking only)
 *
 * @module codegen/capabilities
 */

import type { CodegenTarget } from './types';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Support level for a language feature in a target.
 *
 * - 'supported': Native, faithful implementation with correct semantics
 * - 'emulated': Systematic transform (e.g., field-by-field extraction, Result<T,E>)
 * - 'unsupported': Cannot be emitted or would break semantics
 */
export type SupportLevel = 'supported' | 'emulated' | 'unsupported';

/**
 * Target capability matrix defining feature support.
 *
 * WHY: Hierarchical structure matches feature keys used by detector.
 *      Organized by semantic category (controlFlow, errors, etc).
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
// TARGET SUPPORT MATRIX
// =============================================================================

/**
 * Feature support matrix for all codegen targets.
 *
 * WHY: Single source of truth for target capabilities.
 *      Based on consilia/capabilities.md design document.
 *
 * Phase 1 scope (per issue #102), extended by issue #109:
 * - Includes 5 targets: ts, py, rs, zig, cpp
 * - Excludes go and fab targets (not in scope)
 * - Only tracks features mentioned in issue
 * - Uses 'supported', 'emulated', and 'unsupported' levels
 *
 * Support levels based on target language capabilities:
 *
 * TypeScript:
 * - Full async/await, generators, async generators
 * - Exception handling with try/catch/throw
 * - Object destructuring
 * - Default parameters
 *
 * Python:
 * - Full async/await, generators, async generators
 * - Exception handling with try/except/raise
 * - Object destructuring NOT supported (no direct syntax)
 * - Default parameters supported
 *
 * Rust:
 * - Async/await supported (async fn)
 * - Generators NOT supported in stable (unstable feature)
 * - Exception handling EMULATED (transforms to Result<T,E>)
 * - Throw statements EMULATED (transforms to return Err)
 * - Object destructuring EMULATED (field-by-field extraction)
 * - Default parameters NOT supported (use Option or overloads)
 *
 * Zig:
 * - Async NOT supported in stable (async/await being redesigned)
 * - Generators NOT supported
 * - Exception handling EMULATED (transforms to error unions)
 * - Throw statements EMULATED (transforms to return error.X)
 * - Object destructuring EMULATED (field-by-field extraction)
 * - Default parameters NOT supported (use optional types)
 *
 * C++:
 * - Async NOT supported (no native async/await)
 * - Generators NOT supported (coroutines experimental)
 * - Exception handling supported (try/catch/throw)
 * - Object destructuring EMULATED (field-by-field extraction)
 * - Default parameters supported
 */
export const TARGET_SUPPORT: Record<CodegenTarget, TargetSupport> = {
    ts: {
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
    },

    py: {
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
                object: 'emulated', // field-by-field extraction
            },
        },
        params: {
            defaultValues: 'supported',
        },
    },

    rs: {
        controlFlow: {
            asyncFunction: 'supported',
            generatorFunction: 'unsupported',
        },
        errors: {
            tryCatch: 'emulated',
            throw: 'emulated',
        },
        binding: {
            pattern: {
                object: 'emulated',
            },
        },
        params: {
            defaultValues: 'unsupported',
        },
    },

    zig: {
        controlFlow: {
            asyncFunction: 'unsupported',
            generatorFunction: 'unsupported',
        },
        errors: {
            tryCatch: 'emulated',
            throw: 'emulated',
        },
        binding: {
            pattern: {
                object: 'emulated',
            },
        },
        params: {
            defaultValues: 'unsupported',
        },
    },

    cpp: {
        controlFlow: {
            asyncFunction: 'unsupported',
            generatorFunction: 'unsupported',
        },
        errors: {
            tryCatch: 'supported',
            throw: 'supported',
        },
        binding: {
            pattern: {
                object: 'emulated',
            },
        },
        params: {
            defaultValues: 'supported',
        },
    },

    fab: {
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
    },
};
