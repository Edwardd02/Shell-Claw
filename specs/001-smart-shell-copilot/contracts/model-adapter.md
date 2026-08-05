# Contract: Completion Model Adapter

The daemon must access local model inference through an explicit trait boundary.

## Trait Shape

```rust
pub trait CompletionModel: Send + Sync {
    fn complete_suffix(&self, context: ModelContext, cancel: CancellationToken) -> ModelResult<ModelOutput>;
}
```

Exact async/sync shape may be adjusted during implementation, but scheduler code
must not depend directly on llama.cpp binding internals.

## `ModelContext`

- `line_prefix: String`
- `cwd: String`
- `retrieval_candidates: Vec<RetrievalCandidate>`
- `grammar_id: GrammarId` with initial value `single_line`
- `deadline_ms: u64`

## `ModelOutput`

- `suffix: String`
- `ttft_ms: u64`
- `model_id: String`

## Grammar Contract

```ebnf
root      ::= text-line
text-line ::= [^\r\n\x00]+
```

Implementation may use equivalent llama.cpp GBNF syntax if escaping differs.

## Validation Rules

- Empty prompt, oversized context, expired deadline, and cancellation return
  no-suggestion through the scheduler.
- Output containing CR, LF, NUL, Markdown fences, explanatory prefixes, or text
  that is not a suffix must be rejected.
- Adapter must support startup warmup for resident model and system prompt state.
- Adapter must expose benchmark hooks for warmed TTFT and RSS validation.

## FFI Safety Rules

- Project code must not introduce handwritten llama.cpp FFI unless explicitly
  approved by a future constitution amendment or task exception.
- Any exposed native handle from the binding must be validated or wrapped before
  scheduler use.
- Cancellation and timeout paths must not leak model work into future requests.
