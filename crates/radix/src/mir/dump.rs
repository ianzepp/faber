use crate::mir::{
    MirAggregate, MirAggregateFields, MirAggregateKind, MirBinOp, MirBlockId, MirCallee, MirCollectionOp, MirConstant,
    MirConversion, MirConversionFlavor, MirDiagnosticKind, MirFunctionId, MirIntrinsic, MirKeyValueOperand, MirLocalId,
    MirNamedOperand, MirOperand, MirOptionChainLink, MirOptionOp, MirOptionUnwrapMode, MirPlace, MirPlaceBase,
    MirProgram, MirProjection, MirProvider, MirRuntimeCall, MirStmtKind, MirTempId, MirTerminatorKind, MirType,
    MirUnOp, MirValue, MirValueId, MirValueKind,
};

pub fn dump_program(program: &MirProgram) -> String {
    let mut out = String::new();
    for (index, function) in program.functions.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let error_ty = function
            .error_ty
            .map(|error_ty| format!(" ⇥ {}", ty(error_ty)))
            .unwrap_or_default();
        out.push_str(&format!(
            "function {} -> {}{} {{\n",
            function_id(function.id),
            ty(function.return_ty),
            error_ty
        ));
        if !function.params.is_empty() {
            out.push_str("  params:\n");
            for param in &function.params {
                out.push_str(&format!("    {}: {}\n", local_id(param.local), ty(param.ty)));
            }
        }
        if !function.locals.is_empty() {
            out.push_str("  locals:\n");
            for local in &function.locals {
                let mutability = if local.mutable { "var" } else { "let" };
                out.push_str(&format!("    {} {}: {}\n", mutability, local_id(local.id), ty(local.ty)));
            }
        }
        if !function.temps.is_empty() {
            out.push_str("  temps:\n");
            for temp in &function.temps {
                out.push_str(&format!("    {}: {}\n", temp_id(temp.id), ty(temp.ty)));
            }
        }
        for block in &function.blocks {
            out.push_str(&format!("  {}:\n", block_id(block.id)));
            for stmt in &block.statements {
                out.push_str("    ");
                out.push_str(&stmt_kind(&stmt.kind));
                out.push('\n');
            }
            out.push_str("    ");
            out.push_str(&terminator_kind(&block.terminator.kind));
            out.push('\n');
        }
        out.push_str("}\n");
    }
    out
}

fn stmt_kind(kind: &MirStmtKind) -> String {
    match kind {
        MirStmtKind::Assign { place, value } => format!("{} = {}", place_fmt(place), value_fmt(value)),
        MirStmtKind::Call { destination, callee, args } => {
            let lhs = destination
                .as_ref()
                .map(|place| format!("{} = ", place_fmt(place)))
                .unwrap_or_default();
            format!("{lhs}call {}({})", callee_fmt(callee), operands(args))
        }
        MirStmtKind::RuntimeCall { destination, call } => {
            let lhs = destination
                .as_ref()
                .map(|place| format!("{} = ", place_fmt(place)))
                .unwrap_or_default();
            format!("{lhs}runtime {}", runtime_call(call))
        }
        MirStmtKind::Construct { destination, aggregate } => {
            format!("{} = construct {}", place_fmt(destination), aggregate_fmt(aggregate))
        }
    }
}

