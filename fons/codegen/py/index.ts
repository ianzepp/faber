/**
 * Python Code Generator - Emit Python 3.10+ source code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Python source code.
 * Python uses significant whitespace (indentation) rather than braces,
 * which requires careful depth tracking during code generation.
 *
 * Key transformations:
 * - varia/fixum -> assignment (Python has no const)
 * - functio -> def
 * - futura functio -> async def
 * - genus -> class with dataclass-like pattern
 * - pactum -> typing.Protocol
 * - si/aliter -> if/elif/else
 * - elige -> match/case (Python 3.10+)
 * - ex...pro -> for...in
 * - scribe -> print()
 * - ego -> self
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid Python 3.10+ source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * TARGET DIFFERENCES
 * ==================
 * Python characteristics:
 * - Indentation-based blocks (no braces)
 * - Dynamic typing (type hints are optional annotations)
 * - No `new` keyword (classes called directly)
 * - `self` explicit in method signatures
 * - Pattern matching via match/case (3.10+)
 * - Union types via X | Y syntax (3.10+)
 * - Async via asyncio module
 * - No true constants (convention: UPPER_CASE)
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Python 3.10+
 * INV-2: All Latin type names are mapped to Python type hints
 * INV-3: Indentation is consistently 4 spaces per level
 * INV-4: Generated code follows PEP 8 style guidelines where practical
 */

import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    TypeAliasDeclaration,
    EnumDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    GuardStatement,
    AssertStatement,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    BlockStatement,
    ThrowStatement,
    ScribeStatement,
    TryStatement,
    FacBlockStatement,
    ExpressionStatement,
    ArrayExpression,
    ObjectExpression,
    RangeExpression,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrowFunctionExpression,
    LambdaExpression,
    AssignmentExpression,
    NewExpression,
    Identifier,
    Literal,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    TypeCastExpression,
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
 * Map Latin type names to Python type hints.
 *
 * TARGET MAPPING:
 * | Latin      | Python              |
 * |------------|---------------------|
 * | textus     | str                 |
 * | numerus    | int                 |
 * | fractus    | float               |
 * | decimus    | Decimal             |
 * | bivalens   | bool                |
 * | nihil      | None                |
 * | vacuum     | None                |
 * | octeti     | bytes               |
 * | objectum   | object              |
 * | lista      | list                |
 * | tabula     | dict                |
 * | copia      | set                 |
 * | promissum  | Awaitable           |
 * | erratum    | Exception           |
 * | cursor     | Iterator            |
 *
 * WHY: Python uses `float` for IEEE 754 doubles (same as JS number).
 *      `Decimal` requires `from decimal import Decimal` for arbitrary precision.
 *      `bytes` is Python's native byte array type.
 *      `object` is the base type for all Python objects.
 */
const typeMap: Record<string, string> = {
    textus: 'str',
    numerus: 'int',
    fractus: 'float',
    decimus: 'Decimal',
    magnus: 'int', // WHY: Python ints are arbitrary precision
    bivalens: 'bool',
    nihil: 'None',
    vacuum: 'None',
    numquam: 'NoReturn', // WHY: typing.NoReturn for functions that never return
    octeti: 'bytes',
    objectum: 'object',
    object: 'object',
    lista: 'list',
    tabula: 'dict',
    copia: 'set',
    promissum: 'Awaitable',
    erratum: 'Exception',
    cursor: 'Iterator',
    ignotum: 'Any', // WHY: Python has no unknown, Any is closest
};

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate Python source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Python 3.10+ source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns Python source code
 */
