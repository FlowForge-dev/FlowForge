#!/usr/bin/env sh
set -eu

repo="FlowForge-dev/FlowForge"
bin_dir="${FLOWFORGE_BIN_DIR:-$HOME/.local/bin}"
cargo_bin="$HOME/.cargo/bin"
tmp_dir="$(mktemp -d)"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  *)
    echo "Unsupported platform. Install from source instead:"
    echo "  cargo install --git https://github.com/$repo"
    exit 1
    ;;
esac

url="https://github.com/$repo/releases/latest/download/flowforge-$target.tar.gz"

mkdir -p "$bin_dir"
echo "Downloading FlowForge for $target..."
if curl -fsSL "$url" -o "$tmp_dir/flowforge.tar.gz"; then
  tar -xzf "$tmp_dir/flowforge.tar.gz" -C "$tmp_dir"
  cp "$tmp_dir/forge" "$bin_dir/forge"
  chmod +x "$bin_dir/forge"
  installed_forge="$bin_dir/forge"
else
  echo "No release binary found yet. Building from source instead..."
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust is required for source install: https://rustup.rs/"
    exit 1
  fi
  cargo install --git "https://github.com/$repo" --force
  installed_forge="$cargo_bin/forge"
fi

"$installed_forge" provider list

echo ""
echo "Installed forge."
echo "Starter files were created automatically in ~/.forgeflow."
echo "Next: set your API key, then run: forge provider test openai"
