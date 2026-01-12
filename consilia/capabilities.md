# Target Capability System

**Status**: Proposed
**Author**: Design discussion 2026-01-12
**Related**: Multi-target codegen, Go target proposal

## Problem

Faber compiles to multiple target languages (TypeScript, Python, Rust, Zig, C++, potentially Go). These targets have vastly different capabilities:

- TypeScript has async/await, generators, async generators, destructuring, exceptions
- Python has most features but no optional chaining
- Rust has async but no generators in stable, uses Result not exceptions
- Zig has no async/await, no generators, error unions not exceptions
- C++ has exceptions but no async/generators
- Go has goroutines (not async/await), no generators, no exceptions

**Current behavior**: Codegen attempts to emit code for unsupported features, producing:
- Invalid target code that won't compile
- Runtime errors if code happens to parse
- Silent bugs from semantic mismatches (goroutines ≠ async/await)
- Confusing error messages from target compilers, not Faber

**Desired behavior**: Fail during Faber compilation with clear diagnostics about which features are incompatible with the chosen target.

## Design

### 1. Capability Declaration

Define supported features per target in a single source of truth:

```typescript
// fons/faber/codegen/capabilities.ts

export interface TargetCapabilities {
    // Control flow
    async: boolean;              // futura, figendum
    generators: boolean;         // cursor, fiunt
    asyncGenerators: boolean;    // futura cursor, fient
    exceptions: boolean;         // tempta...cape, iace

    // Syntax features
    destructuring: {
        array: boolean;          // fixum [a, b] = arr
        object: boolean;         // fixum {x, y} = obj
    };
    defaultParams: boolean;      // functio f(numerus x vel 0)
    optionalChaining: boolean;   // obj?.method?.()
    nullCoalescing: boolean;     // x ?? fallback

    // Type system
    nullable: 'builtin' | 'option' | 'pointer';  // How nihil/ignotum work
    errorHandling: 'exceptions' | 'result' | 'multiple-return' | 'union';
}

export const TARGET_CAPABILITIES: Record<CodegenTarget, TargetCapabilities> = {
    ts: {
        async: true,
        generators: true,
        asyncGenerators: true,
        exceptions: true,
        destructuring: { array: true, object: true },
        defaultParams: true,
        optionalChaining: true,
        nullCoalescing: true,
        nullable: 'builtin',
        errorHandling: 'exceptions',
    },

    py: {
        async: true,
        generators: true,
        asyncGenerators: true,
        exceptions: true,
        destructuring: { array: true, object: false },  // No dict unpacking in params
        defaultParams: true,
        optionalChaining: false,  // No ?. operator
        nullCoalescing: false,    // No ?? operator
        nullable: 'builtin',
        errorHandling: 'exceptions',
    },

    rs: {
        async: true,
        generators: false,        // No yield in stable Rust
        asyncGenerators: false,
        exceptions: false,        // Uses Result<T, E>
        destructuring: { array: true, object: true },
        defaultParams: false,     // No default params in Rust
        optionalChaining: false,
        nullCoalescing: false,    // Has .unwrap_or() but not ??
        nullable: 'option',       // Option<T>
        errorHandling: 'result',  // Result<T, E>
    },

    zig: {
        async: false,             // No async/await
        generators: false,
        asyncGenerators: false,
        exceptions: false,        // Error unions !T
        destructuring: { array: true, object: false },
        defaultParams: false,
        optionalChaining: false,
        nullCoalescing: false,    // Has orelse but semantics differ
        nullable: 'option',       // ?T
        errorHandling: 'union',   // !T error unions
    },

    cpp: {
        async: false,             // No native async (coroutines are complex)
        generators: false,
        asyncGenerators: false,
        exceptions: true,
        destructuring: { array: true, object: false },  // Structured bindings limited
        defaultParams: true,
        optionalChaining: false,
        nullCoalescing: false,
        nullable: 'option',       // std::optional
        errorHandling: 'exceptions',
    },

    go: {
        async: false,             // Goroutines are not async/await
        generators: false,        // iter.Seq[T] in 1.23+ but callback-based
        asyncGenerators: false,
        exceptions: false,        // Multiple return values (val, err)
        destructuring: { array: false, object: false },
        defaultParams: false,
        optionalChaining: false,
        nullCoalescing: false,
        nullable: 'pointer',      // *T or zero values
        errorHandling: 'multiple-return',
    },

    fab: {
        // Bootstrap compiler - supports everything
        async: true,
        generators: true,
        asyncGenerators: true,
        exceptions: true,
        destructuring: { array: true, object: true },
        defaultParams: true,
        optionalChaining: true,
        nullCoalescing: true,
        nullable: 'builtin',
        errorHandling: 'exceptions',
    },
};
```

