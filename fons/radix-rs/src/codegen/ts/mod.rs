mod decl;
mod stmt;
mod types;

use super::{CodeWriter, Codegen, CodegenError};
use crate::hir::{
    DefId, HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirItem, HirItemKind, HirLiteral,
    HirOptionalChainKind, HirPattern, HirProgram, HirStmtKind, HirTransformKind, HirUnOp,
};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use crate::TypeScriptOutput;
use rustc_hash::FxHashMap;

pub struct TsCodegen<'a> {
    names: FxHashMap<DefId, Symbol>,
    interner: &'a Interner,
}

impl<'a> TsCodegen<'a> {
    pub fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        let mut codegen = Self { names: FxHashMap::default(), interner };
        codegen.names = codegen.collect_names(hir);
        codegen
    }

    pub(super) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }

    pub(super) fn resolve_def(&self, def_id: DefId) -> &str {
        self.names
            .get(&def_id)
            .map(|sym| self.resolve_symbol(*sym))
            .unwrap_or("unresolved_def")
    }

    fn collect_names(&self, hir: &HirProgram) -> FxHashMap<DefId, Symbol> {
        let mut names = FxHashMap::default();
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    names.insert(item.def_id, func.name);
                    for type_param in &func.type_params {
                        names.insert(type_param.def_id, type_param.name);
                    }
                    for param in &func.params {
                        names.insert(param.def_id, param.name);
                    }
                    self.collect_block_names(&mut names, func.body.as_ref());
                }
                HirItemKind::Struct(strukt) => {
                    names.insert(item.def_id, strukt.name);
                    for type_param in &strukt.type_params {
                        names.insert(type_param.def_id, type_param.name);
                    }
                    for field in &strukt.fields {
                        names.insert(field.def_id, field.name);
                    }
                    for method in &strukt.methods {
                        names.insert(method.def_id, method.func.name);
                        for type_param in &method.func.type_params {
                            names.insert(type_param.def_id, type_param.name);
                        }
                        for param in &method.func.params {
                            names.insert(param.def_id, param.name);
                        }
                        self.collect_block_names(&mut names, method.func.body.as_ref());
                    }
                }
                HirItemKind::Enum(enum_item) => {
                    names.insert(item.def_id, enum_item.name);
                    for type_param in &enum_item.type_params {
                        names.insert(type_param.def_id, type_param.name);
                    }
                    for variant in &enum_item.variants {
                        names.insert(variant.def_id, variant.name);
                    }
                }
                HirItemKind::Interface(interface) => {
                    names.insert(item.def_id, interface.name);
                    for type_param in &interface.type_params {
                        names.insert(type_param.def_id, type_param.name);
                    }
                }
                HirItemKind::TypeAlias(alias) => {
                    names.insert(item.def_id, alias.name);
                }
                HirItemKind::Const(const_item) => {
                    names.insert(item.def_id, const_item.name);
                }
                HirItemKind::Import(import) => {
                    for item in &import.items {
                        names.insert(item.def_id, item.alias.unwrap_or(item.name));
                    }
                }
            }
        }

        if let Some(entry) = &hir.entry {
            self.collect_block_names(&mut names, Some(entry));
        }

        names
    }

    fn collect_block_names(&self, names: &mut FxHashMap<DefId, Symbol>, block: Option<&HirBlock>) {
        let Some(block) = block else {
            return;
        };
        for stmt in &block.stmts {
            match &stmt.kind {
                HirStmtKind::Local(local) => {
                    names.insert(local.def_id, local.name);
                    if let Some(init) = &local.init {
                        self.collect_expr_names(names, init);
                    }
                }
                HirStmtKind::Expr(expr) => self.collect_expr_names(names, expr),
                HirStmtKind::Redde(value) => {
                    if let Some(expr) = value {
                        self.collect_expr_names(names, expr);
                    }
                }
                HirStmtKind::Rumpe | HirStmtKind::Perge => {}
            }
        }
        if let Some(expr) = &block.expr {
            self.collect_expr_names(names, expr);
        }
    }

    fn collect_expr_names(&self, names: &mut FxHashMap<DefId, Symbol>, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.collect_expr_names(names, lhs);
                self.collect_expr_names(names, rhs);
            }
            HirExprKind::Unary(_, operand)
            | HirExprKind::Cede(operand)
            | HirExprKind::Qua(operand, _)
            | HirExprKind::Ref(_, operand)
            | HirExprKind::Deref(operand)
            | HirExprKind::Panic(operand)
            | HirExprKind::Throw(operand) => self.collect_expr_names(names, operand),
            HirExprKind::Innatum { source, map_entries, .. } => {
                self.collect_expr_names(names, source);
                if let Some(entries) = map_entries {
                    for (_, value) in entries {
                        self.collect_expr_names(names, value);
                    }
                }
            }
            HirExprKind::Call(callee, args) => {
                self.collect_expr_names(names, callee);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.collect_expr_names(names, receiver);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Field(object, _) => self.collect_expr_names(names, object),
            HirExprKind::Index(object, index) => {
                self.collect_expr_names(names, object);
                self.collect_expr_names(names, index);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.collect_expr_names(names, object);
                match chain {
                    HirOptionalChainKind::Member(_) => {}
                    HirOptionalChainKind::Index(index) => self.collect_expr_names(names, index),
                    HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_names(names, arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.collect_expr_names(names, source);
                if let Some(filter) = filter {
                    if let HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.collect_expr_names(names, cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.collect_expr_names(names, arg);
                    }
                }
            }
            HirExprKind::Block(block) | HirExprKind::Loop(block) => self.collect_block_names(names, Some(block)),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.collect_expr_names(names, cond);
                self.collect_block_names(names, Some(then_block));
                self.collect_block_names(names, else_block.as_ref());
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.collect_expr_names(names, scrutinee);
                for arm in arms {
                    self.collect_pattern_names(names, &arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_names(names, guard);
                    }
                    self.collect_expr_names(names, &arm.body);
                }
            }
            HirExprKind::Dum(cond, block) => {
                self.collect_expr_names(names, cond);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Itera(_, _, iter, block) => {
                self.collect_expr_names(names, iter);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Array(elements) | HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.collect_expr_names(names, element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.collect_expr_names(names, cond);
                if let Some(message) = message {
                    self.collect_expr_names(names, message);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.collect_expr_names(names, value);
                }
            }
            HirExprKind::Tempta { body, catch, finally } => {
                self.collect_block_names(names, Some(body));
                self.collect_block_names(names, catch.as_ref());
                self.collect_block_names(names, finally.as_ref());
            }
            HirExprKind::Clausura(params, _, body) => {
                for param in params {
                    names.insert(param.def_id, param.name);
                }
                self.collect_expr_names(names, body);
            }
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_pattern_names(&self, names: &mut FxHashMap<DefId, Symbol>, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard | HirPattern::Literal(_) => {}
            HirPattern::Binding(def_id, name) => {
                names.insert(*def_id, *name);
            }
            HirPattern::Variant(_, patterns) => {
                for pattern in patterns {
                    self.collect_pattern_names(names, pattern);
                }
            }
        }
    }

    fn generate_item(&self, item: &HirItem, types: &TypeTable, w: &mut CodeWriter) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => decl::generate_function(self, func, types, w)?,
            HirItemKind::Struct(strukt) => decl::generate_class(self, strukt, types, w)?,
            HirItemKind::Enum(enum_item) => decl::generate_enum(self, enum_item, types, w)?,
            HirItemKind::Interface(interface) => decl::generate_interface(self, interface, types, w)?,
            HirItemKind::TypeAlias(alias) => decl::generate_type_alias(self, alias, types, w)?,
            HirItemKind::Const(constant) => decl::generate_const(self, constant, types, w)?,
            HirItemKind::Import(import) => decl::generate_import(self, import, w)?,
        }
        Ok(())
    }
}

