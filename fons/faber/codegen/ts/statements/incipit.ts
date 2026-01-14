/**
 * TypeScript Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> body (top-level statements)
 *   incipit ergo stmt -> stmt (the ergo statement)
 *   @ cli incipit {} -> CLI argument parser and dispatcher
 *
 * TARGET: TypeScript/JavaScript executes top-level code directly.
 *         No wrapper function needed - just emit the body statements.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { TsGenerator, CliProgram, CliCommandNode, CliParam } from '../generator';

/**
 * Generate help text for a command node (shows its children).
 */
function genNodeHelp(
    cli: CliProgram,
    node: CliCommandNode,
    pathPrefix: string,
    ind: string,
    g: TsGenerator
): string {
    const lines: string[] = [];
    const fullCommand = pathPrefix ? `${cli.name} ${pathPrefix}` : cli.name;

    // Header (only at root level)
    if (!pathPrefix) {
        if (cli.version) {
            lines.push(`${ind}console.log("${cli.name} v${cli.version}");`);
        }
        else {
            lines.push(`${ind}console.log("${cli.name}");`);
        }
        if (cli.description) {
            lines.push(`${ind}console.log("${cli.description}");`);
        }
        lines.push(`${ind}console.log("");`);
    }

    lines.push(`${ind}console.log("Usage: ${fullCommand} <command> [options]");`);
    lines.push(`${ind}console.log("");`);
    lines.push(`${ind}console.log("Commands:");`);

    // List children
    for (const [name, child] of node.children) {
        const aliasStr = child.alias ? `, ${child.alias}` : '';
        const isGroup = child.children.size > 0 && !child.functionName;
        const suffix = isGroup ? ' ...' : '';
        lines.push(`${ind}console.log("  ${name}${aliasStr}${suffix}");`);
    }

    lines.push(`${ind}console.log("");`);
    lines.push(`${ind}console.log("Options:");`);
    lines.push(`${ind}console.log("  --help, -h     Show this help message");`);

    // Version only at root
    if (!pathPrefix && cli.version) {
        lines.push(`${ind}console.log("  --version, -v  Show version number");`);
    }

    return lines.join('\n');
}

/**
 * Generate argument parsing and function call for a leaf command.
 */
function genLeafCommand(
    node: CliCommandNode,
    argsVar: string,
    startIdx: number,
    ind: string
): string {
    const lines: string[] = [];
    const params = node.params ?? [];

    const positionalParams = params.filter(p => !p.optional);
    const optionalParams = params.filter(p => p.optional);

    // Declare variables for parsed args
    for (const p of params) {
        let defaultVal: string;
        if (p.type === 'bivalens') {
            defaultVal = 'false';
        }
        else if (p.defaultValue !== undefined) {
            defaultVal = `"${p.defaultValue}"`;
        }
        else if (p.optional) {
            defaultVal = 'undefined';
        }
        else {
            defaultVal = '""';
        }

        const tsType = p.type === 'bivalens' ? 'boolean' : 'string';
        const nullableType = p.optional && p.type !== 'bivalens' ? ' | undefined' : '';
        lines.push(`${ind}let ${p.name}: ${tsType}${nullableType} = ${defaultVal};`);
    }

    lines.push(`${ind}for (let _i = ${startIdx}; _i < ${argsVar}.length; _i++) {`);
    lines.push(`${ind}  const _arg = ${argsVar}[_i]!;`);

    // Handle optional flags
    for (const p of optionalParams) {
        const longFlag = `--${p.name}`;
        const shortFlag = p.shortFlag ? `-${p.shortFlag}` : null;
        const flagCheck = shortFlag
            ? `_arg === "${longFlag}" || _arg === "${shortFlag}"`
            : `_arg === "${longFlag}"`;

        if (p.type === 'bivalens') {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = true; continue; }`);
        }
        else {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = ${argsVar}[++_i]; continue; }`);
        }
    }

    // Handle positional args
    if (positionalParams.length > 0) {
        lines.push(`${ind}  if (!_arg.startsWith("-")) {`);
        for (const p of positionalParams) {
            lines.push(`${ind}    if (${p.name} === "") { ${p.name} = _arg; continue; }`);
        }
        lines.push(`${ind}  }`);
    }

    lines.push(`${ind}}`);

    // Validate required args
    for (const p of positionalParams) {
        lines.push(`${ind}if (${p.name} === "") {`);
        lines.push(`${ind}  console.error("Missing required argument: ${p.name}");`);
        lines.push(`${ind}  process.exit(1);`);
        lines.push(`${ind}}`);
    }

    // Call the function (with module prefix if from imported module)
    const argList = params.map(p => p.name).join(', ');
    const funcCall = node.modulePrefix
        ? `${node.modulePrefix}.${node.functionName}`
        : node.functionName;
    lines.push(`${ind}${funcCall}(${argList});`);

    return lines.join('\n');
}

