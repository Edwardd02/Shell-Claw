#[cfg(test)]
mod tests {
    struct SuffixValidator;

    impl SuffixValidator {
        fn validate(suffix: &str, line_prefix: &str) -> bool {
            if suffix.is_empty() {
                return false;
            }
            if suffix == line_prefix {
                return false;
            }
            if suffix.contains('\n') || suffix.contains('\r') || suffix.contains('\0') {
                return false;
            }
            if suffix.contains("```") || suffix.starts_with('#') || suffix.starts_with("Here") {
                return false;
            }
            true
        }
    }

    #[test]
    fn test_valid_suffix_passes() {
        assert!(SuffixValidator::validate("ckout main", "git che"));
    }

    #[test]
    fn test_empty_suffix_rejected() {
        assert!(!SuffixValidator::validate("", "git che"));
    }

    #[test]
    fn test_duplicate_of_input_rejected() {
        assert!(!SuffixValidator::validate("git che", "git che"));
    }

    #[test]
    fn test_explanatory_text_rejected() {
        assert!(!SuffixValidator::validate("Here is a suggestion: ls", "git"));
        assert!(!SuffixValidator::validate("# This is a comment", ""));
    }

    #[test]
    fn test_markdown_rejected() {
        assert!(!SuffixValidator::validate("```bash\nls```", ""));
    }

    #[test]
    fn test_newline_rejected() {
        assert!(!SuffixValidator::validate("line\nnext", ""));
    }

    #[test]
    fn test_stale_suffix_for_different_input_rejected() {
        assert!(SuffixValidator::validate("suffix", "prefix"));
        assert!(!SuffixValidator::validate("prefix", "prefix"));
    }
}
