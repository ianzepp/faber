/**
 * Feature Detection - AST visitor to detect language features used in source
 *
 * COMPILER PHASE
 * ==============
 * codegen (pre-validation)
 *
 * ARCHITECTURE
 * ============
 * Walks the AST to identify which Faber language features are actually used.
 * Detects features at the semantic level (async functions, error handling, etc.)
 * rather than specific syntax tokens.
 *
 * Deduplicates by feature key - reports each feature once regardless of how
 * many times it appears in the source. This prevents hundreds of duplicate
 * errors on large programs.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST after parsing
 * OUTPUT: Array of UsedFeature entries (deduplicated by key)
 * ERRORS: None (detection never fails, even on malformed AST)
 *
 * @module codegen/feature-detector
 */

import type {
    Program,
    Statement,
    Expression,
    FunctioDeclaration,
    BlockStatement,
    BaseNode,
} from '../parser/ast';
import { isAsyncFromAnnotations, isGeneratorFromAnnotations } from './types';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Feature key identifying a language feature.
 *
 * WHY: Hierarchical dot notation matches capability structure.
 *      Enables easy lookup in TARGET_SUPPORT.
 */
export type FeatureKey =
    | 'controlFlow.asyncFunction'
    | 'controlFlow.generatorFunction'
    | 'errors.tryCatch'
    | 'errors.throw'
    | 'binding.pattern.object'
    | 'params.defaultValues';

/**
 * Record of a feature being used in the source.
 *
 * WHY: Tracks the AST node for error reporting (position).
 *      Context helps user identify where the feature was used.
 */
export interface UsedFeature {
    key: FeatureKey;
    node: BaseNode;
    context?: string; // e.g., "function fetchData"
}

// =============================================================================
// FEATURE DETECTOR
// =============================================================================

/**
 * Detects language features used in a program.
 *
 * WHY: Traverses full AST including nested blocks to find all uses.
 *      Deduplicates by feature key to avoid overwhelming error output.
 *
 * DESIGN: Uses visitor pattern with type-safe discriminated unions.
 *         Collects features in Map<FeatureKey, UsedFeature> to track
 *         first occurrence of each feature (for error reporting).
 */
export class FeatureDetector {
    private features = new Map<FeatureKey, UsedFeature>();

    /**
     * Detect all features used in a program.
     *
     * @param program - The AST to analyze
     * @returns Array of unique features used (one per key)
     */
    detect(program: Program): UsedFeature[] {
        this.features.clear();
        this.visitProgram(program);
        return Array.from(this.features.values());
    }

    /**
     * Add a feature if not already tracked.
     *
     * WHY: Only records first occurrence to avoid duplicate errors.
     */
    private add(feature: UsedFeature): void {
        if (!this.features.has(feature.key)) {
            this.features.set(feature.key, feature);
        }
    }

    /**
     * Visit program root.
     */
    private visitProgram(program: Program): void {
        for (const stmt of program.body) {
            this.visitStatement(stmt);
        }
    }

    /**
     * Visit a statement node.
     *
     * WHY: Discriminated union on stmt.type enables exhaustive checking.
     *      Only handles statement types that can contain features.
     */
    private visitStatement(stmt: Statement): void {
        switch (stmt.type) {
            case 'FunctioDeclaration':
                this.visitFunctio(stmt);
                break;

            case 'DestructureDeclaration':
                // Object destructuring: ex obj fixum nomen, aetas
                // Target is always an expression, check for object pattern in specifiers
                // WHY: DestructureDeclaration always represents object destructuring
                this.add({
                    key: 'binding.pattern.object',
                    node: stmt,
                    context: 'destructure declaration',
                });
                if (stmt.source) {
                    this.visitExpression(stmt.source);
                }
                break;

            case 'TemptaStatement':
                this.add({
                    key: 'errors.tryCatch',
                    node: stmt,
                    context: 'try-catch block',
                });
                this.visitBlock(stmt.body);
                for (const handler of stmt.handlers) {
                    this.visitBlock(handler.body);
                }
                if (stmt.finalizer) {
                    this.visitBlock(stmt.finalizer);
                }
                break;

            case 'IaceStatement':
                this.add({
                    key: 'errors.throw',
                    node: stmt,
                    context: 'throw statement',
                });
                if (stmt.argument) {
                    this.visitExpression(stmt.argument);
                }
                break;

            case 'SiStatement':
                if (stmt.test) {
                    this.visitExpression(stmt.test);
                }
                this.visitBlock(stmt.consequent);
                if (stmt.alternate) {
                    if (stmt.alternate.type === 'BlockStatement') {
                        this.visitBlock(stmt.alternate);
                    } else {
                        this.visitStatement(stmt.alternate);
                    }
                }
                break;

            case 'DumStatement':
                if (stmt.test) {
                    this.visitExpression(stmt.test);
                }
                this.visitBlock(stmt.body);
                break;

            case 'IteratioStatement':
                if (stmt.iterable) {
                    this.visitExpression(stmt.iterable);
                }
                this.visitBlock(stmt.body);
                break;

            case 'EligeStatement':
                if (stmt.discriminant) {
                    this.visitExpression(stmt.discriminant);
                }
                for (const c of stmt.cases) {
                    if (c.test) {
                        this.visitExpression(c.test);
                    }
                    for (const s of c.consequent) {
                        this.visitStatement(s);
                    }
                }
                break;

            case 'DiscerneStatement':
                if (stmt.discriminant) {
                    this.visitExpression(stmt.discriminant);
                }
                for (const c of stmt.cases) {
                    for (const s of c.consequent) {
                        this.visitStatement(s);
                    }
                }
                break;

            case 'BlockStatement':
            case 'FacBlockStatement':
                this.visitBlock(stmt);
                break;

            case 'CustodiStatement':
                for (const clause of stmt.clauses) {
                    if (clause.test) {
                        this.visitExpression(clause.test);
                    }
                    for (const s of clause.consequent) {
                        this.visitStatement(s);
                    }
                }
                break;

            case 'ReddeStatement':
                if (stmt.argument) {
                    this.visitExpression(stmt.argument);
                }
                break;

            case 'ExpressionStatement':
                this.visitExpression(stmt.expression);
                break;

            case 'VariaDeclaration':
                if (stmt.init) {
                    this.visitExpression(stmt.init);
                }
                break;

            case 'CuraStatement':
                if (stmt.resource) {
                    this.visitExpression(stmt.resource);
                }
                this.visitBlock(stmt.body);
                break;

            case 'InStatement':
                if (stmt.target) {
                    this.visitExpression(stmt.target);
                }
                this.visitBlock(stmt.body);
                break;

            case 'ProbandumStatement':
            case 'ProbaStatement':
            case 'PraeparaBlock':
                this.visitBlock(stmt.body);
                break;

            // Declarations without executable code
            case 'ImportaDeclaration':
            case 'GenusDeclaration':
            case 'PactumDeclaration':
            case 'TypeAliasDeclaration':
            case 'OrdoDeclaration':
            case 'DiscretioDeclaration':
            case 'RumpeStatement':
            case 'PergeStatement':
            case 'ScribeStatement':
            case 'AdfirmaStatement':
            case 'AdStatement':
            case 'IncipitStatement':
            case 'IncipietStatement':
                // No features to detect
                break;
        }
    }

