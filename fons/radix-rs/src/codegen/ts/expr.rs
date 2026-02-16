use super::stmt;
use super::types;
use super::{CodeWriter, CodegenError, TsCodegen};
use crate::hir::{
    HirBinOp, HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirIteraMode, HirLiteral, HirOptionalChainKind,
    HirStmtKind, HirTransformKind, HirUnOp,
};
use crate::semantic::{Primitive, Type, TypeTable};

pub fn generate_expr(
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
        HirExprKind::Unary(op, operand) => generate_unary_expr(codegen, *op, operand, types, w)?,
        HirExprKind::Call(callee, args) => {
            if try_generate_intrinsic_call(codegen, callee, args, types, w)? {
                return Ok(());
            }
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
            if try_generate_translated_method_call(codegen, receiver, *method, args, types, w)? {
                return Ok(());
            }
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
            w.write("`");
            w.write(&render_scriptum_template(codegen.resolve_symbol(*template), args.len()));
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
        HirExprKind::Verte { source, target, entries } => match types.get(*target) {
            Type::Struct(_) => {
                if let Some(entries) = entries {
                    w.write("{ ");
                    for (idx, (name, value)) in entries.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        w.write(codegen.resolve_symbol(*name));
                        w.write(": ");
                        generate_expr(codegen, value, types, w)?;
                    }
                    w.write(" }");
                } else {
                    generate_expr(codegen, source, types, w)?;
                    w.write(" as ");
                    w.write(&types::type_to_ts(codegen, *target, types));
                }
            }
            _ => {
                generate_expr(codegen, source, types, w)?;
                w.write(" as ");
                w.write(&types::type_to_ts(codegen, *target, types));
            }
        },
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
                HirIteraMode::Ex | HirIteraMode::Pro => w.write(" of "),
                HirIteraMode::De => w.write(" in "),
            }
            generate_expr(codegen, iter, types, w)?;
            w.write(") ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Ab { source, filter, transforms } => {
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
        HirExprKind::Error => {
            return Err(CodegenError {
                message: "cannot emit TS for error expression".to_owned(),
            });
        }
    }
    Ok(())
}

fn try_generate_intrinsic_call(
    codegen: &TsCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let HirExprKind::Path(def_id) = callee.kind else {
        return Ok(false);
    };
    let name = codegen.resolve_def(def_id);
    let mapped = match name {
        "scribe" => Some("console.log"),
        "vide" => Some("console.debug"),
        "mone" => Some("console.warn"),
        "lege" => Some("prompt"),
        "pavimentum" => Some("Math.floor"),
        "nunc" => None,
        _ => None,
    };

    if name == "nunc" && args.is_empty() {
        w.write("new Date()");
        return Ok(true);
    }

    let Some(target) = mapped else {
        return Ok(false);
    };
    w.write(target);
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(true)
}

fn try_generate_translated_method_call(
    codegen: &TsCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let method_name = codegen.resolve_symbol(method);
    let receiver_type = receiver.ty.map(|ty| normalize_receiver_type(types.get(ty), types));

    let is_lista = matches!(receiver_type, Some(Type::Array(_)));
    let is_tabula = matches!(receiver_type, Some(Type::Map(_, _)));
    let is_textus = matches!(receiver_type, Some(Type::Primitive(Primitive::Textus)));

    if method_name == "longitudo" && args.is_empty() && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write(".length");
        return Ok(true);
    }

    if method_name == "accipe" && args.len() == 1 && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write("[");
        generate_expr(codegen, &args[0], types, w)?;
        w.write("]");
        return Ok(true);
    }

    if is_tabula {
        match method_name {
            "pone" if args.len() == 2 => {
                w.write("(");
                generate_expr(codegen, receiver, types, w)?;
                w.write("[");
                generate_expr(codegen, &args[0], types, w)?;
                w.write("] = ");
                generate_expr(codegen, &args[1], types, w)?;
                w.write(")");
                return Ok(true);
            }
            "accipe" if args.len() == 1 => {
                generate_expr(codegen, receiver, types, w)?;
                w.write("[");
                generate_expr(codegen, &args[0], types, w)?;
                w.write("]");
                return Ok(true);
            }
            "habet" if args.len() == 1 => {
                w.write("(");
                generate_expr(codegen, &args[0], types, w)?;
                w.write(" in ");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            "claves" if args.is_empty() => {
                w.write("Object.keys(");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            "valores" if args.is_empty() => {
                w.write("Object.values(");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            _ => {}
        }
    }

    let translated = if is_lista {
        match method_name {
            "appende" => Some("push"),
            "decapita" => Some("shift"),
            "detrahe" => Some("pop"),
            "filtrata" => Some("filter"),
            "mappata" => Some("map"),
            "reducta" => Some("reduce"),
            "ordinata" => Some("toSorted"),
            "inversa" => Some("reverse"),
            "coniunge" => Some("join"),
            "continet" => Some("includes"),
            "plana" => Some("flat"),
            "seca" => Some("slice"),
            _ => None,
        }
    } else if is_textus {
        match method_name {
            "maiuscula" => Some("toUpperCase"),
            "minuscula" => Some("toLowerCase"),
            "divide" => Some("split"),
            "continet" => Some("includes"),
            "incipe" => Some("startsWith"),
            "fini" => Some("endsWith"),
            "repone" => Some("replace"),
            "reseca" => Some("trim"),
            _ => None,
        }
    } else {
        None
    };

    let Some(translated) = translated else {
        return Ok(false);
    };
    generate_expr(codegen, receiver, types, w)?;
    w.write(".");
    w.write(translated);
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(true)
}

fn normalize_receiver_type<'a>(mut ty: &'a Type, types: &'a TypeTable) -> &'a Type {
    loop {
        match ty {
            Type::Ref(_, inner) | Type::Alias(_, inner) => {
                ty = types.get(*inner);
            }
            _ => return ty,
        }
    }
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

fn generate_unary_expr(
    codegen: &TsCodegen<'_>,
    op: HirUnOp,
    operand: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match op {
        HirUnOp::Neg => {
            w.write("-");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::Not => {
            w.write("!");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::BitNot => {
            w.write("~");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::IsNull | HirUnOp::IsNil => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == null)");
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" != null)");
        }
        HirUnOp::IsNeg => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" < 0)");
        }
        HirUnOp::IsPos => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" > 0)");
        }
        HirUnOp::IsTrue => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" === true)");
        }
        HirUnOp::IsFalse => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" === false)");
        }
    }
    Ok(())
}

fn binary_op_to_ts(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
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

fn assign_op_to_ts(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
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

fn render_scriptum_template(template: &str, arg_count: usize) -> String {
    let mut rendered = template.to_owned();
    for idx in (1..=arg_count).rev() {
        rendered = rendered.replace(&format!("§{}", idx), &format!("${{{}}}", idx - 1));
    }

    let mut auto_index = 0usize;
    while let Some(pos) = rendered.find('§') {
        rendered.replace_range(pos..=pos, &format!("${{{}}}", auto_index));
        auto_index += 1;
    }

    rendered
}

pub fn contains_await_in_block(block: &HirBlock) -> bool {
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
        | HirExprKind::Throw(operand) => contains_await_in_expr(operand),
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
        HirExprKind::Verte { source, entries, .. } => {
            contains_await_in_expr(source)
                || entries
                    .as_ref()
                    .is_some_and(|entries| entries.iter().any(|(_, value)| contains_await_in_expr(value)))
        }
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => false,
    }
}