### 2. Feature Detection

Walk AST after semantic analysis to collect features actually used:

```typescript
// fons/faber/codegen/feature-detector.ts

export interface UsedFeature {
    name: string;           // Capability key (e.g., "async", "destructuring.array")
    node: BaseNode;         // AST node for error reporting
    context?: string;       // Additional info (e.g., function name)
}

export class FeatureDetector {
    private features: UsedFeature[] = [];

    detect(program: Program): UsedFeature[] {
        this.visitProgram(program);
        return this.features;
    }

    private visitProgram(program: Program): void {
        for (const stmt of program.body) {
            this.visitStatement(stmt);
        }
    }

    private visitStatement(stmt: Statement): void {
        switch (stmt.type) {
            case 'FunctioDeclaration':
                this.visitFunctio(stmt);
                break;

            case 'VariaDeclaration':
                if (stmt.name.type === 'ArrayPattern') {
                    this.features.push({
                        name: 'destructuring.array',
                        node: stmt,
                        context: 'variable declaration',
                    });
                }
                if (stmt.init) {
                    this.visitExpression(stmt.init);
                }
                break;

            case 'DestructureDeclaration':
                if (stmt.target.type === 'ObjectPattern') {
                    this.features.push({
                        name: 'destructuring.object',
                        node: stmt,
                        context: 'variable declaration',
                    });
                }
                break;

            case 'TemptaStatement':
                this.features.push({
                    name: 'exceptions',
                    node: stmt,
                    context: 'try-catch block',
                });
                this.visitBlock(stmt.body);
                for (const handler of stmt.handlers) {
                    this.visitBlock(handler.body);
                }
                break;

            case 'IaceStatement':
                this.features.push({
                    name: 'exceptions',
                    node: stmt,
                    context: 'throw statement',
                });
                break;

            case 'SiStatement':
            case 'DumStatement':
            case 'EligeStatement':
            case 'DiscerneStatement':
            case 'BlockStatement':
                // Visit nested statements
                this.visitBlock(stmt.body);
                break;

            // ... other statement types
        }
    }

    private visitFunctio(func: FunctioDeclaration): void {
        const isAsync = func.async || isAsyncFromAnnotations(func.annotations);
        const isGenerator = func.generator || isGeneratorFromAnnotations(func.annotations);

        if (isAsync && isGenerator) {
            this.features.push({
                name: 'asyncGenerators',
                node: func,
                context: `function ${func.name.name}`,
            });
        } else if (isAsync) {
            this.features.push({
                name: 'async',
                node: func,
                context: `function ${func.name.name}`,
            });
        } else if (isGenerator) {
            this.features.push({
                name: 'generators',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        // Check for default parameters
        if (func.params.some(p => p.default)) {
            this.features.push({
                name: 'defaultParams',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        // Visit function body
        if (func.body) {
            this.visitBlock(func.body);
        }
    }

    private visitExpression(expr: Expression): void {
        switch (expr.type) {
            case 'MemberExpression':
                if (expr.optional) {
                    this.features.push({
                        name: 'optionalChaining',
                        node: expr,
                    });
                }
                this.visitExpression(expr.object);
                if (expr.computed) {
                    this.visitExpression(expr.property);
                }
                break;

            case 'CallExpression':
                if (expr.optional) {
                    this.features.push({
                        name: 'optionalChaining',
                        node: expr,
                    });
                }
                this.visitExpression(expr.callee);
                for (const arg of expr.arguments) {
                    if (arg.type === 'SpreadElement') {
                        this.visitExpression(arg.argument);
                    } else {
                        this.visitExpression(arg);
                    }
                }
                break;

            case 'BinaryExpression':
                if (expr.operator === '??') {
                    this.features.push({
                        name: 'nullCoalescing',
                        node: expr,
                    });
                }
                this.visitExpression(expr.left);
                this.visitExpression(expr.right);
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
                    if (prop.type === 'SpreadElement') {
                        this.visitExpression(prop.argument);
                    } else {
                        this.visitExpression(prop.value);
                    }
                }
                break;

            // ... other expression types
        }
    }

    private visitBlock(block: BlockStatement): void {
        for (const stmt of block.body) {
            this.visitStatement(stmt);
        }
    }
}
```

