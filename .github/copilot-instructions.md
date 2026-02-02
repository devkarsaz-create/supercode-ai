# Copilot / AI Agent Instructions — super-agent

Purpose: concise, actionable guidance so an automated coding agent (Copilot-like) can be productive quickly in this repo.

Summary
- Project: SuperAgentCli — Rust, async (tokio), TUI (ratatui + crossterm), local LLM adapters (llama.cpp via HTTP), local ModelServer (axum), modular agent runtime (SuperAgent, SubAgent, MicroAgent).
- Key goals: local-first, modular, deterministic orchestration (DAG + FSM), extensible LLM adapter/providers, TUI-first UX.

Quick start (dev)
- Build: cargo build --release
- Run TUI: cargo run --release -- tui
- Run agent CLI: cargo run --release -- run --goal "..."
- Run tests: cargo test -- --nocapture
- Model server CLI: cargo run --release -- models serve start <model>

Big-picture architecture (files to read)
- src/main.rs — CLI entry, dispatches run/chat/tui and model CLI
- src/config.rs — RuntimeConfig (XDG config path, theme, model_dir, model_server_addr)
- src/tui/app.rs — All TUI layout, keybindings, command palette, themes, model panel
- src/agent/* — agent abstraction:
  - micro_agent.rs : MicroAgent trait (stateless)
  - sub_agent.rs : SubAgent (role, memory, tools, LLM injection)
  - super_agent.rs : SuperAgent orchestrator (graph + scheduler)
- src/llm/* — LLM abstraction and implementations
  - mod.rs: Llm trait (async trait)
  - llama.rs: HTTP client wrapper to /v1/chat/completions
  - mock.rs: deterministic MockLlm for tests
- src/models/* — model discovery/manager and ModelServer
  - manager.rs: discover/import/remove models (XDG data dir by default)
  - server.rs: ModelServer (axum endpoints) and Provider trait (MockProvider, LlamaProvider skeleton)
- src/tools/registry.rs — Tool trait and registry (sandboxed by registration)
- src/memory/store.rs — MemoryStore (short/long-term) using parking_lot::RwLock
- src/graph/* — AgentGraph (dag.rs) and FSM rules (fsm.rs)

Important patterns & conventions
- Async-first: use tokio runtime and async_trait for async traits (Llm, Provider).
- Dependency injection via Arc<dyn Trait>: LLMs and Providers are injected as Arc<dyn Llm> / Arc<dyn Provider> to allow mocks/tests.
- No unwrap() in core logic. Prefer anyhow for error propagation and thiserror for specialized errors.
- Shared mutable config/runtime state: Arc<RwLock<RuntimeConfig>> stored in TUI and passed to modules.
- Testability: use MockLlm (src/llm/mock.rs) and tempfile for temporary file tests (model manager).
- CLI: uses clap subcommands in src/cli/commands.rs — keep new commands consistent with existing shape.

Integration points / external dependencies
- LLM: expects OpenAI-compatible server endpoint at /v1/chat/completions (llama.cpp server mode). LlamaClient (reqwest) calls it.
- Model Server: internal axum server exposes /v1/models and /v1/chat/completions; ModelServer registers Providers.
- Platform constraints: project aims to run on Termux; avoid OS-specific APIs or ensure fallbacks. Use which crate to detect binaries.

Where to add new features (practical guidance)
- New provider: implement Provider trait in src/models/server.rs and register it with ModelServer (register_provider).
- NativeProvider POC (FFI-based): add module under src/models/native_provider.rs and a small loader wrapper. Add tests and docs in docs/.
- New MicroAgent: implement MicroAgent trait (src/agent/micro_agent.rs) and register via SubAgent tooling or tests.
- TUI changes: modify src/tui/app.rs (render widgets, update keybindings). Keep state in TuiApp and guard with RwLock when needed.
- Persistence: add a persistence layer behind MemoryStore (define a trait Persistence and add implementation under src/memory/persistence.rs).

Testing & debugging tips
- Use MockLlm for deterministic unit tests.
- For integration tests involving the ModelServer, start server.start_local_server().await and use reqwest to call endpoints.
- Run `cargo test -- --nocapture` to see async test logs.
- Use `RUST_LOG=debug cargo run -- ...` to get tracing output when needed.

Code style & PR guidance for AI edits
- Small, focused changes per PR; add unit tests for behavior you change.
- Preserve module boundaries; add new modules under src/ with mod.rs exports.
- Document non-obvious behavior inline and update docs/ and README.md.
- When adding runtime-global changes (config or shared state), prefer explicit fields in RuntimeConfig and use Arc<RwLock<>>.

Examples (copyable snippets)
- Injecting an LLM mock in tests (src/agent/sub_agent.rs test shows pattern):
```rust
let mock = Arc::new(MockLlm::new("this is a plan"));
let agent = SubAgent::new("planner", mock);
```
- Registering a provider at runtime:
```rust
server.register_provider(&model_name, Arc::new(YourProvider::new(...))).await?;
```

Known TODOs / limitations (README/docs reflect these)
- NativeProvider (direct GGUF/ggml inference) is not yet implemented — design in docs/MODEL_SERVICE_FA.md
- Automatic installer for llama.cpp / ollama is not yet implemented — LlamaProvider tries to detect binaries via which and will failover to MockProvider.
- Streaming responses not implemented (v0.1: non-streaming only).

Where to read more (docs + entry points)
- README.md — quick start
- docs/DEVELOPMENT_FA.md — architectural decisions and development plan (Persian)
- docs/MODEL_SERVICE_FA.md — model service design and provider guidance (Persian)
- Key files to open immediately: src/tui/app.rs, src/models/server.rs, src/llm/llama.rs, src/agent/sub_agent.rs

If something is missing or ambiguous
- Ask a short question here in the issue or PR comment: reference file(s), goal, and expected behavior. e.g. "The LlamaProvider health-check should retry 10 times, should I extend retries to 30s backoff?"

--
Please review this instruction file and say what you'd like clarified or expanded (e.g. more code snippets, test examples, or tasks for autonomous agents).