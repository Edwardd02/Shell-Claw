use protocol::CompletionParams;
use protocol::CompletionResult;

pub struct RequestValidator;

impl RequestValidator {
    pub fn validate(params: &CompletionParams) -> Result<(), CompletionResult> {
        if params.line.len() > crate::config::DaemonConfig::load().max_line_length {
            return Err(CompletionResult::no_suggestion());
        }

        if params.cursor > params.line.len() {
            return Err(CompletionResult::no_suggestion());
        }

        if params.line.trim().is_empty() {
            return Err(CompletionResult::no_suggestion());
        }

        if params.session_id.is_empty() {
            return Err(CompletionResult::no_suggestion());
        }

        if !params.cwd.starts_with('/') {
            return Err(CompletionResult::no_suggestion());
        }

        if params.line.contains('\0') || params.line.contains('\r') || params.line.contains('\n') {
            return Err(CompletionResult::no_suggestion());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> CompletionParams {
        CompletionParams {
            session_id: "s1".to_string(),
            shell_kind: "zsh".to_string(),
            line: "git che".to_string(),
            cursor: 7,
            cwd: "/tmp".to_string(),
            deadline_ms: 25,
            client_sent_at_ms: 1000,
        }
    }

    #[test]
    fn test_valid_request_passes() {
        assert!(RequestValidator::validate(&valid_params()).is_ok());
    }

    #[test]
    fn test_empty_line_rejected() {
        let mut p = valid_params();
        p.line = "   ".to_string();
        assert!(RequestValidator::validate(&p).is_err());
    }

    #[test]
    fn test_cursor_past_end_rejected() {
        let mut p = valid_params();
        p.cursor = 999;
        assert!(RequestValidator::validate(&p).is_err());
    }

    #[test]
    fn test_missing_session_id_rejected() {
        let mut p = valid_params();
        p.session_id = "".to_string();
        assert!(RequestValidator::validate(&p).is_err());
    }

    #[test]
    fn test_relative_cwd_rejected() {
        let mut p = valid_params();
        p.cwd = "relative/path".to_string();
        assert!(RequestValidator::validate(&p).is_err());
    }

    #[test]
    fn test_null_byte_rejected() {
        let mut p = valid_params();
        p.line = "bad\0char".to_string();
        assert!(RequestValidator::validate(&p).is_err());
    }
}
