use protocol::{CompletionRequest, CompletionResult, SuggestionSource};
use std::sync::{Arc, Mutex, OnceLock};

use crate::memory::db::Database;
use crate::memory::store::SqliteMemoryStore;
use crate::memory::{CommandMemoryInput, MemoryStore, RetrievalQuery};
use crate::model::adapter::LlamaCppAdapter;
use crate::model::{CompletionModel, ModelContext};

mod deadline;
mod noop;
mod validate;

pub use deadline::DeadlineTracker;
pub use noop::NoopResponse;
pub use validate::RequestValidator;

static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();

pub fn global() -> &'static Scheduler {
    SCHEDULER.get_or_init(|| Scheduler::new())
}

pub struct SchedulerState {
    pub next_request_id: u64,
}

pub struct Scheduler {
    state: tokio::sync::Mutex<SchedulerState>,
    memory: Mutex<Option<Arc<dyn MemoryStore>>>,
    model: Mutex<Option<Arc<dyn CompletionModel>>>,
    daemon_started: std::time::Instant,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            state: tokio::sync::Mutex::new(SchedulerState { next_request_id: 1 }),
            memory: Mutex::new(None),
            model: Mutex::new(None),
            daemon_started: std::time::Instant::now(),
        }
    }

    fn init_pipeline(&self) {
        let config = crate::config::DaemonConfig::load();

        let mut memory_guard = self.memory.lock().unwrap();
        if memory_guard.is_none() {
            if let Ok(db) = Database::open(&config.db_path) {
                *memory_guard = Some(Arc::new(SqliteMemoryStore::new(Arc::new(db))));
            }
        }
        drop(memory_guard);

        let mut model_guard = self.model.lock().unwrap();
        if model_guard.is_none() {
            *model_guard = Some(Arc::new(LlamaCppAdapter::new(config.model_path)));
        }
    }

    pub async fn submit_completion(
        &self,
        req: CompletionRequest,
    ) -> SubmitResult {
        self.init_pipeline();

        if let Err(kind) = RequestValidator::validate(&req.params) {
            return SubmitResult {
                request_id: req.id.clone(),
                outcome: kind,
            };
        }

        let deadline = std::time::Instant::now()
            + std::time::Duration::from_millis(req.params.deadline_ms);
        let deadline_ms = req.params.deadline_ms;

        let memory = self.memory.lock().unwrap().clone();
        let candidates = if let Some(memory) = memory {
            let query = RetrievalQuery {
                cwd: req.params.cwd.clone(),
                line_prefix: req.params.line.clone(),
                limit: 5,
                deadline_ms,
            };
            match memory.retrieve(query) {
                Ok(c) => c,
                Err(_) => vec![],
            }
        } else {
            vec![]
        };

        if std::time::Instant::now() > deadline {
            return SubmitResult {
                request_id: req.id.clone(),
                outcome: CompletionResult::no_suggestion(),
            };
        }

        let model = self.model.lock().unwrap().clone();
        let cancel = tokio_util::sync::CancellationToken::new();

        if let Some(model) = model {
            let context = ModelContext {
                line_prefix: req.params.line.clone(),
                cwd: req.params.cwd.clone(),
                retrieval_candidates: candidates,
                grammar_id: crate::model::GrammarId::SingleLine,
                deadline_ms,
            };

            match model.complete_suffix(context, cancel) {
                Ok(output) => {
                    if crate::model::validate::validate_suffix(
                        &output.suffix,
                        &req.params.line,
                    ) {
                        let mut hasher =
                            std::collections::hash_map::DefaultHasher::new();
                        use std::hash::{Hash, Hasher};
                        req.params.line.hash(&mut hasher);

                        return SubmitResult {
                            request_id: req.id.clone(),
                            outcome: CompletionResult::suggestion(
                                output.suffix,
                                req.params.cursor,
                                format!("{:x}", hasher.finish()),
                                SuggestionSource::Model,
                                output.ttft_ms,
                            ),
                        };
                    }
                }
                Err(_) => {}
            }
        }

        SubmitResult {
            request_id: req.id.clone(),
            outcome: CompletionResult::no_suggestion(),
        }
    }

    pub async fn record_command(&self, cwd: &str, command: &str) {
        self.init_pipeline();

        let memory = self.memory.lock().unwrap().clone();
        if let Some(memory) = memory {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let _ = memory.record_command(CommandMemoryInput {
                cwd: cwd.to_string(),
                command: command.to_string(),
                used_at_unix: now,
            });
        }
    }

    pub async fn cancel_request(&self, _session_id: &str, _request_id: &str) -> bool {
        true
    }

    pub async fn run(&self) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}

pub struct SubmitResult {
    pub request_id: String,
    pub outcome: CompletionResult,
}
