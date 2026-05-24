//! Rust declaration emission for the backend item boundary.
//!
//! This module turns top-level HIR declarations into Rust items: functions,
//! tests, structs, enums, traits, type aliases, and constants. Expression and
//! statement lowering live in sibling modules; this file owns the declaration
//! shape around them, including visibility, type parameters, function return
//! contracts, receiver policy for interfaces, and where generated bodies are
//! allowed to wrap values for Rust's `Result` type.
//!
//! INVARIANTS
//! ==========
//! - Failable functions are emitted as `Result<T, String>` according to the
//!   precomputed failable set owned by [`RustCodegen`].
//! - Faber tests become Rust `#[test]` functions with generated names so user
//!   definitions and test entrypoints do not compete for one Rust symbol.
//! - Faber has no declaration visibility modifiers today, so generated
//!   structs, fields, traits, aliases, and constants are public at the Rust
//!   item boundary.
//! - Struct methods are emitted in inherent `impl` blocks. Interface methods
//!   are emitted as trait items with an implicit `&self` receiver because the
//!   HIR contract for pactum methods is receiver-oriented.
//! - HIR import items are not emitted by these declaration helpers; the parent
//!   Rust backend collects them for the generated prelude alongside helper-type
//!   imports discovered from emitted Rust.
//!
//! TRADE-OFFS
//! ==========
//! Failable declarations use `String` as the error carrier. That keeps throw
//! lowering and `?` propagation simple, but it is a backend policy, not a
//! semantic recheck. Missing or wrong type information should be fixed before
//! this module receives HIR.

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::*;
use crate::semantic::{Type, TypeId, TypeTable};

/// Emit one Rust function item from a HIR function.
///
/// The declaration layer decides the Rust signature: async marker, generated
/// test name, optional CLI argument type, type parameters, and `Result`
/// wrapping for precomputed failable functions. Body emission is delegated to
/// [`generate_block`] with the same failable context so `redde` and tail
/// expressions can produce values matching the signature.
pub fn generate_function(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_function_with_cli_args_type(codegen, def_id, func, types, w, None)
}

pub fn generate_function_with_cli_args_type(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
    cli_args_type: Option<&str>,
) -> Result<(), CodegenError> {
    generate_function_inner(codegen, def_id, func, types, w, cli_args_type, None, None)
}

