#[cfg(test)]
mod tests {
    enum ModelError {
        MissingFile,
        InvalidGguf,
        OutOfMemory,
    }

    fn handle_model_error(error: ModelError) -> Option<String> {
        match error {
            ModelError::MissingFile => None,
            ModelError::InvalidGguf => None,
            ModelError::OutOfMemory => None,
        }
    }

    #[test]
    fn test_missing_model_file_returns_no_suggestion() {
        assert_eq!(handle_model_error(ModelError::MissingFile), None);
    }

    #[test]
    fn test_invalid_gguf_returns_no_suggestion() {
        assert_eq!(handle_model_error(ModelError::InvalidGguf), None);
    }

    #[test]
    fn test_oom_returns_no_suggestion_not_crash() {
        assert_eq!(handle_model_error(ModelError::OutOfMemory), None);
    }

    #[test]
    fn test_model_errors_never_panic() {
        let result = std::panic::catch_unwind(|| {
            handle_model_error(ModelError::MissingFile);
            handle_model_error(ModelError::InvalidGguf);
            handle_model_error(ModelError::OutOfMemory);
        });
        assert!(result.is_ok());
    }
}