fn terminator_kind(kind: &MirTerminatorKind) -> String {
    match kind {
        MirTerminatorKind::Return(Some(value)) => format!("return {}", operand(value)),
        MirTerminatorKind::Return(None) => "return".to_owned(),
        MirTerminatorKind::ReturnError(value) => format!("return_error {}", operand(value)),
        MirTerminatorKind::TryCall { destination, callee, args, ok_block, error_place, error_block } => {
            let lhs = destination
                .as_ref()
                .map(|place| format!("{} = ", place_fmt(place)))
                .unwrap_or_default();
            format!(
                "{lhs}try_call {}({}) ok {} error {} -> {}",
                callee_fmt(callee),
                operands(args),
                block_id(*ok_block),
                place_fmt(error_place),
                block_id(*error_block)
            )
        }
        MirTerminatorKind::Goto(target) => format!("goto {}", block_id(*target)),
        MirTerminatorKind::Branch { condition, then_block, else_block } => {
            format!(
                "branch {} {} {}",
                operand(condition),
                block_id(*then_block),
                block_id(*else_block)
            )
        }
        MirTerminatorKind::Switch { value, cases, default } => {
            let rendered_cases = cases
                .iter()
                .map(|case| format!("{}: {}", constant(&case.value), block_id(case.target)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("switch {} [{}] default {}", operand(value), rendered_cases, block_id(*default))
        }
        MirTerminatorKind::Unreachable => "unreachable".to_owned(),
    }
}

fn value_fmt(value: &MirValue) -> String {
    match &value.kind {
        MirValueKind::Operand(operand_value) => format!("{}: {}", operand(operand_value), ty(value.ty)),
        MirValueKind::Unary { op, operand: inner } => {
            format!("{} {}: {}", unop(*op), operand(inner), ty(value.ty))
        }
        MirValueKind::Binary { op, lhs, rhs } => {
            format!("{} {} {}: {}", operand(lhs), binop(*op), operand(rhs), ty(value.ty))
        }
        MirValueKind::Option(op) => format!("option {}: {}", option_op(op), ty(value.ty)),
    }
}

fn runtime_call(call: &MirRuntimeCall) -> String {
    format!(
        "{}({}) -> {}",
        intrinsic(&call.intrinsic),
        operands(&call.args),
        ty(call.return_ty)
    )
}

fn aggregate_fmt(aggregate: &MirAggregate) -> String {
    let kind = match aggregate.kind {
        MirAggregateKind::Tuple => "tuple".to_owned(),
        MirAggregateKind::Array => "array".to_owned(),
        MirAggregateKind::Map => "map".to_owned(),
        MirAggregateKind::Set => "set".to_owned(),
        MirAggregateKind::Struct(def_id) => format!("struct def#{}", def_id.0),
        MirAggregateKind::EnumVariant(def_id) => format!("variant def#{}", def_id.0),
    };
    format!("{kind}: {} {}", ty(aggregate.ty), aggregate_fields(&aggregate.fields))
}

fn aggregate_fields(fields: &MirAggregateFields) -> String {
    match fields {
        MirAggregateFields::Ordered(items) => format!("[{}]", operands(items)),
        MirAggregateFields::Named(items) => {
            let rendered = items
                .iter()
                .map(named_operand)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{rendered}}}")
        }
        MirAggregateFields::Keyed(items) => {
            let rendered = items
                .iter()
                .map(key_value_operand)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{rendered}}}")
        }
    }
}

fn named_operand(field: &MirNamedOperand) -> String {
    format!("sym#{}: {}", field.name.0, operand(&field.value))
}

fn key_value_operand(entry: &MirKeyValueOperand) -> String {
    format!("{} => {}", operand(&entry.key), operand(&entry.value))
}

fn callee_fmt(callee: &MirCallee) -> String {
    match callee {
        MirCallee::Function(id) => function_id(*id),
        MirCallee::Definition(def_id) => format!("def#{}", def_id.0),
        MirCallee::Value(value) => operand(value),
    }
}

fn operands(args: &[MirOperand]) -> String {
    args.iter().map(operand).collect::<Vec<_>>().join(", ")
}

fn operand(value: &MirOperand) -> String {
    match value {
        MirOperand::Place(place) => place_fmt(place),
        MirOperand::Temp(id) => temp_id(*id),
        MirOperand::Value(id) => value_id(*id),
        MirOperand::Constant(value) => constant(value),
    }
}

fn place_fmt(place: &MirPlace) -> String {
    let mut out = match place.base {
        MirPlaceBase::Local(id) => local_id(id),
        MirPlaceBase::Temp(id) => temp_id(id),
    };
    for projection in &place.projections {
        match projection {
            MirProjection::Field(field) => out.push_str(&format!(".sym#{}", field.0)),
            MirProjection::VariantField { variant, field } => {
                out.push_str(&format!(".def#{}.sym#{}", variant.0, field.0));
            }
            MirProjection::Index(index) => out.push_str(&format!("[{}]", operand(index))),
        }
    }
    out
}

fn constant(value: &MirConstant) -> String {
    match value {
        MirConstant::Int(value) => format!("const int {value}"),
        MirConstant::Float(value) => format!("const float {value:?}"),
        MirConstant::String(symbol) => format!("const string sym#{}", symbol.0),
        MirConstant::Bool(value) => format!("const bool {value}"),
        MirConstant::Nil => "const nil".to_owned(),
        MirConstant::Unit => "const unit".to_owned(),
    }
}

fn intrinsic(value: &MirIntrinsic) -> String {
    match value {
        MirIntrinsic::Diagnostic(kind) => format!("diagnostic {}", diagnostic_kind(*kind)),
        MirIntrinsic::FormatString { template } => format!("format_string template sym#{}", template.0),
        MirIntrinsic::Convert(conversion) => conversion_intrinsic(conversion),
        MirIntrinsic::Collection(op) => format!("collection {}", collection_op(*op)),
        MirIntrinsic::Panic => "panic".to_owned(),
        MirIntrinsic::Provider(provider) => provider_intrinsic(provider),
    }
}