### 3. Validation

Check used features against target capabilities:

```typescript
// fons/faber/codegen/validator.ts

export interface ValidationError {
    feature: string;
    message: string;
    position?: Position;
    context?: string;
    suggestion?: string;
}

export function validateTargetCompatibility(
    program: Program,
    target: CodegenTarget
): ValidationError[] {
    const detector = new FeatureDetector();
    const usedFeatures = detector.detect(program);
    const capabilities = TARGET_CAPABILITIES[target];
    const errors: ValidationError[] = [];

    for (const used of usedFeatures) {
        if (!isFeatureSupported(used.name, capabilities)) {
            errors.push({
                feature: used.name,
                message: formatFeatureError(used.name, target),
                position: used.node.position,
                context: used.context,
                suggestion: getFeatureSuggestion(used.name, target),
            });
        }
    }

    return errors;
}

function isFeatureSupported(
    featureName: string,
    caps: TargetCapabilities
): boolean {
    // Handle nested features like "destructuring.array"
    const parts = featureName.split('.');
    let current: any = caps;

    for (const part of parts) {
        if (current[part] === undefined) return false;
        current = current[part];
    }

    return current === true;
}

function formatFeatureError(feature: string, target: CodegenTarget): string {
    const messages: Record<string, string> = {
        async: `Target '${target}' does not support async functions (futura)`,
        generators: `Target '${target}' does not support generator functions (cursor/fiunt)`,
        asyncGenerators: `Target '${target}' does not support async generators (futura cursor/fient)`,
        exceptions: `Target '${target}' does not support exception handling (tempta...cape)`,
        'destructuring.array': `Target '${target}' does not support array destructuring`,
        'destructuring.object': `Target '${target}' does not support object destructuring`,
        defaultParams: `Target '${target}' does not support default parameters`,
        optionalChaining: `Target '${target}' does not support optional chaining (?.)`,
        nullCoalescing: `Target '${target}' does not support null coalescing (??)`,
    };

    return messages[feature] || `Target '${target}' does not support feature '${feature}'`;
}

function getFeatureSuggestion(feature: string, target: CodegenTarget): string {
    const suggestions: Record<string, Record<string, string>> = {
        async: {
            go: "Use synchronous code or explicit goroutines with channels",
            zig: "Use synchronous code or event loop library",
            cpp: "Use synchronous code or third-party async library",
        },
        generators: {
            rs: "Use iterators with iterator adapters instead",
            zig: "Use while loops or iterator pattern",
            cpp: "Use ranges (C++20) or manual iterator implementation",
            go: "Use channels with goroutines or manual iteration",
        },
        exceptions: {
            rs: "Use Result<T, E> with explicit error handling",
            zig: "Use error unions (!T) with explicit error handling",
            go: "Use multiple return values (val, error) pattern",
        },
        'destructuring.array': {
            go: "Use individual variable assignments: a = arr[0]; b = arr[1]",
            zig: "Use individual variable assignments with array indexing",
        },
        'destructuring.object': {
            py: "Use individual attribute access or dict.get()",
            zig: "Use individual field access",
            cpp: "Use individual member access",
            go: "Use individual field access",
        },
        defaultParams: {
            rs: "Use Option<T> parameters or function overloading",
            zig: "Use optional parameters (?T) with null checks",
            go: "Use variadic parameters or options struct pattern",
        },
        optionalChaining: {
            py: "Use explicit None checks: if x is not None: x.method()",
            rs: "Use Option methods: x.map(|v| v.method())",
            zig: "Use explicit null checks: if (x) |val| val.method()",
            cpp: "Use std::optional with value_or() or explicit checks",
            go: "Use explicit nil checks before accessing",
        },
    };

    const targetSuggestions = suggestions[feature];
    if (targetSuggestions && targetSuggestions[target]) {
        return targetSuggestions[target];
    }

    // Fallback: suggest compatible targets
    const compatibleTargets = getCompatibleTargets(feature);
    if (compatibleTargets.length > 0) {
        return `Consider using ${compatibleTargets.join(', ')} target instead`;
    }

    return "Refactor to avoid this feature";
}

function getCompatibleTargets(feature: string): string[] {
    const targets: CodegenTarget[] = ['ts', 'py', 'rs', 'zig', 'cpp', 'go'];
    return targets.filter(t => {
        const caps = TARGET_CAPABILITIES[t];
        return isFeatureSupported(feature, caps);
    });
}
```

