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

Define target constraints in a _single source of truth_, but split it into:

1. **Support**: “can this target faithfully support this Faber construct?”
2. **Lowering model**: “if supported, what representation does codegen use?”

This avoids mixing boolean checks (support) with configuration choices (lowering), which otherwise leads to subtle bugs (e.g., treating `nullable: 'option'` as “unsupported” because it’s not `true`).

```typescript
// fons/faber/codegen/capabilities.ts

export type SupportLevel = 'supported' | 'emulated' | 'mismatched' | 'unsupported';

export interface TargetSupport {
    controlFlow: {
        asyncFunction: SupportLevel; // futura / figendum
        generatorFunction: SupportLevel; // cursor / fiunt
        asyncGeneratorFunction: SupportLevel; // futura cursor / fient
    };

    errors: {
        tryCatch: SupportLevel; // tempta...cape
        throw: SupportLevel; // iace
    };

    binding: {
        pattern: {
            array: SupportLevel; // pattern binding (e.g., fixum [a, b] = ...)
            object: SupportLevel; // pattern binding (e.g., fixum {x, y} = ...)
        };
    };

    params: {
        defaultValues: SupportLevel; // functio f(numerus x vel 0)
    };

    expressions: {
        nullableMemberAccess: SupportLevel; // “optional chaining”-like semantics
        coalesce: SupportLevel; // “null coalescing”-like semantics
    };
}

export interface TargetLoweringModel {
    // How nihil/ignotum are represented when allowed
    nullable: 'builtin' | 'option' | 'pointer';

    // Primary error model when errors are allowed
    errorHandling: 'exceptions' | 'result' | 'multiple-return' | 'union';
}

export const TARGET_SUPPORT: Record<CodegenTarget, TargetSupport> = {
    ts: {
        controlFlow: { asyncFunction: 'supported', generatorFunction: 'supported', asyncGeneratorFunction: 'supported' },
        errors: { tryCatch: 'supported', throw: 'supported' },
        binding: { pattern: { array: 'supported', object: 'supported' } },
        params: { defaultValues: 'supported' },
        expressions: { nullableMemberAccess: 'supported', coalesce: 'supported' },
    },

    py: {
        controlFlow: { asyncFunction: 'supported', generatorFunction: 'supported', asyncGeneratorFunction: 'supported' },
        errors: { tryCatch: 'supported', throw: 'supported' },
        binding: { pattern: { array: 'supported', object: 'unsupported' } },
        params: { defaultValues: 'supported' },
        expressions: { nullableMemberAccess: 'unsupported', coalesce: 'unsupported' },
    },

    rs: {
        controlFlow: { asyncFunction: 'supported', generatorFunction: 'unsupported', asyncGeneratorFunction: 'unsupported' },
        errors: { tryCatch: 'unsupported', throw: 'unsupported' },
        binding: { pattern: { array: 'supported', object: 'supported' } },
        params: { defaultValues: 'unsupported' },
        expressions: { nullableMemberAccess: 'unsupported', coalesce: 'unsupported' },
    },

    zig: {
        controlFlow: { asyncFunction: 'unsupported', generatorFunction: 'unsupported', asyncGeneratorFunction: 'unsupported' },
        errors: { tryCatch: 'unsupported', throw: 'unsupported' },
        binding: { pattern: { array: 'supported', object: 'unsupported' } },
        params: { defaultValues: 'unsupported' },
        expressions: { nullableMemberAccess: 'unsupported', coalesce: 'mismatched' },
    },

    cpp: {
        controlFlow: { asyncFunction: 'unsupported', generatorFunction: 'unsupported', asyncGeneratorFunction: 'unsupported' },
        errors: { tryCatch: 'supported', throw: 'supported' },
        binding: { pattern: { array: 'supported', object: 'unsupported' } },
        params: { defaultValues: 'supported' },
        expressions: { nullableMemberAccess: 'unsupported', coalesce: 'unsupported' },
    },

    go: {
        controlFlow: { asyncFunction: 'mismatched', generatorFunction: 'unsupported', asyncGeneratorFunction: 'unsupported' },
        errors: { tryCatch: 'unsupported', throw: 'unsupported' },
        binding: { pattern: { array: 'unsupported', object: 'unsupported' } },
        params: { defaultValues: 'unsupported' },
        expressions: { nullableMemberAccess: 'unsupported', coalesce: 'unsupported' },
    },

    fab: {
        controlFlow: { asyncFunction: 'supported', generatorFunction: 'supported', asyncGeneratorFunction: 'supported' },
        errors: { tryCatch: 'supported', throw: 'supported' },
        binding: { pattern: { array: 'supported', object: 'supported' } },
        params: { defaultValues: 'supported' },
        expressions: { nullableMemberAccess: 'supported', coalesce: 'supported' },
    },
};

export const TARGET_LOWERING: Record<CodegenTarget, TargetLoweringModel> = {
    ts: { nullable: 'builtin', errorHandling: 'exceptions' },
    py: { nullable: 'builtin', errorHandling: 'exceptions' },
    rs: { nullable: 'option', errorHandling: 'result' },
    zig: { nullable: 'option', errorHandling: 'union' },
    cpp: { nullable: 'option', errorHandling: 'exceptions' },
    go: { nullable: 'pointer', errorHandling: 'multiple-return' },
    fab: { nullable: 'builtin', errorHandling: 'exceptions' },
};
```

