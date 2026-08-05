use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;

static mut LOG_GUARD: Option<WorkerGuard> = None;

pub fn init(log_path: &PathBuf) {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file_appender = tracing_appender::rolling::never(
        log_path.parent().unwrap_or(&std::path::PathBuf::from(".")),
        log_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("daemon.log"),
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    unsafe {
        LOG_GUARD = Some(guard);
    }

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

pub fn flush() {
    unsafe {
        if let Some(ref guard) = LOG_GUARD {
            let _ = guard;
        }
    }
}

pub struct IpcMetrics {
    pub connections_total: u64,
    pub connections_active: u64,
    pub requests_total: u64,
    pub requests_by_method: std::collections::HashMap<String, u64>,
    pub deadlines_missed: u64,
    pub errors_total: u64,
}

impl Default for IpcMetrics {
    fn default() -> Self {
        Self {
            connections_total: 0,
            connections_active: 0,
            requests_total: 0,
            requests_by_method: std::collections::HashMap::new(),
            deadlines_missed: 0,
            errors_total: 0,
        }
    }
}

pub struct MemoryMetrics {
    pub hit_rate: f64,
    pub retrieval_count: u64,
    pub retrieval_latency_sum_ms: u64,
    pub record_count: u64,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            hit_rate: 0.0,
            retrieval_count: 0,
            retrieval_latency_sum_ms: 0,
            record_count: 0,
        }
    }
}

pub struct ModelMetrics {
    pub inference_count: u64,
    pub ttft_sum_ms: u64,
    pub cancellation_rate: f64,
    pub rejection_rate: f64,
    pub model_load_time_ms: u64,
}