### 4. Integration

Add validation pass before codegen:

```typescript
// fons/faber/codegen/index.ts

export function generate(
    program: Program,
    target: CodegenTarget = 'ts',
    options: CodegenOptions = {}
): string {
    // Validate target compatibility BEFORE attempting codegen
    const validationErrors = validateTargetCompatibility(program, target);

    if (validationErrors.length > 0) {
        throw new TargetCompatibilityError(validationErrors, target);
    }

    // Existing codegen dispatch
    switch (target) {
        case 'ts':
            return generateTs(program, options);
        case 'py':
            return generatePy(program, options);
        case 'rs':
            return generateRs(program, options);
        case 'zig':
            return generateZig(program, options);
        case 'cpp':
            return generateCpp(program, options);
        case 'go':
            return generateGo(program, options);
        case 'fab':
            return generateFab(program, options);
        default:
            throw new Error(`Unknown codegen target: ${target}`);
    }
}

// Custom error class for formatted output
export class TargetCompatibilityError extends Error {
    constructor(
        public errors: ValidationError[],
        public target: CodegenTarget
    ) {
        const formatted = formatValidationErrors(errors, target);
        super(`Target compatibility errors for '${target}':\n\n${formatted}`);
        this.name = 'TargetCompatibilityError';
    }
}

function formatValidationErrors(errors: ValidationError[], target: CodegenTarget): string {
    return errors.map(err => {
        const pos = err.position
            ? `${err.position.line}:${err.position.column}`
            : 'unknown';

        const context = err.context ? ` (in ${err.context})` : '';
        const msg = `error: ${err.message}${context}`;
        const location = `  --> ${pos}`;
        const suggestion = err.suggestion
            ? `  = help: ${err.suggestion}`
            : '';

        return [location, msg, suggestion].filter(Boolean).join('\n');
    }).join('\n\n');
}
```

## User Experience

### Example 1: Async function to Zig

```faber
# hello.fab
futura functio fetch(textus url) fit textus {
    redde "data"
}
```

```bash
$ bun run faber compile hello.fab -t zig
```

**Output**:
```
error: Target 'zig' does not support async functions (futura) (in function fetch)
  --> 2:1
  = help: Use synchronous code or event loop library
```

### Example 2: Multiple errors to Go

```faber
# example.fab
futura functio process(numerus x vel 0) fit numerus[] {
    fixum data = tempta {
        figendum result = fetch()
        redde result?.data ?? []
    } cape err {
        redde []
    }

    redde data
}
```

```bash
$ bun run faber compile example.fab -t go
```

**Output**:
```
error: Target 'go' does not support async functions (futura) (in function process)
  --> 2:1
  = help: Use synchronous code or explicit goroutines with channels

error: Target 'go' does not support default parameters (in function process)
  --> 2:23
  = help: Use variadic parameters or options struct pattern

error: Target 'go' does not support exception handling (tempta...cape) (in try-catch block)
  --> 3:18
  = help: Use multiple return values (val, error) pattern

error: Target 'go' does not support optional chaining (?.)
  --> 5:16
  = help: Use explicit nil checks before accessing

error: Target 'go' does not support null coalescing (??)
  --> 5:28
  = help: Use explicit nil checks before accessing
```

### Example 3: Compatible code

```faber
# valid.fab
functio sum(numerus[] nums) fit numerus {
    varia total = 0
    ex nums pro n {
        total = total + n
    }
    redde total
}
```

```bash
$ bun run faber compile valid.fab -t go
# Succeeds - no async, generators, destructuring, or exceptions
```

## Implementation Plan

### Phase 1: Infrastructure
1. Create `fons/faber/codegen/capabilities.ts` with target capability definitions
2. Create `fons/faber/codegen/feature-detector.ts` with AST visitor
3. Create `fons/faber/codegen/validator.ts` with validation logic
4. Add `TargetCompatibilityError` class for formatted diagnostics

### Phase 2: Integration
1. Modify `fons/faber/codegen/index.ts` to run validation before codegen
2. Update CLI to surface validation errors with proper formatting
3. Add `--strict` flag to fail on warnings (future: soft incompatibilities)

