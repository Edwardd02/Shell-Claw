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
    /// 是否启用本地命令记忆。设 SSС_DISABLE_MEMORY=1 关闭,请求将只走模型推理。
    pub memory_enabled: bool,
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
            model_path: PathBuf::from("models/qwen2.5-coder-0.5b-instruct-finetuned.gguf"),
            db_path: dirs_fallback(),
            log_path: dirs_fallback(),
            max_line_length: 4096,
            default_deadline_ms: 25,
            ranking_weights: RankingWeights::default(),
            memory_enabled: true,
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
            .unwrap_or_else(|_| PathBuf::from("models/qwen2.5-coder-0.5b-instruct-finetuned.gguf"));

        let home = dirs_home();
        let base = std::env::var("SSC_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".smart-shell-copilot"));

        let db_path = base.join("memory.db");

        // 日志目录:SSC_LOG_DIR 优先;否则默认项目根下 logs/(相对当前工作目录)。
        // 典型:从项目根 `./target/release/daemon` 启动 → ./logs/daemon.log
        let log_dir = std::env::var("SSC_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("logs"));
        let log_path = log_dir.join("daemon.log");

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
            memory_enabled: !matches!(
                std::env::var("SSC_DISABLE_MEMORY")
                    .as_deref(),
                Ok("1") | Ok("true") | Ok("TRUE")
            ),
        }
    }
}
