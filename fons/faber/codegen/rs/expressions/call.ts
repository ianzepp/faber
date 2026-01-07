/**
 * Rust Code Generator - Call Expression
 *
 * TRANSFORMS:
 *   foo(a, b) -> foo(a, b)
 *   foo?(a) -> foo.map(|f| f(a))
 *   _scribe(x) -> println!("{}", x)
 *   _vide(x) -> eprintln!("[DEBUG] {}", x)
 *   _mone(x) -> eprintln!("[WARN] {}", x)
 *   _lege() -> std::io::stdin().read_line(...)
 *
 * Collection methods are translated via the unified norma registry.
 */

import type { CallExpression, Identifier } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

// WHY: Unified norma registry for all stdlib translations (from .fab files)
import { getNormaTranslation, applyNormaTemplate, applyNormaModuleCall, validateMorphology } from '../../norma-registry';

/**
 * Rust I/O intrinsic mappings.
 *
 * WHY: Maps Latin I/O intrinsics to Rust equivalents.
 * - _scribe: Standard output via println!
 * - _vide: Debug output to stderr via eprintln! with [DEBUG] prefix
 * - _mone: Warning output to stderr via eprintln! with [WARN] prefix
 * - _lege: Read line from stdin
 */
const RS_INTRINSICS: Record<string, (args: string[]) => string> = {
    _scribe: args => {
        if (args.length === 0) {
            return 'println!()';
        }
        // WHY: Generate format string with {} placeholders for each argument
        const placeholders = args.map(() => '{}').join(' ');
        return `println!("${placeholders}", ${args.join(', ')})`;
    },
    _vide: args => {
        if (args.length === 0) {
            return 'eprintln!("[DEBUG]")';
        }
        const placeholders = args.map(() => '{}').join(' ');
        return `eprintln!("[DEBUG] ${placeholders}", ${args.join(', ')})`;
    },
    _mone: args => {
        if (args.length === 0) {
            return 'eprintln!("[WARN]")';
        }
        const placeholders = args.map(() => '{}').join(' ');
        return `eprintln!("[WARN] ${placeholders}", ${args.join(', ')})`;
    },
    _lege: () => {
        // WHY: Rust stdin reading requires mutable buffer. This pattern reads a line,
        // trims the newline, and returns owned String. Panics on error (simplest).
        return '{ let mut __buf = String::new(); std::io::stdin().read_line(&mut __buf).unwrap(); __buf.trim().to_string() }';
    },
};

export function genCallExpression(node: CallExpression, g: RsGenerator): string {
    // WHY: Build args as array first for intrinsics that need individual args
    const argsArray = node.arguments.map(arg => {
        if (arg.type === 'SpreadElement') {
            return `/* spread */ ${g.genExpression(arg.argument)}`;
        }
        return g.genExpression(arg);
    });
    const args = argsArray.join(', ');

    // Check for intrinsics (bare function calls)
    if (node.callee.type === 'Identifier') {
        const name = node.callee.name;
        const intrinsic = RS_INTRINSICS[name];
        if (intrinsic) {
            return intrinsic(argsArray);
        }

        // Check norma module functions (mathesis, tempus, aleator)
        for (const module of ['mathesis', 'tempus', 'aleator']) {
            const call = applyNormaModuleCall('rs', module, name, [...argsArray]);
            if (call) {
                return call;
            }
        }
    }

    // Check for collection methods (method calls on lista/tabula/copia)
    if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
        const methodName = (node.callee.property as Identifier).name;
        const obj = g.genExpression(node.callee.object);

        // WHY: Use semantic type info to dispatch to correct collection registry.
        const objType = node.callee.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        // Try norma registry for the resolved collection type
        if (collectionName) {
            // WHY: Validate morphology before translation. Catches undefined forms.
            const validation = validateMorphology(collectionName, methodName);
            if (!validation.valid) {
                return `/* MORPHOLOGY: ${validation.error} */ ${obj}.${methodName}(${args})`;
            }

            const norma = getNormaTranslation('rs', collectionName, methodName);
            if (norma) {
                if (norma.method) {
                    return `${obj}.${norma.method}(${args})`;
                }
                if (norma.template && norma.params) {
                    return applyNormaTemplate(norma.template, [...norma.params], obj, [...argsArray]);
                }
            }
        }

        // WHY: No fallback guessing. If type isn't resolved to a known collection,
        //      preserve the Latin method name. Require type annotations for translation.
    }

    const callee = g.genExpression(node.callee);

    if (node.optional) {
        return `${callee}.map(|f| f(${args}))`;
    }

    return `${callee}(${args})`;
}
