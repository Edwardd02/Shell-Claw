use std::path::PathBuf;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
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
        let result = self.with_engine(|engine| generate_suffix(engine, &context, &cancel))?;

        if cancel.is_cancelled() {
            return Err(ModelError::new("request cancelled"));
        }

        let (suffix, source) = match result {
            Some(r) => r,
            None => {
                tracing::warn!("model produced no suffix; prompt={:?}", context.line_prefix);
                return Err(ModelError::new("no suffix generated"));
            }
        };

        // 归一化:0.5B instruct 模型常"复述整个命令"而非只接后缀。
        // 若生成的文本以已输入前缀开头,剥掉前缀只留真正要追加的部分。
        // 例如 line_prefix="git che",模型输出"git checkout main" ->
        // 剥成"ckout main"(再补前导空格交给渲染层衔接)。
        let suffix = if suffix.starts_with(&context.line_prefix) {
            let rest = &suffix[context.line_prefix.len()..];
            rest.to_string()
        } else {
            suffix
        };

        if suffix.is_empty() || !super::validate::validate_suffix(&suffix, &context.line_prefix) {
            return Err(ModelError::new("no valid suffix generated"));
        }

        Ok(ModelOutput {
            suffix,
            ttft_ms: start.elapsed().as_millis() as u64,
            model_id: "llama-cpp-2".to_string(),
            source,
        })
    }
}

fn generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<(String, protocol::SuggestionSource)> {
    // Fast path: a retrieval candidate already extends the typed prefix.
    // Deterministic, correct, and instant — ideal for common completions.
    if let Some(candidate) = context.retrieval_candidates.first() {
        if candidate.command.starts_with(&context.line_prefix) {
            let suffix = &candidate.command[context.line_prefix.len()..];
            if !suffix.is_empty()
                && super::validate::validate_suffix(suffix, &context.line_prefix)
            {
                return Some((
                    suffix.to_string(),
                    protocol::SuggestionSource::Memory,
                ));
            }
        }
    }

    // Fallback path: model generation for unseen prefixes.
    llama_generate_suffix(engine, context, cancel).map(|s| {
        (s, protocol::SuggestionSource::Model)
    })
}

/// Runs greedy token generation to extend the prompt with a completion. The
/// context and sampler are created locally so they are dropped at the end of the
/// call (this is required because `LlamaContext` borrows the model).
fn llama_generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<String> {
    // 用 instruct 模型的 chat template 构造对话式 prompt,而不是裸续写。
    // 这是关键:Qwen2.5-Instruct 必须收到明确的"你在补全 shell 命令"指令,
    // 否则裸续写会把它当作散文/补丁/包名来续(N 是之前垃圾输出根因)。
    let prompt = build_instruct_prompt(&engine.model, &context.line_prefix);
    if let Some(prompt) = prompt {
        let prompt_tokens = engine
            .model
            .str_to_token(&prompt, AddBos::Always)
            .ok()?;
        tracing::debug!("instruct prompt={:?}", prompt);
        tracing::debug!("prompt tokens={}", prompt_tokens.len());
        return llama_complete_from_tokens(
            engine, &prompt_tokens, context, cancel,
        );
    }

    // chat template 不可用时的兜底:回到裸续写。
    let prompt = context.line_prefix.clone();
    let prompt_tokens = engine.model.str_to_token(&prompt, AddBos::Always).ok()?;
    llama_complete_from_tokens(engine, &prompt_tokens, context, cancel)
}

