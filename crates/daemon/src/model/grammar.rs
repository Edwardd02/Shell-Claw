pub const SINGLE_LINE_GRAMMAR: &str = r#"root ::= [^\r\n\x00]+"#;

pub fn single_line_grammar_gbnf() -> String {
    SINGLE_LINE_GRAMMAR.to_string()
}

pub fn validate_grammar_output(output: &str) -> bool {
    if output.is_empty() {
        return false;
    }
    if output.contains('\n') || output.contains('\r') || output.contains('\0') {
        return false;
    }
    if output.contains("```") {
        return false;
    }
    true
}
