//! Pass 2: Name resolution
//!
//! Resolves all identifiers to their definitions.

use crate::lexer::Interner;
use crate::semantic::{
    Resolver, ScopeKind, SemanticError, SemanticErrorKind, Symbol, SymbolKind, TypeTable,
};
use crate::syntax::{
    BindingPattern, BlockStmt, ClausuraBody, DiscerneStmt, Expr, ExprKind, IfBody, Pattern,
    PatternBind, ProbandumDecl, Program, SiStmt, Stmt, StmtKind, TypeExpr, TypeExprKind,
};

/// Resolve all names in the program
pub fn resolve(
    program: &Program,
    resolver: &mut Resolver,
    interner: &Interner,
    _types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();

    for stmt in &program.stmts {
        resolve_stmt(resolver, interner, stmt, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_stmt(
    resolver: &mut Resolver,
    interner: &Interner,
    stmt: &Stmt,
    errors: &mut Vec<SemanticError>,
) {
    let stmt_span = stmt.span;
    match &stmt.kind {
        StmtKind::Var(decl) => {
            if let Some(ty) = &decl.ty {
                resolve_type(resolver, interner, ty, errors);
            }
            if let Some(init) = &decl.init {
                resolve_expr(resolver, interner, init, errors);
            }
            define_binding_pattern(
                resolver,
                &decl.binding,
                decl.mutability == crate::syntax::Mutability::Mutable,
                errors,
            );
        }
        StmtKind::Func(decl) => {
            resolver.enter_scope(ScopeKind::Function);
            for param in &decl.type_params {
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::TypeParam,
                    false,
                    errors,
                );
            }
            for param in &decl.params {
                resolve_type(resolver, interner, &param.ty, errors);
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::Param,
                    param.mode == crate::syntax::ParamMode::MutRef,
                    errors,
                );
                if let Some(default) = &param.default {
                    resolve_expr(resolver, interner, default, errors);
                }
            }
            if let Some(ret) = &decl.ret {
                resolve_type(resolver, interner, ret, errors);
            }
            if let Some(body) = &decl.body {
                resolve_block(resolver, interner, body, errors);
            }
            resolver.exit_scope();
        }
        StmtKind::Class(decl) => {
            resolver.enter_scope(ScopeKind::Module);
            for param in &decl.type_params {
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::TypeParam,
                    false,
                    errors,
                );
            }
            if let Some(base) = &decl.extends {
                resolve_type_ident(resolver, interner, base, errors);
            }
            for iface in &decl.implements {
                resolve_type_ident(resolver, interner, iface, errors);
            }
            for member in &decl.members {
                match &member.kind {
                    crate::syntax::ClassMemberKind::Field(field) => {
                        resolve_type(resolver, interner, &field.ty, errors);
                        if let Some(init) = &field.init {
                            resolve_expr(resolver, interner, init, errors);
                        }
                    }
                    crate::syntax::ClassMemberKind::Method(method) => {
                        resolver.enter_scope(ScopeKind::Function);
                        for param in &method.type_params {
                            define_symbol(
                                resolver,
                                param.name.name,
                                param.name.span,
                                SymbolKind::TypeParam,
                                false,
                                errors,
                            );
                        }
                        for param in &method.params {
                            resolve_type(resolver, interner, &param.ty, errors);
                            define_symbol(
                                resolver,
                                param.name.name,
                                param.name.span,
                                SymbolKind::Param,
                                param.mode == crate::syntax::ParamMode::MutRef,
                                errors,
                            );
                            if let Some(default) = &param.default {
                                resolve_expr(resolver, interner, default, errors);
                            }
                        }
                        if let Some(ret) = &method.ret {
                            resolve_type(resolver, interner, ret, errors);
                        }
                        if let Some(body) = &method.body {
                            resolve_block(resolver, interner, body, errors);
                        }
                        resolver.exit_scope();
                    }
                }
            }
            resolver.exit_scope();
        }
        StmtKind::Interface(decl) => {
            resolver.enter_scope(ScopeKind::Module);
            for param in &decl.type_params {
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::TypeParam,
                    false,
                    errors,
                );
            }
            for method in &decl.methods {
                resolver.enter_scope(ScopeKind::Function);
                for param in &method.params {
                    resolve_type(resolver, interner, &param.ty, errors);
                    define_symbol(
                        resolver,
                        param.name.name,
                        param.name.span,
                        SymbolKind::Param,
                        param.mode == crate::syntax::ParamMode::MutRef,
                        errors,
                    );
                }
                if let Some(ret) = &method.ret {
                    resolve_type(resolver, interner, ret, errors);
                }
                resolver.exit_scope();
            }
            resolver.exit_scope();
        }
        StmtKind::TypeAlias(decl) => {
            resolve_type(resolver, interner, &decl.ty, errors);
        }
        StmtKind::Enum(decl) => {
            let _ = decl;
        }
        StmtKind::Union(decl) => {
            resolver.enter_scope(ScopeKind::Module);
            for param in &decl.type_params {
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::TypeParam,
                    false,
                    errors,
                );
            }
            for variant in &decl.variants {
                for field in &variant.fields {
                    resolve_type(resolver, interner, &field.ty, errors);
                }
            }
            resolver.exit_scope();
        }
        StmtKind::Import(_decl) => {}
        StmtKind::Block(block) => resolve_block(resolver, interner, block, errors),
        StmtKind::Expr(expr) => resolve_expr(resolver, interner, &expr.expr, errors),
        StmtKind::Si(stmt) => resolve_si_stmt(resolver, interner, stmt, errors),
        StmtKind::Dum(stmt) => {
            resolve_expr(resolver, interner, &stmt.cond, errors);
            resolver.enter_scope(ScopeKind::Loop);
            resolve_if_body(resolver, interner, &stmt.body, errors);
            resolver.exit_scope();
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
        }
        StmtKind::Itera(stmt) => {
            resolve_expr(resolver, interner, &stmt.iterable, errors);
            resolver.enter_scope(ScopeKind::Loop);
            define_symbol(
                resolver,
                stmt.binding.name,
                stmt.binding.span,
                SymbolKind::Local,
                stmt.mutability == crate::syntax::Mutability::Mutable,
                errors,
            );
            resolve_if_body(resolver, interner, &stmt.body, errors);
            resolver.exit_scope();
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
        }
        StmtKind::Elige(stmt) => {
            resolve_expr(resolver, interner, &stmt.expr, errors);
            for case in &stmt.cases {
                resolve_expr(resolver, interner, &case.value, errors);
                resolve_if_body(resolver, interner, &case.body, errors);
            }
            if let Some(default) = &stmt.default {
                resolve_if_body(resolver, interner, &default.body, errors);
            }
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
        }
        StmtKind::Discerne(stmt) => resolve_discerne(resolver, interner, stmt, errors),
        StmtKind::Custodi(stmt) => {
            for clause in &stmt.clauses {
                resolve_expr(resolver, interner, &clause.cond, errors);
                resolve_if_body(resolver, interner, &clause.body, errors);
            }
        }
        StmtKind::Fac(stmt) => {
            resolver.enter_scope(ScopeKind::Block);
            resolve_block(resolver, interner, &stmt.body, errors);
            resolver.exit_scope();
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
            if let Some(cond) = &stmt.while_ {
                resolve_expr(resolver, interner, cond, errors);
            }
        }
        StmtKind::Redde(stmt) => {
            if !resolver.in_function() {
                errors.push(SemanticError::new(
                    SemanticErrorKind::ReturnOutsideFunction,
                    "redde outside function",
                    stmt.value
                        .as_ref()
                        .map(|value| value.span)
                        .unwrap_or(stmt_span),
                ));
            }
            if let Some(value) = &stmt.value {
                resolve_expr(resolver, interner, value, errors);
            }
        }
        StmtKind::Rumpe(stmt) => {
            if !resolver.in_loop() {
                errors.push(SemanticError::new(
                    SemanticErrorKind::BreakOutsideLoop,
                    "rumpe outside loop",
                    stmt.span,
                ));
            }
        }
        StmtKind::Perge(stmt) => {
            if !resolver.in_loop() {
                errors.push(SemanticError::new(
                    SemanticErrorKind::ContinueOutsideLoop,
                    "perge outside loop",
                    stmt.span,
                ));
            }
        }
        StmtKind::Iace(stmt) => resolve_expr(resolver, interner, &stmt.value, errors),
        StmtKind::Mori(stmt) => resolve_expr(resolver, interner, &stmt.value, errors),
        StmtKind::Tempta(stmt) => {
            resolve_block(resolver, interner, &stmt.body, errors);
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
            if let Some(finally) = &stmt.finally {
                resolve_block(resolver, interner, finally, errors);
            }
        }
        StmtKind::Adfirma(stmt) => {
            resolve_expr(resolver, interner, &stmt.cond, errors);
            if let Some(message) = &stmt.message {
                resolve_expr(resolver, interner, message, errors);
            }
        }
        StmtKind::Scribe(stmt) => {
            for arg in &stmt.args {
                resolve_expr(resolver, interner, arg, errors);
            }
        }
        StmtKind::Incipit(stmt) => {
            resolver.enter_scope(ScopeKind::Function);
            if let Some(args) = &stmt.args {
                define_symbol(
                    resolver,
                    args.name,
                    args.span,
                    SymbolKind::Param,
                    false,
                    errors,
                );
            }
            if let Some(exitus) = &stmt.exitus {
                resolve_expr(resolver, interner, exitus, errors);
            }
            resolve_if_body(resolver, interner, &stmt.body, errors);
            resolver.exit_scope();
        }
        StmtKind::Cura(stmt) => {
            if let Some(init) = &stmt.init {
                resolve_expr(resolver, interner, init, errors);
            }
            if let Some(ty) = &stmt.ty {
                resolve_type(resolver, interner, ty, errors);
            }
            resolver.enter_scope(ScopeKind::Block);
            define_symbol(
                resolver,
                stmt.binding.name,
                stmt.binding.span,
                SymbolKind::Local,
                stmt.mutability == crate::syntax::Mutability::Mutable,
                errors,
            );
            resolve_block(resolver, interner, &stmt.body, errors);
            resolver.exit_scope();
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
        }
        StmtKind::Ad(stmt) => {
            for arg in &stmt.args {
                resolve_expr(resolver, interner, &arg.value, errors);
            }
            if let Some(binding) = &stmt.binding {
                if let Some(ty) = &binding.ty {
                    resolve_type(resolver, interner, ty, errors);
                }
                define_symbol(
                    resolver,
                    binding.name.name,
                    binding.name.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
            }
            if let Some(body) = &stmt.body {
                resolve_block(resolver, interner, body, errors);
            }
            if let Some(catch) = &stmt.catch {
                resolver.enter_scope(ScopeKind::Block);
                define_symbol(
                    resolver,
                    catch.binding.name,
                    catch.binding.span,
                    SymbolKind::Local,
                    false,
                    errors,
                );
                resolve_block(resolver, interner, &catch.body, errors);
                resolver.exit_scope();
            }
        }
        StmtKind::Probandum(test) => resolve_probandum(resolver, interner, test, errors),
        StmtKind::Proba(case) => {
            resolve_block(resolver, interner, &case.body, errors);
        }
        StmtKind::Ex(stmt) => {
            resolve_expr(resolver, interner, &stmt.source, errors);
            for field in &stmt.fields {
                if let Some(alias) = &field.alias {
                    define_symbol(
                        resolver,
                        alias.name,
                        alias.span,
                        SymbolKind::Local,
                        stmt.mutability == crate::syntax::Mutability::Mutable,
                        errors,
                    );
                } else {
                    define_symbol(
                        resolver,
                        field.name.name,
                        field.name.span,
                        SymbolKind::Local,
                        stmt.mutability == crate::syntax::Mutability::Mutable,
                        errors,
                    );
                }
            }
            if let Some(rest) = &stmt.rest {
                define_symbol(
                    resolver,
                    rest.name,
                    rest.span,
                    SymbolKind::Local,
                    stmt.mutability == crate::syntax::Mutability::Mutable,
                    errors,
                );
            }
        }
    }
}