fn generate_function_inner(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    w: &mut CodeWriter,
    cli_args_type: Option<&str>,
    receiver: Option<&str>,
    return_type_override: Option<&str>,
) -> Result<(), CodegenError> {
    let is_failable = codegen.is_failable_def(def_id);
    let is_test = func.test.is_some();

    // Faber tests are compiled as Rust tests while preserving selection state
    // through ignore reasons rather than deleting unselected tests from output.
    if is_test {
        w.writeln("#[test]");
        if let Some(reason) = codegen.test_ignore_reason(func) {
            w.writeln(&format!("#[ignore = \"{}\"]", escape_ignore_reason(&reason)));
        }
    }

    // Faber `@ futura` maps directly to `async fn`; the async return type
    // remains part of the Rust function contract below.
    if func.is_async {
        w.write("async ");
    }

    if cli_args_type.is_some() {
        w.write("pub(crate) ");
    }
    w.write("fn ");
    if is_test {
        w.write(&format!("proba_{}", def_id.0));
    } else {
        w.write(codegen.resolve_symbol(func.name));
    }

    // Type parameters are emitted unchanged after symbol resolution; trait
    // bounds are not invented here.
    if !func.type_params.is_empty() {
        w.write("<");
        for (i, param) in func.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    // CLI-mounted functions get one synthetic argument slot supplied by the
    // package CLI adapter. Ordinary functions use only HIR parameters.
    w.write("(");
    if let Some(receiver) = receiver {
        w.write(receiver);
    }
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 || receiver.is_some() {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        w.write(": ");
        w.write(&type_to_rust(codegen, param.ty, types));
    }
    if let Some(param) = &func.cli_args {
        if !func.params.is_empty() {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
        w.write(": ");
        w.write(cli_args_type.unwrap_or("CliArgs"));
    }
    w.write(")");

    // Failable status is a whole-function ABI decision: every explicit return
    // and tail expression in the body must match this `Result` wrapper.
    if is_failable {
        w.write(" -> ");
        w.write("Result<");
        if let Some(ret_ty) = return_type_override {
            w.write(ret_ty);
        } else if let Some(ret_ty) = func.ret_ty {
            w.write(&type_to_rust(codegen, ret_ty, types));
        } else {
            w.write("()");
        }
        w.write(", String>");
    } else if let Some(ret_ty) = return_type_override {
        w.write(" -> ");
        w.write(ret_ty);
    } else if let Some(ret_ty) = func.ret_ty {
        w.write(" -> ");
        w.write(&type_to_rust(codegen, ret_ty, types));
    }

    // Declarations without bodies are emitted as Rust signatures, used by
    // trait-like surfaces and imported interface stubs.
    let previous_return_ty = codegen.replace_current_return_ty(func.ret_ty);
    let body_result = if let Some(body) = &func.body {
        w.write(" ");
        generate_block(codegen, body, types, w, is_failable, false, true)
    } else {
        w.write(";");
        Ok(())
    };
    codegen.replace_current_return_ty(previous_return_ty);
    body_result?;

    w.newline();
    Ok(())
}

fn escape_ignore_reason(reason: &str) -> String {
    reason.replace('\\', r"\\").replace('"', "\\\"")
}

/// Emit a Rust struct and any inherent methods declared on the Faber `genus`.
///
/// Instance fields become public Rust fields. Static fields are skipped here
/// because they are not representable inside a Rust struct declaration; method
/// and associated-item support must be added through a separate contract rather
/// than hidden in field emission.
pub fn generate_struct(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    s: &HirStruct,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub struct ");
    w.write(codegen.resolve_symbol(s.name));

    if !s.type_params.is_empty() {
        w.write("<");
        for (i, param) in s.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for field in &s.fields {
            if !field.is_static {
                w.write("pub ");
                w.write(codegen.resolve_symbol(field.name));
                w.write(": ");
                let ty_str = type_to_rust(codegen, field.ty, types);
                if field.sponte && !type_is_option(field.ty, types) {
                    // sponte (voluntary declaration) represented as Option<T> in Rust for
                    // partial construction support; fixus has no target immutability effect here.
                    w.write(&format!("Option<{}>", ty_str));
                } else {
                    w.write(&ty_str);
                }
                w.writeln(",");
            }
        }
    });
    w.writeln("}");

    // Generate impl block for methods
    if !s.methods.is_empty() {
        w.newline();
        w.write("impl ");
        w.write(codegen.resolve_symbol(s.name));
        w.writeln(" {");
        let mut method_result = Ok(());
        w.indented(|w| {
            for method in &s.methods {
                if method_result.is_err() {
                    return;
                }
                method_result = generate_method(codegen, def_id, method, types, w);
            }
        });
        method_result?;
        w.writeln("}");
    }

    Ok(())
}

fn generate_method(
    codegen: &RustCodegen<'_>,
    struct_def: DefId,
    method: &HirMethod,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let receiver = match method.receiver {
        HirReceiver::MutRef => "&mut self",
        HirReceiver::None if method_mutates_self(&method.func, struct_def) => "&mut self",
        HirReceiver::Owned => "self",
        HirReceiver::Ref | HirReceiver::None => "&self",
    };
    let return_type_override = method_self_return_type_override(codegen, struct_def, method, types);
    let previous_self = codegen.replace_current_self_def(Some(struct_def));
    let result = generate_function_inner(
        codegen,
        method.def_id,
        &method.func,
        types,
        w,
        None,
        Some(receiver),
        return_type_override.as_deref(),
    );
    codegen.replace_current_self_def(previous_self);
    result
}

fn method_self_return_type_override(
    codegen: &RustCodegen<'_>,
    struct_def: DefId,
    method: &HirMethod,
    types: &TypeTable,
) -> Option<String> {
    let Some(ret_ty) = method.func.ret_ty else {
        return None;
    };
    if !matches!(resolve_type(ret_ty, types), Type::Struct(def_id) if def_id == struct_def) {
        return None;
    }
    if !method_returns_self(&method.func, struct_def) {
        return None;
    }

    Some(format!("&mut {}", codegen.resolve_def(struct_def)))
}

fn method_mutates_self(func: &HirFunction, self_def: DefId) -> bool {
    struct SelfMutationDetector {
        self_def: DefId,
        found: bool,
    }

    impl HirVisitor for SelfMutationDetector {
        fn visit_expr(&mut self, expr: &HirExpr) {
            match &expr.kind {
                HirExprKind::Assign(target, _) | HirExprKind::AssignOp(_, target, _) => {
                    if assignment_target_starts_at_self(target, self.self_def) {
                        self.found = true;
                        return;
                    }
                }
                _ => {}
            }
            walk_expr(self, expr);
        }
    }

    let mut detector = SelfMutationDetector { self_def, found: false };
    detector.visit_function(func);
    detector.found
}

fn method_returns_self(func: &HirFunction, self_def: DefId) -> bool {
    struct SelfReturnDetector {
        self_def: DefId,
        found: bool,
    }

    impl HirVisitor for SelfReturnDetector {
        fn visit_stmt(&mut self, stmt: &HirStmt) {
            if let HirStmtKind::Redde(Some(expr)) = &stmt.kind {
                if matches!(expr.kind, HirExprKind::Path(def_id) if def_id == self.self_def) {
                    self.found = true;
                    return;
                }
            }
            crate::hir::visit::walk_stmt(self, stmt);
        }
    }

    let mut detector = SelfReturnDetector { self_def, found: false };
    detector.visit_function(func);
    detector.found
}

fn assignment_target_starts_at_self(expr: &HirExpr, self_def: DefId) -> bool {
    match &expr.kind {
        HirExprKind::Path(def_id) => *def_id == self_def,
        HirExprKind::Field(object, _) | HirExprKind::Index(object, _) => {
            assignment_target_starts_at_self(object, self_def)
        }
        _ => false,
    }
}

fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        other => other.clone(),
    }
}