### 2. Feature Detection

Walk the AST after semantic analysis to collect _Faber-level constructs actually used_.

Key points:

- Use **Faber construct keys**, not target spellings like `?.` / `??`.
- Prefer **canonical semantic flags** (produced by analysis) over “syntax sniffing”.
- **Deduplicate/group** results so real programs don’t emit the same error hundreds of times.

```typescript
// fons/faber/codegen/feature-detector.ts

type FeatureKey =
    | 'controlFlow.asyncFunction'
    | 'controlFlow.generatorFunction'
    | 'controlFlow.asyncGeneratorFunction'
    | 'errors.tryCatch'
    | 'errors.throw'
    | 'binding.pattern.array'
    | 'binding.pattern.object'
    | 'params.defaultValues'
    | 'expressions.nullableMemberAccess'
    | 'expressions.coalesce';

export interface UsedFeature {
    key: FeatureKey;
    node: BaseNode;
    context?: string;
}

export class FeatureDetector {
    private featuresByKey = new Map<FeatureKey, UsedFeature[]>();

    detect(program: Program): UsedFeature[] {
        this.visitProgram(program);
        return [...this.featuresByKey.values()].flat();
    }

    private add(feature: UsedFeature): void {
        const list = this.featuresByKey.get(feature.key) ?? [];
        list.push(feature);
        this.featuresByKey.set(feature.key, list);
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
                    this.add({
                        key: 'binding.pattern.array',
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
                    this.add({
                        key: 'binding.pattern.object',
                        node: stmt,
                        context: 'variable declaration',
                    });
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
                break;

            case 'IaceStatement':
                this.add({
                    key: 'errors.throw',
                    node: stmt,
                    context: 'throw statement',
                });
                break;

            case 'SiStatement':
            case 'DumStatement':
            case 'EligeStatement':
            case 'DiscerneStatement':
            case 'BlockStatement':
                this.visitBlock(stmt.body);
                break;

            // ... other statement types
        }
    }

    private visitFunctio(func: FunctioDeclaration): void {
        // Prefer semantic flags if available (single source of truth).
        const isAsync = func.async || isAsyncFromAnnotations(func.annotations);
        const isGenerator = func.generator || isGeneratorFromAnnotations(func.annotations);

        if (isAsync && isGenerator) {
            this.add({
                key: 'controlFlow.asyncGeneratorFunction',
                node: func,
                context: `function ${func.name.name}`,
            });
        } else if (isAsync) {
            this.add({
                key: 'controlFlow.asyncFunction',
                node: func,
                context: `function ${func.name.name}`,
            });
        } else if (isGenerator) {
            this.add({
                key: 'controlFlow.generatorFunction',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        if (func.params.some(p => p.default)) {
            this.add({
                key: 'params.defaultValues',
                node: func,
                context: `function ${func.name.name}`,
            });
        }

        if (func.body) {
            this.visitBlock(func.body);
        }
    }

    private visitExpression(expr: Expression): void {
        switch (expr.type) {
            case 'MemberExpression':
                if (expr.optional) {
                    this.add({ key: 'expressions.nullableMemberAccess', node: expr });
                }
                this.visitExpression(expr.object);
                if (expr.computed) {
                    this.visitExpression(expr.property);
                }
                break;

            case 'CallExpression':
                if (expr.optional) {
                    this.add({ key: 'expressions.nullableMemberAccess', node: expr });
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
                    this.add({ key: 'expressions.coalesce', node: expr });
                }
                this.visitExpression(expr.left);
                this.visitExpression(expr.right);
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

Check used features against target support levels.

This validator should answer two questions:

- Is the construct **supported**?
- If it’s only **emulated** or **mismatched**, do we allow that under the selected policy/flags?

```typescript
// fons/faber/codegen/validator.ts

