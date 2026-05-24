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
use super::type_shape::{resolve_type, type_id_is_option};
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::*;
use crate::semantic::{Type, TypeId, TypeTable};

#[derive(Clone, Copy, Default)]
struct FunctionEmitContext<'a> {
    cli_args_type: Option<&'a str>,
    receiver: Option<&'a str>,
    return_type_override: Option<&'a str>,
}

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
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_function_with_cli_args_type(codegen, def_id, func, types, writer, None)
}

pub fn generate_function_with_cli_args_type(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    writer: &mut CodeWriter,
    cli_args_type: Option<&str>,
) -> Result<(), CodegenError> {
    generate_function_inner(
        codegen,
        def_id,
        func,
        types,
        writer,
        FunctionEmitContext { cli_args_type, ..FunctionEmitContext::default() },
    )
}

fn generate_function_inner(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    func: &HirFunction,
    types: &TypeTable,
    writer: &mut CodeWriter,
    context: FunctionEmitContext<'_>,
) -> Result<(), CodegenError> {
    let is_failable = codegen.is_failable_def(def_id);
    let is_test = func.test.is_some();

    // Faber tests are compiled as Rust tests while preserving selection state
    // through ignore reasons rather than deleting unselected tests from output.
    if is_test {
        writer.writeln("#[test]");
        if let Some(reason) = codegen.test_ignore_reason(func) {
            writer.writeln(&format!("#[ignore = \"{}\"]", escape_ignore_reason(&reason)));
        }
    }

    // Faber `@ futura` maps directly to `async fn`; the async return type
    // remains part of the Rust function contract below.
    if func.is_async {
        writer.write("async ");
    }

    if context.cli_args_type.is_some() {
        writer.write("pub(crate) ");
    }
    writer.write("fn ");
    if is_test {
        writer.write(&format!("proba_{}", def_id.0));
    } else {
        writer.write(codegen.resolve_symbol(func.name));
    }

    // Type parameters are emitted unchanged after symbol resolution; trait
    // bounds are not invented here.
    if !func.type_params.is_empty() {
        writer.write("<");
        for (i, param) in func.type_params.iter().enumerate() {
            if i > 0 {
                writer.write(", ");
            }
            writer.write(codegen.resolve_symbol(param.name));
        }
        writer.write(">");
    }

    // CLI-mounted functions get one synthetic argument slot supplied by the
    // package CLI adapter. Ordinary functions use only HIR parameters.
    writer.write("(");
    if let Some(receiver) = context.receiver {
        writer.write(receiver);
    }
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 || context.receiver.is_some() {
            writer.write(", ");
        }
        writer.write(codegen.resolve_symbol(param.name));
        writer.write(": ");
        if param.optional && param.default.is_none() {
            writer.write("Option<");
            writer.write(&type_to_rust(codegen, param.ty, types));
            writer.write(">");
        } else {
            writer.write(&type_to_rust(codegen, param.ty, types));
        }
    }
    if let Some(param) = &func.cli_args {
        if !func.params.is_empty() {
            writer.write(", ");
        }
        writer.write(codegen.resolve_symbol(param.name));
        writer.write(": ");
        writer.write(context.cli_args_type.unwrap_or("CliArgs"));
    }
    writer.write(")");

    // Failable status is a whole-function ABI decision: every explicit return
    // and tail expression in the body must match this `Result` wrapper.
    if is_failable {
        writer.write(" -> ");
        writer.write("Result<");
        if let Some(ret_ty) = context.return_type_override {
            writer.write(ret_ty);
        } else if let Some(ret_ty) = func.ret_ty {
            writer.write(&type_to_rust(codegen, ret_ty, types));
        } else {
            writer.write("()");
        }
        writer.write(", String>");
    } else if func.is_generator {
        writer.write(" -> Vec<");
        if let Some(ret_ty) = func.ret_ty {
            writer.write(&type_to_rust(codegen, ret_ty, types));
        } else {
            writer.write("()");
        }
        writer.write(">");
    } else if let Some(ret_ty) = context.return_type_override {
        writer.write(" -> ");
        writer.write(ret_ty);
    } else if let Some(ret_ty) = func.ret_ty {
        writer.write(" -> ");
        writer.write(&type_to_rust(codegen, ret_ty, types));
    }

    // Declarations without bodies are emitted as Rust signatures, used by
    // trait-like surfaces and imported interface stubs.
    let previous_return_ty = codegen.replace_current_return_ty(func.ret_ty);
    let previous_generator_yield_ty = codegen.replace_current_generator_yield_ty(
        func.is_generator.then_some(
            func.ret_ty
                .unwrap_or_else(|| types.primitive(crate::semantic::Primitive::Vacuum)),
        ),
    );
    let body_result = if let Some(body) = &func.body {
        writer.write(" ");
        if func.is_generator {
            generate_generator_block(codegen, body, func.ret_ty, types, writer, is_failable)
        } else {
            generate_block(codegen, body, types, writer, is_failable, false, true)
        }
    } else {
        writer.write(";");
        Ok(())
    };
    codegen.replace_current_return_ty(previous_return_ty);
    codegen.replace_current_generator_yield_ty(previous_generator_yield_ty);
    body_result?;

    writer.newline();
    Ok(())
}

