use std::path::PathBuf;
use std::sync::Mutex;
use std::{num::NonZeroU32, time::Instant};
use tokio_util::sync::CancellationToken;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

use super::grammar::validate_grammar_output;
use super::{CompletionModel, ModelContext, ModelError, ModelOutput, ModelResult};

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
    state: Mutex<EngineState>,
}

struct EngineState {
    engine: Option<Engine>,
    last_used: Instant,
}

impl LlamaCppAdapter {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            state: Mutex::new(EngineState { engine: None, last_used: Instant::now() }),
        }
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Engine) -> T) -> ModelResult<T> {
        if !self.model_path.exists() {
            return Err(ModelError::new(format!("Model file not found: {:?}", self.model_path)));
        }

        let mut guard =
            self.state.lock().map_err(|_| ModelError::new("model engine lock poisoned"))?;

        if guard.engine.is_none() {
            let backend = LlamaBackend::init()
                .map_err(|e| ModelError::new(format!("backend init failed: {e}")))?;
            let params = LlamaModelParams::default().with_use_mmap(true).with_use_mlock(false);
            let model = LlamaModel::load_from_file(&backend, &self.model_path, &params)
                .map_err(|e| ModelError::new(format!("model load failed: {e}")))?;
            guard.engine = Some(Engine { backend, model });
        }

        let engine = guard.engine.as_ref().ok_or_else(|| ModelError::new("model unavailable"))?;
        let result = f(engine);
        guard.last_used = Instant::now();
        Ok(result)
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

        let result = self.with_engine(|engine| generate_suffix(engine, &context, &cancel))?;

        if cancel.is_cancelled() {
            return Err(ModelError::new("request cancelled"));
        }

        let (suffix, source, ttft_ms) = match result {
            Some(r) => r,
            None => {
                tracing::warn!("model produced no suffix; prompt={:?}", context.line_prefix);
                return Err(ModelError::new("no suffix generated"));
            }
        };

        // 减法前先整齐化模型输出:去掉前导/尾随空白。
        let out = suffix.trim();
        if out.is_empty() {
            return Err(ModelError::new("no valid suffix generated"));
        }

        // 减法 —— 按"模型输出 vs 用户输入"的三种前缀关系决定:
        //   分支1 (模型输出以用户输入开头): 模型补全了剩余部分
        //         out="git status", user="git " → suffix="status"
        //         out="git status", user="git"  → suffix=" status"
        //         减法得后缀。
        //   分支2 (用户输入以模型输出开头, 且更长): 用户已打出超过模型建议
        //         user="git commit ", out="git commit" → 无建议
        //         用户输入比模型建议还完整, 无需补全 → 返回无建议。
        //   分支3 (互不包含): 模型只输出纯后缀(漏了命令行首词)
        //         user="git", out="diff-tree -r" → out 本就是待追加后缀
        //         把 out 整体视为后缀, 渲染时 user+out 即可。
        //         若 out 以 `-`/参数开头但明显不是命令名, 也视为后缀。
        let suffix = if out.starts_with(&context.line_prefix) {
            // 分支1: 模型输出长于用户输入, 减法
            out[context.line_prefix.len()..].to_string()
        } else if context.line_prefix.starts_with(out) && context.line_prefix != out {
            // 分支2: 用户输入长于模型输出(已超出模型建议) → 无建议
            tracing::debug!(
                "no suggestion (user already typed beyond model); prefix={:?} out={:?}",
                context.line_prefix,
                out
            );
            return Err(ModelError::new("user input already extends past model output"));
        } else {
            // 分支3: 模型输出是独立后缀(漏了命令行首词, 如 user='git' out='diff-tree -r')
            //         把它整体作为后缀。
            tracing::debug!(
                "model output used as bare suffix; prefix={:?} out={:?}",
                context.line_prefix,
                out
            );
            // 若 out 首字符是字母/'-'(像是命令或参数), 前面补一个空格与 user 分隔
            let first = out.chars().next().unwrap_or(' ');
            if first.is_alphabetic() || first == '-' {
                format!(" {out}")
            } else {
                out.to_string()
            }
        };

        if suffix.is_empty() || !super::validate::validate_suffix(&suffix, &context.line_prefix) {
            return Err(ModelError::new("no valid suffix generated"));
        }

        Ok(ModelOutput { suffix, ttft_ms, source })
    }

    fn unload_if_idle(&self, max_idle: std::time::Duration) {
        if let Ok(mut state) = self.state.try_lock() {
            if state.engine.is_some() && state.last_used.elapsed() >= max_idle {
                state.engine = None;
                tracing::debug!("unloaded idle model runtime");
            }
        }
    }
}

fn generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<(String, protocol::SuggestionSource, u64)> {
    // Memory hits are resolved by the scheduler before the model is loaded.
    llama_generate_suffix(engine, context, cancel)
        .map(|(suffix, ttft)| (suffix, protocol::SuggestionSource::Model, ttft))
}

