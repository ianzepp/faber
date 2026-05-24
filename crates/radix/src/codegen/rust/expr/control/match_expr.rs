//! Match/`elige` expression lowering.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_match_expr(
    codegen: &RustCodegen<'_>,
    scrutinees: &[HirExpr],
    arms: &[HirCasuArm],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("match ");
    if scrutinees.len() == 1 {
        generate_expr(
            codegen,
            &scrutinees[0],
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        if matches!(
            scrutinees[0].ty.map(|ty| resolve_type(ty, types)),
            Some(Type::Primitive(Primitive::Textus))
        ) {
            w.write(".as_str()");
        }
    } else {
        w.write("(");
        for (idx, scrutinee) in scrutinees.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(
                codegen,
                scrutinee,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        w.write(")");
    }
    w.writeln(" {");
    let mut discerne_result = Ok(());
    w.indented(|w| {
        for arm in arms {
            if arm.patterns.len() == 1 {
                generate_pattern(codegen, &arm.patterns[0], w);
            } else {
                w.write("(");
                for (idx, pattern) in arm.patterns.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    generate_pattern(codegen, pattern, w);
                }
                w.write(")");
            }
            if let Some(guard) = &arm.guard {
                w.write(" if ");
                if discerne_result.is_err() {
                    return;
                }
                discerne_result =
                    generate_expr(codegen, guard, types, w, in_failable_fn, in_entry, suppress_error_propagation);
            }
            w.write(" => ");
            if discerne_result.is_err() {
                return;
            }
            discerne_result = generate_expr(
                codegen,
                &arm.body,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            w.writeln(",");
        }
        if !arms.iter().any(arm_has_wildcard_pattern) && match_scrutinee_is_enum(scrutinees, types) {
            w.writeln("_ => unreachable!(),");
        } else if !arms.iter().any(arm_has_wildcard_pattern) {
            w.writeln("_ => {},");
        }
    });
    discerne_result?;
    w.write("}");
    Ok(())
}

fn arm_has_wildcard_pattern(arm: &HirCasuArm) -> bool {
    arm.patterns
        .iter()
        .any(|pattern| matches!(pattern, HirPattern::Wildcard))
}

fn match_scrutinee_is_enum(scrutinees: &[HirExpr], types: &TypeTable) -> bool {
    matches!(
        scrutinees,
        [scrutinee]
            if matches!(scrutinee.ty.map(|ty| resolve_type(ty, types)), Some(Type::Enum(_)))
    )
}