fn generate_generator_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    yield_ty: Option<TypeId>,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
) -> Result<(), CodegenError> {
    writer.writeln("{");
    let mut block_result = Ok(());
    writer.indented(|writer| {
        writer.write("let mut __faber_yielded: Vec<");
        if let Some(yield_ty) = yield_ty {
            writer.write(&type_to_rust(codegen, yield_ty, types));
        } else {
            writer.write("()");
        }
        writer.writeln("> = Vec::new();");
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::stmt::generate_stmt(codegen, stmt, types, writer, in_failable_fn, false, false);
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            block_result = super::expr::generate_expr(codegen, expr, types, writer, in_failable_fn, false, false);
            writer.writeln(";");
        }
        writer.writeln("__faber_yielded");
    });
    block_result?;
    writer.write("}");
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
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    writer.writeln("#[derive(Clone, Debug)]");
    writer.write("pub struct ");
    writer.write(codegen.resolve_symbol(s.name));

    if !s.type_params.is_empty() {
        writer.write("<");
        for (i, param) in s.type_params.iter().enumerate() {
            if i > 0 {
                writer.write(", ");
            }
            writer.write(codegen.resolve_symbol(param.name));
        }
        writer.write(">");
    }

    writer.writeln(" {");
    writer.indented(|writer| {
        for field in &s.fields {
            if !field.is_static {
                writer.write("pub ");
                writer.write(codegen.resolve_symbol(field.name));
                writer.write(": ");
                let ty_str = type_to_rust(codegen, field.ty, types);
                if field.sponte && !type_id_is_option(field.ty, types) {
                    // sponte (voluntary declaration) represented as Option<T> in Rust for
                    // partial construction support; fixus has no target immutability effect here.
                    writer.write(&format!("Option<{}>", ty_str));
                } else {
                    writer.write(&ty_str);
                }
                writer.writeln(",");
            }
        }
    });
    writer.writeln("}");

    // Generate impl block for methods
    if !s.methods.is_empty() {
        writer.newline();
        writer.write("impl ");
        writer.write(codegen.resolve_symbol(s.name));
        writer.writeln(" {");
        let mut method_result = Ok(());
        writer.indented(|writer| {
            for method in &s.methods {
                if method_result.is_err() {
                    return;
                }
                method_result = generate_method(codegen, def_id, method, types, writer);
            }
        });
        method_result?;
        writer.writeln("}");
    }

    Ok(())
}