fn resolve_block(
    resolver: &mut Resolver,
    interner: &Interner,
    block: &BlockStmt,
    errors: &mut Vec<SemanticError>,
) {
    resolver.enter_scope(ScopeKind::Block);
    for stmt in &block.stmts {
        resolve_stmt(resolver, interner, stmt, errors);
    }
    resolver.exit_scope();
}

fn resolve_if_body(
    resolver: &mut Resolver,
    interner: &Interner,
    body: &IfBody,
    errors: &mut Vec<SemanticError>,
) {
    match body {
        IfBody::Block(block) => resolve_block(resolver, interner, block, errors),
        IfBody::Ergo(stmt) => resolve_stmt(resolver, interner, stmt, errors),
        IfBody::InlineReturn(ret) => match ret {
            crate::syntax::InlineReturn::Reddit(expr)
            | crate::syntax::InlineReturn::Iacit(expr)
            | crate::syntax::InlineReturn::Moritor(expr) => {
                resolve_expr(resolver, interner, expr, errors)
            }
            crate::syntax::InlineReturn::Tacet => {}
        },
    }
}

fn resolve_si_stmt(
    resolver: &mut Resolver,
    interner: &Interner,
    stmt: &SiStmt,
    errors: &mut Vec<SemanticError>,
) {
    resolve_expr(resolver, interner, &stmt.cond, errors);
    resolve_if_body(resolver, interner, &stmt.then, errors);
    if let Some(catch) = &stmt.catch {
        resolver.enter_scope(ScopeKind::Block);
        define_symbol(
            resolver,
            catch.binding.name,
            catch.binding.span,
            SymbolKind::Local,
            false,
            errors,
        );
        resolve_block(resolver, interner, &catch.body, errors);
        resolver.exit_scope();
    }
    if let Some(else_) = &stmt.else_ {
        resolve_secus_clause(resolver, interner, else_, errors);
    }
}

