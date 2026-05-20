use super::*;
pub(super) fn render_scriptum_template(template: &str, arg_count: usize) -> String {
    let mut rendered = template.to_owned();
    // Replace §N placeholders with Go's %v format verbs
    for idx in (1..=arg_count).rev() {
        rendered = rendered.replace(&format!("§{}", idx), "%v");
    }
    // Replace bare § with %v
    rendered = rendered.replace('§', "%v");
    rendered
}
pub(super) fn generate_literal(codegen: &GoCodegen<'_>, literal: &HirLiteral, w: &mut CodeWriter) {
    match literal {
        HirLiteral::Int(v) => w.write(&v.to_string()),
        HirLiteral::Float(v) => w.write(&v.to_string()),
        HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        HirLiteral::Regex(pattern, flags) => {
            w.write("regexp.MustCompile(`");
            w.write(codegen.resolve_symbol(*pattern));
            if let Some(flags) = flags {
                w.write("(?");
                w.write(codegen.resolve_symbol(*flags));
                w.write(")");
            }
            w.write("`)");
        }
        HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        HirLiteral::Nil => w.write("nil"),
    }
}
