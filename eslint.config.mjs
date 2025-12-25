/**
 * ESLint Configuration - Faber Romanus
 *
 * Prettier handles formatting (indent, semi, spacing).
 * ESLint handles code quality (unused vars, type errors, patterns).
 *
 * @module eslint-config
 */

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import prettierConfig from 'eslint-config-prettier';

export default tseslint.config(
    eslint.configs.recommended,
    ...tseslint.configs.recommended,
    prettierConfig,
    {
        rules: {
            // WHY: Always require braces for clarity
            'curly': ['error', 'all'],

            // =================================================================
            // TYPESCRIPT SPECIFIC
            // =================================================================

            '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
            '@typescript-eslint/consistent-type-imports': ['error', {
                prefer: 'type-imports',
                disallowTypeAnnotations: false,
            }],

            // WHY: Stubs for future implementation
            '@typescript-eslint/no-unused-vars': 'off',
            '@typescript-eslint/no-unused-expressions': 'off',

            // WHY: Allow `const self = this` where context is needed
            '@typescript-eslint/no-this-alias': ['error', {
                allowedNames: ['self'],
            }],

            // WHY: Allow namespaces for grouped exports
            '@typescript-eslint/no-namespace': 'off',

            // WHY: Generators implementing interfaces don't need yield
            'require-yield': 'off',

            '@typescript-eslint/no-explicit-any': 'warn',
            '@typescript-eslint/no-empty-object-type': 'off',
        },
    },
    {
        files: ['**/*.test.ts'],
        rules: {
            '@typescript-eslint/no-explicit-any': 'off',
        },
    },
    {
        // Generated code from .fab files - relaxed rules
        // These are examples/demos, not production code
        files: ['opus/exempla/**/*.ts'],
        rules: {
            'prefer-const': 'warn',
            'no-empty': 'warn',
            'no-constant-condition': 'warn',
            'no-constant-binary-expression': 'warn',
            'no-extra-boolean-cast': 'warn',
            'for-direction': 'warn',
        },
    },
    {
        ignores: [
            'node_modules/**',
            'dist/**',
            'build/**',
            '.git/**',
        ],
    },
);
