//! Pass 3 AST-to-HIR lowering for compiler-friendly program structure.
//!
//! This module is the boundary between parsed, name-resolved syntax and the
//! High-level Intermediate Representation consumed by type checking, analysis,
//! and backend code generation. It preserves source spans and resolver `DefId`s
//! where they already exist, creates synthetic ids for bindings that only become
//! concrete during lowering, and normalizes entrypoint/declaration layout so
//! later passes do not need to understand every surface syntax form.
//!
//! HIR lowering is intentionally not a semantic pass. It resolves the shape of
//! declarations, statements, local scopes, and desugared control-flow nodes, but
//! leaves type compatibility, exhaustiveness, reachability, and most target
//! legality to the semantic/typecheck passes that have the full HIR graph.
//!
//! ERROR STRATEGY
//! ==============
//! Lowering reports structural problems and continues whenever possible. Invalid
//! expressions become `HirExprKind::Error`, lowering-created bindings receive
//! synthetic `DefId`s, and every produced node keeps the best source span known
//! at that point. This lets later diagnostics report more than the first
//! lowering issue without guessing about source locations.
//!
//! INVARIANTS
//! ==========
//! - Top-level declarations lower into `HirProgram::items`; executable
//!   top-level statements lower into an explicit or implicit entry block.
//! - Resolver-owned names keep their resolver `DefId`; lowering-created locals,
//!   catch bindings, pattern bindings, and CLI bindings receive synthetic ids.
//! - The lowerer scope stack only tracks function/block-local names. Module and
//!   item lookup stays with the resolver.
//! - HIR construction preserves AST spans, falling back to the current lowering
//!   span only for nodes synthesized from a larger source form.
//!
//! MODULE SHAPE
//! ============
//! Declaration, statement, expression, pattern, and type lowering live in
//! focused sibling modules because each owns a different compiler contract:
//! declarations build item-level HIR, statements normalize executable flow,
//! expressions preserve evaluation shape, patterns bind names, and types map
//! parsed type syntax into the semantic type table.

mod decl;
mod expr;
mod pattern;
mod stmt;
mod types;

use super::{
    HirBlock, HirExpr, HirExprKind, HirId, HirItem, HirProgram, HirStmt, HirStmtKind, HirTestMetadata, HirTestModifier,
};
use crate::cli::{CliProgram, CliType};
use crate::lexer::{Interner, Span, Symbol};
use crate::semantic::{Primitive, Resolver, Type, TypeId, TypeTable};
use crate::syntax::{PraeparaBlock, PraeparaKind, ProbaCase, ProbaModifier, ProbandumDecl, Program, Stmt, StmtKind};
use rustc_hash::FxHashMap;

/// Stateful coordinator for one AST-to-HIR lowering run.
///
/// A `Lowerer` borrows the resolver and type table for the duration of one
/// program. The resolver remains the source of truth for item-level symbols,
/// while this type owns per-run HIR ids, synthetic local `DefId`s, collected
/// lowering diagnostics, local scopes, and optional CLI metadata that can shape
/// generated entrypoint argument records.
pub struct Lowerer<'a> {
    /// Resolver produced by the previous compiler pass.
    resolver: &'a Resolver,

    /// Shared semantic type table used when lowering type syntax and CLI records.
    types: &'a mut TypeTable,

    /// Symbol interner used for annotation and diagnostic-sensitive spelling.
    interner: &'a Interner,

    /// Next per-HIR-node id assigned during this lowering run.
    next_id: u32,

    /// Next synthetic definition id for bindings that did not exist in resolver output.
    next_def_id: u32,

    /// Recoverable lowering diagnostics collected instead of aborting the pass.
    errors: Vec<LowerError>,

    /// Best source span for diagnostics emitted by helpers without a narrower span.
    current_span: Span,

    /// Stack of function/block-local bindings visible only during lowering.
    local_scopes: Vec<FxHashMap<Symbol, crate::hir::DefId>>,

    /// Current self type for lowering `ego` expressions inside methods.
    current_ego_struct: Option<crate::hir::DefId>,

    /// Validated CLI contract, when the entry point is a runnable CLI.
    cli_program: Option<&'a CliProgram>,
}