    /**
     * Visit a function declaration.
     *
     * WHY: Detects async, generator, and default parameter features.
     *      Uses semantic flags (async/generator) not syntax tokens.
     */
    private visitFunctio(func: FunctioDeclaration): void {
        // Detect async: check both flag and annotations
        const isAsync = func.async || isAsyncFromAnnotations(func.annotations);

        // Detect generator: check both flag and annotations
        const isGenerator = func.generator || isGeneratorFromAnnotations(func.annotations);

        if (isAsync) {
            this.add({
                key: 'controlFlow.asyncFunction',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        if (isGenerator) {
            this.add({
                key: 'controlFlow.generatorFunction',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        // Detect default parameters
        if (func.params.some(p => p.defaultValue)) {
            this.add({
                key: 'params.defaultValues',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        // Visit function body
        if (func.body) {
            this.visitBlock(func.body);
        }
    }

    /**
     * Visit an expression node.
     *
     * WHY: Recursively visits nested expressions.
     *      Phase 1 doesn't detect expression-level features yet.
     */
    private visitExpression(expr: Expression): void {
        switch (expr.type) {
            case 'BinaryExpression':
            case 'LogicalExpression':
                this.visitExpression(expr.left);
                this.visitExpression(expr.right);
                break;

            case 'UnaryExpression':
                this.visitExpression(expr.argument);
                break;

            case 'CallExpression':
                this.visitExpression(expr.callee);
                for (const arg of expr.arguments) {
                    if (arg.type === 'SpreadElement') {
                        this.visitExpression(arg.argument);
                    } else {
                        this.visitExpression(arg);
                    }
                }
                break;

            case 'MemberExpression':
                this.visitExpression(expr.object);
                if (expr.computed && expr.property.type !== 'Identifier') {
                    this.visitExpression(expr.property);
                }
                break;

            case 'ConditionalExpression':
                this.visitExpression(expr.test);
                this.visitExpression(expr.consequent);
                this.visitExpression(expr.alternate);
                break;

            case 'ArrayExpression':
                for (const elem of expr.elements) {
                    if (elem.type === 'SpreadElement') {
                        this.visitExpression(elem.argument);
                    } else {
                        this.visitExpression(elem);
                    }
                }
                break;

            case 'ObjectExpression':
                for (const prop of expr.properties) {
                    if (prop.type === 'ObjectProperty') {
                        this.visitExpression(prop.value);
                    } else if (prop.type === 'SpreadElement') {
                        this.visitExpression(prop.argument);
                    }
                }
                break;

            case 'ParenthesizedExpression':
                this.visitExpression(expr.expression);
                break;

            case 'AwaitExpression':
                this.visitExpression(expr.argument);
                break;

            case 'YieldExpression':
                if (expr.argument) {
                    this.visitExpression(expr.argument);
                }
                break;

            case 'AssignmentExpression':
                this.visitExpression(expr.left);
                this.visitExpression(expr.right);
                break;

            case 'UpdateExpression':
                this.visitExpression(expr.argument);
                break;

            // Terminals
            case 'Identifier':
            case 'Literal':
            case 'NihilLiteral':
            case 'BivalensLiteral':
            case 'TextusLiteral':
            case 'NumerusLiteral':
            case 'TemplateStringLiteral':
            case 'RegexLiteral':
            case 'FunctionExpression':
            case 'ArrowFunctionExpression':
            case 'GenusExpression':
            case 'NewExpression':
            case 'SuperExpression':
            case 'ThisExpression':
            case 'SequenceExpression':
                // No nested expressions to visit
                break;
        }
    }

    /**
     * Visit a block statement.
     *
     * WHY: Recursively visits all statements in block.
     */
    private visitBlock(block: BlockStatement): void {
        for (const stmt of block.body) {
            this.visitStatement(stmt);
        }
    }
}