/// Runs greedy token generation to extend the prompt with a completion. The
/// context and sampler are created locally so they are dropped at the end of the
/// call (this is required because `LlamaContext` borrows the model).
fn llama_generate_suffix(
    engine: &Engine,
    context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<(String, u64)> {
    // 用 instruct 模型的 chat template 构造对话式 prompt,而不是裸续写。
    // 这是关键:Qwen2.5-Instruct 必须收到明确的"你在补全 shell 命令"指令,
    // 否则裸续写会把它当作散文/补丁/包名来续(N 是之前垃圾输出根因)。
    let prompt = build_instruct_prompt(&engine.model, &context.line_prefix);
    if let Some(prompt) = prompt {
        let prompt_tokens = engine.model.str_to_token(&prompt, AddBos::Always).ok()?;
        tracing::debug!("instruct prompt={:?}", prompt);
        tracing::debug!("prompt tokens={}", prompt_tokens.len());
        return llama_complete_from_tokens(engine, &prompt_tokens, context, cancel);
    }

    // chat template 不可用时的兜底:回到裸续写。
    let prompt = context.line_prefix.clone();
    let prompt_tokens = engine.model.str_to_token(&prompt, AddBos::Always).ok()?;
    llama_complete_from_tokens(engine, &prompt_tokens, context, cancel)
}

/// 构造与训练完全一致的 prompt。
/// 重要:训练脚本(lora_finetune.py 的 tokenize_sample)用的是
///   user=prefix, assistant=完整命令, 且【没有 system 消息】。
/// 推理时若加了 system / 指令,会偏离训练分布,导致微调模型发挥失常。
/// 因此这里只放一个 user 消息(内容=前缀),让模型按训练学到的续出完整命令。
fn build_instruct_prompt(model: &LlamaModel, prefix: &str) -> Option<String> {
    let tmpl = model.chat_template(None).ok()?;

    // 训练/推理对齐:用训练时见过的 Qwen 默认 system prompt。
    // 0.5B 小模型对没见过的 system 指令没有鲁棒性 —— 自创的 autocomplete
    // 指令会让它退化(对照实验:默认 system -> "-Force"; 自创 system -> "..")。
    // 这里必须和训练脚本 tokenize_sample 里的 system 完全一致。
    let system = LlamaChatMessage::new(
        "system".to_string(),
        "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.".to_string(),
    )
    .ok()?;

    let user = LlamaChatMessage::new("user".to_string(), prefix.to_string()).ok()?;

    // add_ass=true:在末尾补上 assistant 起始符,让模型从这里续写。
    let rendered = model.apply_chat_template(&tmpl, &[system, user], /*add_ass=*/ true).ok()?;

    Some(rendered)
}

/// 给定已经 tokenize 好的 prompt,执行采样生成一个后缀。
fn llama_complete_from_tokens(
    engine: &Engine,
    prompt_tokens: &[LlamaToken],
    _context: &ModelContext,
    cancel: &CancellationToken,
) -> Option<(String, u64)> {
    const MAX_OUTPUT_TOKENS: usize = 24;
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(256))
        .with_n_batch(256)
        .with_n_ubatch(256);
    let mut ctx = engine.model.new_context(&engine.backend, ctx_params).ok()?;

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
        /*penalty_last_n=*/ 64, /*penalty_repeat=*/ 1.20, /*penalty_freq=*/ 0.15,
        /*penalty_present=*/ 0.0,
    ));
    samplers.push(LlamaSampler::greedy());
    let mut sampler = LlamaSampler::chain_simple(samplers);

    if prompt_tokens.len() + MAX_OUTPUT_TOKENS > 256 {
        return None;
    }
    let mut batch = LlamaBatch::new(prompt_tokens.len() + MAX_OUTPUT_TOKENS, 1);

    // Prefill the prompt.
    for (i, token) in prompt_tokens.iter().enumerate() {
        batch.add(*token, i as i32, &[0], i + 1 == prompt_tokens.len()).ok()?;
    }
    ctx.decode(&mut batch).ok()?;

    let mut generated = String::new();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let generation_started = Instant::now();
    let mut ttft_ms = None;
    let eos_token: i32 = -1; // llama's EOS is -1 when no explicit EOS token id is set

    for pos in prompt_tokens.len()..prompt_tokens.len() + MAX_OUTPUT_TOKENS {
        if cancel.is_cancelled() {
            return None;
        }

        let token: LlamaToken = sampler.sample(&ctx, -1);
        ttft_ms.get_or_insert_with(|| generation_started.elapsed().as_millis() as u64);

        // Stop on end-of-string/turn markers.
        if token.0 == eos_token || token_string_marker(&token) {
            tracing::debug!("stopped on token {token:?} (eos/marker)");
            break;
        }

        let piece = match engine.model.token_to_piece(token, &mut decoder, false, None) {
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
    tracing::debug!("llama_complete raw={:?} cleaned={:?}", generated, cleaned);
    cleaned.map(|suffix| (suffix, ttft_ms.unwrap_or_default()))
}

/// Heuristic: stop when a token is the EOS (id 0 for most tokenizers).
fn token_string_marker(t: &LlamaToken) -> bool {
    t.0 == 0
}

fn clean_suffix(generated: &str) -> Option<String> {
    // 注意:不去 trim 前导空格!训练数据(lora_finetune.py split_pairs)的后缀
    // 是带前导空格的(如 ' push origin main'),且 Ghost Text 渲染 = BUFFER + suffix,
    // 需要一个空格衔接(kubectl get 接 ' all' -> 'kubectl get all')。
    // 若这里 trim 掉前导空格,会拼成 'kubectl getall'(缺空格 bug)。
    let mut out = generated.to_string();
    if let Some(idx) = out.find('\n').or_else(|| out.find('\r')) {
        out.truncate(idx);
    }
    // 去掉模型的包裹字符(反引号/引号),instruct 模型常给输出加这些。
    // 但是 DON'T trim 前导空格(那是补全衔接所需的)。
    out = out.trim_end_matches(&['`', '\'', '"'][..]).to_string();
    out = out.trim_start_matches(&['`', '\''][..]).to_string();
    if out.is_empty() {
        return None;
    }
    if out.contains('\0') || out.contains("```") || out.starts_with('#') {
        return None;
    }
    Some(out)
}