fn resolve_probandum(
    resolver: &mut Resolver,
    interner: &Interner,
    test: &ProbandumDecl,
    errors: &mut Vec<SemanticError>,
) {
    resolver.enter_scope(ScopeKind::Module);
    for setup in &test.body.setup {
        resolve_block(resolver, interner, &setup.body, errors);
    }
    for case in &test.body.tests {
        resolve_block(resolver, interner, &case.body, errors);
    }
    for nested in &test.body.nested {
        resolve_probandum(resolver, interner, nested, errors);
    }
    resolver.exit_scope();
}

fn resolve_secus_clause(
    resolver: &mut Resolver,
    interner: &Interner,
    clause: &crate::syntax::SecusClause,
    errors: &mut Vec<SemanticError>,
) {
    match clause {
        crate::syntax::SecusClause::Sin(stmt) => resolve_si_stmt(resolver, interner, stmt, errors),
        crate::syntax::SecusClause::Block(block) => {
            resolve_block(resolver, interner, block, errors)
        }
        crate::syntax::SecusClause::Stmt(stmt) => resolve_stmt(resolver, interner, stmt, errors),
        crate::syntax::SecusClause::InlineReturn(ret) => match ret {
            crate::syntax::InlineReturn::Reddit(expr)
            | crate::syntax::InlineReturn::Iacit(expr)
            | crate::syntax::InlineReturn::Moritor(expr) => {
                resolve_expr(resolver, interner, expr, errors)
            }
            crate::syntax::InlineReturn::Tacet => {}
        },
    }
}

