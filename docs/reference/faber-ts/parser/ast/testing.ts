/**
 * AST Testing Types - Test infrastructure declarations (proba, probandum, praepara, cura)
 *
 * @module parser/ast/testing
 */

import type { BaseNode } from './base';
import type { Expression, Identifier } from './expressions';
import type { TypeAnnotation } from './types';
import type { BlockStatement } from './control';
import type { CapeClause } from './actions';

// =============================================================================
// TEST DECLARATIONS
// =============================================================================

/**
 * Test suite declaration (probandum).
 *
 * GRAMMAR (in EBNF):
 *   probandumDecl := 'probandum' STRING '{' probandumBody '}'
 *   probandumBody := (anteBlock | postBlock | probandumDecl | probaStmt)*
 *
 * INVARIANT: name is the suite description string.
 * INVARIANT: body contains setup/teardown blocks, nested suites, and tests.
 *
 * WHY: Latin "probandum" (gerundive of probare) = "that which must be tested".
 *      Analogous to describe() in Jest/Vitest or test module in Zig.
 *
 * Target mappings:
 *   TypeScript: describe("name", () => { ... })
 *   Python:     class TestName: ...
 *   Zig:        test "name" { ... } (flattened with prefix)
 *   Rust:       mod tests { ... } (flattened with prefix)
 *   C++:        void test_name() { ... } (flattened with prefix)
 *
 * Examples:
 *   probandum "Tokenizer" {
 *       praepara { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 */
export interface ProbandumStatement extends BaseNode {
    type: 'ProbandumStatement';
    name: string;
    skip?: boolean;
    skipReason?: string;
    solum?: boolean;
    tags?: string[];
    body: (PraeparaBlock | ProbandumStatement | ProbaStatement)[];
}

/**
 * Test modifier for skipped or todo tests.
 *
 * WHY: Two modifiers:
 *   omitte = skip (imperative: "skip!")
 *   futurum = todo/pending (noun: "the future")
 */
export type ProbaModifier = 'omitte' | 'futurum';

/**
 * Individual test case (proba).
 *
 * GRAMMAR (in EBNF):
 *   probaStmt := probaAnnotation* 'proba' STRING blockStmt
 *   probaAnnotation := '@' ('omitte' | 'futurum' | 'solum' | 'tag' | 'temporis' | 'metior' | 'repete' | 'fragilis' | 'requirit' | 'solum_in') STRING?
 *
 * INVARIANT: name is the test description string.
 * INVARIANT: annotations control test execution behavior.
 * INVARIANT: body is the test block.
 *
 * WHY: Latin "proba" (imperative of probare) = "test!" / "prove!".
 *      Analogous to test() or it() in Jest/Vitest.
 *
 * Annotation meanings:
 *   @ omitte "reason"     - Skip test with reason
 *   @ futurum "reason"    - Mark as todo/pending
 *   @ solum               - Only run this test (exclusive)
 *   @ tag "name"          - Tag for filtering (can appear multiple times)
 *   @ temporis 5000       - Timeout in milliseconds
 *   @ metior              - Benchmark mode
 *   @ repete 100          - Repeat test N times
 *   @ fragilis 3          - Retry flaky test N times
 *   @ requirit "ENV_VAR"  - Skip if environment variable missing
 *   @ solum_in "darwin"   - Only run on specified platform
 *
 * Target mappings:
 *   TypeScript: __proba_suite_name() function + registry
 *   Python:     def test_name(): ...
 *   Zig:        test "name" { ... }
 *   Rust:       #[test] fn name() { ... }
 *   C++:        void test_name() { ... }
 *
 * Examples:
 *   proba "parses integers" { adfirma parse("42") est 42 }
 *   @ omitte "blocked by #42"
 *   proba "skipped test" { ... }
 *   @ tag "slow"
 *   @ temporis 10000
 *   proba "slow test" { ... }
 */
export interface ProbaStatement extends BaseNode {
    type: 'ProbaStatement';
    name: string;
    modifier?: ProbaModifier;
    modifierReason?: string;
    solum?: boolean;
    tags?: string[];
    temporis?: number;
    metior?: boolean;
    repete?: number;
    fragilis?: number;
    requirit?: string;
    solumIn?: string;
    body: BlockStatement;
}

