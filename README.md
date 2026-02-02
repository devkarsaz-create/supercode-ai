# Super-Agent v0.1

A local-first, Rust-native Super Multi-Agent CLI runtime (v0.1). This repo provides a foundation: CLI + TUI (ratatui) + agent runtime + llama.cpp adapter (HTTP OpenAI-compatible server mode).

Quick start

1. Build (Termux / Linux):

   - Install Rust toolchain (`rustup`) and ensure `cargo` is in PATH.

   - Build the project:

     ```bash
     cargo build --release
     ```

2. Start a llama.cpp-compatible HTTP server on your device, listening on 127.0.0.1:8080. Example (depending on your llama.cpp build):

   ```bash
   # make sure you have a GGUF model file locally, e.g., model.gguf
   ./server --model ./model.gguf --http 8080
   ```

   The adapter expects an OpenAI-compatible endpoint at `/v1/chat/completions`.

3. Run the agent:

   ```bash
   export LLAMA_ENDPOINT="http://127.0.0.1:8080"
   export LLAMA_MODEL="local.gguf"
   cargo run --release -- run --goal "Write a short plan to automate backups"
   ```

4. TUI

   ```bash
   cargo run --release -- tui
   ```

TUI features (v0.1):
- Splash header with project name `SuperAgentCli`.
- Chat panel with input box at the bottom. Type and press Enter to send.
- Command palette: press `/` to open, type to filter, Enter to select.
- Ten built-in themes (DarkPlus, Light, Monokai, SolarizedDark/Light, Dracula, OneDark, Nord, Gruvbox, Peacocks). Cycle themes via command palette and save configuration.
- Settings stored at `$XDG_CONFIG_HOME/super-agent/config.toml` (or platform default config dir).
- Model manager: press `m` in TUI to open Models panel. Press `i` to import a model file path.
- CLI model commands: `agent models list`, `agent models import <path>`, `agent models remove <name>`, `agent models serve start <model>` — starts local model server and registers a mock provider for quick testing.

Installer helper

- A convenience install script for building `llama.cpp` is available at `scripts/install_llama.sh` (Termux and Linux friendly). You can run it directly or use the CLI wrapper:

```bash
./scripts/install_llama.sh
# or
cargo run -- models install llama
```

Notes

- v0.1 uses llama.cpp in HTTP server mode only (no FFI). No cloud APIs are used.
- Fully local, no Docker, and designed to be buildable on Termux.