fn type_is_option(type_id: TypeId, types: &TypeTable) -> bool {
    match types.get(type_id) {
        Type::Option(_) => true,
        Type::Alias(_, resolved) => type_is_option(*resolved, types),
        _ => false,
    }
}

/// Emit a Rust enum using struct-like variants for any variant payloads.
///
/// Faber enum fields are named in HIR, so the Rust backend preserves that
/// shape instead of lowering to tuple variants. This keeps generated pattern
/// syntax aligned with the source-level field names.
pub fn generate_enum(
    codegen: &RustCodegen<'_>,
    e: &HirEnum,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub enum ");
    w.write(codegen.resolve_symbol(e.name));

    if !e.type_params.is_empty() {
        w.write("<");
        for (i, param) in e.type_params.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for variant in &e.variants {
            w.write(codegen.resolve_symbol(variant.name));
            if !variant.fields.is_empty() {
                w.writeln(" {");
                w.indented(|w| {
                    for field in &variant.fields {
                        w.write(codegen.resolve_symbol(field.name));
                        w.write(": ");
                        w.write(&type_to_rust(codegen, field.ty, types));
                        w.writeln(",");
                    }
                });
                w.write("}");
            }
            w.writeln(",");
        }
    });
    w.writeln("}");

    Ok(())
}

/// Emit a Rust trait for a Faber interface.
///
/// Interface methods receive an implicit immutable `&self` receiver. Static
/// interface functions are not modeled by this HIR shape, so this function does
/// not infer static methods from parameter lists or names.
pub fn generate_trait(
    codegen: &RustCodegen<'_>,
    i: &HirInterface,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("pub trait ");
    w.write(codegen.resolve_symbol(i.name));

    if !i.type_params.is_empty() {
        w.write("<");
        for (idx, param) in i.type_params.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            w.write(codegen.resolve_symbol(param.name));
        }
        w.write(">");
    }

    w.writeln(" {");
    w.indented(|w| {
        for method in &i.methods {
            w.write("fn ");
            w.write(codegen.resolve_symbol(method.name));
            w.write("(");
            w.write("&self");
            for param in &method.params {
                w.write(", ");
                w.write(codegen.resolve_symbol(param.name));
                w.write(": ");
                w.write(&type_to_rust(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret) = method.ret_ty {
                w.write(" -> ");
                w.write(&type_to_rust(codegen, ret, types));
            }
            w.writeln(";");
        }
    });
    w.writeln("}");

    Ok(())
}

pub fn generate_type_alias(
    codegen: &RustCodegen<'_>,
    a: &HirTypeAlias,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Aliases intentionally render their resolved target type. Name retention
    // is tracked by semantic definitions, not by emitting a Rust newtype here.
    w.write("pub type ");
    w.write(codegen.resolve_symbol(a.name));
    w.write(" = ");
    w.write(&type_to_rust(codegen, a.ty, types));
    w.writeln(";");

    Ok(())
}

pub fn generate_const(
    codegen: &RustCodegen<'_>,
    c: &HirConst,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // A missing const annotation type has already survived semantic analysis;
    // this backend emits `()` rather than guessing from the initializer.
    w.write("pub const ");
    w.write(codegen.resolve_symbol(c.name));
    w.write(": ");
    if let Some(ty) = c.ty {
        w.write(&type_to_rust(codegen, ty, types));
    } else {
        w.write("()");
    }
    w.write(" = ");
    super::expr::generate_expr(codegen, &c.value, types, w, false, false, false)?;
    w.writeln(";");

    Ok(())
}

fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    wrap_tail_ok: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut block_result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::stmt::generate_stmt(codegen, stmt, types, w, in_failable_fn, in_entry, false);
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            // Tail expressions carry the same Result contract as explicit
            // `redde`; entry blocks are excluded because entry throws lower to
            // panic paths instead of caller-visible propagation.
            if wrap_tail_ok && in_failable_fn && !in_entry {
                w.write("Ok(");
                block_result = super::expr::generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, false);
                w.writeln(")");
            } else {
                block_result = super::expr::generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, false);
            }
        }
    });
    block_result?;
    w.write("}");
    Ok(())
}
