use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub model_path: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub max_line_length: usize,
    pub default_deadline_ms: u64,
    pub ranking_weights: RankingWeights,
}

#[derive(Debug, Clone)]
pub struct RankingWeights {
    pub bm25: f64,
    pub cwd: f64,
    pub frequency: f64,
    pub recency_lambda: f64,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            bm25: 0.40,
            cwd: 0.25,
            frequency: 0.20,
            recency_lambda: 0.15,
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/smart-shell-copilot.sock"),
            model_path: PathBuf::from("models/qwen3-0.6b-base.gguf"),
            db_path: dirs_fallback(),
            log_path: dirs_fallback(),
            max_line_length: 4096,
            default_deadline_ms: 25,
            ranking_weights: RankingWeights::default(),
        }
    }
}

fn dirs_fallback() -> PathBuf {
    dirs_home().join(".smart-shell-copilot")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

impl DaemonConfig {
    pub fn load() -> Self {
        let socket_path = std::env::var("SSC_SOCKET_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/smart-shell-copilot.sock"));

        let model_path = std::env::var("SSC_MODEL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("models/qwen3-0.6b-base.gguf"));

        let home = dirs_home();
        let base = std::env::var("SSC_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".smart-shell-copilot"));

        let db_path = base.join("memory.db");
        let log_path = base.join("daemon.log");

        Self {
            socket_path,
            model_path,
            db_path,
            log_path,
            max_line_length: std::env::var("SSC_MAX_LINE_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
            default_deadline_ms: std::env::var("SSC_DEADLINE_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(25),
            ranking_weights: RankingWeights::default(),
        }
    }
}
