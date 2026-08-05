#[cfg(test)]
mod tests {
    struct GrammarValidator;

    impl GrammarValidator {
        fn is_valid_single_line(output: &str) -> bool {
            if output.is_empty() {
                return false;
            }
            if output.contains('\n') || output.contains('\r') || output.contains('\0') {
                return false;
            }
            if output.starts_with("```") || output.contains("```") {
                return false;
            }
            true
        }
    }

    #[test]
    fn test_valid_single_line_passes() {
        assert!(GrammarValidator::is_valid_single_line("ckout main"));
        assert!(GrammarValidator::is_valid_single_line("-la"));
        assert!(GrammarValidator::is_valid_single_line("build --release"));
    }

    #[test]
    fn test_newline_rejected() {
        assert!(!GrammarValidator::is_valid_single_line("line1\nline2"));
    }

    #[test]
    fn test_carriage_return_rejected() {
        assert!(!GrammarValidator::is_valid_single_line("line\r"));
    }

    #[test]
    fn test_null_byte_rejected() {
        assert!(!GrammarValidator::is_valid_single_line("bad\0char"));
    }

    #[test]
    fn test_markdown_fence_rejected() {
        assert!(!GrammarValidator::is_valid_single_line("```bash\nls```"));
    }

    #[test]
    fn test_empty_rejected() {
        assert!(!GrammarValidator::is_valid_single_line(""));
    }

    #[test]
    fn test_multiline_rejected() {
        assert!(!GrammarValidator::is_valid_single_line("ls\necho hello"));
    }
}