/**
 * Generate dispatcher for a command node (recursive).
 */
function genNodeDispatcher(
    cli: CliProgram,
    node: CliCommandNode,
    pathPrefix: string,
    argsVar: string,
    argIdx: number,
    ind: string,
    g: TsGenerator
): string {
    const lines: string[] = [];
    const cmdVar = `_cmd${argIdx}`;

    lines.push(`${ind}const ${cmdVar} = ${argsVar}[${argIdx}];`);

    // Help flag or no command
    lines.push(`${ind}if (${cmdVar} === "--help" || ${cmdVar} === "-h" || ${cmdVar} === undefined) {`);
    lines.push(genNodeHelp(cli, node, pathPrefix, ind + g.indent, g));
    lines.push(`${ind}  process.exit(0);`);
    lines.push(`${ind}}`);

    // Version flag (only at root)
    if (!pathPrefix && cli.version) {
        lines.push(`${ind}if (${cmdVar} === "--version" || ${cmdVar} === "-v") {`);
        lines.push(`${ind}  console.log("${cli.version}");`);
        lines.push(`${ind}  process.exit(0);`);
        lines.push(`${ind}}`);
    }

    // Command dispatch
    lines.push(`${ind}switch (${cmdVar}) {`);

    for (const [name, child] of node.children) {
        // Case labels (include alias if present)
        if (child.alias) {
            lines.push(`${ind}  case "${name}":`);
            lines.push(`${ind}  case "${child.alias}": {`);
        }
        else {
            lines.push(`${ind}  case "${name}": {`);
        }

        const childPath = pathPrefix ? `${pathPrefix} ${name}` : name;

        if (child.functionName && child.children.size === 0) {
            // Leaf node - call the function
            lines.push(genLeafCommand(child, argsVar, argIdx + 1, ind + g.indent + g.indent));
        }
        else if (child.children.size > 0) {
            // Branch node - recurse
            lines.push(genNodeDispatcher(cli, child, childPath, argsVar, argIdx + 1, ind + g.indent + g.indent, g));
        }

        lines.push(`${ind}    break;`);
        lines.push(`${ind}  }`);
    }

    // Default case
    const errorContext = pathPrefix ? `${cli.name} ${pathPrefix}` : cli.name;
    lines.push(`${ind}  default: {`);
    lines.push(`${ind}    console.error(\`Unknown command: \${${cmdVar}}\`);`);
    lines.push(`${ind}    console.error("Run '${errorContext} --help' for usage.");`);
    lines.push(`${ind}    process.exit(1);`);
    lines.push(`${ind}  }`);
    lines.push(`${ind}}`);

    return lines.join('\n');
}

/**
 * Generate CLI dispatcher code (entry point).
 */
function genCliDispatcher(cli: CliProgram, g: TsGenerator): string {
    const ind = g.ind();
    const lines: string[] = [];

    // Generate imports for CLI command modules
    // These are injected here because they're not part of the original AST
    for (const [alias, path] of g.cliModuleImports) {
        lines.push(`import * as ${alias} from "${path}";`);
    }
    if (g.cliModuleImports.size > 0) {
        lines.push(``);
    }

    lines.push(`${ind}const _args = process.argv.slice(2);`);
    lines.push(``);
    lines.push(genNodeDispatcher(cli, cli.root, '', '_args', 0, ind, g));

    return lines.join('\n');
}

export function genIncipitStatement(node: IncipitStatement, g: TsGenerator): string {
    // CLI mode: generate argument parser and dispatcher
    if (g.cli) {
        return genCliDispatcher(g.cli, g);
    }

    // Handle ergo form: incipit ergo <statement>
    if (node.ergoStatement) {
        return g.genStatement(node.ergoStatement);
    }

    // Just emit the body statements - no wrapper needed for TS
    return g.genBlockStatementContent(node.body!);
}