/// Recoverable diagnostic emitted while converting resolved AST into HIR.
///
/// Lowering errors describe shape problems, unsupported constructs, or missing
/// context discovered before type checking. They intentionally keep only a
/// message and source span because richer semantic explanations belong to later
/// passes once HIR and type information are available.
#[derive(Debug, Clone)]
pub struct LowerError {
    /// Human-readable diagnostic text.
    pub message: String,

    /// Source span associated with the problematic AST or synthesized node.
    pub span: Span,
}

impl<'a> Lowerer<'a> {
    /// Create a lowerer for one program.
    ///
    /// `cli_program` is optional because ordinary file compilation and package
    /// CLI compilation share the same lowering pipeline. When present, it is
    /// used only to type synthetic entrypoint/command argument records.
    pub fn new(
        resolver: &'a Resolver,
        types: &'a mut TypeTable,
        interner: &'a Interner,
        cli_program: Option<&'a CliProgram>,
    ) -> Self {
        Self {
            resolver,
            types,
            interner,
            next_id: 0,
            next_def_id: 1_000_000,
            errors: Vec::new(),
            current_span: Span::default(),
            local_scopes: Vec::new(),
            current_ego_struct: None,
            cli_program,
        }
    }

    /// Lower a complete resolved syntax program into HIR.
    ///
    /// The program boundary is where declaration and entrypoint partitioning is
    /// established. Explicit `incipit` owns `HirProgram::entry`; otherwise
    /// executable top-level statements become an implicit entry block, while
    /// top-level declarations stay as items. Later passes can therefore treat
    /// HIR like a target-language module with a separate runnable body.
    pub fn lower_program(&mut self, program: &Program) -> HirProgram {
        let mut items = Vec::new();
        let mut entry = None;
        let mut implicit_entry_stmts = Vec::new();
        let mut implicit_entry_scope = false;

        for stmt in &program.stmts {
            self.current_span = stmt.span;

            match &stmt.kind {
                StmtKind::Incipit(entry_stmt) => {
                    // `incipit` introduces an entry-local scope and may synthesize
                    // a CLI args local before the user-authored body statements.
                    self.push_scope();
                    let mut args_binding = None;
                    if let Some(args) = &entry_stmt.args {
                        let def_id = self.next_def_id();
                        self.bind_local(args.name, def_id);
                        args_binding = Some((args.name, args.span, def_id));
                    }
                    let mut block = self.lower_ergo_body(&entry_stmt.body);
                    if let Some((name, span, def_id)) = args_binding {
                        block.stmts.insert(
                            0,
                            HirStmt {
                                id: self.next_hir_id(),
                                kind: HirStmtKind::Local(crate::hir::HirLocal {
                                    def_id,
                                    name,
                                    ty: Some(self.incipit_args_type()),
                                    init: None,
                                    mutable: false,
                                }),
                                span,
                            },
                        );
                    }
                    self.pop_scope();
                    entry = Some(block);
                }
                _ => {
                    let lowered_items = self.lower_stmt_items(stmt);
                    if !lowered_items.is_empty() {
                        items.extend(lowered_items);
                    } else {
                        if !implicit_entry_scope {
                            self.push_scope();
                            implicit_entry_scope = true;
                        }
                        implicit_entry_stmts.push(stmt::lower_stmt(self, stmt));
                    }
                }
            }
        }

        if implicit_entry_scope {
            self.pop_scope();
        }

        if entry.is_none() && !implicit_entry_stmts.is_empty() {
            let span = implicit_entry_stmts
                .first()
                .map(|stmt| stmt.span)
                .unwrap_or_else(Span::default);
            entry = Some(HirBlock { stmts: implicit_entry_stmts, expr: None, span });
        }

        HirProgram { items, entry }
    }

    /// Drain diagnostics accumulated during the lowering run.
    pub fn take_errors(&mut self) -> Vec<LowerError> {
        std::mem::take(&mut self.errors)
    }

    /// Allocate a per-node HIR id.
    fn next_hir_id(&mut self) -> HirId {
        let id = self.next_id;
        self.next_id += 1;
        HirId(id)
    }