fn option_op(op: &MirOptionOp) -> String {
    match op {
        MirOptionOp::None => "none".to_owned(),
        MirOptionOp::Some(value) => format!("some({})", operand(value)),
        MirOptionOp::IsNil(value) => format!("is_nil({})", operand(value)),
        MirOptionOp::IsNonNil(value) => format!("is_non_nil({})", operand(value)),
        MirOptionOp::Unwrap { value, mode } => {
            format!("unwrap_{}({})", option_unwrap_mode(*mode), operand(value))
        }
        MirOptionOp::Coalesce { value, fallback } => {
            format!("coalesce({}, {})", operand(value), operand(fallback))
        }
        MirOptionOp::Chain { base, link } => {
            format!("chain({}, {})", operand(base), option_chain_link(link))
        }
    }
}

fn option_unwrap_mode(mode: MirOptionUnwrapMode) -> &'static str {
    match mode {
        MirOptionUnwrapMode::Assert => "assert",
        MirOptionUnwrapMode::Assume => "assume",
    }
}

fn option_chain_link(link: &MirOptionChainLink) -> String {
    match link {
        MirOptionChainLink::Field(field) => format!(".sym#{}", field.0),
        MirOptionChainLink::VariantField { variant, field } => {
            format!(".def#{}.sym#{}", variant.0, field.0)
        }
        MirOptionChainLink::Index(index) => format!("[{}]", operand(index)),
        MirOptionChainLink::Call { callee, args } => {
            format!("call {}({})", callee_fmt(callee), operands(args))
        }
    }
}

fn diagnostic_kind(kind: MirDiagnosticKind) -> &'static str {
    match kind {
        MirDiagnosticKind::Nota => "nota",
        MirDiagnosticKind::Vide => "vide",
        MirDiagnosticKind::Mone => "mone",
        MirDiagnosticKind::Scribe => "scribe",
    }
}

fn conversion_intrinsic(conversion: &MirConversion) -> String {
    let fallback = conversion
        .fallback
        .as_ref()
        .map(|value| format!(" fallback {}", operand(value)))
        .unwrap_or_default();
    format!(
        "convert {} -> {}{}",
        conversion_flavor(conversion.flavor),
        ty(conversion.target_ty),
        fallback
    )
}

fn conversion_flavor(flavor: MirConversionFlavor) -> &'static str {
    match flavor {
        MirConversionFlavor::Cast => "cast",
        MirConversionFlavor::Runtime => "runtime",
    }
}

fn collection_op(op: MirCollectionOp) -> &'static str {
    match op {
        MirCollectionOp::Append => "append",
        MirCollectionOp::AppendImmutable => "append_immutable",
        MirCollectionOp::Index => "index",
        MirCollectionOp::Length => "length",
        MirCollectionOp::Contains => "contains",
    }
}

fn provider_intrinsic(provider: &MirProvider) -> String {
    let module = provider
        .module
        .iter()
        .map(|symbol| format!("sym#{}", symbol.0))
        .collect::<Vec<_>>()
        .join("/");
    if module.is_empty() {
        format!("provider sym#{}", provider.name.0)
    } else {
        format!("provider {module}::sym#{}", provider.name.0)
    }
}

fn binop(op: MirBinOp) -> &'static str {
    match op {
        MirBinOp::Add => "+",
        MirBinOp::Sub => "-",
        MirBinOp::Mul => "*",
        MirBinOp::Div => "/",
        MirBinOp::Mod => "%",
        MirBinOp::Eq => "==",
        MirBinOp::NotEq => "!=",
        MirBinOp::Lt => "<",
        MirBinOp::Gt => ">",
        MirBinOp::LtEq => "<=",
        MirBinOp::GtEq => ">=",
        MirBinOp::And => "and",
        MirBinOp::Or => "or",
        MirBinOp::Coalesce => "coalesce",
        MirBinOp::BitAnd => "&",
        MirBinOp::BitOr => "|",
        MirBinOp::BitXor => "^",
        MirBinOp::Shl => "<<",
        MirBinOp::Shr => ">>",
    }
}

fn unop(op: MirUnOp) -> &'static str {
    match op {
        MirUnOp::Neg => "neg",
        MirUnOp::Not => "not",
        MirUnOp::BitNot => "bit_not",
        MirUnOp::IsNil => "is_nil",
        MirUnOp::IsNonNil => "is_non_nil",
    }
}

fn ty(value: MirType) -> String {
    match value.layout_id() {
        Some(layout) => format!("ty#{} layout#{}", value.semantic_id().0, layout.0),
        None => format!("ty#{}", value.semantic_id().0),
    }
}

fn function_id(id: MirFunctionId) -> String {
    format!("f{}", id.0)
}

fn block_id(id: MirBlockId) -> String {
    format!("bb{}", id.0)
}

fn local_id(id: MirLocalId) -> String {
    format!("_{}", id.0)
}

fn temp_id(id: MirTempId) -> String {
    format!("%{}", id.0)
}

fn value_id(id: MirValueId) -> String {
    format!("v{}", id.0)
}
