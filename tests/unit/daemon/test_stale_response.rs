#[cfg(test)]
mod tests {
    struct StaleResponseValidator;

    impl StaleResponseValidator {
        fn response_matches_request(
            response_request_id: &str,
            active_request_id: &str,
            response_line_hash: &str,
            current_line: &str,
        ) -> bool {
            response_request_id == active_request_id
                && response_line_hash == StaleResponseValidator::hash_line(current_line)
        }

        fn hash_line(line: &str) -> String {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            line.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        }
    }

    #[test]
    fn test_response_id_mismatch_rejected() {
        assert!(!StaleResponseValidator::response_matches_request(
            "r1",
            "r2",
            "hash1",
            "line1"
        ));
    }

    #[test]
    fn test_response_line_mismatch_rejected() {
        assert!(!StaleResponseValidator::response_matches_request(
            "r1",
            "r1",
            "wrong_hash",
            "line1"
        ));
    }

    #[test]
    fn test_response_id_and_line_match_accepted() {
        let line = "git che";
        let hash = StaleResponseValidator::hash_line(line);
        assert!(StaleResponseValidator::response_matches_request(
            "r1", "r1", &hash, line
        ));
    }

    #[test]
    fn test_response_cursor_moved_rejected() {
        let line1 = "git ch";
        let line2 = "git checkout";
        let hash1 = StaleResponseValidator::hash_line(line1);
        assert!(!StaleResponseValidator::response_matches_request(
            "r1", "r1", &hash1, line2
        ));
    }

    #[test]
    fn test_response_empty_line_rejected_when_user_erased() {
        let line1 = "git che";
        let line2 = "";
        let hash1 = StaleResponseValidator::hash_line(line1);
        assert!(!StaleResponseValidator::response_matches_request(
            "r1", "r1", &hash1, line2
        ));
    }
}
