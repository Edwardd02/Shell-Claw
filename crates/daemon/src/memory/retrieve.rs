use std::time::{SystemTime, UNIX_EPOCH};

use super::db::Database;
use super::{MemoryError, MemoryResult, RetrievalCandidate, RetrievalQuery};

pub fn retrieve(db: &Database, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>> {
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(query.deadline_ms);

    let fts_query = format!("\"{}\"*", query.line_prefix.replace('"', "\"\""));
    let raw = db.retrieve(&fts_query, query.limit)?;

    if std::time::Instant::now() > deadline {
        return Ok(vec![]);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut candidates: Vec<RetrievalCandidate> = raw
        .into_iter()
        .map(|(id, cmd, cwd, last_used, use_count, bm25)| {
            let cwd_score = cwd_match(&query.cwd, &cwd);
            let freq_score = frequency_score(use_count);
            let recency_score = recency_score(last_used, now);
            let final_score = 0.40 * bm25
                + 0.25 * cwd_score
                + 0.20 * freq_score
                + 0.15 * recency_score;

            RetrievalCandidate {
                entry_id: id,
                command: cmd,
                cwd,
                bm25_score: bm25,
                cwd_score,
                frequency_score: freq_score,
                recency_score,
                final_score,
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(candidates)
}

fn cwd_match(query_cwd: &str, entry_cwd: &str) -> f64 {
    if query_cwd == entry_cwd {
        1.0
    } else if entry_cwd.starts_with(query_cwd) {
        0.5
    } else if query_cwd.starts_with(entry_cwd) {
        0.3
    } else {
        0.0
    }
}

fn frequency_score(use_count: i32) -> f64 {
    (1.0 + use_count as f64).ln() / 10.0
}

fn recency_score(last_used: i64, now: i64) -> f64 {
    let delta = (now - last_used) as f64;
    let lambda = 0.15;
    (-lambda * delta / 86400.0).exp()
}
