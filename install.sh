#!/usr/bin/env sh
set -eu

repo="FlowForge-dev/FlowForge"
bin_dir="${FLOWFORGE_BIN_DIR:-$HOME/.local/bin}"
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
curl -fsSL "$url" -o "$tmp_dir/flowforge.tar.gz"
tar -xzf "$tmp_dir/flowforge.tar.gz" -C "$tmp_dir"
cp "$tmp_dir/forge" "$bin_dir/forge"
chmod +x "$bin_dir/forge"
"$bin_dir/forge" init

echo "Installed forge to $bin_dir/forge"
echo "Run: forge provider list"