fn resolve_discerne(
    resolver: &mut Resolver,
    interner: &Interner,
    stmt: &DiscerneStmt,
    errors: &mut Vec<SemanticError>,
) {
    for subject in &stmt.subjects {
        resolve_expr(resolver, interner, subject, errors);
    }

    resolver.enter_scope(ScopeKind::Match);
    for arm in &stmt.arms {
        resolver.enter_scope(ScopeKind::Block);
        for pattern in &arm.patterns {
            resolve_pattern(resolver, interner, pattern, errors);
        }
        resolve_if_body(resolver, interner, &arm.body, errors);
        resolver.exit_scope();
    }
    if let Some(default) = &stmt.default {
        resolve_if_body(resolver, interner, &default.body, errors);
    }
    resolver.exit_scope();
}

fn resolve_pattern(
    resolver: &mut Resolver,
    interner: &Interner,
    pattern: &Pattern,
    errors: &mut Vec<SemanticError>,
) {
    match pattern {
        Pattern::Wildcard(_) => {}
        Pattern::Literal(_, _) => {}
        Pattern::Ident(ident, bind) => {
            define_symbol(
                resolver,
                ident.name,
                ident.span,
                SymbolKind::Local,
                false,
                errors,
            );
            resolve_pattern_bind(resolver, bind.as_ref(), errors);
        }
        Pattern::Path(path) => {
            if let Some(last) = path.segments.last() {
                resolve_variant_ident(resolver, last, errors);
            }
            resolve_pattern_bind(resolver, path.bind.as_ref(), errors);
        }
    }
}