/**
 * Timing for praepara/postpara blocks in test context.
 *
 * WHY: 'praepara' = setup (before), 'postpara' = teardown (after)
 */
export type PraeparaTiming = 'praepara' | 'postpara';

/**
 * Curator kinds for cura statements.
 *
 * WHY: Explicit curator kind declares resource management type.
 *      - arena: Arena allocator (memory freed on scope exit)
 *      - page: Page allocator (memory freed on scope exit)
 *      - (future: curator, liber, transactio, mutex, conexio)
 */
export type CuratorKind = 'arena' | 'page';

/**
 * Test setup/teardown block.
 *
 * GRAMMAR (in EBNF):
 *   praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
 *
 * INVARIANT: timing distinguishes setup (praepara) vs teardown (postpara).
 * INVARIANT: async flag distinguishes sync (-a) vs async (-bit) variants.
 * INVARIANT: omnia flag distinguishes all vs each.
 *
 * WHY: Latin "praepara" (prepare!) for test setup, "postpara" (cleanup!) for teardown.
 *      Uses -bit suffix for async (future tense), matching fit/fiet pattern.
 *
 * Target mappings:
 *   TypeScript: beforeEach() / beforeAll() / afterEach() / afterAll()
 *   Python:     @pytest.fixture / setup_module / teardown
 *   Zig:        inlined into each test
 *   Rust:       inlined into each test
 *   C++:        inlined into each test
 *
 * Examples:
 *   praepara { lexer = init() }
 *   praepara omnia { db = connect() }
 *   praeparabit omnia { db = cede connect() }
 *   postpara { cleanup() }
 *   postpara omnia { db.close() }
 *   postparabit omnia { cede db.close() }
 */
export interface PraeparaBlock extends BaseNode {
    type: 'PraeparaBlock';
    timing: PraeparaTiming;
    async: boolean;
    omnia: boolean;
    body: BlockStatement;
}

/**
 * Resource management statement.
 *
 * GRAMMAR (in EBNF):
 *   curaStmt := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
 *   curatorKind := 'arena' | 'page'
 *
 * INVARIANT: curatorKind is optional; when present, declares allocator type.
 * INVARIANT: resource is optional for allocator kinds (arena/page create their own).
 * INVARIANT: binding is the identifier that receives the resource/allocator.
 * INVARIANT: typeAnnotation is optional explicit type for the binding.
 * INVARIANT: async flag indicates fiet (async) vs pro/fit (sync).
 * INVARIANT: body is the scoped block where resource is used.
 * INVARIANT: catchClause is optional error handling.
 *
 * WHY: Latin "cura" (care, concern) + binding verb for scoped resources.
 *      - pro: neutral binding ("for")
 *      - fit: sync binding ("it becomes")
 *      - fiet: async binding ("it will become")
 *      Guarantees cleanup via solve() on scope exit.
 *
 * Target mappings:
 *   TypeScript: try { } finally { binding.solve?.(); } (allocators stripped)
 *   Python:     with expr as binding: ... (allocators stripped)
 *   Zig:        ArenaAllocator / PageAllocator with defer deinit()
 *   Rust:       RAII / Drop at scope end
 *
 * Examples:
 *   cura arena fixum mem { ... }                    // arena allocator
 *   cura page fixum mem { ... }                     // page allocator
 *   cura aperi("data.bin") fixum fd { lege(fd) }   // generic resource
 *   cura connect(url) fixum conn { ... }           // resource binding
 *   cura aperi("config.json") fixum File fd { }    // with type annotation
 */
export interface CuraStatement extends BaseNode {
    type: 'CuraStatement';
    curatorKind?: CuratorKind;
    resource?: Expression;
    binding: Identifier;
    typeAnnotation?: TypeAnnotation;
    async: boolean;
    mutable: boolean; // true for varia (let), false for fixum (const)
    body: BlockStatement;
    catchClause?: CapeClause;
}