fn generate_method(
    codegen: &RustCodegen<'_>,
    struct_def: DefId,
    method: &HirMethod,
    types: &TypeTable,
    writer: &mut CodeWriter,
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
        writer,
        FunctionEmitContext {
            receiver: Some(receiver),
            return_type_override: return_type_override.as_deref(),
            ..FunctionEmitContext::default()
        },
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
    let ret_ty = method.func.ret_ty?;
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
                HirExprKind::Assign(target, _) | HirExprKind::AssignOp(_, target, _)
                    if assignment_target_starts_at_self(target, self.self_def) =>
                {
                    self.found = true;
                    return;
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

/// Emit a Rust enum using struct-like variants for any variant payloads.
///
/// Faber enum fields are named in HIR, so the Rust backend preserves that
/// shape instead of lowering to tuple variants. This keeps generated pattern
/// syntax aligned with the source-level field names.
pub fn generate_enum(
    codegen: &RustCodegen<'_>,
    e: &HirEnum,
    types: &TypeTable,
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    writer.write("pub enum ");
    writer.write(codegen.resolve_symbol(e.name));

    if !e.type_params.is_empty() {
        writer.write("<");
        for (i, param) in e.type_params.iter().enumerate() {
            if i > 0 {
                writer.write(", ");
            }
            writer.write(codegen.resolve_symbol(param.name));
        }
        writer.write(">");
    }

    writer.writeln(" {");
    writer.indented(|writer| {
        for variant in &e.variants {
            writer.write(codegen.resolve_symbol(variant.name));
            if !variant.fields.is_empty() {
                writer.writeln(" {");
                writer.indented(|writer| {
                    for field in &variant.fields {
                        writer.write(codegen.resolve_symbol(field.name));
                        writer.write(": ");
                        writer.write(&type_to_rust(codegen, field.ty, types));
                        writer.writeln(",");
                    }
                });
                writer.write("}");
            }
            writer.writeln(",");
        }
    });
    writer.writeln("}");

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
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    writer.write("pub trait ");
    writer.write(codegen.resolve_symbol(i.name));

    if !i.type_params.is_empty() {
        writer.write("<");
        for (idx, param) in i.type_params.iter().enumerate() {
            if idx > 0 {
                writer.write(", ");
            }
            writer.write(codegen.resolve_symbol(param.name));
        }
        writer.write(">");
    }

    writer.writeln(" {");
    writer.indented(|writer| {
        for method in &i.methods {
            writer.write("fn ");
            writer.write(codegen.resolve_symbol(method.name));
            writer.write("(");
            writer.write("&self");
            for param in &method.params {
                writer.write(", ");
                writer.write(codegen.resolve_symbol(param.name));
                writer.write(": ");
                writer.write(&type_to_rust(codegen, param.ty, types));
            }
            writer.write(")");
            if let Some(ret) = method.ret_ty {
                writer.write(" -> ");
                writer.write(&type_to_rust(codegen, ret, types));
            }
            writer.writeln(";");
        }
    });
    writer.writeln("}");

    Ok(())
}

pub fn generate_type_alias(
    codegen: &RustCodegen<'_>,
    a: &HirTypeAlias,
    types: &TypeTable,
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Aliases intentionally render their resolved target type. Name retention
    // is tracked by semantic definitions, not by emitting a Rust newtype here.
    writer.write("pub type ");
    writer.write(codegen.resolve_symbol(a.name));
    writer.write(" = ");
    writer.write(&type_to_rust(codegen, a.ty, types));
    writer.writeln(";");

    Ok(())
}

pub fn generate_const(
    codegen: &RustCodegen<'_>,
    c: &HirConst,
    types: &TypeTable,
    writer: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // A missing const annotation type has already survived semantic analysis;
    // this backend emits `()` rather than guessing from the initializer.
    writer.write("pub const ");
    writer.write(codegen.resolve_symbol(c.name));
    writer.write(": ");
    if let Some(ty) = c.ty {
        writer.write(&type_to_rust(codegen, ty, types));
    } else {
        writer.write("()");
    }
    writer.write(" = ");
    super::expr::generate_expr(codegen, &c.value, types, writer, false, false, false)?;
    writer.writeln(";");

    Ok(())
}

fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    wrap_tail_ok: bool,
) -> Result<(), CodegenError> {
    writer.writeln("{");
    let mut block_result = Ok(());
    writer.indented(|writer| {
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::stmt::generate_stmt(codegen, stmt, types, writer, in_failable_fn, in_entry, false);
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            // Tail expressions carry the same Result contract as explicit
            // `redde`; entry blocks are excluded because entry throws lower to
            // panic paths instead of caller-visible propagation.
            if wrap_tail_ok && in_failable_fn && !in_entry {
                writer.write("Ok(");
                block_result =
                    super::expr::generate_expr(codegen, expr, types, writer, in_failable_fn, in_entry, false);
                writer.writeln(")");
            } else {
                block_result =
                    super::expr::generate_expr(codegen, expr, types, writer, in_failable_fn, in_entry, false);
            }
        }
    });
    block_result?;
    writer.write("}");
    Ok(())
}