fn resolve_pattern_bind(
    resolver: &mut Resolver,
    bind: Option<&PatternBind>,
    errors: &mut Vec<SemanticError>,
) {
    if let Some(bind) = bind {
        match bind {
            PatternBind::Alias(alias) => define_symbol(
                resolver,
                alias.name,
                alias.span,
                SymbolKind::Local,
                false,
                errors,
            ),
            PatternBind::Bindings { mutability, names } => {
                for name in names {
                    define_symbol(
                        resolver,
                        name.name,
                        name.span,
                        SymbolKind::Local,
                        *mutability == crate::syntax::Mutability::Mutable,
                        errors,
                    );
                }
            }
        }
    }
}

fn resolve_expr(
    resolver: &mut Resolver,
    interner: &Interner,
    expr: &Expr,
    errors: &mut Vec<SemanticError>,
) {
    match &expr.kind {
        ExprKind::Ident(ident) => {
            if resolver.lookup(ident.name).is_none() {
                errors.push(SemanticError::new(
                    SemanticErrorKind::UndefinedVariable,
                    "unknown identifier",
                    ident.span,
                ));
            }
        }
        ExprKind::Literal(_) => {}
        ExprKind::Binary(expr) => {
            resolve_expr(resolver, interner, &expr.lhs, errors);
            resolve_expr(resolver, interner, &expr.rhs, errors);
        }
        ExprKind::Unary(expr) => resolve_expr(resolver, interner, &expr.operand, errors),
        ExprKind::Ternary(expr) => {
            resolve_expr(resolver, interner, &expr.cond, errors);
            resolve_expr(resolver, interner, &expr.then, errors);
            resolve_expr(resolver, interner, &expr.else_, errors);
        }
        ExprKind::Call(expr) => {
            resolve_expr(resolver, interner, &expr.callee, errors);
            for arg in &expr.args {
                resolve_expr(resolver, interner, &arg.value, errors);
            }
        }
        ExprKind::Member(expr) => {
            resolve_expr(resolver, interner, &expr.object, errors);
        }
        ExprKind::Index(expr) => {
            resolve_expr(resolver, interner, &expr.object, errors);
            resolve_expr(resolver, interner, &expr.index, errors);
        }
        ExprKind::OptionalChain(expr) => {
            resolve_expr(resolver, interner, &expr.object, errors);
            match &expr.chain {
                crate::syntax::OptionalChainKind::Member(_) => {}
                crate::syntax::OptionalChainKind::Index(expr) => {
                    resolve_expr(resolver, interner, expr, errors)
                }
                crate::syntax::OptionalChainKind::Call(args) => {
                    for arg in args {
                        resolve_expr(resolver, interner, &arg.value, errors);
                    }
                }
            }
        }
        ExprKind::NonNull(expr) => {
            resolve_expr(resolver, interner, &expr.object, errors);
            match &expr.chain {
                crate::syntax::NonNullKind::Member(_) => {}
                crate::syntax::NonNullKind::Index(expr) => {
                    resolve_expr(resolver, interner, expr, errors)
                }
                crate::syntax::NonNullKind::Call(args) => {
                    for arg in args {
                        resolve_expr(resolver, interner, &arg.value, errors);
                    }
                }
            }
        }
        ExprKind::Assign(expr) => {
            resolve_expr(resolver, interner, &expr.target, errors);
            resolve_expr(resolver, interner, &expr.value, errors);
        }
        ExprKind::Qua(expr) => {
            resolve_expr(resolver, interner, &expr.expr, errors);
            resolve_type(resolver, interner, &expr.ty, errors);
        }
        ExprKind::Innatum(expr) => {
            resolve_expr(resolver, interner, &expr.expr, errors);
            resolve_type(resolver, interner, &expr.ty, errors);
        }
        ExprKind::Novum(expr) => {
            resolve_type_ident(resolver, interner, &expr.ty, errors);
            if let Some(args) = &expr.args {
                for arg in args {
                    resolve_expr(resolver, interner, &arg.value, errors);
                }
            }
            if let Some(init) = &expr.init {
                match init {
                    crate::syntax::NovumInit::Object(fields) => {
                        for field in fields {
                            if let Some(value) = &field.value {
                                resolve_expr(resolver, interner, value, errors);
                            }
                        }
                    }
                    crate::syntax::NovumInit::From(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                }
            }
        }
        ExprKind::Finge(expr) => {
            resolve_variant_ident(resolver, &expr.variant, errors);
            for field in &expr.fields {
                resolve_expr(resolver, interner, &field.value, errors);
            }
            if let Some(cast) = &expr.cast {
                resolve_type_ident(resolver, interner, cast, errors);
            }
        }
        ExprKind::Clausura(expr) => {
            resolver.enter_scope(ScopeKind::Function);
            for param in &expr.params {
                resolve_type(resolver, interner, &param.ty, errors);
                define_symbol(
                    resolver,
                    param.name.name,
                    param.name.span,
                    SymbolKind::Param,
                    false,
                    errors,
                );
            }
            if let Some(ret) = &expr.ret {
                resolve_type(resolver, interner, ret, errors);
            }
            match &expr.body {
                ClausuraBody::Expr(expr) => resolve_expr(resolver, interner, expr, errors),
                ClausuraBody::Block(block) => resolve_block(resolver, interner, block, errors),
            }
            resolver.exit_scope();
        }
        ExprKind::Cede(expr) => resolve_expr(resolver, interner, &expr.expr, errors),
        ExprKind::Array(expr) => {
            for element in &expr.elements {
                match element {
                    crate::syntax::ArrayElement::Expr(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                    crate::syntax::ArrayElement::Spread(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                }
            }
        }
        ExprKind::Object(expr) => {
            for field in &expr.fields {
                match &field.key {
                    crate::syntax::ObjectKey::Computed(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                    crate::syntax::ObjectKey::Spread(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                    _ => {}
                }
                if let Some(value) = &field.value {
                    resolve_expr(resolver, interner, value, errors);
                }
            }
        }
        ExprKind::Intervallum(expr) => {
            resolve_expr(resolver, interner, &expr.start, errors);
            resolve_expr(resolver, interner, &expr.end, errors);
            if let Some(step) = &expr.step {
                resolve_expr(resolver, interner, step, errors);
            }
        }
        ExprKind::Ab(expr) => {
            resolve_expr(resolver, interner, &expr.source, errors);
            if let Some(filter) = &expr.filter {
                match &filter.kind {
                    crate::syntax::CollectionFilterKind::Condition(expr) => {
                        resolve_expr(resolver, interner, expr, errors)
                    }
                    crate::syntax::CollectionFilterKind::Property(ident) => {
                        if resolver.lookup(ident.name).is_none() {
                            errors.push(SemanticError::new(
                                SemanticErrorKind::UndefinedVariable,
                                "unknown identifier",
                                ident.span,
                            ));
                        }
                    }
                }
            }
            for transform in &expr.transforms {
                if let Some(arg) = &transform.arg {
                    resolve_expr(resolver, interner, arg, errors);
                }
            }
        }
        ExprKind::Conversio(expr) => {
            resolve_expr(resolver, interner, &expr.expr, errors);
            for param in &expr.type_params {
                resolve_type(resolver, interner, param, errors);
            }
            if let Some(fallback) = &expr.fallback {
                resolve_expr(resolver, interner, fallback, errors);
            }
        }
        ExprKind::Scriptum(expr) => {
            for arg in &expr.args {
                resolve_expr(resolver, interner, arg, errors);
            }
        }
        ExprKind::Lege(_) => {}
        ExprKind::Sed(_) => {}
        ExprKind::Praefixum(expr) => match &expr.body {
            crate::syntax::PraefixumBody::Block(block) => {
                resolve_block(resolver, interner, block, errors)
            }
            crate::syntax::PraefixumBody::Expr(expr) => {
                resolve_expr(resolver, interner, expr, errors)
            }
        },
        ExprKind::Ego(_) => {}
        ExprKind::Paren(expr) => resolve_expr(resolver, interner, expr, errors),
    }
}

fn resolve_type(
    resolver: &mut Resolver,
    interner: &Interner,
    ty: &TypeExpr,
    errors: &mut Vec<SemanticError>,
) {
    match &ty.kind {
        TypeExprKind::Named(name, params) => {
            resolve_type_ident(resolver, interner, name, errors);
            for param in params {
                resolve_type(resolver, interner, param, errors);
            }
        }
        TypeExprKind::Array(inner) => resolve_type(resolver, interner, inner, errors),
        TypeExprKind::Func(func) => {
            for param in &func.params {
                resolve_type(resolver, interner, param, errors);
            }
            resolve_type(resolver, interner, &func.ret, errors);
        }
    }
}

fn resolve_type_ident(
    resolver: &mut Resolver,
    interner: &Interner,
    ident: &crate::syntax::Ident,
    errors: &mut Vec<SemanticError>,
) {
    let name = interner.resolve(ident.name);

    if is_builtin_type(name) {
        return;
    }

    let Some(def_id) = resolver.lookup(ident.name) else {
        errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedType,
            "unknown type",
            ident.span,
        ));
        return;
    };

    let Some(symbol) = resolver.get_symbol(def_id) else {
        errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedType,
            "unknown type",
            ident.span,
        ));
        return;
    };

    match symbol.kind {
        SymbolKind::Struct
        | SymbolKind::Enum
        | SymbolKind::Interface
        | SymbolKind::TypeAlias
        | SymbolKind::TypeParam => {}
        _ => errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedType,
            "name does not refer to a type",
            ident.span,
        )),
    }
}

