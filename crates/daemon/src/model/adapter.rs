use std::path::PathBuf;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

use super::{CompletionModel, ModelContext, ModelError, ModelOutput, ModelResult};
use super::grammar::validate_grammar_output;

struct Engine {
    backend: LlamaBackend,
    model: LlamaModel,
}

/// The `CompletionModel` that drives real llama.cpp inference.
///
/// The `LlamaContext` borrows the model, so it cannot be stored long-lived
/// alongside it in this `Send + Sync` struct. Instead the model is kept resident
/// here and a fresh context is created per request and dropped at the end.
pub struct LlamaCppAdapter {
    model_path: PathBuf,
    engine: Mutex<Option<Engine>>,
}

impl LlamaCppAdapter {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            engine: Mutex::new(None),
        }
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Engine) -> T) -> ModelResult<T> {
        if !self.model_path.exists() {
            return Err(ModelError::new(format!(
                "Model file not found: {:?}",
                self.model_path
            )));
        }

        let mut guard = self
            .engine
            .lock()
            .map_err(|_| ModelError::new("model engine lock poisoned"))?;

        if guard.is_none() {
            let backend = LlamaBackend::init()
                .map_err(|e| ModelError::new(format!("backend init failed: {e}")))?;
            let model = LlamaModel::load_from_file(
                &backend,
                &self.model_path,
                &Default::default(),
            )
            .map_err(|e| ModelError::new(format!("model load failed: {e}")))?;
            *guard = Some(Engine { backend, model });
        }

        Ok(f(guard.as_ref().unwrap()))
    }
}

impl CompletionModel for LlamaCppAdapter {
    fn complete_suffix(
        &self,
        context: ModelContext,
        cancel: CancellationToken,
    ) -> ModelResult<ModelOutput> {
        if cancel.is_cancelled() {
            return Err(ModelError::new("request cancelled"));
        }
        if context.line_prefix.trim().is_empty() {
            return Err(ModelError::new("empty prompt"));
        }
        if context.line_prefix.len() > 4096 {
            return Err(ModelError::new("prompt too long"));
        }

        let start = std::time::Instant::now();
        let suffix = self.with_engine(|engine| generate_suffix(engine, &context, &cancel))?;

        if cancel.is_cancelled() {
            return Err(ModelError::new("request cancelled"));
        }

        let suffix = match suffix {
            Some(s) => s,
            None => {
                tracing::warn!("model produced no suffix; prompt={:?}", context.line_prefix);
                return Err(ModelError::new("no suffix generated"));
            }
        };

        if suffix.is_empty() || !super::validate::validate_suffix(&suffix, &context.line_prefix) {
            return Err(ModelError::new("no valid suffix generated"));
        }

        Ok(ModelOutput {
            suffix,
            ttft_ms: start.elapsed().as_millis() as u64,
            model_id: "llama-cpp-2".to_string(),
        })
    }
}

fn generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<String> {
    // Fast path: a retrieval candidate already extends the typed prefix.
    // Deterministic, correct, and instant — ideal for common completions.
    if let Some(candidate) = context.retrieval_candidates.first() {
        if candidate.command.starts_with(&context.line_prefix) {
            let suffix = &candidate.command[context.line_prefix.len()..];
            if !suffix.is_empty()
                && super::validate::validate_suffix(suffix, &context.line_prefix)
            {
                return Some(suffix.to_string());
            }
        }
    }

    // Fallback path: model generation for unseen prefixes.
    llama_generate_suffix(engine, context, cancel)
}

/// Runs greedy token generation to extend the prompt with a completion. The
/// context and sampler are created locally so they are dropped at the end of the
/// call (this is required because `LlamaContext` borrows the model).
fn llama_generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<String> {
    let ctx_params = LlamaContextParams::default();
    let mut ctx = engine
        .model
        .new_context(&engine.backend, ctx_params)
        .ok()?;

    // Model fallback operates on the raw typed prefix only (any candidate that
    // extends it was already returned on the fast path above).
    let prompt = context.line_prefix.clone();

    let prompt_tokens = engine.model.str_to_token(&prompt, AddBos::Always).ok()?;
    tracing::debug!("prompt={:?} tokens={}", prompt, prompt_tokens.len());

    let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);

    let mut batch = LlamaBatch::new(prompt_tokens.len() + 64, 1);

    // Prefill the prompt.
    for (i, token) in prompt_tokens.iter().enumerate() {
        batch
            .add(*token, i as i32, &[0], i + 1 == prompt_tokens.len())
            .ok()?;
    }
    ctx.decode(&mut batch).ok()?;

    let mut generated = String::new();
    let eos_token: i32 = -1; // llama's EOS is -1 when no explicit EOS token id is set

    for pos in prompt_tokens.len()..prompt_tokens.len() + 64 {
        if cancel.is_cancelled() {
            return None;
        }

        let token: LlamaToken = sampler.sample(&ctx, -1);

        // Stop on end-of-string/turn markers.
        if token.0 == eos_token || token_string_marker(&token) {
            tracing::debug!("stopped on token {token:?} (eos/marker)");
            break;
        }

        let piece = engine.model.token_to_str(token, Special::Plaintext).ok()?;
        generated.push_str(&piece);
        tracing::debug!("step: token={:?} piece={:?} so_far={:?}", token.0, piece, generated);

        if !validate_grammar_output(&generated) {
            break;
        }

        // Continue decoding this single token.
        batch.clear();
        batch.add(token, pos as i32, &[0], true).ok()?;
        ctx.decode(&mut batch).ok()?;
    }

    // Reject empty or multiline leftovers.
    clean_suffix(&generated)
}

/// Heuristic: stop when a token is the EOS (id 0 for most tokenizers).
fn token_string_marker(t: &LlamaToken) -> bool {
    t.0 == 0
}

fn clean_suffix(generated: &str) -> Option<String> {
    let mut out = generated.trim_start_matches(' ').to_string();
    if let Some(idx) = out.find('\n').or_else(|| out.find('\r')) {
        out.truncate(idx);
    }
    if out.is_empty() {
        return None;
    }
    if out.contains('\0') || out.contains("```") || out.starts_with('#') {
        return None;
    }
    Some(out)
}
