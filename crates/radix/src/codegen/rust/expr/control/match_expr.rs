//! Match/`elige` expression lowering.

use super::*;

pub(in crate::codegen::rust::expr) fn generate_match_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    scrutinees: &[HirExpr],
    arms: &[HirCasuArm],
) -> Result<(), CodegenError> {
    emitter.writer.write("match ");
    if scrutinees.len() == 1 {
        emitter.expr(&scrutinees[0])?;
        if matches!(
            scrutinees[0].ty.map(|ty| resolve_type(ty, emitter.types)),
            Some(Type::Primitive(Primitive::Textus))
        ) {
            emitter.writer.write(".as_str()");
        }
    } else {
        emitter.writer.write("(");
        for (idx, scrutinee) in scrutinees.iter().enumerate() {
            if idx > 0 {
                emitter.writer.write(", ");
            }
            emitter.expr(scrutinee)?;
        }
        emitter.writer.write(")");
    }
    emitter.writer.writeln(" {");
    let mut discerne_result = Ok(());
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|w| {
        let mut arm_emitter = ExprEmitter::new(codegen, types, w, policy);
        for arm in arms {
            if arm.patterns.len() == 1 {
                generate_pattern(codegen, &arm.patterns[0], arm_emitter.writer);
            } else {
                arm_emitter.writer.write("(");
                for (idx, pattern) in arm.patterns.iter().enumerate() {
                    if idx > 0 {
                        arm_emitter.writer.write(", ");
                    }
                    generate_pattern(codegen, pattern, arm_emitter.writer);
                }
                arm_emitter.writer.write(")");
            }
            if let Some(guard) = &arm.guard {
                arm_emitter.writer.write(" if ");
                if discerne_result.is_err() {
                    return;
                }
                discerne_result = arm_emitter.expr(guard);
            }
            arm_emitter.writer.write(" => ");
            if discerne_result.is_err() {
                return;
            }
            discerne_result = arm_emitter.expr(&arm.body);
            arm_emitter.writer.writeln(",");
        }
        if !arms.iter().any(arm_has_wildcard_pattern) && match_scrutinee_is_enum(scrutinees, types) {
            arm_emitter.writer.writeln("_ => unreachable!(),");
        } else if !arms.iter().any(arm_has_wildcard_pattern) {
            arm_emitter.writer.writeln("_ => {},");
        }
    });
    discerne_result?;
    emitter.writer.write("}");
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
