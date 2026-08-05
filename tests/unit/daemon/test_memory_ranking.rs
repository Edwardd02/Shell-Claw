#[cfg(test)]
mod tests {
    struct HybridRanker {
        bm25_weight: f64,
        cwd_weight: f64,
        freq_weight: f64,
        recency_weight: f64,
    }

    impl HybridRanker {
        fn new() -> Self {
            Self {
                bm25_weight: 0.40,
                cwd_weight: 0.25,
                freq_weight: 0.20,
                recency_weight: 0.15,
            }
        }

        fn cwd_match_score(&self, query_cwd: &str, entry_cwd: &str) -> f64 {
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

        fn frequency_score(&self, use_count: u32) -> f64 {
            (1.0 + use_count as f64).ln() / 10.0
        }

        fn recency_score(&self, last_used: i64, now: i64, lambda: f64) -> f64 {
            let delta = (now - last_used) as f64;
            (-lambda * delta / 86400.0).exp()
        }

        fn final_score(
            &self,
            bm25: f64,
            cwd: f64,
            use_count: u32,
            last_used: i64,
            now: i64,
        ) -> f64 {
            self.bm25_weight * bm25
                + self.cwd_weight * cwd
                + self.freq_weight * self.frequency_score(use_count)
                + self.recency_weight * self.recency_score(last_used, now, 0.15)
        }
    }

    #[test]
    fn test_bm25_higher_for_better_match() {
        let ranker = HybridRanker::new();
        let s1 = ranker.final_score(2.5, 1.0, 5, 100, 100);
        let s2 = ranker.final_score(1.0, 1.0, 5, 100, 100);
        assert!(s1 > s2, "Higher BM25 should yield higher final score");
    }

    #[test]
    fn test_cwd_same_directory_scores_highest() {
        let ranker = HybridRanker::new();
        let same = ranker.cwd_match_score("/home/user/project", "/home/user/project");
        let parent = ranker.cwd_match_score("/home/user", "/home/user/project");
        let unrelated = ranker.cwd_match_score("/tmp", "/home/user/project");
        assert!(same > parent);
        assert!(parent > unrelated);
        assert_eq!(unrelated, 0.0);
    }

    #[test]
    fn test_frequency_higher_use_count_scores_higher() {
        let ranker = HybridRanker::new();
        let s1 = ranker.frequency_score(10);
        let s2 = ranker.frequency_score(1);
        assert!(s1 > s2, "Higher usage should yield higher frequency score");
    }

    #[test]
    fn test_recency_newer_scores_higher() {
        let ranker = HybridRanker::new();
        let recent = ranker.recency_score(1000, 1000, 0.15);
        let old = ranker.recency_score(500, 1000, 0.15);
        assert!(recent > old, "More recent should yield higher recency score");
    }

    #[test]
    fn test_combined_ranking_favors_relevant_cwd() {
        let ranker = HybridRanker::new();
        let score_same_dir = ranker.final_score(2.0, 1.0, 5, 1000, 1000);
        let score_diff_dir = ranker.final_score(2.5, 0.0, 5, 1000, 1000);
        assert!(
            score_same_dir > score_diff_dir,
            "Same cwd with slightly lower BM25 should beat higher BM25 in different dir"
        );
    }

    #[test]
    fn test_combined_ranking_favors_frequent() {
        let ranker = HybridRanker::new();
        let score_frequent = ranker.final_score(2.0, 1.0, 100, 1000, 1000);
        let score_rare = ranker.final_score(2.0, 1.0, 1, 1000, 1000);
        assert!(score_frequent > score_rare);
    }

    #[test]
    fn test_combined_ranking_favors_recent() {
        let ranker = HybridRanker::new();
        let score_recent = ranker.final_score(2.0, 1.0, 5, 1000, 1000);
        let score_old = ranker.final_score(2.0, 1.0, 5, 100, 1000);
        assert!(score_recent > score_old);
    }
}
