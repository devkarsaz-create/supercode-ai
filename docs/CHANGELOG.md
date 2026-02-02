# Changelog (selected)

## Unreleased

- TUI: avoid blocking awaits during provider start — provider launch now runs in background using `tokio::spawn`; UI receives status updates via an internal channel and shows logs. ✅
- TUI: removed unsafe `unwrap()` usage around config locks; added `get_config_clone()` helper and safer lock handling for saves and theme changes. ✅
- ModelServer: added unit test `test_register_mock_for_model` to validate mock registration. ✅
- LlamaProvider: improved health-check with exponential backoff and clearer error behavior. ✅
- CLI: added `agent models install llama` to run local convenience installer script. ✅
- Added `scripts/install_llama.sh` - helper to build `llama.cpp` on Termux/Linux. ✅


If you'd like, I can continue by:
1. Running `cargo fmt`/`clippy` and iterating on warnings (needs Rust toolchain locally).
2. Finishing markdown lint cleanup and docs polish.
3. Implementing a more robust provider installer (Rust wrapper with progress & auto-detection).
4. Starting a NativeProvider POC (ggml/gguf FFI) as a separate branch.

Tell me which to prioritize next and I will proceed.