/**
 * Zig Code Generator - GenusDeclaration (struct/class)
 *
 * TRANSFORMS:
 *   genus persona { textus nomen: "X" }
 *   -> const persona = struct {
 *          nomen: []const u8 = "X",
 *          const Self = @This();
 *          pub fn init(overrides: anytype) Self { ... }
 *      };
 *
 * TARGET: Zig uses structs with methods. The init() pattern replaces constructors.
 *         Self = @This() enables methods to reference their own type.
 *
 * ALLOCATOR: If struct has collection fields (lista/tabula/copia), an alloc field
 *            is added and init() takes allocator as first param. Methods use self.alloc.
 */

import type { GenusDeclaration, FieldDeclaration, FunctioDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';
import { isExternaFromAnnotations } from '../../types';

/** Collection type names that require allocators */
const COLLECTION_TYPES = ['lista', 'tabula', 'copia'];

/** Check if a field type is a collection that needs allocator */
function isCollectionField(field: FieldDeclaration): boolean {
    const typeName = field.fieldType?.name?.toLowerCase() ?? '';
    return COLLECTION_TYPES.includes(typeName);
}

/** Check if struct has any collection fields */
function hasCollectionFields(node: GenusDeclaration): boolean {
    return node.fields.some(isCollectionField);
}

export function genGenusDeclaration(node: GenusDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];
    const needsAlloc = hasCollectionFields(node);

    lines.push(`${g.ind()}const ${name} = struct {`);
    g.depth++;

    // WHY: Add allocator field if struct has collections
    // Methods will use self.alloc for allocating operations
    if (needsAlloc) {
        lines.push(`${g.ind()}alloc: std.mem.Allocator,`);
    }

    // Fields with defaults
    for (const field of node.fields) {
        lines.push(genFieldDeclaration(field, g));
    }

    // Self type reference (needed for init, methods)
    // WHY: init() uses Self for return type and self variable
    if (node.fields.length > 0 || node.methods.length > 0 || node.constructor || needsAlloc) {
        if (node.fields.length > 0 || needsAlloc) {
            lines.push('');
        }
        lines.push(`${g.ind()}const Self = @This();`);
    }

    // Auto-merge init function
    if (node.fields.length > 0 || needsAlloc) {
        lines.push('');
        lines.push(genStructInit(node, g, needsAlloc));
    }

    // User's creo as private post-init method
    if (node.constructor) {
        lines.push('');
        lines.push(genCreoMethod(node.constructor, g, needsAlloc));
    }

    // Methods
    if (node.methods.length > 0) {
        lines.push('');
        for (const method of node.methods) {
            lines.push(genStructMethod(method, g, needsAlloc));
        }
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    return lines.join('\n');
}

/**
 * Generate field declaration within a struct.
 *
 * TRANSFORMS:
 *   textus nomen: "X" -> nomen: []const u8 = "X",
 *   numerus aetas -> aetas: i64 = undefined,
 */
function genFieldDeclaration(node: FieldDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const type = g.genType(node.fieldType);
    const init = node.init ? g.genExpression(node.init) : 'undefined';

    // TARGET: Zig struct fields use = for defaults, end with comma
    return `${g.ind()}${name}: ${type} = ${init},`;
}

/**
 * Generate init function for struct (auto-merge constructor).
 *
 * TRANSFORMS:
 *   genus persona { textus nomen: "X", numerus aetas: 0 }
 *   -> pub fn init(overrides: anytype) Self {
 *          var self = Self{
 *              .nomen = if (@hasField(@TypeOf(overrides), "nomen")) overrides.nomen else "X",
 *              .aetas = if (@hasField(@TypeOf(overrides), "aetas")) overrides.aetas else 0,
 *          };
 *          return self;
 *      }
 *
 * TARGET: Uses comptime @hasField to check if override was provided.
 *         If needsAlloc, init takes allocator as first param and stores it.
 */
function genStructInit(node: GenusDeclaration, g: ZigGenerator, needsAlloc: boolean): string {
    const lines: string[] = [];

    // WHY: If struct has collections, init takes allocator as first param
    const initParams = needsAlloc ? 'alloc: std.mem.Allocator, overrides: anytype' : 'overrides: anytype';
    lines.push(`${g.ind()}pub fn init(${initParams}) Self {`);
    g.depth++;

    // WHY: Use 'var' if creo mutates self, otherwise 'const' to satisfy Zig linter
    const selfKind = node.constructor ? 'var' : 'const';
    lines.push(`${g.ind()}${selfKind} self = Self{`);
    g.depth++;

    // WHY: Store allocator first so it's available for collection init
    if (needsAlloc) {
        lines.push(`${g.ind()}.alloc = alloc,`);
    }

    for (const field of node.fields) {
        const name = field.name.name;
        const defaultVal = field.init ? g.genExpression(field.init) : 'undefined';

        lines.push(`${g.ind()}.${name} = if (@hasField(@TypeOf(overrides), "${name}")) overrides.${name} else ${defaultVal},`);
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    // Call creo if defined
    if (node.constructor) {
        lines.push(`${g.ind()}self.creo();`);
    }

    lines.push(`${g.ind()}return self;`);

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate user's creo as a method.
 *
 * TRANSFORMS:
 *   functio creo() { si ego.aetas < 0 { ego.aetas = 0 } }
 *   -> fn creo(self: *Self) void { if (self.aetas < 0) { self.aetas = 0; } }
 */
function genCreoMethod(node: FunctioDeclaration, g: ZigGenerator, needsAlloc: boolean): string {
    // EDGE: External declarations are not allowed as constructors (creo must have a body)
    if (isExternaFromAnnotations(node.annotations)) {
        throw new Error('External declarations (@ externa) not supported for constructors');
    }

    // EDGE: Abstract methods have no body - Zig doesn't support abstract methods
    if (!node.body) {
        throw new Error('Abstract methods not supported for Zig target');
    }

    const lines: string[] = [];

    lines.push(`${g.ind()}fn creo(self: *Self) void {`);
    g.depth++;

    // WHY: Push self.alloc onto curator stack so collection ops use it
    if (needsAlloc) {
        g.pushCurator('self.alloc');
    }

    // Generate body, replacing ego with self
    for (const stmt of node.body.body) {
        const code = g.genStatement(stmt);
        lines.push(code.replace(/\bego\b/g, 'self'));
    }

    if (needsAlloc) {
        g.popCurator();
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate method declaration within a struct.
 *
 * TRANSFORMS:
 *   functio saluta() -> textus { redde ego.nomen }
 *   -> pub fn saluta(self: *const Self) []const u8 { return self.nomen; }
 *
 * EDGE: anytype is not valid as a return type in Zig.
 */
function genStructMethod(node: FunctioDeclaration, g: ZigGenerator, needsAlloc: boolean): string {
    // EDGE: External declarations are not allowed as methods (methods must have a body)
    if (isExternaFromAnnotations(node.annotations)) {
        throw new Error('External declarations (@ externa) not supported for methods');
    }

    // EDGE: Abstract methods have no body - Zig doesn't support abstract methods
    if (!node.body) {
        throw new Error('Abstract methods not supported for Zig target');
    }

    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p));
    const returnType = node.returnType ? g.genType(node.returnType) : 'void';

    // Add self parameter - use *Self for methods that might mutate
    const selfParam = 'self: *const Self';
    const allParams = [selfParam, ...params].join(', ');

    // EDGE: anytype is not valid as return type in Zig
    if (returnType === 'anytype') {
        return `${g.ind()}pub fn ${name}(${allParams}) void { @compileError("Method '${name}' returns objectum/ignotum which has no Zig equivalent - use a concrete type"); }`;
    }

    const lines: string[] = [];

    lines.push(`${g.ind()}pub fn ${name}(${allParams}) ${returnType} {`);
    g.depth++;

    // WHY: Push self.alloc onto curator stack so collection ops use it
    if (needsAlloc) {
        g.pushCurator('self.alloc');
    }

    // Generate body, replacing ego with self
    for (const stmt of node.body.body) {
        const code = g.genStatement(stmt);
        lines.push(code.replace(/\bego\b/g, 'self'));
    }

    if (needsAlloc) {
        g.popCurator();
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
