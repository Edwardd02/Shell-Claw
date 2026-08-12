use std::time::{SystemTime, UNIX_EPOCH};

use super::db::Database;
use super::{MemoryResult, RetrievalCandidate, RetrievalQuery};

pub fn retrieve(db: &Database, query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(query.deadline_ms);

    let fts_query = format!("\"{}\"*", query.line_prefix.replace('"', "\"\""));
    let raw = db.retrieve(&fts_query, query.limit)?;

    if std::time::Instant::now() > deadline {
        return Ok(vec![]);
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

    let mut candidates: Vec<RetrievalCandidate> = raw
        .into_iter()
        .map(|(id, cmd, cwd, last_used, use_count, raw_bm25)| {
            // FTS5 bm25/rank is smaller for a better match, often negative.
            // Convert it into a positive score before sorting descending.
            let bm25 = -raw_bm25;
            let cwd_score = cwd_match(&query.cwd, &cwd);
            let freq_score = frequency_score(use_count);
            let recency_score = recency_score(last_used, now);
            let final_score =
                0.40 * bm25 + 0.25 * cwd_score + 0.20 * freq_score + 0.15 * recency_score;

            let _ = id;
            RetrievalCandidate { command: cmd, final_score }
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal)
    });

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::db::Database;

    #[test]
    fn fts_prefix_retrieval_is_fast_and_deduplicates_commands() {
        let path = std::env::temp_dir().join(format!(
            "shellclaw-memory-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let db = Database::open(&path).unwrap();
        for i in 0..2_000 {
            db.execute_insert("/tmp/project", &format!("git checkout feature-{i}"), i).unwrap();
        }
        db.execute_insert("/tmp/project", "git checkout feature-1999", 3_000).unwrap();

        let started = std::time::Instant::now();
        let results = retrieve(
            &db,
            RetrievalQuery {
                cwd: "/tmp/project".into(),
                line_prefix: "git checkout feature-1999".into(),
                limit: 5,
                deadline_ms: 50,
            },
        )
        .unwrap();
        let elapsed = started.elapsed();

        assert!(!results.is_empty());
        assert_eq!(
            results
                .iter()
                .filter(|candidate| candidate.command == "git checkout feature-1999")
                .count(),
            1
        );
        assert!(elapsed < std::time::Duration::from_millis(20), "{elapsed:?}");

        drop(db);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
    }
}
