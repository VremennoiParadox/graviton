#!/usr/bin/env bash
# Record graviton demo assets per PLANNING.md (VHS → GIF, optional asciinema).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Building release binary..."
cargo build --release

if command -v vhs >/dev/null 2>&1; then
  echo "Rendering assets/demo.gif with VHS..."
  vhs assets/demo.tape
  echo "Wrote assets/demo.gif"
else
  echo "VHS not found. Install from https://github.com/charmbracelet/vhs"
  echo "Then run: vhs assets/demo.tape"
  exit 1
fi

if command -v asciinema >/dev/null 2>&1; then
  echo ""
  echo "Optional: record asciinema with:"
  echo "  asciinema rec assets/demo.cast"
else
  echo "asciinema not installed (optional)."
fi
