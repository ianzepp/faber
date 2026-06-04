//! Runtime module bridge helpers for built-in norma calls.

pub(super) fn norma_runtime_module_path(receiver_name: &str) -> Option<&'static str> {
    match receiver_name {
        "json" => Some("norma::json"),
        "toml" => Some("norma::toml"),
        "consolum" => Some("norma::hal::consolum"),
        "http" => Some("norma::hal::http"),
        _ => None,
    }
}

pub(super) fn norma_runtime_method_name(method_name: &str) -> String {
    let mut lowered = String::with_capacity(method_name.len());
    for (i, ch) in method_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                lowered.push('_');
            }
            lowered.push(ch.to_ascii_lowercase());
        } else {
            lowered.push(ch);
        }
    }
    lowered
}