### Phase 3: Testing
1. Create test suite in `fons/proba/capabilities/` with YAML test cases
2. Test each feature against each target
3. Verify error messages are clear and actionable
4. Test combinations (async + generator, destructure in function params, etc.)

### Phase 4: Documentation
1. Document capability system in `fons/grammatica/targets.md`
2. Add target compatibility matrix to README
3. Update error message suggestions based on user feedback

## Edge Cases

### 1. Conditional compilation

Future extension: Allow conditional target code:

```faber
@ si target ts {
    futura functio fetch() { ... }
}

@ si target go {
    functio fetch() { ... }
}
```

Not needed initially - users can maintain separate files per target.

### 2. Soft vs hard incompatibilities

Some features can be emulated with warnings:

- `async` on Go → synchronous code (semantic mismatch, warn)
- Default params on Rust → overloading (verbose, warn)
- Destructuring on Go → multiple statements (annoying, warn)

Initial implementation: all incompatibilities are hard errors. Future: add `--allow-compat` flag for soft errors with code generation fallbacks.

### 3. Version-specific features

Go 1.23 has `iter.Seq[T]` for iteration. Zig 0.13 added labeled blocks. Rust nightly has generators.

Solution: Add optional `version` field to capabilities:

```typescript
export interface TargetVersion {
    target: CodegenTarget;
    version?: string;  // Semantic version or "nightly"
}

// Usage: validateTargetCompatibility(program, { target: 'go', version: '1.23' })
```

Deferred to future work.

### 4. Feature interactions

Some combinations are invalid even if both features are supported:

- Async + generator = async generator (different capability)
- Default params + destructuring in params (complex)

Solution: Feature detector already handles combinations by detecting the composite feature (`asyncGenerators` not just `async` + `generators`).

### 5. Library/runtime dependencies

Some features need runtime support even if language supports them:

- Python `@dataclass` needs import
- TypeScript Decimal needs decimal.js
- Zig Lista needs runtime library

Current system: `RequiredFeatures` tracks this for preamble generation. Orthogonal to capability validation - if target supports the feature, codegen handles runtime deps.

## Alternatives Considered

### 1. Codegen-time detection

**Rejected**: Too late. Codegen is already attempting to emit code when it discovers incompatibility. Error messages reference target constructs, not Faber source.

### 2. Lint pass instead of hard errors

**Rejected initially**: Users expect code to compile if it passes validation. Lints are for style/performance, not correctness. However, could add `--strict` mode later for warnings.

### 3. Annotations-based gating

```faber
@ capacitas async, generators
functio process() { ... }
```

**Rejected**: Annotation burden on users. Capability validation should be automatic based on what's written.

### 4. Runtime feature detection

**Rejected**: Faber is compiled, not interpreted. No runtime to check features. Must be compile-time.

### 5. Per-file target specification

```faber
@ meta target ts

futura functio f() { ... }
```

**Rejected**: Creates fragmented codebase. Better to organize by directories if different files need different targets. CLI flag is sufficient.

## Future Extensions

1. **Capability polyfills**: Automatic transformation of unsupported features to supported equivalents
   - `async` on Go → synchronous execution (with warning)
   - Destructuring → multiple statements
   - Requires opt-in flag: `--compat-mode`

2. **Target profiles**: Named capability sets
   - `modern` = all features
   - `embedded` = no async, no exceptions, no GC
   - `wasm` = no file I/O, limited concurrency

3. **Custom capabilities**: User-defined capability constraints
   ```typescript
   // fabfile.json
   {
       "capabilities": {
           "async": false,  // Disallow even if target supports it
           "generators": false
       }
   }
   ```

4. **Migration tooling**: CLI command to check compatibility before porting
   ```bash
   $ bun run faber check --target go src/**/*.fab
   Found 23 incompatibilities in 8 files
   Run with --verbose for details
   ```

## References

- Rust feature gates: https://doc.rust-lang.org/unstable-book/
- TypeScript target/lib options: https://www.typescriptlang.org/tsconfig#target
- Swift availability: https://docs.swift.org/swift-book/ReferenceManual/Attributes.html
- C# target frameworks: https://learn.microsoft.com/en-us/dotnet/standard/frameworks

## Decision

**Status**: Awaiting approval

This design provides:
- Clear, actionable error messages
- Single source of truth for target capabilities
- Easy extension as new targets are added
- Consistent with PL industry standards

Implementation can proceed once approved.
