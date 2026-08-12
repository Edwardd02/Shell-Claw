use protocol::{CompletionRequest, CompletionResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio_util::sync::CancellationToken;

use crate::memory::db::Database;
use crate::memory::store::SqliteMemoryStore;
use crate::memory::{CommandMemoryInput, MemoryStore, RetrievalQuery};
use crate::model::adapter::LlamaCppAdapter;
use crate::model::{CompletionModel, ModelContext};

mod validate;

pub use validate::RequestValidator;

static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();

pub fn global() -> &'static Scheduler {
    SCHEDULER.get_or_init(Scheduler::new)
}

pub struct Scheduler {
    memory: Mutex<Option<Arc<dyn MemoryStore>>>,
    model: Mutex<Option<Arc<dyn CompletionModel>>>,
    active_requests: Mutex<HashMap<String, ActiveRequest>>,
}

struct ActiveRequest {
    request_id: String,
    arrival_sequence: u64,
    cancel: CancellationToken,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(None),
            model: Mutex::new(None),
            active_requests: Mutex::new(HashMap::new()),
        }
    }

    fn memory(&self) -> Option<Arc<dyn MemoryStore>> {
        let config = crate::config::DaemonConfig::load();

        let mut memory_guard = self.memory.lock().ok()?;
        if memory_guard.is_none() {
            if let Ok(db) = Database::open(&config.db_path) {
                *memory_guard = Some(Arc::new(SqliteMemoryStore::new(Arc::new(db))));
            }
        }
        memory_guard.clone()
    }

    fn model(&self) -> Option<Arc<dyn CompletionModel>> {
        let config = crate::config::DaemonConfig::load();
        let mut model_guard = self.model.lock().ok()?;
        if model_guard.is_none() {
            *model_guard = Some(Arc::new(LlamaCppAdapter::new(config.model_path)));
        }
        model_guard.clone()
    }

    pub async fn submit_completion(
        &self,
        req: CompletionRequest,
        arrival_sequence: u64,
    ) -> SubmitResult {
        let request_started = std::time::Instant::now();
        if let Err(kind) = RequestValidator::validate(&req.params) {
            return SubmitResult { request_id: req.id.clone(), outcome: kind };
        }

        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(req.params.deadline_ms);
        let deadline_ms = req.params.deadline_ms;

        let session_id = req.params.session_id.clone();
        let cancel = CancellationToken::new();
        {
            let Ok(mut active) = self.active_requests.lock() else {
                return SubmitResult {
                    request_id: req.id,
                    outcome: CompletionResult::no_suggestion(),
                };
            };
            if let Some(current) = active.get(&session_id) {
                if current.arrival_sequence > arrival_sequence {
                    return SubmitResult {
                        request_id: req.id,
                        outcome: CompletionResult::no_suggestion(),
                    };
                }
                current.cancel.cancel();
            }
            active.insert(
                session_id.clone(),
                ActiveRequest {
                    request_id: req.id.clone(),
                    arrival_sequence,
                    cancel: cancel.clone(),
                },
            );
        }

        let memory = self.memory();
        // 本地命令记忆始终启用(记忆优先级于模型推理)。
        let candidates = if let Some(memory) = memory {
            let query = RetrievalQuery {
                cwd: req.params.cwd.clone(),
                line_prefix: req.params.line.clone(),
                limit: 5,
                deadline_ms,
            };
            match tokio::task::spawn_blocking(move || memory.retrieve(query)).await {
                Ok(Ok(c)) => c,
                _ => vec![],
            }
        } else {
            vec![]
        };

        if cancel.is_cancelled() || std::time::Instant::now() > deadline {
            self.finish_request(&session_id, &req.id);
            return SubmitResult {
                request_id: req.id.clone(),
                outcome: CompletionResult::no_suggestion(),
            };
        }

        if let Some(candidate) =
            candidates.iter().find(|candidate| candidate.command.starts_with(&req.params.line))
        {
            let suffix = candidate.command[req.params.line.len()..].to_string();
            if !suffix.is_empty()
                && crate::model::validate::validate_suffix(&suffix, &req.params.line)
            {
                self.finish_request(&session_id, &req.id);
                return SubmitResult {
                    request_id: req.id.clone(),
                    outcome: CompletionResult::suggestion(
                        suffix,
                        req.params.cursor,
                        line_hash(&req.params.line),
                        protocol::SuggestionSource::Memory,
                        request_started.elapsed().as_millis() as u64,
                        None,
                    ),
                };
            }
        }

        let model = self.model();

        if let Some(model) = model {
            let context = ModelContext { line_prefix: req.params.line.clone() };

            let model_cancel = cancel.clone();
            let completed =
                tokio::task::spawn_blocking(move || model.complete_suffix(context, model_cancel));
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match tokio::time::timeout(remaining, completed).await {
                Ok(Ok(Ok(output))) if !cancel.is_cancelled() => {
                    if crate::model::validate::validate_suffix(&output.suffix, &req.params.line) {
                        self.finish_request(&session_id, &req.id);
                        return SubmitResult {
                            request_id: req.id.clone(),
                            outcome: CompletionResult::suggestion(
                                output.suffix,
                                req.params.cursor,
                                line_hash(&req.params.line),
                                output.source,
                                request_started.elapsed().as_millis() as u64,
                                Some(output.ttft_ms),
                            ),
                        };
                    }
                }
                Err(_) => cancel.cancel(),
                _ => {}
            }
        }

        self.finish_request(&session_id, &req.id);
        SubmitResult { request_id: req.id.clone(), outcome: CompletionResult::no_suggestion() }
    }

    pub async fn record_command(&self, cwd: &str, command: &str) {
        let memory = self.memory();
        if let Some(memory) = memory {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let entry = CommandMemoryInput {
                cwd: cwd.to_string(),
                command: command.to_string(),
                used_at_unix: now,
            };
            let _ = tokio::task::spawn_blocking(move || memory.record_command(entry)).await;
        }
    }

    pub async fn cancel_request(&self, session_id: &str, request_id: &str) -> bool {
        let Ok(mut active) = self.active_requests.lock() else {
            return false;
        };
        let token = active
            .get(session_id)
            .filter(|request| request.request_id == request_id)
            .map(|request| request.cancel.clone());
        if let Some(token) = token {
            active.remove(session_id);
            token.cancel();
            true
        } else {
            false
        }
    }

    pub async fn run(&self) {
        let idle_seconds = std::env::var("SHELLCLAW_MODEL_IDLE_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30)
            .max(5);
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            if let Ok(model_guard) = self.model.lock() {
                let Some(model) = model_guard.as_ref() else {
                    continue;
                };
                model.unload_if_idle(std::time::Duration::from_secs(idle_seconds));
            }
        }
    }

    fn finish_request(&self, session_id: &str, request_id: &str) {
        let Ok(mut active) = self.active_requests.lock() else {
            return;
        };
        if active.get(session_id).is_some_and(|request| request.request_id == request_id) {
            active.remove(session_id);
        }
    }
}

