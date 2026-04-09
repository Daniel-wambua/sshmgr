#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_BIN="$HOME/.cargo/bin"
SSHMGR_BIN="$CARGO_BIN/sshmgr"

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

run_as_root() {
  if [[ ${EUID:-$(id -u)} -eq 0 ]]; then
    "$@"
  elif need_cmd sudo; then
    sudo "$@"
  else
    echo "Error: sudo is required to install tmux with the detected package manager." >&2
    echo "Install tmux manually or rerun this script with elevated privileges." >&2
    return 1
  fi
}

install_tmux() {
  if need_cmd tmux; then
    return 0
  fi

  echo "Installing tmux..."

  if need_cmd apt-get; then
    run_as_root apt-get update
    run_as_root apt-get install -y tmux
  elif need_cmd dnf; then
    run_as_root dnf install -y tmux
  elif need_cmd yum; then
    run_as_root yum install -y tmux
  elif need_cmd pacman; then
    run_as_root pacman -Sy --noconfirm tmux
  elif need_cmd zypper; then
    run_as_root zypper --non-interactive install tmux
  elif need_cmd apk; then
    run_as_root apk add tmux
  elif need_cmd brew; then
    brew install tmux
  else
    echo "Error: tmux is required, but no supported package manager was found." >&2
    echo "Install tmux manually, then rerun this script." >&2
    exit 1
  fi
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

install_tmux

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
