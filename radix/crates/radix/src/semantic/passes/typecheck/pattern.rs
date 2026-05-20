use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_pattern(&mut self, pattern: &HirPattern, expected: TypeId, span: crate::lexer::Span) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Binding(def_id, _name) => {
                self.insert_binding(*def_id, expected, false);
            }
            HirPattern::Alias(def_id, _name, pattern) => {
                self.insert_binding(*def_id, expected, false);
                if let HirPattern::Variant(variant_def, patterns) = pattern.as_ref() {
                    if patterns.is_empty() {
                        let Some(parent) = self.variant_parent.get(variant_def).copied() else {
                            self.error(SemanticErrorKind::UndefinedVariable, "unknown variant", span);
                            return;
                        };
                        if let Some(expected_parent) = self.enum_def_from_type(expected) {
                            if expected_parent != parent {
                                self.error(
                                    SemanticErrorKind::TypeMismatch,
                                    "variant does not match scrutinee type",
                                    span,
                                );
                            }
                        }
                        return;
                    }
                }
                self.check_pattern(pattern, expected, span);
            }
            HirPattern::Literal(lit) => {
                let lit_ty = self.literal_type(lit);
                self.unify(lit_ty, expected, span, "pattern type mismatch");
            }
            HirPattern::Variant(variant_def, patterns) => {
                let Some(parent) = self.variant_parent.get(variant_def).copied() else {
                    self.error(SemanticErrorKind::UndefinedVariable, "unknown variant", span);
                    return;
                };

                if let Some(expected_parent) = self.enum_def_from_type(expected) {
                    if expected_parent != parent {
                        self.error(SemanticErrorKind::TypeMismatch, "variant does not match scrutinee type", span);
                    }
                }

                let fields = self
                    .variant_fields
                    .get(variant_def)
                    .cloned()
                    .unwrap_or_default();
                if fields.len() != patterns.len() {
                    self.error(SemanticErrorKind::WrongArity, "variant pattern arity mismatch", span);
                }
                for (sub, field_ty) in patterns.iter().zip(fields.iter()) {
                    self.check_pattern(sub, *field_ty, span);
                }
            }
        }
    }
}