fn line_hash(line: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    line.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub struct SubmitResult {
    pub request_id: String,
    pub outcome: CompletionResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{MemoryResult, RetrievalCandidate};
    use crate::model::{ModelError, ModelOutput, ModelResult};
    use protocol::{CompletionParams, SuggestionSource};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct EmptyMemory;

    impl MemoryStore for EmptyMemory {
        fn record_command(&self, _entry: CommandMemoryInput) -> MemoryResult<()> {
            Ok(())
        }

        fn retrieve(&self, _query: RetrievalQuery) -> MemoryResult<Vec<RetrievalCandidate>> {
            Ok(vec![])
        }
    }

    struct CancellableModel {
        first_started: Arc<AtomicBool>,
        first_cancelled: Arc<AtomicBool>,
    }

    impl CompletionModel for CancellableModel {
        fn complete_suffix(
            &self,
            context: ModelContext,
            cancel: CancellationToken,
        ) -> ModelResult<ModelOutput> {
            if context.line_prefix == "git a" {
                self.first_started.store(true, Ordering::SeqCst);
                while !cancel.is_cancelled() {
                    std::thread::yield_now();
                }
                self.first_cancelled.store(true, Ordering::SeqCst);
                return Err(ModelError::new("cancelled"));
            }
            Ok(ModelOutput {
                suffix: " status".into(),
                ttft_ms: 1,
                source: SuggestionSource::Model,
            })
        }
    }

    fn request(id: &str, line: &str) -> CompletionRequest {
        CompletionRequest::new(
            id.into(),
            CompletionParams {
                session_id: "session-1".into(),
                shell_kind: "zsh".into(),
                line: line.into(),
                cursor: line.len(),
                cwd: "/tmp".into(),
                deadline_ms: 1_000,
                client_sent_at_ms: 0,
            },
        )
    }

    #[tokio::test]
    async fn newer_request_cancels_inference_in_same_session() {
        let first_started = Arc::new(AtomicBool::new(false));
        let first_cancelled = Arc::new(AtomicBool::new(false));
        let scheduler = Scheduler {
            memory: Mutex::new(Some(Arc::new(EmptyMemory))),
            model: Mutex::new(Some(Arc::new(CancellableModel {
                first_started: first_started.clone(),
                first_cancelled: first_cancelled.clone(),
            }))),
            active_requests: Mutex::new(HashMap::new()),
        };

        let first = scheduler.submit_completion(request("request-1", "git a"), 1);
        let second = async {
            while !first_started.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
            scheduler.submit_completion(request("request-2", "git b"), 2).await
        };
        let (first_result, second_result) = tokio::join!(first, second);

        assert!(matches!(first_result.outcome, CompletionResult::None));
        assert!(matches!(second_result.outcome, CompletionResult::Suggestion(_)));
        assert!(first_cancelled.load(Ordering::SeqCst));
    }
}
