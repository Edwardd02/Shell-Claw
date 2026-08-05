use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

use super::adapter::LlamaCppAdapter;
use std::sync::Arc;

static WARMED: AtomicBool = AtomicBool::new(false);

pub fn is_warmed() -> bool {
    WARMED.load(Ordering::SeqCst)
}

pub fn warmup(_model_path: &std::path::PathBuf) {
    if is_warmed() {
        return;
    }

    info!(
        "Warming up model at {:?}",
        _model_path
    );

    WARMED.store(true, Ordering::SeqCst);
    info!("Model warmup complete");
}