export interface ValidationError {
    feature: string;
    message: string;
    position?: Position;
    context?: string;
    suggestion?: string;
}

export interface SupportPolicy {
    allowEmulated: boolean;
    allowMismatched: boolean;
}

export function validateTargetCompatibility(
    program: Program,
    target: CodegenTarget,
    policy: SupportPolicy = { allowEmulated: false, allowMismatched: false },
): ValidationError[] {
    const detector = new FeatureDetector();
    const usedFeatures = detector.detect(program);
    const support = TARGET_SUPPORT[target];
    const errors: ValidationError[] = [];

    for (const used of usedFeatures) {
        const level = getSupportLevel(used.key, support);
        if (!isAllowed(level, policy)) {
            errors.push({
                feature: used.key,
                message: formatFeatureError(used.key, target, level),
                position: used.node.position,
                context: used.context,
                suggestion: getFeatureSuggestion(used.key, target),
            });
        }
    }

    return errors;
}

function isAllowed(level: SupportLevel, policy: SupportPolicy): boolean {
    switch (level) {
        case 'supported':
            return true;
        case 'emulated':
            return policy.allowEmulated;
        case 'mismatched':
            return policy.allowMismatched;
        case 'unsupported':
            return false;
    }
}

function getSupportLevel(featureKey: string, support: TargetSupport): SupportLevel {
    // Feature keys are hierarchical, e.g. "errors.tryCatch".
    const parts = featureKey.split('.');
    let current: any = support;

    for (const part of parts) {
        if (current?.[part] === undefined) {
            // Unknown key: treat as unsupported so we fail loudly.
            return 'unsupported';
        }
        current = current[part];
    }

    return current as SupportLevel;
}

function formatFeatureError(feature: string, target: CodegenTarget, level: SupportLevel): string {
    const suffix = level === 'mismatched' ? ' (semantic mismatch)' : level === 'emulated' ? ' (requires emulation)' : '';

    const messages: Record<string, string> = {
        'controlFlow.asyncFunction': `Target '${target}' does not support async functions (futura)${suffix}`,
        'controlFlow.generatorFunction': `Target '${target}' does not support generator functions (cursor/fiunt)${suffix}`,
        'controlFlow.asyncGeneratorFunction': `Target '${target}' does not support async generators (futura cursor/fient)${suffix}`,
        'errors.tryCatch': `Target '${target}' does not support exception handling (tempta...cape)${suffix}`,
        'errors.throw': `Target '${target}' does not support throw statements (iace)${suffix}`,
        'binding.pattern.array': `Target '${target}' does not support array pattern binding${suffix}`,
        'binding.pattern.object': `Target '${target}' does not support object pattern binding${suffix}`,
        'params.defaultValues': `Target '${target}' does not support default parameters${suffix}`,
        'expressions.nullableMemberAccess': `Target '${target}' does not support nullable member access (optional chaining semantics)${suffix}`,
        'expressions.coalesce': `Target '${target}' does not support coalescing (null-coalescing semantics)${suffix}`,
    };

    return messages[feature] || `Target '${target}' does not support feature '${feature}'${suffix}`;
}