impl Codegen for TsCodegen<'_> {
    type Output = TypeScriptOutput;

    fn generate(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
        _interner: &Interner,
    ) -> Result<TypeScriptOutput, CodegenError> {
        let mut w = CodeWriter::new();
        w.writeln("// Generated by radix - do not edit");
        w.newline();

        for item in &hir.items {
            self.generate_item(item, types, &mut w)?;
            w.newline();
        }

        if let Some(entry) = &hir.entry {
            let entry_is_async = contains_await_in_block(entry);
            if entry_is_async {
                w.writeln("(async () => {");
            } else {
                w.writeln("(() => {");
            }
            let mut block_result = Ok(());
            w.indented(|w| {
                block_result = stmt::generate_block(self, entry, types, w);
            });
            block_result?;
            if entry_is_async {
                w.writeln("})();");
            } else {
                w.writeln("})();");
            }
        }
        Ok(TypeScriptOutput { code: w.finish() })
    }
}

pub(super) fn generate_expr(
    codegen: &TsCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => w.write(codegen.resolve_def(*def_id)),
        HirExprKind::Literal(lit) => generate_literal(codegen, lit, w),
        HirExprKind::Binary(op, lhs, rhs) => {
            w.write("(");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(binary_op_to_ts(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(")");
        }
        HirExprKind::Unary(op, operand) => {
            w.write(unary_op_to_ts(*op));
            generate_expr(codegen, operand, types, w)?;
        }
        HirExprKind::Call(callee, args) => {
            generate_expr(codegen, callee, types, w)?;
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_expr(codegen, receiver, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*method));
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Field(object, field) => {
            generate_expr(codegen, object, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*field));
        }
        HirExprKind::Index(object, index) => {
            generate_expr(codegen, object, types, w)?;
            w.write("[");
            generate_expr(codegen, index, types, w)?;
            w.write("]");
        }
        HirExprKind::OptionalChain(object, chain) => {
            generate_expr(codegen, object, types, w)?;
            match chain {
                HirOptionalChainKind::Member(field) => {
                    w.write("?.");
                    w.write(codegen.resolve_symbol(*field));
                }
                HirOptionalChainKind::Index(index) => {
                    w.write("?.[");
                    generate_expr(codegen, index, types, w)?;
                    w.write("]");
                }
                HirOptionalChainKind::Call(args) => {
                    w.write("?.(");
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        generate_expr(codegen, arg, types, w)?;
                    }
                    w.write(")");
                }
            }
        }
        HirExprKind::Assign(lhs, rhs) => {
            generate_expr(codegen, lhs, types, w)?;
            w.write(" = ");
            generate_expr(codegen, rhs, types, w)?;
        }
        HirExprKind::AssignOp(op, lhs, rhs) => {
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(assign_op_to_ts(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
        }
        HirExprKind::Array(elements) => {
            w.write("[");
            for (idx, element) in elements.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, element, types, w)?;
            }
            w.write("]");
        }
        HirExprKind::Struct(_, fields) => {
            w.write("{ ");
            for (idx, (name, value)) in fields.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(*name));
                w.write(": ");
                generate_expr(codegen, value, types, w)?;
            }
            w.write(" }");
        }
        HirExprKind::Tuple(elements) => {
            w.write("[");
            for (idx, element) in elements.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, element, types, w)?;
            }
            w.write("]");
        }
        HirExprKind::Scribe(args) => {
            w.write("console.log(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Scriptum(template, args) => {
            let mut rendered = codegen.resolve_symbol(*template).to_owned();
            for idx in 0..args.len() {
                rendered = rendered.replace(&format!("§{}", idx + 1), &format!("${{{}}}", idx));
                rendered = rendered.replace('§', "${}");
            }
            w.write("`");
            w.write(&rendered);
            w.write("`");
        }
        HirExprKind::Panic(value) | HirExprKind::Throw(value) => {
            w.write("(() => { throw new Error(String(");
            generate_expr(codegen, value, types, w)?;
            w.write(")); })()");
        }
        HirExprKind::Tempta { body, catch, finally } => {
            w.write("(() => { try ");
            stmt::generate_inline_block(codegen, body, types, w)?;
            if let Some(catch) = catch {
                w.write(" catch (_err) ");
                stmt::generate_inline_block(codegen, catch, types, w)?;
            }
            if let Some(finally) = finally {
                w.write(" finally ");
                stmt::generate_inline_block(codegen, finally, types, w)?;
            }
            w.write(" })()");
        }
        HirExprKind::Clausura(params, ret_ty, body) => {
            w.write("(");
            for (idx, param) in params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                if param.optional {
                    w.write("?");
                }
                w.write(": ");
                w.write(&types::type_to_ts(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = ret_ty {
                w.write(": ");
                w.write(&types::type_to_ts(codegen, *ret_ty, types));
            }
            w.write(" => ");
            generate_expr(codegen, body, types, w)?;
        }
        HirExprKind::Cede(inner) => {
            w.write("await ");
            generate_expr(codegen, inner, types, w)?;
        }
        HirExprKind::Qua(inner, ty) => {
            generate_expr(codegen, inner, types, w)?;
            w.write(" as ");
            w.write(&types::type_to_ts(codegen, *ty, types));
        }
        HirExprKind::Innatum { source, target, .. } => {
            generate_expr(codegen, source, types, w)?;
            w.write(" as ");
            w.write(&types::type_to_ts(codegen, *target, types));
        }
        HirExprKind::Ref(_, inner) | HirExprKind::Deref(inner) => generate_expr(codegen, inner, types, w)?,
        HirExprKind::Block(block) => {
            w.write("(() => ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(")()");
        }
        HirExprKind::Si(cond, then_block, else_block) => {
            w.write("(");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ? ");
            w.write("(() => ");
            stmt::generate_inline_block(codegen, then_block, types, w)?;
            w.write(")()");
            w.write(" : ");
            if let Some(else_block) = else_block {
                w.write("(() => ");
                stmt::generate_inline_block(codegen, else_block, types, w)?;
                w.write(")()");
            } else {
                w.write("undefined");
            }
            w.write(")");
        }
        HirExprKind::Discerne(_, _) => {
            w.write("undefined");
        }
        HirExprKind::Loop(block) => {
            w.write("(() => { while (true) ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Dum(cond, block) => {
            w.write("(() => { while (");
            generate_expr(codegen, cond, types, w)?;
            w.write(") ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Itera(mode, def_id, iter, block) => {
            w.write("(() => { for (const ");
            w.write(codegen.resolve_def(*def_id));
            match mode {
                crate::hir::HirIteraMode::Ex | crate::hir::HirIteraMode::Pro => w.write(" of "),
                crate::hir::HirIteraMode::De => w.write(" in "),
            }
            generate_expr(codegen, iter, types, w)?;
            w.write(") ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Ab {
            source,
            filter,
            transforms,
        } => {
            generate_expr(codegen, source, types, w)?;
            if let Some(filter) = filter {
                match &filter.kind {
                    HirCollectionFilterKind::Property(name) => {
                        w.write(".filter((x) => ");
                        if filter.negated {
                            w.write("!");
                        }
                        w.write("x.");
                        w.write(codegen.resolve_symbol(*name));
                        w.write(")");
                    }
                    HirCollectionFilterKind::Condition(cond) => {
                        w.write(".filter((_x) => ");
                        generate_expr(codegen, cond, types, w)?;
                        w.write(")");
                    }
                }
            }
            for transform in transforms {
                match transform.kind {
                    HirTransformKind::First => {
                        w.write(".slice(0, ");
                        if let Some(arg) = &transform.arg {
                            generate_expr(codegen, arg, types, w)?;
                        } else {
                            w.write("1");
                        }
                        w.write(")");
                    }
                    HirTransformKind::Last => {
                        w.write(".slice(-");
                        if let Some(arg) = &transform.arg {
                            generate_expr(codegen, arg, types, w)?;
                        } else {
                            w.write("1");
                        }
                        w.write(")");
                    }
                    HirTransformKind::Sum => {
                        w.write(".reduce((acc, value) => acc + value, 0)");
                    }
                }
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            w.write("(() => { if (!(");
            generate_expr(codegen, cond, types, w)?;
            w.write(")) { throw new Error(");
            if let Some(message) = message {
                generate_expr(codegen, message, types, w)?;
            } else {
                w.write("\"assertion failed\"");
            }
            w.write("); } })()");
        }
        HirExprKind::Error => return Err(CodegenError { message: "cannot emit TS for error expression".to_owned() }),
    }
    Ok(())
}

fn generate_literal(codegen: &TsCodegen<'_>, literal: &HirLiteral, w: &mut CodeWriter) {
    match literal {
        HirLiteral::Int(v) => w.write(&v.to_string()),
        HirLiteral::Float(v) => w.write(&v.to_string()),
        HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        HirLiteral::Regex(pattern, flags) => {
            w.write("/");
            w.write(codegen.resolve_symbol(*pattern));
            w.write("/");
            if let Some(flags) = flags {
                w.write(codegen.resolve_symbol(*flags));
            }
        }
        HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        HirLiteral::Nil => w.write("null"),
    }
}

fn binary_op_to_ts(op: crate::hir::HirBinOp) -> &'static str {
    use crate::hir::HirBinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        Eq | StrictEq => "===",
        NotEq | StrictNotEq => "!==",
        Lt => "<",
        Gt => ">",
        LtEq => "<=",
        GtEq => ">=",
        And => "&&",
        Or => "||",
        Coalesce => "??",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        Is => "===",
        IsNot => "!==",
        InRange | Between => "&&",
    }
}

fn assign_op_to_ts(op: crate::hir::HirBinOp) -> &'static str {
    use crate::hir::HirBinOp::*;
    match op {
        Add => "+=",
        Sub => "-=",
        Mul => "*=",
        Div => "/=",
        Mod => "%=",
        BitAnd => "&=",
        BitOr => "|=",
        BitXor => "^=",
        Shl => "<<=",
        Shr => ">>=",
        _ => "=",
    }
}

fn unary_op_to_ts(op: HirUnOp) -> &'static str {
    match op {
        HirUnOp::Neg => "-",
        HirUnOp::Not => "!",
        HirUnOp::BitNot => "~",
        HirUnOp::IsNull | HirUnOp::IsNil => "===",
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => "!==",
        HirUnOp::IsNeg => "<",
        HirUnOp::IsPos => ">",
        HirUnOp::IsTrue => "===",
        HirUnOp::IsFalse => "!==",
    }
}

fn contains_await_in_block(block: &HirBlock) -> bool {
    block.stmts.iter().any(contains_await_in_stmt)
        || block.expr.as_ref().is_some_and(|expr| contains_await_in_expr(expr))
}

fn contains_await_in_stmt(stmt: &crate::hir::HirStmt) -> bool {
    match &stmt.kind {
        HirStmtKind::Local(local) => local.init.as_ref().is_some_and(contains_await_in_expr),
        HirStmtKind::Expr(expr) => contains_await_in_expr(expr),
        HirStmtKind::Redde(expr) => expr.as_ref().is_some_and(contains_await_in_expr),
        HirStmtKind::Rumpe | HirStmtKind::Perge => false,
    }
}

fn contains_await_in_expr(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Cede(_) => true,
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            contains_await_in_expr(lhs) || contains_await_in_expr(rhs)
        }
        HirExprKind::Unary(_, operand)
        | HirExprKind::Ref(_, operand)
        | HirExprKind::Deref(operand)
        | HirExprKind::Panic(operand)
        | HirExprKind::Throw(operand)
        | HirExprKind::Qua(operand, _) => contains_await_in_expr(operand),
        HirExprKind::Call(callee, args) | HirExprKind::MethodCall(callee, _, args) => {
            contains_await_in_expr(callee) || args.iter().any(contains_await_in_expr)
        }
        HirExprKind::Field(object, _) => contains_await_in_expr(object),
        HirExprKind::Index(object, index) => contains_await_in_expr(object) || contains_await_in_expr(index),
        HirExprKind::OptionalChain(object, chain) => {
            contains_await_in_expr(object)
                || match chain {
                    HirOptionalChainKind::Member(_) => false,
                    HirOptionalChainKind::Index(index) => contains_await_in_expr(index),
                    HirOptionalChainKind::Call(args) => args.iter().any(contains_await_in_expr),
                }
        }
        HirExprKind::Ab { source, filter, transforms } => {
            contains_await_in_expr(source)
                || filter.as_ref().is_some_and(|filter| match &filter.kind {
                    HirCollectionFilterKind::Condition(cond) => contains_await_in_expr(cond),
                    HirCollectionFilterKind::Property(_) => false,
                })
                || transforms
                    .iter()
                    .any(|transform| transform.arg.as_ref().is_some_and(|arg| contains_await_in_expr(arg)))
        }
        HirExprKind::Block(block) | HirExprKind::Loop(block) => contains_await_in_block(block),
        HirExprKind::Si(cond, then_block, else_block) => {
            contains_await_in_expr(cond)
                || contains_await_in_block(then_block)
                || else_block.as_ref().is_some_and(contains_await_in_block)
        }
        HirExprKind::Discerne(scrutinee, arms) => {
            contains_await_in_expr(scrutinee)
                || arms.iter().any(|arm| {
                    arm.guard.as_ref().is_some_and(contains_await_in_expr) || contains_await_in_expr(&arm.body)
                })
        }
        HirExprKind::Dum(cond, block) => contains_await_in_expr(cond) || contains_await_in_block(block),
        HirExprKind::Itera(_, _, iter, block) => contains_await_in_expr(iter) || contains_await_in_block(block),
        HirExprKind::Array(values) | HirExprKind::Tuple(values) | HirExprKind::Scribe(values) => {
            values.iter().any(contains_await_in_expr)
        }
        HirExprKind::Scriptum(_, args) => args.iter().any(contains_await_in_expr),
        HirExprKind::Adfirma(cond, msg) => {
            contains_await_in_expr(cond) || msg.as_ref().is_some_and(|msg| contains_await_in_expr(msg))
        }
        HirExprKind::Struct(_, fields) => fields.iter().any(|(_, value)| contains_await_in_expr(value)),
        HirExprKind::Tempta { body, catch, finally } => {
            contains_await_in_block(body)
                || catch.as_ref().is_some_and(contains_await_in_block)
                || finally.as_ref().is_some_and(contains_await_in_block)
        }
        HirExprKind::Clausura(_, _, body) => contains_await_in_expr(body),
        HirExprKind::Innatum { source, map_entries, .. } => {
            contains_await_in_expr(source)
                || map_entries
                    .as_ref()
                    .is_some_and(|entries| entries.iter().any(|(_, value)| contains_await_in_expr(value)))
        }
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => false,
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
