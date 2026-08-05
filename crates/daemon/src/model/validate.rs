pub fn validate_suffix(suffix: &str, line_prefix: &str) -> bool {
    if suffix.is_empty() {
        return false;
    }

    if suffix.contains('\n') || suffix.contains('\r') || suffix.contains('\0') {
        return false;
    }

    if suffix.contains("```") {
        return false;
    }

    let explanatory = ["Here", "Sure", "The command", "You should", "I suggest"];
    for prefix in &explanatory {
        if suffix.starts_with(prefix) {
            return false;
        }
    }

    if suffix == line_prefix {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_suffix() {
        assert!(validate_suffix("ckout main", "git che"));
    }

    #[test]
    fn test_empty_suffix() {
        assert!(!validate_suffix("", "git"));
    }

    #[test]
    fn test_newline_suffix() {
        assert!(!validate_suffix("line\nbreak", ""));
    }

    #[test]
    fn test_explanatory_suffix() {
        assert!(!validate_suffix("Here is a command: ls", ""));
    }

    #[test]
    fn test_duplicate_suffix() {
        assert!(!validate_suffix("git", "git"));
    }
}
