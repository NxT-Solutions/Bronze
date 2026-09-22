#!/bin/sh
# Build-time / developer vendor only. Never invoked at capture or runtime.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ID="${1:-smol-360}"

vendor_one() {
  id="$1"
  manifest="$ROOT/vendor/manifests/$id"
  if [ ! -f "$manifest" ]; then
    echo "unknown title tier: $id" >&2
    echo "usage: $0 [smol-135|smol-360|qwen-05|all]" >&2
    exit 1
  fi
  filename="$(sed -n 's/^filename = //p' "$manifest")"
  sha="$(sed -n 's/^sha256 = //p' "$manifest")"
  repo="$(sed -n 's/^gguf_repo = //p' "$manifest")"
  rev="$(sed -n 's/^revision = //p' "$manifest")"
  dest="$ROOT/vendor/$filename"
  url="https://huggingface.co/${repo}/resolve/${rev}/${filename}"

  mkdir -p "$ROOT/vendor"
  if [ -f "$dest" ]; then
    actual="$(shasum -a 256 "$dest" | awk '{print $1}')"
    if [ "$actual" = "$sha" ]; then
      echo "already vendored: $dest"
      return 0
    fi
    echo "hash mismatch, re-downloading $id" >&2
    rm -f "$dest"
  fi

  curl -L --fail --proto '=https' --tlsv1.2 -o "$dest" "$url"
  actual="$(shasum -a 256 "$dest" | awk '{print $1}')"
  if [ "$actual" != "$sha" ]; then
    echo "SHA-256 mismatch: got $actual want $sha" >&2
    rm -f "$dest"
    exit 1
  fi
  echo "vendored $dest"
  echo "sha256 $actual"
}

if [ "$ID" = "all" ]; then
  vendor_one smol-135
  vendor_one smol-360
  vendor_one qwen-05
  exit 0
fi

vendor_one "$ID"