fn resolve_variant_ident(
    resolver: &mut Resolver,
    ident: &crate::syntax::Ident,
    errors: &mut Vec<SemanticError>,
) {
    let Some(def_id) = resolver.lookup(ident.name) else {
        errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedVariable,
            "unknown variant",
            ident.span,
        ));
        return;
    };

    let Some(symbol) = resolver.get_symbol(def_id) else {
        errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedVariable,
            "unknown variant",
            ident.span,
        ));
        return;
    };

    if symbol.kind != SymbolKind::Variant {
        errors.push(SemanticError::new(
            SemanticErrorKind::UndefinedVariable,
            "name does not refer to a variant",
            ident.span,
        ));
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "textus"
            | "numerus"
            | "fractus"
            | "bivalens"
            | "nihil"
            | "vacuum"
            | "numquam"
            | "ignotum"
            | "octeti"
            | "lista"
            | "tabula"
            | "copia"
    )
}

fn define_binding_pattern(
    resolver: &mut Resolver,
    pattern: &BindingPattern,
    mutable: bool,
    errors: &mut Vec<SemanticError>,
) {
    match pattern {
        BindingPattern::Ident(ident) => define_symbol(
            resolver,
            ident.name,
            ident.span,
            SymbolKind::Local,
            mutable,
            errors,
        ),
        BindingPattern::Wildcard(_) => {}
        BindingPattern::Array { elements, rest, .. } => {
            for element in elements {
                define_binding_pattern(resolver, element, mutable, errors);
            }
            if let Some(rest) = rest {
                define_symbol(
                    resolver,
                    rest.name,
                    rest.span,
                    SymbolKind::Local,
                    mutable,
                    errors,
                );
            }
        }
    }
}

fn define_symbol(
    resolver: &mut Resolver,
    name: crate::lexer::Symbol,
    span: crate::lexer::Span,
    kind: SymbolKind,
    mutable: bool,
    errors: &mut Vec<SemanticError>,
) {
    let def_id = resolver.fresh_def_id();
    let symbol = Symbol {
        def_id,
        name,
        kind,
        ty: None,
        mutable,
        span,
    };

    if resolver.define(symbol).is_err() {
        errors.push(SemanticError::new(
            SemanticErrorKind::DuplicateDefinition,
            "duplicate definition",
            span,
        ));
    }
}
