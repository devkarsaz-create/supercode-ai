#!/usr/bin/env sh
set -e
# Simple helper to guide building a basic llama.cpp + HTTP server (non-privileged)
# Intended as a convenience script — review before running on your machine.

echo "SuperAgent: Llama.cpp installer helper"

echo "Checking for required tools: git, make, cmake, gcc/clang"
for t in git make cmake; do
  if ! command -v "$t" >/dev/null 2>&1; then
    echo "Missing $t — please install it (e.g., 'pkg install $t' on Termux, 'apt install $t' on Debian/Ubuntu)" >&2
    exit 2
  fi
done

# On Termux, ensure clang is present
if ! command -v clang >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
  echo "No compiler found (clang/gcc). Install clang or gcc first." >&2
  exit 2
fi

# Clone and build llama.cpp as a simple suggestion (may change with upstream)
WORKDIR="$PWD/third_party"
mkdir -p "$WORKDIR"
cd "$WORKDIR"
if [ ! -d llama.cpp ]; then
  echo "Cloning llama.cpp (https://github.com/ggerganov/llama.cpp)..."
  git clone https://github.com/ggerganov/llama.cpp.git
fi
cd llama.cpp
echo "Pulling latest..."
git pull --rebase || true

echo "Building... (this may take a while)"
# Use Makefile build which tries to detect arch and builds lib/backends
make clean || true
make -j$(nproc || echo 1)

echo "Build done. For an HTTP server wrapper you may want to build a server that exposes /v1/chat/completions — see docs/MODEL_SERVICE_FA.md for recommended adapters and the 'server' sample." 

echo "If you built a binary, add it to PATH or pass its path to the CLI (models serve start ...)."

echo "Done."