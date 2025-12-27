/**
 * TypeScript Code Generator - Emit JavaScript with type annotations
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into TypeScript source code.
 * It preserves JavaScript runtime semantics while adding TypeScript type
 * annotations derived from Latin type declarations.
 *
 * The generator uses a recursive descent pattern that mirrors the AST structure.
 * Each AST node type has a corresponding gen* function that produces a string
 * fragment. These fragments are composed bottom-up to build the complete output.
 *
 * Indentation is managed via a depth counter that tracks nesting level. The
 * ind() helper function generates the appropriate indentation string for the
 * current depth.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid TypeScript source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * TypeScript preserves JavaScript semantics:
 * - Dynamic typing with optional annotations
 * - Prototype-based objects
 * - Async/await for concurrency
 * - Exception-based error handling
 * - Nullable types via union with null
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid TypeScript
 * INV-2: All Latin type names are mapped to TypeScript equivalents
 * INV-3: Indentation depth is correctly maintained (incremented/decremented)
 */

import type {
    Program,
    Statement,
    Expression,
    ImportaDeclaration,
    VariaDeclaration,
    FunctioDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    TypeAliasDeclaration,
    SiStatement,
    DumStatement,
    IteratioStatement,
    InStatement,
    EligeStatement,
    DiscerneStatement,
    DiscretioDeclaration,
    VariantDeclaration,
    VariantField,
    CustodiStatement,
    AdfirmaStatement,
    ReddeStatement,
    RumpeStatement,
    PergeStatement,
    BlockStatement,
    OrdoDeclaration,
    IaceStatement,
    ScribeStatement,
    TemptaStatement,
    ExpressionStatement,
    ArrayExpression,
    ObjectExpression,
    RangeExpression,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrowFunctionExpression,
    AssignmentExpression,
    NovumExpression,
    Identifier,
    Literal,
    EgoExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    SpreadElement,
    FacBlockStatement,
    LambdaExpression,
    QuaExpression,
    EstExpression,
    ProbandumStatement,
    ProbaStatement,
    CuraBlock,
    CuraStatement,
    PraefixumExpression,
} from '../../parser/ast';
import type { CodegenOptions, RequiredFeatures } from '../types';
import { createRequiredFeatures } from '../types';
import { getListaMethod } from './norma/lista';
import { getTabulaMethod } from './norma/tabula';
import { getCopiaMethod } from './norma/copia';

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to TypeScript types.
 *
 * WHY: Latin uses descriptive names (textus = text), TypeScript uses
 *      JavaScript primitive names (string). This mapping preserves semantics
 *      while emitting idiomatic TypeScript.
 *
 * TARGET MAPPING:
 * | Latin      | TypeScript |
 * |------------|------------|
 * | textus     | string     |
 * | numerus    | number     |
 * | bivalens   | boolean    |
 * | nihil      | null       |
 * | lista      | Array      |
 * | tabula     | Map        |
 * | copia      | Set        |
 * | promissum  | Promise    |
 * | erratum    | Error      |
 * | cursor     | Iterator   |
 *
 * CASE: Keys are lowercase. Lookup normalizes input to lowercase for
 *       case-insensitive matching (textus, textus, TEXTUS all work).
 */
const typeMap: Record<string, string> = {
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
    decimus: 'Decimal',
    magnus: 'bigint',
    bivalens: 'boolean',
    nihil: 'null',
    vacuum: 'void',
    numquam: 'never',
    octeti: 'Uint8Array',
    lista: 'Array',
    tabula: 'Map',
    copia: 'Set',
    promissum: 'Promise',
    erratum: 'Error',
    cursor: 'Iterator',
    objectum: 'object',
    // WHY: Map Object -> object for TS compatibility
    object: 'object',
    ignotum: 'unknown',
};

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate TypeScript source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> TypeScript source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent, semicolons)
 * @returns TypeScript source code
 */