function getFeatureSuggestion(feature: string, target: CodegenTarget): string {
    const suggestions: Record<string, Record<string, string>> = {
        'controlFlow.asyncFunction': {
            go: 'Refactor to synchronous code; goroutines are not async/await',
            zig: 'Refactor to synchronous code; consider explicit callbacks/event loop',
            cpp: 'Refactor to synchronous code or adopt an async runtime model',
        },
        'controlFlow.generatorFunction': {
            rs: 'Use iterators and iterator adapters instead',
            zig: 'Use a while loop or explicit iterator type',
            cpp: 'Use ranges/coroutines only with an explicit design',
            go: 'Use channels + goroutines or manual iteration',
        },
        'errors.tryCatch': {
            rs: 'Use Result<T, E> and propagate errors explicitly',
            zig: 'Use error unions (!T) and handle errors explicitly',
            go: 'Use (val, err) returns and handle errors explicitly',
        },
        'binding.pattern.array': {
            go: 'Use explicit indexing assignments',
            zig: 'Use explicit indexing assignments',
        },
        'binding.pattern.object': {
            py: 'Use explicit field/dict access',
            zig: 'Use explicit field access',
            cpp: 'Use explicit member access',
            go: 'Use explicit field access',
        },
        'params.defaultValues': {
            rs: 'Use Option<T> params or provide helper overloads',
            zig: 'Use optional params (?T) and handle null',
            go: 'Use variadic params or an options struct',
        },
        'expressions.nullableMemberAccess': {
            py: 'Use explicit None checks',
            rs: 'Use Option combinators (map/and_then)',
            zig: 'Use explicit null checks',
            cpp: 'Use std::optional and explicit checks',
            go: 'Use explicit nil checks',
        },
    };

    const targetSuggestions = suggestions[feature];
    if (targetSuggestions?.[target]) {
        return targetSuggestions[target];
    }

    return 'Refactor to avoid this construct';
}
```

### 4. Integration

Add validation **before codegen**.

Important integration note: prefer reporting these as normal compiler diagnostics (accumulated alongside parse/semantic errors), not as a thrown exception. Throwing a `TargetCompatibilityError` is acceptable as an implementation shortcut, but the goal is consistent error formatting and multi-error reporting.

```typescript
// fons/faber/codegen/index.ts

export function generate(program: Program, target: CodegenTarget = 'ts', options: CodegenOptions = {}): string {
    // Validate target compatibility BEFORE attempting codegen.
    // (Policy can later be wired to CLI flags like --allow-emulated.)
    const validationErrors = validateTargetCompatibility(program, target);

    if (validationErrors.length > 0) {
        throw new TargetCompatibilityError(validationErrors, target);
    }

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
        public target: CodegenTarget,
    ) {
        const formatted = formatValidationErrors(errors, target);
        super(`Target compatibility errors for '${target}':\n\n${formatted}`);
        this.name = 'TargetCompatibilityError';
    }
}

function formatValidationErrors(errors: ValidationError[], target: CodegenTarget): string {
    return errors
        .map(err => {
            const pos = err.position ? `${err.position.line}:${err.position.column}` : 'unknown';

            const context = err.context ? ` (in ${err.context})` : '';
            const msg = `error: ${err.message}${context}`;
            const location = `  --> ${pos}`;
            const suggestion = err.suggestion ? `  = help: ${err.suggestion}` : '';

            return [location, msg, suggestion].filter(Boolean).join('\n');
        })
        .join('\n\n');
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

1. Create a table-driven suite in `fons/proba/capabilities/` (small programs + expected outcomes)
2. Start with targeted assertions per feature (avoid full feature×target matrix explosion)
3. Assert on feature key + primary message (wording can evolve without rewriting tests)
4. Add a small set of interaction tests (async+generator, pattern binding in params vs locals, etc.)

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

Not all incompatibilities are the same. Model them explicitly:

- `supported`: native, faithful semantics
- `emulated`: possible with a systematic transform/polyfill (may have perf/ergonomics cost)
- `mismatched`: can be “made to work” but semantics differ (dangerous by default)
- `unsupported`: cannot be emitted without changing the language model

Initial implementation policy:

- Treat `emulated` and `mismatched` as **errors** by default.
- Future: add a `--compat-mode`/`--allow-emulated`/`--allow-mismatched` flag that relaxes the policy and requires explicit opt-in.

### 3. Version-specific features

Go 1.23 has `iter.Seq[T]` for iteration. Zig 0.13 added labeled blocks. Rust nightly has generators.

Solution: Add optional `version` field to capabilities:

```typescript
export interface TargetVersion {
    target: CodegenTarget;
    version?: string; // Semantic version or "nightly"
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
- Single source of truth for target constraints
- Explicit handling of emulation vs semantic mismatch
- Easy extension as new targets are added
- Consistent with PL industry standards

Implementation can proceed once approved.
