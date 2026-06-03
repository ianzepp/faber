//! Package-mode path detection for CLI inputs.

use super::compile::should_treat_as_package;

pub(crate) fn should_treat_as_package_from_input(input: &[String]) -> bool {
    if input.is_empty() || input[0] == "-" {
        return false;
    }
    let path = std::path::Path::new(&input[0]);
    should_treat_as_package(path)
}