/// 用模型的 chat template 把(裸前缀)包装成一段补全指令的 prompt。
fn build_instruct_prompt(
    model: &LlamaModel,
    prefix: &str,
) -> Option<String> {
    let tmpl = model.chat_template(None).ok()?;

    let system = LlamaChatMessage::new(
        "system".to_string(),
        "You are a shell command auto-completion engine. The user has typed a \
         PARTIAL command line. Your job: output ONLY the text that should be \
         APPENDED after what they typed, to make a complete shell command. \
         Rules: \
         - Never repeat the already-typed prefix. \
         - Output only the plain suffix text. No quotes, no backticks, no \
           explanation, no markdown. Output raw command text only. \
         - If the prefix is already a complete command, output nothing. \
         - Example: prefix git che -> output ckout main \
         - Example: prefix pip insta -> output ll requests \
         - Example: prefix cd Des -> output ktop"
            .to_string(),
    )
    .ok()?;

    let user = LlamaChatMessage::new(
        "user".to_string(),
        format!(
            "The user has typed: {} Output only the suffix to append.",
            prefix
        ),
    )
    .ok()?;

    let rendered = model
        .apply_chat_template(&tmpl, &[system, user], /*add_ass=*/ true)
        .ok()?;

    Some(rendered)
}

/// 给定已经 tokenize 好的 prompt,执行采样生成一个后缀。
fn llama_complete_from_tokens(
    engine: &Engine,
    prompt_tokens: &[LlamaToken],
    _context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<String> {
    let ctx_params = LlamaContextParams::default();
    let mut ctx = engine
        .model
        .new_context(&engine.backend, ctx_params)
        .ok()?;

    //   grammar → penalties(重复惩罚) → greedy
    // 1) grammar:仅允许符合 single_line 的 token(格式约束)
    // 2) penalties:对已出现 token 降权,消灭 Deskripsi×N 式无限循环
    //    - penalty_repeat : repetition_penalty, >1.0 惩罚重复(对应 vllm repetition_penalty)
    //    - penalty_freq   : frequency_penalty,  >0 按出现频率惩罚
    //    - penalty_present: presence_penalty,   >0 对出现过的 token 惩罚
    // 3) greedy:在约束+惩罚后取最高概率
    let grammar_res = LlamaSampler::grammar(
        &engine.model,
        super::grammar::single_line_grammar_gbnf().as_str(),
        "root",
    );
    let mut samplers: Vec<LlamaSampler> = Vec::new();
    match grammar_res {
        Ok(gs) => samplers.push(gs),
        Err(e) => tracing::warn!("GBNF grammar init failed ({}); unconstrained", e),
    }
    // 重复惩罚:覆盖最近 64 个 token。repeat>1 打击任何已出现 token(防循环),
    // freq/present>0 让模型倾向新内容而非车轱辘话。
    samplers.push(LlamaSampler::penalties(
        /*penalty_last_n=*/ 64,
        /*penalty_repeat=*/ 1.20,
        /*penalty_freq=*/ 0.15,
        /*penalty_present=*/ 0.0,
    ));
    samplers.push(LlamaSampler::greedy());
    let mut sampler = LlamaSampler::chain_simple(samplers);

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

        let piece = match engine.model.token_to_str(token, Special::Plaintext) {
            Ok(p) => p,
            Err(_) => break,
        };
        generated.push_str(&piece);
        tracing::debug!("step: token={:?} piece={:?} so_far={:?}", token.0, piece, generated);

        if !validate_grammar_output(&generated) {
            break;
        }

        // Continue decoding this single token.
        batch.clear();
        if batch.add(token, pos as i32, &[0], true).is_err() {
            break;
        }
        if ctx.decode(&mut batch).is_err() {
            break;
        }
    }

    // Reject empty or multiline leftovers.
    let cleaned = clean_suffix(&generated);
    tracing::debug!(
        "llama_complete raw={:?} cleaned={:?}",
        generated, cleaned
    );
    cleaned
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
    // 去掉模型的包裹字符(反引号/引号),instruct 模型常给输出加这些。
    out = out.trim_matches(&['`', '\'', '"'][..]).to_string();
    if out.is_empty() {
        return None;
    }
    if out.contains('\0') || out.contains("```") || out.starts_with('#') {
        return None;
    }
    Some(out)
}