    /// Allocate a synthetic `DefId` for a binding introduced during lowering.
    pub(super) fn next_def_id(&mut self) -> crate::hir::DefId {
        let id = self.next_def_id;
        self.next_def_id += 1;
        crate::hir::DefId(id)
    }

    /// Resolve an item-level name, falling back to a synthetic id for recovery.
    ///
    /// A fallback here keeps malformed or partially resolved programs lowering
    /// far enough for later passes to attach more diagnostics to the same HIR.
    pub(super) fn def_id_for(&mut self, name: Symbol) -> crate::hir::DefId {
        self.resolver
            .lookup(name)
            .unwrap_or_else(|| self.next_def_id())
    }

    pub(super) fn push_scope(&mut self) {
        self.local_scopes.push(FxHashMap::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    pub(super) fn bind_local(&mut self, name: Symbol, def_id: crate::hir::DefId) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, def_id);
        }
    }

    pub(super) fn lookup_name(&self, name: Symbol) -> Option<crate::hir::DefId> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(def_id) = scope.get(&name) {
                return Some(*def_id);
            }
        }
        self.resolver.lookup(name)
    }

    pub(super) fn incipit_args_type(&mut self) -> TypeId {
        if let Some(cli) = self.cli_program {
            let mut fields = FxHashMap::default();
            for option in cli.global_options.iter().chain(cli.options.iter()) {
                let ty = self.cli_value_type(&option.ty, option.default.is_none() && !option.flag, false);
                fields.insert(option.binding_symbol, ty);
            }
            for operand in cli.global_operands.iter().chain(cli.operands.iter()) {
                let ty = self.cli_value_type(&operand.ty, false, operand.rest);
                fields.insert(operand.binding_symbol, ty);
            }
            return self.types.intern(Type::Record(fields));
        }

        let textus = self.types.primitive(Primitive::Textus);
        self.types.array(textus)
    }

    /// Build the synthetic argument record type for a mounted CLI command.
    ///
    /// A command function receives already-parsed CLI data as a record. Missing
    /// CLI metadata falls back to `ignotum` so ordinary compilation does not
    /// need a second lowering path.
    pub(super) fn command_args_type(&mut self, function: Symbol) -> TypeId {
        if let Some(cli) = self.cli_program {
            if let Some(command) = cli
                .commands
                .iter()
                .find(|command| command.module_path.is_none() && command.function_symbol == function)
            {
                let mut fields = FxHashMap::default();
                for option in cli.global_options.iter().chain(command.options.iter()) {
                    let ty = self.cli_value_type(&option.ty, option.default.is_none() && !option.flag, false);
                    fields.insert(option.binding_symbol, ty);
                }
                for operand in cli.global_operands.iter().chain(command.operands.iter()) {
                    let ty = self.cli_value_type(&operand.ty, false, operand.rest);
                    fields.insert(operand.binding_symbol, ty);
                }
                return self.types.intern(Type::Record(fields));
            }
        }

        self.types.primitive(Primitive::Ignotum)
    }

    /// Translate validated CLI value metadata into a HIR-visible semantic type.
    ///
    /// Optionality here means "the parser may omit this value", not nullable
    /// source syntax. Rest operands become arrays unless the CLI type is already
    /// list-shaped.
    fn cli_value_type(&mut self, ty: &CliType, optional: bool, rest: bool) -> TypeId {
        let base = match ty {
            CliType::Textus | CliType::Ignotum => self.types.primitive(Primitive::Textus),
            CliType::Numerus => self.types.primitive(Primitive::Numerus),
            CliType::Fractus => self.types.primitive(Primitive::Fractus),
            CliType::Bivalens => self.types.primitive(Primitive::Bivalens),
            CliType::Octeti => self.types.primitive(Primitive::Octeti),
            CliType::ListaTextus => {
                let textus = self.types.primitive(Primitive::Textus);
                self.types.array(textus)
            }
            CliType::ListaNumerus => {
                let numerus = self.types.primitive(Primitive::Numerus);
                self.types.array(numerus)
            }
        };
        let value = if rest && !matches!(ty, CliType::ListaTextus | CliType::ListaNumerus) {
            self.types.array(base)
        } else {
            base
        };
        if optional {
            self.types.option(value)
        } else {
            value
        }
    }

    /// Record a recoverable lowering diagnostic at the current source span.
    fn error(&mut self, message: impl Into<String>) {
        self.errors
            .push(LowerError { message: message.into(), span: self.current_span });
    }

    /// Lower a top-level syntax statement when it represents one or more items.
    ///
    /// Non-item statements return an empty vector so `lower_program` can route
    /// them into the explicit or implicit entry block.
    fn lower_stmt_items(&mut self, stmt: &Stmt) -> Vec<HirItem> {
        match &stmt.kind {
            StmtKind::Var(decl) => self.lower_varia(stmt, decl).into_iter().collect(),
            StmtKind::Func(decl) => self.lower_functio(stmt, decl).into_iter().collect(),
            StmtKind::Class(decl) => self.lower_gens(stmt, decl).into_iter().collect(),
            StmtKind::Enum(decl) => self.lower_ordo(stmt, decl).into_iter().collect(),
            StmtKind::Union(decl) => self.lower_discretio(stmt, decl).into_iter().collect(),
            StmtKind::Interface(decl) => self.lower_pactum(stmt, decl).into_iter().collect(),
            StmtKind::TypeAlias(decl) => self.lower_typus(stmt, decl).into_iter().collect(),
            StmtKind::Import(decl) => self.lower_importa(stmt, decl).into_iter().collect(),
            StmtKind::Proba(case) => vec![self.lower_proba_item(case, &[], &[], &[])],
            StmtKind::Probandum(suite) => {
                let mut items = Vec::new();
                self.lower_probandum_items(suite, &[], &[], &[], &mut items);
                items
            }
            _ => Vec::new(),
        }
    }

    fn lower_probandum_items(
        &mut self,
        suite: &ProbandumDecl,
        inherited_setup: &[&PraeparaBlock],
        inherited_modifiers: &[ProbaModifier],
        suite_path: &[Symbol],
        out: &mut Vec<HirItem>,
    ) {
        // Only `praepara ... omnia` setup/teardown blocks flow into nested
        // suites. Local setup blocks are intentionally owned by the suite level
        // where they were written so generated tests do not inherit more fixture
        // work than the source requested.
        let mut combined_setup = inherited_setup.to_vec();
        for setup in &suite.body.setup {
            if setup.all {
                combined_setup.push(setup);
            }
        }

        let mut combined_suite_path = suite_path.to_vec();
        combined_suite_path.push(suite.name);

        let mut combined_modifiers = inherited_modifiers.to_vec();
        combined_modifiers.extend(suite.modifiers.iter().cloned());

        for case in &suite.body.tests {
            out.push(self.lower_proba_item(case, &combined_setup, &combined_modifiers, &combined_suite_path));
        }

        for nested in &suite.body.nested {
            self.lower_probandum_items(nested, &combined_setup, &combined_modifiers, &combined_suite_path, out);
        }
    }

    fn lower_proba_item(
        &mut self,
        case: &ProbaCase,
        inherited_setup: &[&PraeparaBlock],
        inherited_modifiers: &[ProbaModifier],
        suite_path: &[Symbol],
    ) -> HirItem {
        let def_id = self.next_def_id();
        self.push_scope();

        // Test cases become ordinary zero-argument HIR functions with metadata.
        // Setup and teardown blocks are spliced into the function body here so
        // typecheck and codegen can use the normal function path instead of a
        // separate test-body AST model.
        let mut stmts = Vec::new();
        for setup in inherited_setup {
            if matches!(setup.kind, PraeparaKind::Praepara | PraeparaKind::Praeparabit) {
                stmts.extend(self.lower_block(&setup.body).stmts);
            }
        }
        stmts.extend(self.lower_block(&case.body).stmts);
        for setup in inherited_setup {
            if matches!(setup.kind, PraeparaKind::Postpara | PraeparaKind::Postparabit) {
                stmts.extend(self.lower_block(&setup.body).stmts);
            }
        }

        self.pop_scope();

        let mut modifiers = inherited_modifiers.to_vec();
        modifiers.extend(case.modifiers.iter().cloned());

        HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: crate::hir::HirItemKind::Function(crate::hir::HirFunction {
                name: case.name,
                type_params: Vec::new(),
                params: Vec::new(),
                cli_args: None,
                ret_ty: Some(self.types.primitive(crate::semantic::Primitive::Vacuum)),
                err_ty: None,
                body: Some(HirBlock { stmts, expr: None, span: case.span }),
                is_async: false,
                is_generator: false,
                test: Some(HirTestMetadata {
                    name: case.name,
                    suite_path: suite_path.to_vec(),
                    modifiers: Self::lower_test_modifiers(&modifiers),
                    span: case.span,
                }),
            }),
            span: case.span,
        }
    }

    fn lower_test_modifiers(modifiers: &[ProbaModifier]) -> Vec<HirTestModifier> {
        modifiers.iter().map(Self::lower_test_modifier).collect()
    }

    /// Preserve test-selection metadata without interpreting harness policy.
    fn lower_test_modifier(modifier: &ProbaModifier) -> HirTestModifier {
        match modifier {
            ProbaModifier::Omitte(reason) => HirTestModifier::Omitte(*reason),
            ProbaModifier::Futurum(reason) => HirTestModifier::Futurum(*reason),
            ProbaModifier::Solum => HirTestModifier::Solum,
            ProbaModifier::Tag(tag) => HirTestModifier::Tag(*tag),
            ProbaModifier::Temporis(n) => HirTestModifier::Temporis(*n),
            ProbaModifier::Metior => HirTestModifier::Metior,
            ProbaModifier::Repete(n) => HirTestModifier::Repete(*n),
            ProbaModifier::Fragilis(n) => HirTestModifier::Fragilis(*n),
            ProbaModifier::Requirit(req) => HirTestModifier::Requirit(*req),
            ProbaModifier::SolumIn(env) => HirTestModifier::SolumIn(*env),
        }
    }

    /// Lower either a brace block or single-statement `ergo` body to a block.
    ///
    /// Control-flow consumers downstream see one HIR block shape regardless of
    /// whether source used Stroustrup braces or the compact single-statement
    /// form.
    fn lower_ergo_body(&mut self, body: &crate::syntax::IfBody) -> HirBlock {
        match body {
            crate::syntax::IfBody::Block(block) => self.lower_block(block),
            crate::syntax::IfBody::Ergo(stmt) => {
                let stmts = stmt::lower_stmt_expanded(self, stmt);
                HirBlock { stmts, expr: None, span: self.current_span }
            }
        }
    }

    /// Lower a lexical block with its own local scope.
    fn lower_block(&mut self, block: &crate::syntax::BlockStmt) -> HirBlock {
        self.current_span = block.span;
        let mut stmts = Vec::new();
        self.push_scope();

        for stmt in &block.stmts {
            stmts.extend(stmt::lower_stmt_expanded(self, stmt));
        }
        self.pop_scope();

        HirBlock { stmts, expr: None, span: block.span }
    }

    /// Lower an expression (delegates to expr.rs)
    fn lower_expr(&mut self, expr: &crate::syntax::Expr) -> HirExpr {
        expr::lower_expr(self, expr)
    }
}

/// Lower resolved AST to HIR without package CLI metadata.
///
/// This is the ordinary compiler entrypoint for sources whose entry arguments
/// are the default `lista<textus>` shape rather than a validated CLI contract.
pub fn lower(
    program: &Program,
    resolver: &Resolver,
    types: &mut TypeTable,
    interner: &Interner,
) -> (HirProgram, Vec<LowerError>) {
    lower_with_cli(program, resolver, types, interner, None)
}

/// Lower resolved AST to HIR with an optional validated CLI contract.
///
/// Package CLI analysis runs before lowering and can provide record-shaped
/// argument metadata for `incipit` and command functions. The HIR still carries
/// ordinary locals and type ids; no CLI-specific semantic checks are performed
/// here beyond translating the already validated contract into HIR-friendly
/// binding types.
pub fn lower_with_cli(
    program: &Program,
    resolver: &Resolver,
    types: &mut TypeTable,
    interner: &Interner,
    cli_program: Option<&CliProgram>,
) -> (HirProgram, Vec<LowerError>) {
    let mut lowerer = Lowerer::new(resolver, types, interner, cli_program);
    let hir = lowerer.lower_program(program);
    let errors = lowerer.take_errors();
    (hir, errors)
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
