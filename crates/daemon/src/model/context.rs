use crate::memory::RetrievalCandidate;
use super::{GrammarId, ModelContext};

const MAX_CANDIDATES: usize = 5;
const MAX_CONTEXT_TOKENS: usize = 512;

pub fn build_context(
    line_prefix: &str,
    cwd: &str,
    candidates: &[RetrievalCandidate],
    deadline_ms: u64,
) -> Option<ModelContext> {
    if line_prefix.trim().is_empty() {
        return None;
    }

    if line_prefix.len() > 4096 {
        return None;
    }

    let limited_candidates: Vec<RetrievalCandidate> = candidates
        .iter()
        .take(MAX_CANDIDATES)
        .cloned()
        .collect();

    let estimated_tokens = line_prefix.split_whitespace().count()
        + limited_candidates.len() * 10;

    if estimated_tokens > MAX_CONTEXT_TOKENS {
        return None;
    }

    Some(ModelContext {
        line_prefix: line_prefix.to_string(),
        cwd: cwd.to_string(),
        retrieval_candidates: limited_candidates,
        grammar_id: GrammarId::SingleLine,
        deadline_ms,
    })
}
