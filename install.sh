#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_BIN="$HOME/.cargo/bin"
SSHMGR_BIN="$CARGO_BIN/sshmgr"

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

if ! need_cmd curl; then
  echo "Error: curl is required to install Rust toolchain." >&2
  exit 1
fi

if [[ ! -x "$CARGO_BIN/cargo" ]]; then
  echo "Installing Rust toolchain (rustup + cargo)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi

# shellcheck disable=SC1090
source "$HOME/.cargo/env"

echo "Installing sshmgr globally..."
cd "$REPO_DIR"
cargo install --path .

if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
  for rc_file in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [[ -f "$rc_file" ]] && ! grep -Fq 'export PATH="$HOME/.cargo/bin:$PATH"' "$rc_file"; then
      printf '\nexport PATH="$HOME/.cargo/bin:$PATH"\n' >> "$rc_file"
    fi
  done
  export PATH="$CARGO_BIN:$PATH"
fi

if ! need_cmd sshmgr; then
  echo "Error: sshmgr was installed but is not available on PATH in this shell." >&2
  echo "Run: export PATH=\"$HOME/.cargo/bin:\$PATH\"" >&2
  exit 1
fi

echo "Success: sshmgr is installed and ready."
echo "Try: sshmgr -h"
echo "Binary: $SSHMGR_BIN"
