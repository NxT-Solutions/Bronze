#!/bin/sh
# Build-time / developer vendor only. Never invoked at capture or runtime.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
MANIFEST="$ROOT/vendor/MANIFEST"
FILENAME="$(sed -n 's/^filename = //p' "$MANIFEST")"
SHA="$(sed -n 's/^sha256 = //p' "$MANIFEST")"
REPO="$(sed -n 's/^gguf_repo = //p' "$MANIFEST")"
REV="$(sed -n 's/^revision = //p' "$MANIFEST")"
DEST="$ROOT/vendor/$FILENAME"
URL="https://huggingface.co/${REPO}/resolve/${REV}/${FILENAME}"

mkdir -p "$ROOT/vendor"
if [ -f "$DEST" ]; then
  ACTUAL="$(shasum -a 256 "$DEST" | awk '{print $1}')"
  if [ "$ACTUAL" = "$SHA" ]; then
    echo "already vendored: $DEST"
    exit 0
  fi
  echo "hash mismatch, re-downloading" >&2
  rm -f "$DEST"
fi

curl -L --fail --proto '=https' --tlsv1.2 -o "$DEST" "$URL"
ACTUAL="$(shasum -a 256 "$DEST" | awk '{print $1}')"
if [ "$ACTUAL" != "$SHA" ]; then
  echo "SHA-256 mismatch: got $ACTUAL want $SHA" >&2
  rm -f "$DEST"
  exit 1
fi
echo "vendored $DEST"
echo "sha256 $ACTUAL"