export function generateTs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 2 spaces is TypeScript convention
    const indent = options.indent ?? '  ';
    // WHY: Semicolons are recommended in TypeScript style guides
    const semi = options.semicolons ?? true;

    // Track indentation depth for nested blocks
    let depth = 0;

    // Track if we're inside a generator function (for cede -> yield vs await)
    let inGenerator = false;

    // Track which features are used (for preamble generation)
    const features = createRequiredFeatures();

    /**
     * Generate indentation for current depth.
     * WHY: Centralized indentation logic ensures consistent formatting.
     */
    function ind(): string {
        return indent.repeat(depth);
    }

    /**
     * Generate preamble based on features used.
     * WHY: Only emit setup code for features actually used in the program.
     */
    function genPreamble(): string {
        const imports: string[] = [];
        const definitions: string[] = [];

        if (features.decimal) {
            imports.push("import type Decimal from 'decimal.js';");
        }

        if (features.panic) {
            definitions.push('class Panic extends Error { name = "Panic"; }');
        }

        const lines = [...imports, ...definitions];
        return lines.length > 0 ? lines.join('\n') + '\n\n' : '';
    }

    // ---------------------------------------------------------------------------
    // Top-level
    // ---------------------------------------------------------------------------

    function genProgram(node: Program): string {
        // First pass: generate body (this populates features)
        const body = node.body.map(genStatement).join('\n');

        // Second: prepend preamble based on detected features
        const preamble = genPreamble();

        return preamble + body;
    }

    // ---------------------------------------------------------------------------
    // Statements
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any statement type.
     * WHY: Exhaustive switch ensures all statement types are handled.
     */
    function genStatement(node: Statement): string {
        switch (node.type) {
            case 'ImportaDeclaration':
                return genImportaDeclaration(node);
            case 'VariaDeclaration':
                return genVariaDeclaration(node);
            case 'FunctioDeclaration':
                return genFunctioDeclaration(node);
            case 'GenusDeclaration':
                return genGenusDeclaration(node);
            case 'PactumDeclaration':
                return genPactumDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'OrdoDeclaration':
                return genOrdoDeclaration(node);
            case 'DiscretioDeclaration':
                return genDiscretioDeclaration(node);
            case 'SiStatement':
                return genSiStatement(node);
            case 'DumStatement':
                return genDumStatement(node);
            case 'IteratioStatement':
                return genIteratioStatement(node);
            case 'InStatement':
                return genInStatement(node);
            case 'EligeStatement':
                return genEligeStatement(node);
            case 'DiscerneStatement':
                return genDiscerneStatement(node);
            case 'CustodiStatement':
                return genCustodiStatement(node);
            case 'AdfirmaStatement':
                return genAdfirmaStatement(node);
            case 'ReddeStatement':
                return genReddeStatement(node);
            case 'RumpeStatement':
                return genRumpeStatement();
            case 'PergeStatement':
                return genPergeStatement();
            case 'IaceStatement':
                return genIaceStatement(node);
            case 'ScribeStatement':
                return genScribeStatement(node);
            case 'TemptaStatement':
                return genTemptaStatement(node);
            case 'BlockStatement':
                return genBlockStatement(node);
            case 'FacBlockStatement':
                return genFacBlockStatement(node);
            case 'ExpressionStatement':
                return genExpressionStatement(node);
            case 'ProbandumStatement':
                return genProbandumStatement(node);
            case 'ProbaStatement':
                return genProbaStatement(node);
            case 'CuraBlock':
                return genCuraBlock(node);
            case 'CuraStatement':
                return genCuraStatement(node);
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate import declaration.
     *
     * TRANSFORMS:
     *   ex "norma/tempus" importa nunc -> (no output, handled via intrinsics)
     *   ex "@hono/hono" importa Hono -> import { Hono } from "@hono/hono"
     *
     * WHY: norma/* imports are compiler-handled via intrinsics, not runtime imports.
     *      External packages pass through as native imports.
     */
    function genImportaDeclaration(node: ImportaDeclaration): string {
        const source = node.source;

        // Skip norma imports - these are handled via intrinsics
        if (source === 'norma' || source.startsWith('norma/')) {
            return '';
        }

        if (node.wildcard) {
            return `${ind()}import * as ${source} from "${source}"${semi ? ';' : ''}`;
        }

        const names = node.specifiers.map(s => s.name).join(', ');

        return `${ind()}import { ${names} } from "${source}"${semi ? ';' : ''}`;
    }

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   varia x: numerus = 5 -> let x: number = 5
     *   fixum y: textus = "hello" -> const y: string = "hello"
     *   fixum { nomen, aetas } = persona -> const { nomen, aetas } = persona
     *   figendum data = fetchData() -> const data = await fetchData()
     *   variandum result = fetch() -> let result = await fetch()
     *
     * WHY: Async bindings (figendum/variandum) imply await without explicit cede.
     *      The gerundive form ("that which will be fixed/varied") signals async intent.
     */
    function genVariaDeclaration(node: VariaDeclaration): string {
        // Map kind to JS keyword and determine if async
        const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
        const kind = node.kind === 'varia' || node.kind === 'variandum' ? 'let' : 'const';

        let name: string;

        if (node.name.type === 'ObjectPattern') {
            // Generate object destructuring pattern with rest support
            const props = node.name.properties.map(prop => {
                // Rest pattern: ceteri restName -> ...restName
                if (prop.rest) {
                    return `...${prop.value.name}`;
                }
                if (prop.key.name === prop.value.name) {
                    return prop.key.name;
                }

                return `${prop.key.name}: ${prop.value.name}`;
            });

            name = `{ ${props.join(', ')} }`;
        } else if (node.name.type === 'ArrayPattern') {
            // Generate array destructuring pattern with rest support
            // [a, b, ceteri rest] -> [a, b, ...rest]
            // [_, b, _] -> [, b, ]
            const elems = node.name.elements.map(elem => {
                if (elem.skip) {
                    return ''; // skip position (underscore becomes empty slot)
                }
                if (elem.rest) {
                    return `...${elem.name.name}`;
                }
                return elem.name.name;
            });

            name = `[${elems.join(', ')}]`;
        } else {
            name = node.name.name;
        }

        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';

        // Async bindings wrap initializer in await
        let init = '';
        if (node.init) {
            const expr = genExpression(node.init);
            init = isAsync ? ` = await ${expr}` : ` = ${expr}`;
        }

        return `${ind()}${kind} ${name}${typeAnno}${init}${semi ? ';' : ''}`;
    }

    /**
     * Generate genus (struct) declaration as TypeScript class.
     *
     * TRANSFORMS:
     *   genus persona { textus nomen: "X" numerus aetas: 0 }
     *   ->
     *   class persona {
     *       nomen: string = "X";
     *       aetas: number = 0;
     *       constructor(overrides: { nomen?: string, aetas?: number } = {}) {
     *           if (overrides.nomen !== undefined) { this.nomen = overrides.nomen; }
     *           if (overrides.aetas !== undefined) { this.aetas = overrides.aetas; }
     *           this.creo(); // if defined
     *       }
     *       private creo() { ... } // user's creo body, no args
     *   }
     *
     * WHY: Auto-merge design - field defaults + property overrides merged before creo runs.
     */
    function genGenusDeclaration(node: GenusDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
        const impl = node.implements ? ` implements ${node.implements.map(i => i.name).join(', ')}` : '';

        const lines: string[] = [];
        lines.push(`${ind()}class ${name}${typeParams}${impl} {`);

        depth++;

        const sections: string[][] = [];

        if (node.fields.length > 0) {
            sections.push(node.fields.map(genFieldDeclaration));
        }

        // Always generate constructor for auto-merge
        sections.push([genAutoMergeConstructor(node)]);

        // Emit user's creo as private no-args method (if defined)
        if (node.constructor) {
            sections.push([genCreoMethod(node.constructor)]);
        }

        if (node.methods.length > 0) {
            sections.push(node.methods.map(genMethodDeclaration));
        }

        sections.forEach((section, index) => {
            if (index > 0) {
                lines.push('');
            }
            lines.push(...section);
        });

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate auto-merge constructor.
     *
     * WHY: The constructor handles merging field defaults with { ... } or de expr overrides,
     *      then calls creo() if defined. User never writes merge boilerplate.
     */
    function genAutoMergeConstructor(node: GenusDeclaration): string {
        const lines: string[] = [];

        // Build overrides type: { fieldName?: fieldType, ... }
        const overrideProps = node.fields.map(f => {
            const fieldName = f.name.name;
            const fieldType = genType(f.fieldType);
            return `${fieldName}?: ${fieldType}`;
        });
        const overridesType = overrideProps.length > 0 ? `{ ${overrideProps.join(', ')} }` : 'Record<string, never>';

        lines.push(`${ind()}constructor(overrides: ${overridesType} = {}) {`);
        depth++;

        // Apply each override if provided
        for (const field of node.fields) {
            const fieldName = field.name.name;
            lines.push(`${ind()}if (overrides.${fieldName} !== undefined) { this.${fieldName} = overrides.${fieldName}${semi ? ';' : ''} }`);
        }

        // Call creo() if user defined it
        if (node.constructor) {
            lines.push(`${ind()}this.creo()${semi ? ';' : ''}`);
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate user's creo as a private no-args method.
     *
     * WHY: creo() is a post-initialization hook. By the time it runs,
     *      ego (this) already has merged field values. No args needed.
     */
    function genCreoMethod(node: FunctioDeclaration): string {
        const body = genBlockStatement(node.body);
        return `${ind()}private creo() ${body}`;
    }

    /**
     * Generate field declaration within a class.
     *
     * TRANSFORMS:
     *   textus nomen: "X" -> private nomen: string = "X"
     *   publicus numerus aetas: 0 -> aetas: number = 0
     *   nexum numerus count: 0 -> #count = 0; get count() { ... } set count(v) { ... }
     *
     * WHY: Reactive (nexum) fields emit getter/setter with invalidation hook.
     *      This allows libraries to track changes without special proxy magic.
     */
    function genFieldDeclaration(node: FieldDeclaration): string {
        const name = node.name.name;
        const type = genType(node.fieldType);

        // Reactive fields emit private backing field + getter/setter
        if (node.isReactive) {
            const init = node.init ? ` = ${genExpression(node.init)}` : '';
            const lines: string[] = [];

            // Private backing field with # prefix
            lines.push(`${ind()}#${name}${init}${semi ? ';' : ''}`);

            // Getter
            lines.push(`${ind()}get ${name}(): ${type} { return this.#${name}; }`);

            // Setter with invalidation
            lines.push(`${ind()}set ${name}(v: ${type}) { this.#${name} = v; this.__invalidate?.('${name}'); }`);

            return lines.join('\n');
        }

        // Regular fields - public by default (struct semantics), private with 'privatus'
        const visibility = node.isPrivate ? 'private ' : '';
        const staticMod = node.isStatic ? 'static ' : '';
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        return `${ind()}${visibility}${staticMod}${name}: ${type}${init}${semi ? ';' : ''}`;
    }

    /**
     * Generate method declaration within a class.
     */
    function genMethodDeclaration(node: FunctioDeclaration): string {
        const asyncMod = node.async ? 'async ' : '';
        const star = node.generator ? '*' : '';
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');

        // Wrap return type based on async/generator semantics
        let returnType = '';
        if (node.returnType) {
            let baseType = genType(node.returnType);
            if (node.async && node.generator) {
                baseType = `AsyncGenerator<${baseType}>`;
            } else if (node.generator) {
                baseType = `Generator<${baseType}>`;
            } else if (node.async) {
                baseType = `Promise<${baseType}>`;
            }
            returnType = `: ${baseType}`;
        }

        // Track generator context for cede -> yield vs await
        const prevInGenerator = inGenerator;
        inGenerator = node.generator;
        const body = genBlockStatement(node.body);
        inGenerator = prevInGenerator;

        return `${ind()}${asyncMod}${star}${name}(${params})${returnType} ${body}`;
    }

    /**
     * Generate pactum declaration as a TypeScript interface.
     */
    function genPactumDeclaration(node: PactumDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
        const lines: string[] = [];

        lines.push(`${ind()}interface ${name}${typeParams} {`);
        depth++;

        for (const method of node.methods) {
            lines.push(genPactumMethod(method));
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    function genPactumMethod(node: PactumMethod): string {
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        let returnType = node.returnType ? genType(node.returnType) : 'void';

        // Wrap return type based on async/generator semantics
        if (node.async && node.generator) {
            returnType = `AsyncGenerator<${returnType}>`;
        } else if (node.generator) {
            returnType = `Generator<${returnType}>`;
        } else if (node.async) {
            returnType = `Promise<${returnType}>`;
        }

        return `${ind()}${name}(${params}): ${returnType}${semi ? ';' : ''}`;
    }

    /**
     * Generate type alias declaration.
     *
     * TRANSFORMS:
     *   typus ID = textus -> type ID = string;
     *   typus UserID = numerus<32, Naturalis> -> type UserID = number;
     *   typus ConfigTypus = typus config -> type ConfigTypus = typeof config;
     *
     * WHY: TypeScript type aliases provide semantic naming without runtime cost.
     *      The `typus x` form extracts the type of a value at compile time.
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;

        // Check for typeof form: `typus X = typus y`
        if (node.typeofTarget) {
            const target = node.typeofTarget.name;
            return `${ind()}type ${name} = typeof ${target}${semi ? ';' : ''}`;
        }

        const typeAnno = genType(node.typeAnnotation);

        return `${ind()}type ${name} = ${typeAnno}${semi ? ';' : ''}`;
    }

    /**
     * Generate enum declaration.
     *
     * TRANSFORMS:
     *   ordo color { rubrum, viridis } -> enum color { rubrum, viridis }
     *   ordo status { pendens = 0 } -> enum status { pendens = 0 }
     *   ordo dir { north = "n" } -> enum dir { north = "n" }
     *
     * WHY: TypeScript enums map directly from Latin 'ordo'.
     */
    function genOrdoDeclaration(node: OrdoDeclaration): string {
        const name = node.name.name;

        const members = node.members.map(member => {
            const memberName = member.name.name;

            if (member.value !== undefined) {
                const value = typeof member.value.value === 'string' ? `"${member.value.value}"` : member.value.value;
                return `${memberName} = ${value}`;
            }

            return memberName;
        });

        return `${ind()}enum ${name} { ${members.join(', ')} }`;
    }

    /**
     * Generate discretio (tagged union) declaration.
     *
     * TRANSFORMS:
     *   discretio Event { Click { numerus x, numerus y }, Quit }
     *   -> type Event = { tag: 'Click'; x: number; y: number } | { tag: 'Quit' };
     *
     * WHY: TypeScript discriminated unions use a 'tag' property for narrowing.
     */
    function genDiscretioDeclaration(node: DiscretioDeclaration): string {
        const name = node.name.name;

        // Generate type parameters if present
        let typeParams = '';
        if (node.typeParameters && node.typeParameters.length > 0) {
            typeParams = '<' + node.typeParameters.map(p => p.name).join(', ') + '>';
        }

        // Generate variant types
        const variants = node.variants.map(variant => {
            const variantName = variant.name.name;

            if (variant.fields.length === 0) {
                // Unit variant: just the tag
                return `{ tag: '${variantName}' }`;
            }

            // Variant with fields: tag + field properties
            const fields = variant.fields.map(field => {
                const fieldName = field.name.name;
                const fieldType = genType(field.fieldType);
                return `${fieldName}: ${fieldType}`;
            });

            return `{ tag: '${variantName}'; ${fields.join('; ')} }`;
        });

        return `${ind()}type ${name}${typeParams} = ${variants.join(' | ')};`;
    }

    /**
     * Generate type parameter (type, literal, or modifier).
     *
     * TRANSFORMS:
     *   textus -> string (nested type)
     *   32 -> ignored (size constraint)
     *
     * WHY: TypeScript doesn't support numeric size constraints.
     *      These are semantic hints for other targets (Zig, Rust) but not TS.
     */
    function genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return genType(param);
        }

        if (param.type === 'Literal') {
            // EDGE: Numeric params like numerus<32> indicate size/width
            // For TypeScript, we ignore size - all numbers are float64
            return null;
        }

        return null;
    }

    /**
     * Generate type annotation from Latin type.
     *
     * TRANSFORMS:
     *   textus -> string
     *   lista<numerus> -> Array<number>
     *   textus? -> string | null
     *   numerus<32> -> number (size ignored)
     *   numerus<i32> -> number (size type ignored)
     */
    function genType(node: TypeAnnotation): string {
        // Map Latin type name to TS type
        const typeName = node.name;
        const base = typeMap[typeName] ?? node.name;

        // Track decimal.js dependency
        if (typeName === 'decimus') {
            features.decimal = true;
        }

        // Handle generic type parameters: lista<textus> -> Array<string>
        let result = base;

        if (node.typeParameters && node.typeParameters.length > 0) {
            // Filter out null params (literals and modifiers that TS doesn't support)
            const params = node.typeParameters.map(genTypeParameter).filter((p): p is string => p !== null);

            // Only add type params if we have any left after filtering
            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> string | null
        if (node.nullable) {
            result = `${result} | null`;
        }

        // Handle union types
        if (node.union && node.union.length > 0) {
            result = node.union.map(genType).join(' | ');
        }

        return result;
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(nomen: textus): nihil -> function salve(nomen: string): null
     *   futura functio f(): numerus -> async function f(): Promise<number>
     *   cursor functio f(): numerus -> function* f(): Generator<number>
     *   futura cursor functio f(): numerus -> async function* f(): AsyncGenerator<number>
     */
    function genFunctioDeclaration(node: FunctioDeclaration): string {
        const async = node.async ? 'async ' : '';
        const star = node.generator ? '*' : '';
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');

        // Generate type parameters: prae typus T -> <T>
        const typeParams = node.typeParams ? `<${node.typeParams.map(tp => tp.name.name).join(', ')}>` : '';

        // Wrap return type based on async/generator semantics
        let returnType = '';
        if (node.returnType) {
            let baseType = genType(node.returnType);
            if (node.async && node.generator) {
                baseType = `AsyncGenerator<${baseType}>`;
            } else if (node.generator) {
                baseType = `Generator<${baseType}>`;
            } else if (node.async) {
                baseType = `Promise<${baseType}>`;
            }
            returnType = `: ${baseType}`;
        }

        // Track generator context for cede -> yield vs await
        const prevInGenerator = inGenerator;
        inGenerator = node.generator;
        const body = genBlockStatement(node.body);
        inGenerator = prevInGenerator;

        return `${ind()}${async}function${star} ${name}${typeParams}(${params})${returnType} ${body}`;
    }

    /**
     * Generate function parameter.
     *
     * TRANSFORMS:
     *   nomen: textus -> nomen: string
     *   ceteri lista<numerus> nums -> ...nums: number[]
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';
        const prefix = node.rest ? '...' : '';

        return `${prefix}${name}${typeAnno}`;
    }

    /**
     * Generate if statement.
     *
     * TRANSFORMS:
     *   si (conditio) { ... } -> if (conditio) { ... }
     *   si (conditio) { ... } aliter { ... } -> if (conditio) { ... } else { ... }
     *
     * WHY: Latin if-statements can have optional catch clauses for exception handling.
     *      When present, we wrap the entire if in a try-catch block.
     */
    function genSiStatement(node: SiStatement): string {
        let result = '';

        // WHY: Latin allows 'capta' (catch) clause on if-statements for brevity
        if (node.catchClause) {
            result += `${ind()}try {\n`;
            depth++;
            result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;
        } else {
            result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;
        }

        if (node.alternate) {
            if (node.alternate.type === 'SiStatement') {
                result += ` else ${genSiStatement(node.alternate).trim()}`;
            } else {
                result += ` else ${genBlockStatement(node.alternate)}`;
            }
        }

        return result;
    }

    function genDumStatement(node: DumStatement): string {
        const test = genExpression(node.test);
        const body = genBlockStatement(node.body);

        if (node.catchClause) {
            let result = `${ind()}try {\n`;

            depth++;
            result += `${ind()}while (${test}) ${body}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

            return result;
        }

        return `${ind()}while (${test}) ${body}`;
    }

    /**
     * Generate for statement.
     *
     * TRANSFORMS:
     *   ex 0..10 pro i { } -> for (let i = 0; i < 10; i++) { }
     *   ex 0..10 per 2 pro i { } -> for (let i = 0; i < 10; i += 2) { }
     *   ex items pro item { } -> for (const item of items) { }
     *   ex items fit item { } -> for (const item of items) { }
     *   ex stream fiet chunk { } -> for await (const chunk of stream) { }
     *
     * WHY: Range expressions compile to efficient traditional for loops
     *      instead of allocating arrays. The 'fiet' verb form generates
     *      'for await' for async iteration.
     */
    function genIteratioStatement(node: IteratioStatement): string {
        const varName = node.variable.name;
        const body = genBlockStatement(node.body);
        const awaitKeyword = node.async ? ' await' : '';

        // Check if iterable is a range expression for efficient loop generation
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);
            const cmp = range.inclusive ? '<=' : '<';

            let forHeader: string;

            if (range.step) {
                const step = genExpression(range.step);

                // With step: need to handle positive/negative direction
                // For simplicity, assume positive step uses </<= based on inclusive
                forHeader = `for${awaitKeyword} (let ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName} += ${step})`;
            } else {
                // Default step of 1
                forHeader = `for${awaitKeyword} (let ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName}++)`;
            }

            if (node.catchClause) {
                let result = `${ind()}try {\n`;

                depth++;
                result += `${ind()}${forHeader} ${body}`;
                depth--;
                result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

                return result;
            }

            return `${ind()}${forHeader} ${body}`;
        }

        // Standard for-of/for-in loop
        const iterable = genExpression(node.iterable);
        const keyword = node.kind === 'in' ? 'in' : 'of';

        if (node.catchClause) {
            let result = `${ind()}try {\n`;

            depth++;
            result += `${ind()}for${awaitKeyword} (const ${varName} ${keyword} ${iterable}) ${body}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

            return result;
        }

        return `${ind()}for${awaitKeyword} (const ${varName} ${keyword} ${iterable}) ${body}`;
    }

    /**
     * Generate with statement (mutation block).
     *
     * TRANSFORMS:
     *   in user { nomen = "Marcus" } -> user.nomen = "Marcus";
     *
     * WHY: Bare identifier assignments inside 'in' blocks become property
     *      assignments on the context object.
     */
    function genInStatement(node: InStatement): string {
        const context = genExpression(node.object);
        const lines: string[] = [];

        for (const stmt of node.body.body) {
            if (
                stmt.type === 'ExpressionStatement' &&
                stmt.expression.type === 'AssignmentExpression' &&
                stmt.expression.left.type === 'Identifier'
            ) {
                const prop = stmt.expression.left.name;
                const value = genExpression(stmt.expression.right);
                const op = stmt.expression.operator;

                lines.push(`${ind()}${context}.${prop} ${op} ${value}${semi ? ';' : ''}`);
            } else {
                lines.push(genStatement(stmt));
            }
        }

        return lines.join('\n');
    }

    /**
     * Generate switch statement (value matching).
     *
     * TRANSFORMS:
     *   elige x { si 1 { a() } si 2 { b() } aliter { c() } }
     *   -> if (x === 1) { a(); } else if (x === 2) { b(); } else { c(); }
     *
     * WHY: Always emit if/else for all targets. Simpler codegen, no type
     *      detection needed, and downstream compilers optimize anyway.
     */
    function genEligeStatement(node: EligeStatement): string {
        const discriminant = genExpression(node.discriminant);
        let result = '';

        if (node.catchClause) {
            result += `${ind()}try {\n`;
            depth++;
        }

        // Generate if/else chain
        for (let i = 0; i < node.cases.length; i++) {
            const caseNode = node.cases[i]!;
            const keyword = i === 0 ? 'if' : 'else if';

            // Value matching: si expression { ... }
            const test = genExpression(caseNode.test);
            result += `${ind()}${keyword} (${discriminant} === ${test}) {\n`;
            depth++;

            for (const stmt of caseNode.consequent.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}`;

            // Add newline if more cases or default follows
            if (i < node.cases.length - 1 || node.defaultCase) {
                result += '\n';
            }
        }

        if (node.defaultCase) {
            // Add "else" for default case
            if (node.cases.length > 0) {
                result += `${ind()}else {\n`;
            } else {
                // No cases, just default — emit as bare block
                result += `${ind()}{\n`;
            }

            depth++;

            for (const stmt of node.defaultCase.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}`;
        }

        if (node.catchClause) {
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;
        }

        return result;
    }

    /**
     * Generate variant matching statement (for discretio types).
     *
     * TRANSFORMS:
     *   discerne event { si Click pro x, y { use(x, y) } si Quit { exit() } }
     *   -> if (event.tag === 'Click') { const { x, y } = event; use(x, y); }
     *      else if (event.tag === 'Quit') { exit(); }
     *
     * WHY: TypeScript discriminated unions use a 'tag' property for discrimination.
     */
    function genDiscerneStatement(node: DiscerneStatement): string {
        const discriminant = genExpression(node.discriminant);
        let result = '';

        // Generate if/else chain
        for (let i = 0; i < node.cases.length; i++) {
            const caseNode = node.cases[i]!;
            const keyword = i === 0 ? 'if' : 'else if';

            // Variant matching: si VariantName pro bindings { ... }
            const variantName = caseNode.variant.name;
            result += `${ind()}${keyword} (${discriminant}.tag === '${variantName}') {\n`;
            depth++;

            // Destructure bindings if any
            if (caseNode.bindings.length > 0) {
                const bindingNames = caseNode.bindings.map((b: Identifier) => b.name).join(', ');
                result += `${ind()}const { ${bindingNames} } = ${discriminant};\n`;
            }

            for (const stmt of caseNode.consequent.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}`;

            // Add newline if more cases follow
            if (i < node.cases.length - 1) {
                result += '\n';
            }
        }

        return result;
    }

    /**
     * Generate guard statement.
     *
     * TRANSFORMS:
     *   custodi { si x == nihil { redde } si y < 0 { iace "error" } }
     *   -> if (x == null) { return; } if (y < 0) { throw "error"; }
     *
     * WHY: Guard clauses are just sequential if statements with early exits.
     */
    function genCustodiStatement(node: CustodiStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            const body = genBlockStatement(clause.consequent);

            lines.push(`${ind()}if (${test}) ${body}`);
        }

        return lines.join('\n');
    }

    /**
     * Generate assert statement.
     *
     * TRANSFORMS:
     *   adfirma x > 0 -> if (!(x > 0)) throw new Error("Assertion failed: x > 0");
     *   adfirma x > 0, "custom" -> if (!(x > 0)) throw new Error("custom");
     *
     * WHY: Always-on runtime checks that throw on failure.
     *      Auto-generates message from expression if not provided.
     */
    function genAdfirmaStatement(node: AdfirmaStatement): string {
        const test = genExpression(node.test);

        let message: string;

        if (node.message) {
            message = genExpression(node.message);
        } else {
            // Auto-generate message from the test expression
            message = `"Assertion failed: ${test.replace(/"/g, '\\"')}"`;
        }

        return `${ind()}if (!(${test})) { throw new Error(${message}); }`;
    }

    function genReddeStatement(node: ReddeStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)}${semi ? ';' : ''}`;
        }

        return `${ind()}return${semi ? ';' : ''}`;
    }

    /**
     * Generate break statement.
     *
     * TRANSFORMS:
     *   rumpe -> break
     */
    function genRumpeStatement(): string {
        return `${ind()}break${semi ? ';' : ''}`;
    }

    /**
     * Generate continue statement.
     *
     * TRANSFORMS:
     *   perge -> continue
     */
    function genPergeStatement(): string {
        return `${ind()}continue${semi ? ';' : ''}`;
    }

    /**
     * Generate throw/panic statement.
     *
     * TRANSFORMS:
     *   iace "message" -> throw "message"
     *   iace error     -> throw error
     *   mori "message" -> throw new Panic("message")
     *   mori error     -> throw new Panic(String(error))
     *
     * WHY: mori indicates unrecoverable errors (like Rust's panic! or Zig's @panic).
     *      Using a dedicated Panic class allows catching panics separately from
     *      regular errors if needed, and makes stack traces clearer.
     */
    function genIaceStatement(node: IaceStatement): string {
        const expr = genExpression(node.argument);

        if (node.fatal) {
            // Track that we need the Panic class in preamble
            features.panic = true;

            // mori (panic) - wrap in Panic class
            if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
                return `${ind()}throw new Panic(${JSON.stringify(node.argument.value)})${semi ? ';' : ''}`;
            }
            // Other expressions: convert to string
            return `${ind()}throw new Panic(String(${expr}))${semi ? ';' : ''}`;
        }

        return `${ind()}throw ${expr}${semi ? ';' : ''}`;
    }

    function genScribeStatement(node: ScribeStatement): string {
        const args = node.arguments.map(genExpression).join(', ');
        const method = node.level === 'debug' ? 'debug' : node.level === 'warn' ? 'warn' : 'log';
        return `${ind()}console.${method}(${args})${semi ? ';' : ''}`;
    }

    function genTemptaStatement(node: TemptaStatement): string {
        let result = `${ind()}try ${genBlockStatement(node.block)}`;

        if (node.handler) {
            result += ` catch (${node.handler.param.name}) ${genBlockStatement(node.handler.body)}`;
        }

        if (node.finalizer) {
            result += ` finally ${genBlockStatement(node.finalizer)}`;
        }

        return result;
    }

    /**
     * Generate fac block statement (explicit scope block).
     *
     * TRANSFORMS:
     *   fac { x() } -> { x(); }
     *   fac { x() } cape e { y() } -> try { x(); } catch (e) { y(); }
     *
     * WHY: fac alone is just a scope block. With cape, it becomes try-catch.
     */
    function genFacBlockStatement(node: FacBlockStatement): string {
        if (node.catchClause) {
            // With cape, emit as try-catch
            let result = `${ind()}try ${genBlockStatement(node.body)}`;
            result += ` catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;
            return result;
        }

        // Without cape, just emit the block
        return `${ind()}${genBlockStatement(node.body)}`;
    }

    // ---------------------------------------------------------------------------
    // Test Declarations (Proba)
    // ---------------------------------------------------------------------------

    /**
     * Generate test suite declaration.
     *
     * TRANSFORMS:
     *   probandum "Tokenizer" {
     *       ante { lexer = init() }
     *       proba "parses numbers" { ... }
     *   }
     *   ->
     *   describe("Tokenizer", () => {
     *       beforeEach(() => { lexer = init(); });
     *       test("parses numbers", () => { ... });
     *   });
     *
     * WHY: Maps to Bun/Jest/Vitest describe() for test organization.
     *      Setup/teardown blocks map to beforeEach/afterEach or beforeAll/afterAll.
     */
    function genProbandumStatement(node: ProbandumStatement): string {
        const lines: string[] = [];
        lines.push(`${ind()}describe("${node.name}", () => {`);

        depth++;

        for (const member of node.body) {
            switch (member.type) {
                case 'ProbandumStatement':
                    lines.push(genProbandumStatement(member));
                    break;
                case 'ProbaStatement':
                    lines.push(genProbaStatement(member));
                    break;
                case 'CuraBlock':
                    lines.push(genCuraBlock(member));
                    break;
            }
        }

        depth--;
        lines.push(`${ind()}})${semi ? ';' : ''}`);

        return lines.join('\n');
    }

    /**
     * Generate individual test case.
     *
     * TRANSFORMS:
     *   proba "parses integers" { adfirma parse("42") est 42 }
     *   -> test("parses integers", () => { ... });
     *
     *   proba omitte "blocked by #42" { ... }
     *   -> test.skip("blocked by #42", () => { ... });
     *
     *   proba futurum "needs feature" { ... }
     *   -> test.todo("needs feature");
     *
     * WHY: Maps to test()/test.skip()/test.todo() for test runners.
     */
    function genProbaStatement(node: ProbaStatement): string {
        if (node.modifier === 'omitte') {
            // Skip: test.skip("reason: name", () => { ... })
            const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
            const body = genBlockStatement(node.body);
            return `${ind()}test.skip("${reason}${node.name}", () => ${body})${semi ? ';' : ''}`;
        }

        if (node.modifier === 'futurum') {
            // Todo: test.todo("reason: name")
            const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
            return `${ind()}test.todo("${reason}${node.name}")${semi ? ';' : ''}`;
        }

        // Regular test
        const body = genBlockStatement(node.body);
        return `${ind()}test("${node.name}", () => ${body})${semi ? ';' : ''}`;
    }

    /**
     * Generate cura block (test setup/teardown).
     *
     * TRANSFORMS:
     *   cura ante { lexer = init() }
     *   -> beforeEach(() => { lexer = init(); });
     *
     *   cura ante omnia { db = connect() }
     *   -> beforeAll(() => { db = connect(); });
     *
     *   cura post { cleanup() }
     *   -> afterEach(() => { cleanup(); });
     *
     *   cura post omnia { db.close() }
     *   -> afterAll(() => { db.close(); });
     *
     * WHY: Maps timing (ante/post) and scope (omnia) to JS test hooks.
     */
    function genCuraBlock(node: CuraBlock): string {
        let hook: string;
        if (node.timing === 'ante') {
            hook = node.omnia ? 'beforeAll' : 'beforeEach';
        } else {
            hook = node.omnia ? 'afterAll' : 'afterEach';
        }
        const body = genBlockStatement(node.body);
        return `${ind()}${hook}(() => ${body})${semi ? ';' : ''}`;
    }

    /**
     * Generate cura statement (resource management).
     *
     * TRANSFORMS:
     *   cura aperi("file.txt") fit fd { lege(fd) }
     *   -> {
     *        const fd = aperi("file.txt");
     *        try {
     *          lege(fd);
     *        } finally {
     *          fd.solve?.();
     *        }
     *      }
     *
     *   cura cede connect(url) fit conn { ... }
     *   -> { const conn = await connect(url); try { ... } finally { conn.solve?.(); } }
     *
     *   cura resource() fit r { ... } cape err { ... }
     *   -> { const r = resource(); try { ... } catch (err) { ... } finally { r.solve?.(); } }
     *
     * WHY: Wraps in try/finally to guarantee cleanup via solve().
     *      Uses optional chaining (?.) so it works even if object doesn't implement curator.
     */
    function genCuraStatement(node: CuraStatement): string {
        const lines: string[] = [];
        const binding = node.binding.name;
        const resource = genExpression(node.resource);
        const awaitPrefix = node.async ? 'await ' : '';

        // Opening block scope
        lines.push(`${ind()}{`);
        depth++;

        // Resource acquisition: const <binding> = [await] <resource>;
        lines.push(`${ind()}const ${binding} = ${awaitPrefix}${resource}${semi ? ';' : ''}`);

        // Try block
        lines.push(`${ind()}try ${genBlockStatement(node.body)}`);

        // Optional catch clause
        if (node.catchClause) {
            const catchParam = node.catchClause.param.name;
            const catchBody = genBlockStatement(node.catchClause.body);
            lines.push(`${ind()}catch (${catchParam}) ${catchBody}`);
        }

        // Finally block with solve?.()
        lines.push(`${ind()}finally {`);
        depth++;
        lines.push(`${ind()}${binding}.solve?.()${semi ? ';' : ''}`);
        depth--;
        lines.push(`${ind()}}`);

        // Close block scope
        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    function genBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return '{}';
        }

        depth++;
        const body = node.body.map(genStatement).join('\n');

        depth--;

        return `{\n${body}\n${ind()}}`;
    }

    function genExpressionStatement(node: ExpressionStatement): string {
        return `${ind()}${genExpression(node.expression)}${semi ? ';' : ''}`;
    }

    // ---------------------------------------------------------------------------
    // Expressions
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any expression type.
     * WHY: Exhaustive switch ensures all expression types are handled.
     */
    function genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier': {
                // Check for constant intrinsics (norma/tempus duration constants, etc.)
                const constant = TS_CONSTANTS[node.name];
                if (constant) {
                    return constant;
                }
                return node.name;
            }
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                return `\`${node.raw}\``;
            case 'EgoExpression':
                return 'this';
            case 'ArrayExpression':
                return genArrayExpression(node);
            case 'BinaryExpression':
                return genBinaryExpression(node);
            case 'UnaryExpression':
                return genUnaryExpression(node);
            case 'CallExpression':
                return genCallExpression(node);
            case 'MemberExpression':
                return genMemberExpression(node);
            case 'ArrowFunctionExpression':
                return genArrowFunction(node);
            case 'AssignmentExpression':
                return genAssignmentExpression(node);
            case 'CedeExpression':
                // WHY: cede maps to yield in generators, await in async functions
                return `${inGenerator ? 'yield' : 'await'} ${genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node);
            case 'ConditionalExpression':
                return `${genExpression(node.test)} ? ${genExpression(node.consequent)} : ${genExpression(node.alternate)}`;
            case 'RangeExpression':
                return genRangeExpression(node);
            case 'ObjectExpression':
                return genObjectExpression(node);
            case 'LambdaExpression':
                return genLambdaExpression(node);
            case 'QuaExpression':
                return genQuaExpression(node);
            case 'EstExpression':
                return genEstExpression(node);
            case 'PraefixumExpression':
                return genPraefixumExpression(node);
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    /**
     * Generate literal value.
     *
     * TRANSFORMS:
     *   "hello" -> "hello" (JSON-escaped)
     *   42 -> 42
     *   verum -> true
     *   nihil -> null
     *
     * WHY: JSON.stringify ensures proper escaping of string literals.
     */
    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return 'null';
        }

        if (typeof node.value === 'string') {
            return JSON.stringify(node.value);
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'true' : 'false';
        }

        if (typeof node.value === 'bigint') {
            // WHY: Use raw to preserve hex format, raw already includes 'n' suffix
            return node.raw;
        }

        // WHY: Use raw to preserve original format (hex: 0xFF, decimal: 123)
        if (typeof node.value === 'number') {
            return node.raw;
        }

        return String(node.value);
    }

    /**
     * Generate array literal.
     *
     * TRANSFORMS:
     *   [] -> []
     *   [1, 2, 3] -> [1, 2, 3]
     *   [[1], [2]] -> [[1], [2]]
     *   [sparge a, sparge b] -> [...a, ...b]
     */
    function genArrayExpression(node: ArrayExpression): string {
        const elements = node.elements
            .map(el => {
                if (el.type === 'SpreadElement') {
                    return `...${genExpression(el.argument)}`;
                }
                return genExpression(el);
            })
            .join(', ');

        return `[${elements}]`;
    }

    /**
     * Generate object literal.
     *
     * TRANSFORMS:
     *   { nomen: "Marcus" } -> { nomen: "Marcus" }
     *   { nomen: x, aetas: y } -> { nomen: x, aetas: y }
     *   { sparge defaults, x: 1 } -> { ...defaults, x: 1 }
     */
    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '{}';
        }

        const props = node.properties.map(prop => {
            if (prop.type === 'SpreadElement') {
                return `...${genExpression(prop.argument)}`;
            }
            const key = prop.key.type === 'Identifier' ? prop.key.name : genLiteral(prop.key);
            const value = genExpression(prop.value);

            return `${key}: ${value}`;
        });

        return `{ ${props.join(', ')} }`;
    }

    /**
     * Generate range expression as array.
     *
     * TRANSFORMS:
     *   0..5 -> Array.from({length: 5}, (_, i) => i)           // exclusive
     *   0 usque 5 -> Array.from({length: 6}, (_, i) => i)      // inclusive
     *   0..10 per 2 -> Array.from({length: 5}, (_, i) => i * 2)
     *
     * WHY: When used outside a for-loop, ranges become arrays.
     *      End is exclusive by default, inclusive with 'usque'.
     *      In for-loops, they compile to efficient traditional loops instead.
     */
    function genRangeExpression(node: RangeExpression): string {
        const start = genExpression(node.start);
        const end = genExpression(node.end);
        const adj = node.inclusive ? ' + 1' : '';

        if (node.step) {
            const step = genExpression(node.step);

            // With step: more complex calculation
            // WHY: inclusive adds 1 to length calculation
            if (node.inclusive) {
                return `Array.from({length: Math.floor((${end} - ${start}) / ${step}) + 1}, (_, i) => ${start} + i * ${step})`;
            }

            return `Array.from({length: Math.ceil((${end} - ${start}) / ${step})}, (_, i) => ${start} + i * ${step})`;
        }

        // Simple range: adjust length for inclusive
        return `Array.from({length: ${end} - ${start}${adj}}, (_, i) => ${start} + i)`;
    }

    /**
     * Generate binary expression.
     *
     * TRANSFORMS:
     *   x + y -> (x + y)
     *   a && b -> (a && b)
     *
     * WHY: Parentheses ensure correct precedence in all contexts.
     */
    function genBinaryExpression(node: BinaryExpression): string {
        const left = genExpression(node.left);
        const right = genExpression(node.right);

        return `(${left} ${node.operator} ${right})`;
    }

    /**
     * Generate unary expression.
     *
     * TRANSFORMS:
     *   !x -> !x (prefix)
     *   x++ -> x++ (postfix)
     *   nulla x -> inline empty check
     *   nonnulla x -> inline non-empty check
     *   nihil x -> (x === null)
     *   nonnihil x -> (x !== null)
     */
    function genUnaryExpression(node: UnaryExpression): string {
        const arg = genExpression(node.argument);

        // nulla: check if null/empty
        if (node.operator === 'nulla') {
            return `(${arg} == null || (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length === 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length === 0 : !${arg}))`;
        }

        // nonnulla: check if non-null and has content
        if (node.operator === 'nonnulla') {
            return `(${arg} != null && (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length > 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length > 0 : Boolean(${arg})))`;
        }

        // nihil: check if null
        if (node.operator === 'nihil') {
            return `(${arg} === null)`;
        }

        // nonnihil: check if not null
        if (node.operator === 'nonnihil') {
            return `(${arg} !== null)`;
        }

        // negativum: check if less than zero
        if (node.operator === 'negativum') {
            return `(${arg} < 0)`;
        }

        // positivum: check if greater than zero
        if (node.operator === 'positivum') {
            return `(${arg} > 0)`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate type cast expression.
     *
     * TRANSFORMS:
     *   x qua textus -> (x as string)
     *   response.body qua objectum -> (response.body as object)
     *
     * WHY: TypeScript uses 'as' for type assertions. Parentheses ensure
     *      correct precedence when the cast appears in larger expressions.
     */
    function genQuaExpression(node: QuaExpression): string {
        const expr = genExpression(node.expression);
        const targetType = genType(node.targetType);

        return `(${expr} as ${targetType})`;
    }

    /**
     * Primitive types that use typeof for runtime checks.
     *
     * WHY: These Latin type names map to JavaScript typeof strings.
     *      Other types (user-defined, collections) use instanceof.
     */
    const TYPEOF_PRIMITIVES: Record<string, string> = {
        textus: 'string',
        numerus: 'number',
        fractus: 'number',
        bivalens: 'boolean',
        functio: 'function',
        signum: 'symbol',
        magnus: 'bigint',
        incertum: 'undefined',
        objectum: 'object',
    };

    /**
     * Generate type check expression (est/non est with type).
     *
     * TRANSFORMS:
     *   x est textus      -> (typeof x === "string")
     *   x est numerus     -> (typeof x === "number")
     *   x est persona     -> (x instanceof persona)
     *   x non est textus  -> (typeof x !== "string")
     *   x non est persona -> !(x instanceof persona)
     *
     * WHY: JavaScript distinguishes primitive type checks (typeof) from
     *      class/constructor checks (instanceof). Latin 'est' unifies these
     *      semantically; codegen chooses the right runtime mechanism.
     *
     * NOTE: For null checks, use `nihil x` or `nonnihil x` unary operators.
     */
    function genEstExpression(node: EstExpression): string {
        const expr = genExpression(node.expression);
        const typeName = node.targetType.name;
        const op = node.negated ? '!==' : '===';

        // Check if it's a primitive type (uses typeof)
        const jsType = TYPEOF_PRIMITIVES[typeName];
        if (jsType) {
            return `(typeof ${expr} ${op} "${jsType}")`;
        }

        // User-defined type or collection (uses instanceof)
        const targetType = genType(node.targetType);
        if (node.negated) {
            return `!(${expr} instanceof ${targetType})`;
        }
        return `(${expr} instanceof ${targetType})`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   scribe("hello") -> console.log("hello")
     *   _pavimentum(x) -> Math.floor(x)
     *   foo(x, y) -> foo(x, y)
     *   lista.adde(x) -> lista.push(x)
     *   lista.filtrata(fn) -> lista.filter(fn)
     *   f(sparge nums) -> f(...nums)
     *
     * Intrinsics are mapped to target-specific implementations.
     * Lista methods (Latin array methods) are translated to JS equivalents.
     */
    function genCallExpression(node: CallExpression): string {
        const args = node.arguments
            .map(arg => {
                if (arg.type === 'SpreadElement') {
                    return `...${genExpression(arg.argument)}`;
                }
                return genExpression(arg);
            })
            .join(', ');

        // Check for intrinsics (bare function calls)
        if (node.callee.type === 'Identifier') {
            const name = node.callee.name;
            const intrinsic = TS_INTRINSICS[name];

            if (intrinsic) {
                return intrinsic(args);
            }
        }

        // Check for collection methods (method calls on lista/tabula/copia)
        if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
            const methodName = (node.callee.property as Identifier).name;
            const obj = genExpression(node.callee.object);

            // WHY: Use semantic type info to dispatch to correct collection registry.
            // This prevents method name collisions (e.g., accipe means different
            // things for lista vs tabula).
            const objType = node.callee.object.resolvedType;
            const collectionName = objType?.kind === 'generic' ? objType.name : null;

            // Dispatch based on resolved type
            if (collectionName === 'tabula') {
                const method = getTabulaMethod(methodName);
                if (method) {
                    if (typeof method.ts === 'function') {
                        return method.ts(obj, args);
                    }
                    return `${obj}.${method.ts}(${args})`;
                }
            } else if (collectionName === 'copia') {
                const method = getCopiaMethod(methodName);
                if (method) {
                    if (typeof method.ts === 'function') {
                        return method.ts(obj, args);
                    }
                    return `${obj}.${method.ts}(${args})`;
                }
            } else if (collectionName === 'lista') {
                const method = getListaMethod(methodName);
                if (method) {
                    if (typeof method.ts === 'function') {
                        return method.ts(obj, args);
                    }
                    return `${obj}.${method.ts}(${args})`;
                }
            }

            // Fallback: no type info or unknown type - try lista (most common)
            const listaMethod = getListaMethod(methodName);
            if (listaMethod) {
                if (typeof listaMethod.ts === 'function') {
                    return listaMethod.ts(obj, args);
                }
                return `${obj}.${listaMethod.ts}(${args})`;
            }
        }

        const callee = genExpression(node.callee);

        // Handle optional call: callback?() -> callback?.()
        if (node.optional) {
            return `${callee}?.(${args})`;
        }

        // Handle non-null assertion call: handler!() -> handler!()
        if (node.nonNull) {
            return `${callee}!(${args})`;
        }

        return `${callee}(${args})`;
    }

    /**
     * TypeScript intrinsic mappings.
     *
     * Maps Latin intrinsic names to TypeScript/JavaScript equivalents.
     *
     * Two categories:
     * - _prefixed: Internal intrinsics called from arca/norma/index.fab
     * - unprefixed: Direct stdlib functions (norma/tempus, etc.)
     */
    const TS_INTRINSICS: Record<string, (args: string) => string> = {
        // I/O (internal intrinsics used by arca/norma/index.fab)
        _scribe: args => `console.log(${args})`,
        _vide: args => `console.debug(${args})`,
        _mone: args => `console.warn(${args})`,
        _lege: () => `prompt() ?? ""`,

        // Math (internal intrinsics used by arca/norma/index.fab)
        _fortuitus: () => `Math.random()`,
        _pavimentum: args => `Math.floor(${args})`,
        _tectum: args => `Math.ceil(${args})`,
        _radix: args => `Math.sqrt(${args})`,
        _potentia: args => `Math.pow(${args})`,

        // norma/tempus - Time functions
        nunc: () => `Date.now()`,
        nunc_nano: () => `BigInt(Date.now()) * 1000000n`,
        nunc_secunda: () => `Math.floor(Date.now() / 1000)`,
        dormi: args => `new Promise(r => setTimeout(r, ${args}))`,
    };

    /**
     * TypeScript constant intrinsics.
     *
     * Maps Latin constant names to literal values.
     * Used for identifier references (not function calls).
     */
    const TS_CONSTANTS: Record<string, string> = {
        // norma/tempus - Duration constants (milliseconds)
        MILLISECUNDUM: '1',
        SECUNDUM: '1000',
        MINUTUM: '60000',
        HORA: '3600000',
        DIES: '86400000',
    };

    /**
     * Generate member access.
     *
     * TRANSFORMS:
     *   obj.prop      -> obj.prop
     *   obj[key]      -> obj[key]
     *   obj?.prop     -> obj?.prop
     *   obj?[key]     -> obj?.[key]   (TS requires dot before bracket)
     *   obj!.prop     -> obj!.prop
     *   obj![key]     -> obj![key]
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);

        if (node.computed) {
            // Check for slice syntax: arr[1..3] or arr[1 usque 3]
            if (node.property.type === 'RangeExpression') {
                return genSliceExpression(obj, node.property, node.optional);
            }

            // Check for negative index: arr[-1]
            if (isNegativeIndex(node.property)) {
                const idx = genExpression(node.property);
                if (node.optional) {
                    return `${obj}?.at(${idx})`;
                }
                return `${obj}.at(${idx})`;
            }

            const prop = genExpression(node.property);

            // WHY: TypeScript requires ?. before [ for optional computed access
            if (node.optional) {
                return `${obj}?.[${prop}]`;
            }
            if (node.nonNull) {
                return `${obj}![${prop}]`;
            }
            return `${obj}[${prop}]`;
        }

        // WHY: Non-computed access always has Identifier property by grammar
        const prop = (node.property as Identifier).name;

        if (node.optional) {
            return `${obj}?.${prop}`;
        }
        if (node.nonNull) {
            return `${obj}!.${prop}`;
        }
        return `${obj}.${prop}`;
    }

    /**
     * Check if an expression is a negative numeric literal.
     *
     * WHY: JavaScript arrays don't support negative indices natively.
     *      We detect negative literals to emit .at() instead of bracket access.
     */
    function isNegativeIndex(expr: Expression): boolean {
        // Direct negative literal: -1
        if (expr.type === 'UnaryExpression' && expr.operator === '-' && expr.argument.type === 'Literal') {
            return typeof expr.argument.value === 'number';
        }
        // Negative literal already parsed as negative number (rare)
        if (expr.type === 'Literal' && typeof expr.value === 'number' && expr.value < 0) {
            return true;
        }
        return false;
    }

    /**
     * Generate slice expression from range inside brackets.
     *
     * TRANSFORMS:
     *   arr[1..3]       -> arr.slice(1, 3)
     *   arr[1 usque 3]  -> arr.slice(1, 4)  // inclusive adds 1 to end
     *   arr[1..]        -> arr.slice(1)     // to end (if end is omitted)
     *   arr[..3]        -> arr.slice(0, 3)  // from start (if start is omitted)
     *   arr[-3..-1]     -> arr.slice(-3, -1)
     *   arr[-3 usque -1] -> arr.slice(-3)   // inclusive of -1 means to end
     *
     * WHY: JavaScript .slice() is always exclusive of end index, so inclusive
     *      ranges need adjustment. Negative indices work natively in slice().
     */
    function genSliceExpression(obj: string, range: RangeExpression, optional?: boolean): string {
        const start = genExpression(range.start);
        const end = genExpression(range.end);
        const optChain = optional ? '?' : '';

        // For inclusive ranges (usque), we need to add 1 to the end
        // unless end is negative (then we'd need different handling)
        if (range.inclusive) {
            // Check if end is a literal number for simple +1
            if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
                const inclusiveEnd = range.end.value + 1;
                // If inclusive end is 0, it means "to the end"
                if (inclusiveEnd === 0) {
                    return `${obj}${optChain}.slice(${start})`;
                }
                return `${obj}${optChain}.slice(${start}, ${inclusiveEnd})`;
            }
            // Check for negative literal in unary expression
            if (
                range.end.type === 'UnaryExpression' &&
                range.end.operator === '-' &&
                range.end.argument.type === 'Literal' &&
                typeof range.end.argument.value === 'number'
            ) {
                const negVal = -range.end.argument.value;
                const inclusiveEnd = negVal + 1;
                if (inclusiveEnd === 0) {
                    return `${obj}${optChain}.slice(${start})`;
                }
                return `${obj}${optChain}.slice(${start}, ${inclusiveEnd})`;
            }
            // Dynamic end: need runtime +1
            return `${obj}${optChain}.slice(${start}, ${end} + 1)`;
        }

        // Exclusive range (.. or ante): direct slice
        return `${obj}${optChain}.slice(${start}, ${end})`;
    }

    /**
     * Generate arrow function.
     *
     * TRANSFORMS:
     *   (x) => x + 1 -> (x) => x + 1
     *   (x) => { redde x + 1; } -> (x) => { return x + 1; }
     */
    function genArrowFunction(node: ArrowFunctionExpression): string {
        const params = node.params.map(genParameter).join(', ');

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `(${params}) => ${body}`;
        }

        const body = genExpression(node.body as Expression);

        return `(${params}) => ${body}`;
    }

    /**
     * Generate lambda expression (pro ... redde or pro ... { }).
     *
     * TRANSFORMS:
     *   pro x redde x * 2 -> (x) => x * 2
     *   pro x, y redde x + y -> (x, y) => x + y
     *   pro redde 42 -> () => 42
     *   pro x { redde x * 2 } -> (x) => { return x * 2; }
     *   pro { scribe "hi" } -> () => { console.log("hi"); }
     *
     * WHY: Latin pro (for) + redde (return) creates arrow functions.
     */
    function genLambdaExpression(node: LambdaExpression): string {
        const params = node.params.map(p => p.name).join(', ');
        const asyncPrefix = node.async ? 'async ' : '';

        // Add return type annotation if present
        const returnTypeAnno = node.returnType ? `: ${genType(node.returnType)}` : '';

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);
            return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
        }

        const body = genExpression(node.body);
        return `${asyncPrefix}(${params})${returnTypeAnno} => ${body}`;
    }

    function genAssignmentExpression(node: AssignmentExpression): string {
        const left = node.left.type === 'Identifier' ? node.left.name : genExpression(node.left);

        return `${left} ${node.operator} ${genExpression(node.right)}`;
    }

    /**
     * Generate compile-time evaluation expression as runtime IIFE.
     *
     * TRANSFORMS:
     *   praefixum(expr) -> (expr)
     *   praefixum { ... } -> (() => { ... })()
     *
     * TARGET: TypeScript lacks compile-time evaluation. We emit an IIFE
     *         so the code compiles and runs, even though it won't have
     *         true compile-time semantics.
     *
     * WHY: Rather than crashing the build, we degrade gracefully.
     *      The code still works correctly at runtime—it just doesn't
     *      benefit from compile-time optimization.
     */
    function genPraefixumExpression(node: PraefixumExpression): string {
        if (node.body.type === 'BlockStatement') {
            // Block form: wrap in IIFE
            const body = genBlockStatement(node.body);
            return `(() => ${body})()`;
        }

        // Expression form: just parenthesize
        return `(${genExpression(node.body)})`;
    }

    function genNovumExpression(node: NovumExpression): string {
        const callee = node.callee.name;
        const args: string[] = node.arguments.filter((arg): arg is Expression => arg.type !== 'SpreadElement').map(genExpression);

        if (node.withExpression) {
            args.push(genObjectExpression(node.withExpression as ObjectExpression));
        }

        const argsText = args.join(', ');

        return `new ${callee}(${argsText})`;
    }

    return genProgram(program);
}
