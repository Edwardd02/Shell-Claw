use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub model_path: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub log_enabled: bool,
    pub max_line_length: usize,
}

/// ShellClaw 用户数据目录: `~/.shellclaw/`
pub fn data_dir() -> PathBuf {
    let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"));
    std::env::var("SHELLCLAW_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".shellclaw"))
}

/// 配置文件路径: `~/.shellclaw/config`
pub fn config_path() -> PathBuf {
    data_dir().join("config")
}

/// 默认模型路径: `~/.shellclaw/models/qwen2.5-coder-0.5b-instruct-finetuned.gguf`
///
/// 模型由 brew post_install / download-model.sh 下载到这里,daemon 无需
/// 用户手动设置 SHELLCLAW_MODEL_PATH 就能加载。
pub fn default_model_path() -> PathBuf {
    let filename = "qwen2.5-coder-0.5b-instruct-finetuned.gguf";
    std::env::var("SHELLCLAW_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| data_dir().join("models").join(filename))
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: data_dir().join("daemon.sock"),
            model_path: default_model_path(),
            db_path: data_dir().join("memory.db"),
            log_path: data_dir().join("daemon.log"),
            log_enabled: false,
            max_line_length: 4096,
        }
    }
}

impl DaemonConfig {
    /// 读取用户配置 + 环境变量,构造 daemon 配置。
    pub fn load() -> Self {
        let base = data_dir();

        // 读 ~/.shellclaw/config 判断日志开关
        let log_enabled = read_config_bool("log_enabled");

        let socket_path = std::env::var("SHELLCLAW_SOCKET_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| base.join("daemon.sock"));

        let model_path = std::env::var("SHELLCLAW_MODEL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_model_path());

        let db_path = base.join("memory.db");
        let log_path = base.join("daemon.log");

        Self {
            socket_path,
            model_path,
            db_path,
            log_path,
            log_enabled,
            max_line_length: std::env::var("SHELLCLAW_MAX_LINE_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
        }
    }
}

/// 从 ~/.shellclaw/config 读一个布尔配置项。
fn read_config_bool(key: &str) -> bool {
    let cfg = config_path();
    let content = std::fs::read_to_string(cfg).unwrap_or_default();
    for line in content.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return matches!(v.trim(), "1" | "true" | "yes" | "on");
            }
        }
    }
    false
}

/// 写入配置布尔项(用于 shellclaw log on/off 持久化)。
pub fn set_config_bool(key: &str, enabled: bool) -> std::io::Result<()> {
    let cfg = config_path();
    if let Some(parent) = cfg.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // 读取现有配置,保留其他项,更新目标项
    let mut lines: Vec<String> = std::fs::read_to_string(&cfg)
        .map(|s| s.lines().map(String::from).collect())
        .unwrap_or_default();
    let value = if enabled { "1" } else { "0" };
    let new_line = format!("{key}={value}");
    if let Some(idx) = lines.iter().position(|l| l.trim().starts_with(&format!("{key}="))) {
        lines[idx] = new_line;
    } else {
        lines.push(new_line);
    }
    std::fs::write(&cfg, lines.join("\n") + "\n")
}