export function generatePy(program: Program, options: CodegenOptions = {}): string {
    // WHY: 4 spaces is PEP 8 standard
    const indent = options.indent ?? '    ';

    // Track indentation depth for Python's significant whitespace
    let depth = 0;

    // Track if we're inside a generator function (for cede -> yield vs await)
    let inGenerator = false;

    // Track which features are used (for preamble generation)
    const features = createRequiredFeatures();

    /**
     * Generate indentation for current depth.
     */
    function ind(): string {
        return indent.repeat(depth);
    }

    /**
     * Generate preamble based on features used.
     *
     * WHY: Python requires explicit imports for standard library features.
     *      Only emit imports for features actually used in the program.
     */
    function genPreamble(): string {
        const imports: string[] = [];

        if (features.enum) {
            imports.push('from enum import Enum, auto');
        }

        if (features.decimal) {
            imports.push('from decimal import Decimal');
        }

        // functools needed for reduce
        // TODO: Track when reducta is used

        return imports.length > 0 ? imports.join('\n') + '\n\n' : '';
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

    function genStatement(node: Statement): string {
        switch (node.type) {
            case 'ImportDeclaration':
                return genImportDeclaration(node);
            case 'VariableDeclaration':
                return genVariableDeclaration(node);
            case 'FunctionDeclaration':
                return genFunctionDeclaration(node);
            case 'GenusDeclaration':
                return genGenusDeclaration(node);
            case 'PactumDeclaration':
                return genPactumDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'EnumDeclaration':
                return genEnumDeclaration(node);
            case 'IfStatement':
                return genIfStatement(node);
            case 'WhileStatement':
                return genWhileStatement(node);
            case 'ForStatement':
                return genForStatement(node);
            case 'WithStatement':
                return genWithStatement(node);
            case 'SwitchStatement':
                return genSwitchStatement(node);
            case 'GuardStatement':
                return genGuardStatement(node);
            case 'AssertStatement':
                return genAssertStatement(node);
            case 'ReturnStatement':
                return genReturnStatement(node);
            case 'BreakStatement':
                return genBreakStatement();
            case 'ContinueStatement':
                return genContinueStatement();
            case 'ThrowStatement':
                return genThrowStatement(node);
            case 'ScribeStatement':
                return genScribeStatement(node);
            case 'TryStatement':
                return genTryStatement(node);
            case 'FacBlockStatement':
                return genFacBlockStatement(node);
            case 'BlockStatement':
                return genBlockStatementContent(node);
            case 'ExpressionStatement':
                return genExpressionStatement(node);
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate import declaration.
     *
     * TRANSFORMS:
     *   ex norma importa * -> import norma
     *   ex norma importa scribe, lege -> from norma import scribe, lege
     */
    function genImportDeclaration(node: ImportDeclaration): string {
        const source = node.source;

        if (node.wildcard) {
            return `${ind()}import ${source}`;
        }

        const names = node.specifiers.map(s => s.name).join(', ');
        return `${ind()}from ${source} import ${names}`;
    }

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   varia x: numerus = 5 -> x: int = 5
     *   fixum y: textus = "hello" -> y: str = "hello"
     *   fixum { nomen, aetas } = persona -> nomen = persona.nomen; aetas = persona.aetas
     *   figendum data = fetchData() -> data = await fetchData()
     *   variandum result = getResult() -> result = await getResult()
     *
     * WHY: Python has no const, so both varia and fixum become simple assignment.
     * WHY: Async bindings (figendum/variandum) imply await without explicit cede.
     */
    function genVariableDeclaration(node: VariableDeclaration): string {
        // Check if this is an async binding (figendum/variandum)
        const isAsync = node.kind === 'figendum' || node.kind === 'variandum';

        // Handle object pattern destructuring
        if (node.name.type === 'ObjectPattern') {
            const initExpr = node.init ? genExpression(node.init) : 'None';
            const awaitPrefix = isAsync ? 'await ' : '';
            const lines: string[] = [];

            // Separate regular properties from rest property
            const regularProps = node.name.properties.filter(p => !p.rest);
            const restProp = node.name.properties.find(p => p.rest);

            // Generate regular property extractions
            for (const prop of regularProps) {
                const key = prop.key.name;
                const localName = prop.value.name;
                lines.push(`${ind()}${localName} = ${awaitPrefix}${initExpr}["${key}"]`);
            }

            // Generate rest collection (remaining properties after extracting named ones)
            // WHY: Python doesn't have native rest destructuring, so we manually collect
            //      the remaining keys using dict comprehension
            if (restProp) {
                const restName = restProp.value.name;
                const excludeKeys = regularProps.map(p => `"${p.key.name}"`).join(', ');
                lines.push(`${ind()}${restName} = {k: v for k, v in ${awaitPrefix}${initExpr}.items() if k not in [${excludeKeys}]}`);
            }

            return lines.join('\n');
        }

        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';

        if (isAsync && node.init) {
            const init = genExpression(node.init);
            return `${ind()}${name}${typeAnno} = await ${init}`;
        }

        const init = node.init ? ` = ${genExpression(node.init)}` : '';
        return `${ind()}${name}${typeAnno}${init}`;
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(nomen: textus) -> textus { ... }
     *   -> def salve(nomen: str) -> str:
     *          ...
     *
     *   futura functio f() -> numerus { ... }
     *   -> async def f() -> int:
     *          ...
     *
     *   cursor functio f() -> numerus { ... }
     *   -> def f() -> Iterator[int]:
     *          ... (with yield)
     */
    function genFunctionDeclaration(node: FunctionDeclaration): string {
        const asyncMod = node.async ? 'async ' : '';
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');

        // Generate type parameters for generics (Python uses TypeVar)
        // prae typus T -> requires TypeVar('T') to be defined before function
        const typeParamDefs: string[] = [];
        if (node.typeParams && node.typeParams.length > 0) {
            for (const tp of node.typeParams) {
                typeParamDefs.push(`${ind()}${tp.name.name} = TypeVar('${tp.name.name}')`);
            }
        }

        // Build return type with generator/async wrapping
        let returnType = '';
        if (node.returnType) {
            let baseType = genType(node.returnType);
            if (node.async && node.generator) {
                baseType = `AsyncIterator[${baseType}]`;
            } else if (node.generator) {
                baseType = `Iterator[${baseType}]`;
            } else if (node.async) {
                baseType = `Awaitable[${baseType}]`;
            }
            returnType = ` -> ${baseType}`;
        }

        // Track generator context for cede -> yield vs await
        const prevInGenerator = inGenerator;
        inGenerator = node.generator;

        const header = `${ind()}${asyncMod}def ${name}(${params})${returnType}:`;
        depth++;
        const body = genBlockStatementContent(node.body);
        depth--;

        inGenerator = prevInGenerator;

        // Handle empty body
        if (node.body.body.length === 0) {
            const funcDef = `${header}\n${indent.repeat(depth + 1)}pass`;
            // Prepend TypeVar definitions if needed
            return typeParamDefs.length > 0 ? `${typeParamDefs.join('\n')}\n${funcDef}` : funcDef;
        }

        const funcDef = `${header}\n${body}`;
        // Prepend TypeVar definitions if needed
        return typeParamDefs.length > 0 ? `${typeParamDefs.join('\n')}\n${funcDef}` : funcDef;
    }

    /**
     * Generate function parameter.
     *
     * TRANSFORMS:
     *   nomen: textus -> nomen: str
     *   ceteri lista<textus> messages -> *messages: list[str]
     *
     * WHY: Python uses * for rest/variadic parameters.
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';
        const prefix = node.rest ? '*' : '';
        return `${prefix}${name}${typeAnno}`;
    }

    /**
     * Generate genus (class) declaration.
     *
     * TRANSFORMS:
     *   genus persona { textus nomen: "X" numerus aetas: 0 }
     *   ->
     *   class persona:
     *       nomen: str = "X"
     *       aetas: int = 0
     *
     *       def __init__(self, overrides: dict = {}):
     *           if 'nomen' in overrides:
     *               self.nomen = overrides['nomen']
     *           ...
     *
     *   genus Config { generis textus VERSION: "1.0" }
     *   ->
     *   class Config:
     *       VERSION: str = "1.0"  # Class-level (static)
     */
    function genGenusDeclaration(node: GenusDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters ? `[${node.typeParameters.map(p => p.name).join(', ')}]` : '';

        // Python uses Protocol for interfaces
        const impl = node.implements ? `(${node.implements.map(i => i.name).join(', ')})` : '';

        const lines: string[] = [];
        lines.push(`${ind()}class ${name}${typeParams}${impl}:`);
        depth++;

        // Track if we have any content
        let hasContent = false;

        // Separate fields by type: static, regular, reactive
        const staticFields = node.fields.filter(f => f.isStatic);
        const instanceFields = node.fields.filter(f => !f.isStatic && !f.isReactive);
        const reactiveFields = node.fields.filter(f => f.isReactive);

        // Static fields first (class-level variables in Python)
        if (staticFields.length > 0) {
            for (const field of staticFields) {
                lines.push(genFieldDeclaration(field));
            }
            hasContent = true;
        }

        // Instance fields
        if (instanceFields.length > 0) {
            if (hasContent) {
                lines.push('');
            }
            for (const field of instanceFields) {
                lines.push(genFieldDeclaration(field));
            }
            hasContent = true;
        }

        // Reactive fields as properties with backing field
        if (reactiveFields.length > 0) {
            if (hasContent) {
                lines.push('');
            }
            for (const field of reactiveFields) {
                lines.push(genReactiveFieldDeclaration(field));
            }
            hasContent = true;
        }

        // Constructor (only for non-static instance fields)
        const nonStaticFields = node.fields.filter(f => !f.isStatic);
        if (nonStaticFields.length > 0 || node.constructor) {
            if (hasContent) {
                lines.push('');
            }
            lines.push(genAutoMergeConstructor(node));
            hasContent = true;
        }

        // User's creo as private method
        if (node.constructor) {
            lines.push('');
            lines.push(genCreoMethod(node.constructor));
        }

        // Methods
        if (node.methods.length > 0) {
            lines.push('');
            for (const method of node.methods) {
                lines.push(genMethodDeclaration(method));
            }
            hasContent = true;
        }

        // Empty class needs pass
        if (!hasContent) {
            lines.push(`${ind()}pass`);
        }

        depth--;
        return lines.join('\n');
    }

    /**
     * Generate auto-merge constructor for genus.
     *
     * WHY: Static fields are class-level and not set via __init__.
     */
    function genAutoMergeConstructor(node: GenusDeclaration): string {
        const lines: string[] = [];
        lines.push(`${ind()}def __init__(self, overrides: dict = {}):`);
        depth++;

        // Only process non-static fields in constructor
        const instanceFields = node.fields.filter(f => !f.isStatic);

        if (instanceFields.length === 0 && !node.constructor) {
            lines.push(`${ind()}pass`);
        } else {
            // Apply each override if provided
            for (const field of instanceFields) {
                const fieldName = field.name.name;
                lines.push(`${ind()}if '${fieldName}' in overrides:`);
                depth++;
                lines.push(`${ind()}self.${fieldName} = overrides['${fieldName}']`);
                depth--;
            }

            // Call _creo() if user defined it
            if (node.constructor) {
                lines.push(`${ind()}self._creo()`);
            }
        }

        depth--;
        return lines.join('\n');
    }

    /**
     * Generate user's creo as a private method.
     */
    function genCreoMethod(node: FunctionDeclaration): string {
        const lines: string[] = [];
        lines.push(`${ind()}def _creo(self):`);
        depth++;

        if (node.body.body.length === 0) {
            lines.push(`${ind()}pass`);
        } else {
            lines.push(genBlockStatementContent(node.body));
        }

        depth--;
        return lines.join('\n');
    }

    /**
     * Generate field declaration within a class.
     */
    function genFieldDeclaration(node: FieldDeclaration): string {
        const name = node.name.name;
        const type = genType(node.fieldType);
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        // Python doesn't have private/static modifiers in the same way
        // Use underscore prefix convention for private
        const prefix = node.isPrivate ? '_' : '';

        return `${ind()}${prefix}${name}: ${type}${init}`;
    }

    /**
     * Generate reactive (nexum) field declaration as property with backing field.
     *
     * TRANSFORMS:
     *   nexum numerus count: 0
     *   ->
     *   _count: int = 0
     *   @property
     *   def count(self) -> int:
     *       return self._count
     *   @count.setter
     *   def count(self, value: int):
     *       self._count = value
     *       if hasattr(self, '_pingo'):
     *           self._pingo()
     */
    function genReactiveFieldDeclaration(node: FieldDeclaration): string {
        const name = node.name.name;
        const type = genType(node.fieldType);
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        const lines: string[] = [];

        // Private backing field
        lines.push(`${ind()}_${name}: ${type}${init}`);
        lines.push('');

        // Getter property
        lines.push(`${ind()}@property`);
        lines.push(`${ind()}def ${name}(self) -> ${type}:`);
        depth++;
        lines.push(`${ind()}return self._${name}`);
        depth--;
        lines.push('');

        // Setter with invalidation
        lines.push(`${ind()}@${name}.setter`);
        lines.push(`${ind()}def ${name}(self, value: ${type}):`);
        depth++;
        lines.push(`${ind()}self._${name} = value`);
        lines.push(`${ind()}if hasattr(self, '_pingo'):`);
        depth++;
        lines.push(`${ind()}self._pingo()`);
        depth--;
        depth--;

        return lines.join('\n');
    }

    /**
     * Generate method declaration within a class.
     */
    function genMethodDeclaration(node: FunctionDeclaration): string {
        const asyncMod = node.async ? 'async ' : '';
        const name = node.name.name;

        // Add self as first parameter
        const params = ['self', ...node.params.map(genParameter)].join(', ');

        // Build return type
        let returnType = '';
        if (node.returnType) {
            let baseType = genType(node.returnType);
            if (node.async && node.generator) {
                baseType = `AsyncIterator[${baseType}]`;
            } else if (node.generator) {
                baseType = `Iterator[${baseType}]`;
            } else if (node.async) {
                baseType = `Awaitable[${baseType}]`;
            }
            returnType = ` -> ${baseType}`;
        }

        const prevInGenerator = inGenerator;
        inGenerator = node.generator;

        const header = `${ind()}${asyncMod}def ${name}(${params})${returnType}:`;
        depth++;
        const body = node.body.body.length === 0 ? `${ind()}pass` : genBlockStatementContent(node.body);
        depth--;

        inGenerator = prevInGenerator;

        return `${header}\n${body}`;
    }

    /**
     * Generate pactum declaration as a Protocol.
     */
    function genPactumDeclaration(node: PactumDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters ? `[${node.typeParameters.map(p => p.name).join(', ')}]` : '';

        const lines: string[] = [];
        lines.push(`${ind()}class ${name}${typeParams}(Protocol):`);
        depth++;

        if (node.methods.length === 0) {
            lines.push(`${ind()}pass`);
        } else {
            for (const method of node.methods) {
                lines.push(genPactumMethod(method));
            }
        }

        depth--;
        return lines.join('\n');
    }

    /**
     * Generate pactum method signature.
     */
    function genPactumMethod(node: PactumMethod): string {
        const asyncMod = node.async ? 'async ' : '';
        const name = node.name.name;
        const params = ['self', ...node.params.map(genParameter)].join(', ');

        let returnType = node.returnType ? genType(node.returnType) : 'None';
        if (node.async && node.generator) {
            returnType = `AsyncIterator[${returnType}]`;
        } else if (node.generator) {
            returnType = `Iterator[${returnType}]`;
        } else if (node.async) {
            returnType = `Awaitable[${returnType}]`;
        }

        return `${ind()}${asyncMod}def ${name}(${params}) -> ${returnType}: ...`;
    }

    /**
     * Generate type alias declaration.
     *
     * TRANSFORMS:
     *   typus ID = textus -> ID = str  (or TypeAlias)
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;
        const typeAnno = genType(node.typeAnnotation);
        return `${ind()}${name} = ${typeAnno}`;
    }

    /**
     * Generate enum declaration.
     *
     * TRANSFORMS:
     *   ordo Color { rubrum, viridis, caeruleum }
     *   ->
     *   class Color(Enum):
     *       rubrum = auto()
     *       viridis = auto()
     *       caeruleum = auto()
     *
     *   ordo Status { pendens = 0, actum = 1 }
     *   ->
     *   class Status(Enum):
     *       pendens = 0
     *       actum = 1
     *
     * WHY: Python uses class-based Enum from the enum module.
     *      Members without explicit values use auto() for automatic numbering.
     */
    function genEnumDeclaration(node: EnumDeclaration): string {
        // Track that we need the Enum import in preamble
        features.enum = true;

        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}class ${name}(Enum):`);
        depth++;

        for (const member of node.members) {
            const memberName = member.name.name;

            if (member.value !== undefined) {
                const value = typeof member.value.value === 'string' ? `"${member.value.value}"` : member.value.value;
                lines.push(`${ind()}${memberName} = ${value}`);
            } else {
                lines.push(`${ind()}${memberName} = auto()`);
            }
        }

        depth--;
        return lines.join('\n');
    }

    /**
     * Generate type annotation from Latin type.
     */
    function genType(node: TypeAnnotation): string {
        // Track feature usage for preamble
        const lowerName = node.name.toLowerCase();
        if (lowerName === 'decimus' || lowerName === 'decim') {
            features.decimal = true;
        }

        // Map Latin type name to Python type (case-insensitive lookup)
        const base = typeMap[lowerName] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(genTypeParameter).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}[${params.join(', ')}]`;
            }
        }

        // Handle nullable: textus? -> str | None
        if (node.nullable) {
            result = `${result} | None`;
        }

        // Handle union types
        if (node.union && node.union.length > 0) {
            result = node.union.map(genType).join(' | ');
        }

        return result;
    }

    /**
     * Generate type parameter.
     */
    function genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return genType(param);
        }

        // Ignore numeric parameters (e.g., numerus<32>)
        if (param.type === 'Literal') {
            return null;
        }

        return null;
    }

    /**
     * Generate if statement.
     *
     * TRANSFORMS:
     *   si (conditio) { ... } -> if conditio:
     *   si (conditio) { ... } aliter { ... } -> if conditio: ... else: ...
     */
    function genIfStatement(node: IfStatement): string {
        const lines: string[] = [];

        // Handle catch clause by wrapping in try
        if (node.catchClause) {
            lines.push(`${ind()}try:`);
            depth++;
        }

        lines.push(`${ind()}if ${genExpression(node.test)}:`);
        depth++;
        lines.push(genBlockStatementContent(node.consequent));
        depth--;

        if (node.alternate) {
            if (node.alternate.type === 'IfStatement') {
                // elif chain
                const elifLines = genIfStatement(node.alternate).split('\n');
                // Replace 'if' with 'elif' on first line
                if (elifLines[0]) {
                    elifLines[0] = elifLines[0].replace(/^(\s*)if /, '$1elif ');
                }
                lines.push(elifLines.join('\n'));
            } else {
                lines.push(`${ind()}else:`);
                depth++;
                lines.push(genBlockStatementContent(node.alternate));
                depth--;
            }
        }

        if (node.catchClause) {
            depth--;
            lines.push(`${ind()}except Exception as ${node.catchClause.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.catchClause.body));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate while statement.
     */
    function genWhileStatement(node: WhileStatement): string {
        const lines: string[] = [];

        if (node.catchClause) {
            lines.push(`${ind()}try:`);
            depth++;
        }

        lines.push(`${ind()}while ${genExpression(node.test)}:`);
        depth++;
        lines.push(genBlockStatementContent(node.body));
        depth--;

        if (node.catchClause) {
            depth--;
            lines.push(`${ind()}except Exception as ${node.catchClause.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.catchClause.body));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate for statement.
     *
     * TRANSFORMS:
     *   ex 0..10 pro i { } -> for i in range(0, 10):
     *   ex items pro item { } -> for item in items:
     *   ex stream fiet chunk { } -> async for chunk in stream:
     */
    function genForStatement(node: ForStatement): string {
        const lines: string[] = [];
        const varName = node.variable.name;
        const asyncKw = node.async ? 'async ' : '';

        if (node.catchClause) {
            lines.push(`${ind()}try:`);
            depth++;
        }

        // Check if iterable is a range expression
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);
            // WHY: Python range() is exclusive, so add 1 for inclusive ranges
            const endExpr = range.inclusive ? `${end} + 1` : end;

            let rangeCall: string;
            if (range.step) {
                const step = genExpression(range.step);
                rangeCall = `range(${start}, ${endExpr}, ${step})`;
            } else {
                rangeCall = `range(${start}, ${endExpr})`;
            }

            lines.push(`${ind()}${asyncKw}for ${varName} in ${rangeCall}:`);
        } else {
            const iterable = genExpression(node.iterable);
            lines.push(`${ind()}${asyncKw}for ${varName} in ${iterable}:`);
        }

        depth++;
        lines.push(genBlockStatementContent(node.body));
        depth--;

        if (node.catchClause) {
            depth--;
            lines.push(`${ind()}except Exception as ${node.catchClause.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.catchClause.body));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate with statement (mutation block).
     *
     * TRANSFORMS:
     *   in user { nomen = "Marcus" } -> user.nomen = "Marcus"
     */
    function genWithStatement(node: WithStatement): string {
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
                lines.push(`${ind()}${context}.${prop} ${op} ${value}`);
            } else {
                lines.push(genStatement(stmt));
            }
        }

        return lines.join('\n');
    }

    /**
     * Generate switch statement using match/case (Python 3.10+).
     */
    function genSwitchStatement(node: SwitchStatement): string {
        const lines: string[] = [];
        const discriminant = genExpression(node.discriminant);

        if (node.catchClause) {
            lines.push(`${ind()}try:`);
            depth++;
        }

        lines.push(`${ind()}match ${discriminant}:`);
        depth++;

        for (const caseNode of node.cases) {
            const test = genExpression(caseNode.test);
            lines.push(`${ind()}case ${test}:`);
            depth++;
            lines.push(genBlockStatementContent(caseNode.consequent));
            depth--;
        }

        if (node.defaultCase) {
            lines.push(`${ind()}case _:`);
            depth++;
            lines.push(genBlockStatementContent(node.defaultCase));
            depth--;
        }

        depth--;

        if (node.catchClause) {
            depth--;
            lines.push(`${ind()}except Exception as ${node.catchClause.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.catchClause.body));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate guard statement.
     */
    function genGuardStatement(node: GuardStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            lines.push(`${ind()}if ${test}:`);
            depth++;
            lines.push(genBlockStatementContent(clause.consequent));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate assert statement.
     *
     * TRANSFORMS:
     *   adfirma x > 0 -> assert x > 0
     *   adfirma x > 0, "msg" -> assert x > 0, "msg"
     */
    function genAssertStatement(node: AssertStatement): string {
        const test = genExpression(node.test);

        if (node.message) {
            const message = genExpression(node.message);
            return `${ind()}assert ${test}, ${message}`;
        }

        return `${ind()}assert ${test}`;
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)}`;
        }
        return `${ind()}return`;
    }

    /**
     * Generate break statement.
     *
     * TRANSFORMS:
     *   rumpe -> break
     */
    function genBreakStatement(): string {
        return `${ind()}break`;
    }

    /**
     * Generate continue statement.
     *
     * TRANSFORMS:
     *   perge -> continue
     */
    function genContinueStatement(): string {
        return `${ind()}continue`;
    }

    /**
     * Generate throw/panic statement.
     *
     * TRANSFORMS:
     *   iace "message" -> raise Exception("message")
     *   mori "message" -> raise SystemExit("message")
     */
    function genThrowStatement(node: ThrowStatement): string {
        const arg = genExpression(node.argument);
        const exceptionType = node.fatal ? 'SystemExit' : 'Exception';

        // If throwing a string literal, wrap in exception type
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${ind()}raise ${exceptionType}(${arg})`;
        }

        // If throwing a new Error, convert to exception type
        if (node.argument.type === 'NewExpression') {
            const callee = node.argument.callee.name;
            if (callee === 'Error' || callee === 'erratum') {
                const args = node.argument.arguments.map(genExpression).join(', ');
                return `${ind()}raise ${exceptionType}(${args})`;
            }
        }

        if (node.fatal) {
            return `${ind()}raise SystemExit(${arg})`;
        }
        return `${ind()}raise ${arg}`;
    }

    /**
     * Generate scribe/vide/mone statement.
     *
     * TRANSFORMS:
     *   scribe "hello" -> print("hello")
     *   vide x         -> print("[DEBUG]", x)
     *   mone "oops"    -> print("[WARN]", "oops")
     */
    function genScribeStatement(node: ScribeStatement): string {
        const args = node.arguments.map(genExpression);
        if (node.level === 'debug') {
            args.unshift('"[DEBUG]"');
        } else if (node.level === 'warn') {
            args.unshift('"[WARN]"');
        }
        return `${ind()}print(${args.join(', ')})`;
    }

    /**
     * Generate try statement.
     */
    function genTryStatement(node: TryStatement): string {
        const lines: string[] = [];

        lines.push(`${ind()}try:`);
        depth++;
        lines.push(genBlockStatementContent(node.block));
        depth--;

        if (node.handler) {
            lines.push(`${ind()}except Exception as ${node.handler.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.handler.body));
            depth--;
        }

        if (node.finalizer) {
            lines.push(`${ind()}finally:`);
            depth++;
            lines.push(genBlockStatementContent(node.finalizer));
            depth--;
        }

        return lines.join('\n');
    }

    /**
     * Generate fac block statement (do-catch).
     *
     * TRANSFORMS:
     *   fac { x() } cape e { y() } -> try: x() except Exception as e: y()
     */
    function genFacBlockStatement(node: FacBlockStatement): string {
        const lines: string[] = [];

        // If there's a catch clause, wrap in try-except
        if (node.catchClause) {
            lines.push(`${ind()}try:`);
            depth++;
            lines.push(genBlockStatementContent(node.body));
            depth--;
            lines.push(`${ind()}except Exception as ${node.catchClause.param.name}:`);
            depth++;
            lines.push(genBlockStatementContent(node.catchClause.body));
            depth--;
        } else {
            // No catch - just emit the block contents
            lines.push(genBlockStatementContent(node.body));
        }

        return lines.join('\n');
    }

    /**
     * Generate block statement content (without braces).
     */
    function genBlockStatementContent(node: BlockStatement): string {
        if (node.body.length === 0) {
            return `${ind()}pass`;
        }
        return node.body.map(genStatement).join('\n');
    }

    function genExpressionStatement(node: ExpressionStatement): string {
        return `${ind()}${genExpression(node.expression)}`;
    }

    // ---------------------------------------------------------------------------
    // Expressions
    // ---------------------------------------------------------------------------

    function genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return genIdentifier(node);
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                return `f"${node.raw}"`;
            case 'ThisExpression':
                return 'self';
            case 'ArrayExpression':
                return genArrayExpression(node);
            case 'ObjectExpression':
                return genObjectExpression(node);
            case 'RangeExpression':
                return genRangeExpression(node);
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
            case 'LambdaExpression':
                return genLambdaExpression(node);
            case 'AssignmentExpression':
                return genAssignmentExpression(node);
            case 'AwaitExpression':
                // cede maps to yield in generators, await in async functions
                return `${inGenerator ? 'yield' : 'await'} ${genExpression(node.argument)}`;
            case 'NewExpression':
                return genNewExpression(node);
            case 'ConditionalExpression':
                // Python ternary: consequent if test else alternate
                return `${genExpression(node.consequent)} if ${genExpression(node.test)} else ${genExpression(node.alternate)}`;
            case 'TypeCastExpression':
                // WHY: Python is dynamically typed, type casts have no runtime effect.
                // Just emit the expression — the cast is a compile-time annotation only.
                return genExpression(node.expression);
            case 'PraefixumExpression':
                // WHY: Python has no compile-time evaluation.
                // Rather than silently dropping praefixum, throw an error.
                throw new Error(
                    'praefixum requires compile-time evaluation which Python does not support. ' + 'Use a literal value or remove praefixum.',
                );
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    /**
     * Generate identifier.
     */
    function genIdentifier(node: Identifier): string {
        switch (node.name) {
            case 'verum':
                return 'True';
            case 'falsum':
                return 'False';
            case 'nihil':
                return 'None';
            default:
                return node.name;
        }
    }

    /**
     * Generate literal value.
     */
    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return 'None';
        }

        if (typeof node.value === 'string') {
            return JSON.stringify(node.value);
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'True' : 'False';
        }

        // WHY: Python integers are arbitrary precision, no 'n' suffix needed
        if (typeof node.value === 'bigint') {
            // Strip 'n' suffix from raw (e.g., "0xFFn" -> "0xFF")
            return node.raw.replace(/n$/, '');
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
     *   [1, 2, 3] -> [1, 2, 3]
     *   [sparge a, sparge b] -> [*a, *b]
     *
     * WHY: Python uses * for unpacking iterables in list literals.
     */
    function genArrayExpression(node: ArrayExpression): string {
        const elements = node.elements
            .map(el => {
                if (el.type === 'SpreadElement') {
                    return `*${genExpression(el.argument)}`;
                }
                return genExpression(el);
            })
            .join(', ');
        return `[${elements}]`;
    }

    /**
     * Generate object literal as dict.
     *
     * TRANSFORMS:
     *   { nomen: "Marcus" } -> {"nomen": "Marcus"}
     *   { sparge defaults, x: 1 } -> {**defaults, "x": 1}
     *
     * WHY: Python uses ** for unpacking dicts in dict literals.
     */
    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '{}';
        }

        const props = node.properties.map(prop => {
            if (prop.type === 'SpreadElement') {
                return `**${genExpression(prop.argument)}`;
            }
            const key = prop.key.type === 'Identifier' ? `"${prop.key.name}"` : genLiteral(prop.key);
            const value = genExpression(prop.value);
            return `${key}: ${value}`;
        });

        return `{${props.join(', ')}}`;
    }

    /**
     * Generate range expression.
     *
     * WHY: Python range() is exclusive. For inclusive ranges (usque),
     *      we add 1 to the end value.
     */
    function genRangeExpression(node: RangeExpression): string {
        const start = genExpression(node.start);
        const end = genExpression(node.end);
        const endExpr = node.inclusive ? `${end} + 1` : end;

        if (node.step) {
            const step = genExpression(node.step);
            return `list(range(${start}, ${endExpr}, ${step}))`;
        }

        return `list(range(${start}, ${endExpr}))`;
    }

    /**
     * Generate binary expression.
     *
     * WHY: Python doesn't have nullish coalescing (??), so we expand to
     *      conditional expression. This evaluates left twice, which is
     *      acceptable for simple expressions.
     */
    function genBinaryExpression(node: BinaryExpression): string {
        const left = genExpression(node.left);
        const right = genExpression(node.right);

        // WHY: Python has no ?? operator; use conditional expression
        if (node.operator === '??') {
            return `(${left} if ${left} is not None else ${right})`;
        }

        const op = mapOperator(node.operator);

        return `(${left} ${op} ${right})`;
    }

    /**
     * Map operators to Python equivalents.
     */
    function mapOperator(op: string): string {
        switch (op) {
            case '&&':
                return 'and';
            case '||':
                return 'or';
            case '===':
                return '==';
            case '!==':
                return '!=';
            default:
                return op;
        }
    }

    /**
     * Generate unary expression.
     */
    function genUnaryExpression(node: UnaryExpression): string {
        const arg = genExpression(node.argument);

        // nulla: check if empty
        if (node.operator === 'nulla') {
            return `(not ${arg} or len(${arg}) == 0 if hasattr(${arg}, '__len__') else not ${arg})`;
        }

        // nonnulla: check if non-empty
        if (node.operator === 'nonnulla') {
            return `(${arg} and (len(${arg}) > 0 if hasattr(${arg}, '__len__') else bool(${arg})))`;
        }

        // negativum: check if less than zero
        if (node.operator === 'negativum') {
            return `(${arg} < 0)`;
        }

        // positivum: check if greater than zero
        if (node.operator === 'positivum') {
            return `(${arg} > 0)`;
        }

        // Map ! to not
        if (node.operator === '!') {
            return `not ${arg}`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   fn()    -> fn()
     *   fn?()   -> (fn() if fn is not None else None)
     *   fn!()   -> fn()  (Python has no assertion, just call)
     *   f(sparge nums) -> f(*nums)
     *
     * WHY: Python uses * for unpacking iterables in function calls.
     */
    function genCallExpression(node: CallExpression): string {
        const args = node.arguments
            .map(arg => {
                if (arg.type === 'SpreadElement') {
                    return `*${genExpression(arg.argument)}`;
                }
                return genExpression(arg);
            })
            .join(', ');

        // Check for intrinsics
        if (node.callee.type === 'Identifier') {
            const name = node.callee.name;
            const intrinsic = PY_INTRINSICS[name];
            if (intrinsic) {
                return intrinsic(args);
            }
        }

        // Check for collection methods (lista, tabula, copia)
        if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
            const methodName = (node.callee.property as Identifier).name;
            const obj = genExpression(node.callee.object);

            // Try lista methods
            const listaMethod = getListaMethod(methodName);
            if (listaMethod) {
                if (typeof listaMethod.py === 'function') {
                    return listaMethod.py(obj, args);
                }
                return `${obj}.${listaMethod.py}(${args})`;
            }

            // Try tabula methods
            const tabulaMethod = getTabulaMethod(methodName);
            if (tabulaMethod) {
                if (typeof tabulaMethod.py === 'function') {
                    return tabulaMethod.py(obj, args);
                }
                return `${obj}.${tabulaMethod.py}(${args})`;
            }

            // Try copia methods
            const copiaMethod = getCopiaMethod(methodName);
            if (copiaMethod) {
                if (typeof copiaMethod.py === 'function') {
                    return copiaMethod.py(obj, args);
                }
                return `${obj}.${copiaMethod.py}(${args})`;
            }
        }

        const callee = genExpression(node.callee);

        // WHY: Python has no native optional chaining; expand to conditional
        if (node.optional) {
            return `(${callee}(${args}) if ${callee} is not None else None)`;
        }
        // WHY: Python has no non-null assertion; just call directly
        return `${callee}(${args})`;
    }

    /**
     * Python intrinsic mappings.
     */
    const PY_INTRINSICS: Record<string, (args: string) => string> = {
        _scribe: args => `print(${args})`,
        _vide: args => `print(${args}, file=sys.stderr)`,
        _mone: args => `warnings.warn(${args})`,
        _lege: () => `input()`,
        _fortuitus: () => `random.random()`,
        _pavimentum: args => `math.floor(${args})`,
        _tectum: args => `math.ceil(${args})`,
        _radix: args => `math.sqrt(${args})`,
        _potentia: args => `math.pow(${args})`,
    };

    /**
     * Generate member access.
     *
     * TRANSFORMS:
     *   obj.prop      -> obj.prop
     *   obj?.prop     -> (obj.prop if obj is not None else None)
     *   obj!.prop     -> obj.prop  (Python has no assertion, just access)
     *   obj[idx]      -> obj[idx]
     *   obj?[idx]     -> (obj[idx] if obj is not None else None)
     *   obj![idx]     -> obj[idx]
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);

        if (node.computed) {
            const prop = genExpression(node.property);
            // WHY: Python has no native optional chaining; expand to conditional
            if (node.optional) {
                return `(${obj}[${prop}] if ${obj} is not None else None)`;
            }
            // WHY: Python has no non-null assertion; just access directly
            return `${obj}[${prop}]`;
        }

        const prop = (node.property as Identifier).name;
        if (node.optional) {
            return `(${obj}.${prop} if ${obj} is not None else None)`;
        }
        return `${obj}.${prop}`;
    }

    /**
     * Generate arrow function as lambda.
     */
    function genArrowFunction(node: ArrowFunctionExpression): string {
        const params = node.params.map(p => p.name.name).join(', ');

        // Simple expression body -> lambda
        if (node.body.type !== 'BlockStatement') {
            const body = genExpression(node.body as Expression);
            return `lambda ${params}: ${body}`;
        }

        // Block body - extract return expression if simple
        const block = node.body;
        const firstStmt = block.body[0];
        if (block.body.length === 1 && firstStmt?.type === 'ReturnStatement') {
            if (firstStmt.argument) {
                const body = genExpression(firstStmt.argument);
                return `lambda ${params}: ${body}`;
            }
        }

        // Complex block body - Python lambdas can't have statements
        // Use None as fallback; these should ideally be lifted to named functions
        return `lambda ${params}: None`;
    }

    /**
     * Generate Latin lambda expression (pro x redde expr).
     *
     * TARGET: Python lambdas don't support type annotations, so returnType
     *         is ignored. Use def with type hints for typed functions.
     */
    function genLambdaExpression(node: LambdaExpression): string {
        // Note: node.returnType is ignored in Python - lambdas can't have type hints
        const params = node.params.map(p => p.name).join(', ');

        // Simple expression body -> lambda
        if (node.body.type !== 'BlockStatement') {
            const body = genExpression(node.body as Expression);
            return `lambda ${params}: ${body}`;
        }

        // Block body - extract return expression if simple
        const block = node.body;
        const firstStmt = block.body[0];
        if (block.body.length === 1 && firstStmt?.type === 'ReturnStatement') {
            if (firstStmt.argument) {
                const body = genExpression(firstStmt.argument);
                return `lambda ${params}: ${body}`;
            }
        }

        // Complex block body - Python lambdas can't have statements
        return `lambda ${params}: None`;
    }

    /**
     * Generate assignment expression.
     */
    function genAssignmentExpression(node: AssignmentExpression): string {
        const left = node.left.type === 'Identifier' ? node.left.name : genExpression(node.left);

        return `${left} ${node.operator} ${genExpression(node.right)}`;
    }

    /**
     * Generate new expression (no 'new' keyword in Python).
     *
     * TRANSFORMS:
     *   novum Persona() -> Persona()
     *   novum Persona { nomen: "X" } -> Persona({"nomen": "X"})
     *   novum Persona de props -> Persona(props)
     *
     * WHY: Python classes are called directly without `new`.
     */
    function genNewExpression(node: NewExpression): string {
        const callee = node.callee.name;
        const args: string[] = node.arguments.map(genExpression);

        if (node.withExpression) {
            // withExpression can be ObjectExpression (inline) or any Expression (de X)
            args.push(genExpression(node.withExpression));
        }

        return `${callee}(${args.join(', ')})`;
    }

    return genProgram(program);
}
