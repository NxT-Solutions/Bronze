#!/usr/bin/env bash
# Local debug packaging (story 9.1, SEC-005). No notarization, no network updater.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${BRONZE_PACKAGE_OUT:-$ROOT/dist/debug-pack}"
HOST="$(uname -m)"
case "$HOST" in
  arm64|aarch64) ARCH="arm64" ;;
  x86_64) ARCH="x86_64" ;;
  *) ARCH="$HOST" ;;
esac

mkdir -p "$OUT"
printf '%s\n' "$ARCH" >"$OUT/arch.txt"
printf '%s\n' "arm64" "x86_64" >"$OUT/release-arches.txt"

ENTITLEMENTS="$ROOT/apps/desktop/src-tauri/entitlements/macos.release.plist"
CONF="$ROOT/apps/desktop/src-tauri/tauri.conf.json"
if grep -q 'get-task-allow' "$ENTITLEMENTS" "$CONF"; then
  echo "release config must not contain get-task-allow" >&2
  exit 1
fi

{
  echo "# SBOM stub — cargo + pnpm lists (not a notarized CycloneDX)"
  echo "## cargo"
  (cd "$ROOT" && cargo metadata --format-version 1 --no-deps --offline 2>/dev/null \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); print("\n".join(sorted(p["name"]+"@"+p["version"] for p in d.get("packages",[]))))') \
    || (cd "$ROOT" && cargo tree --prefix none --offline | sort -u)
  echo "## pnpm"
  (cd "$ROOT" && pnpm list --depth 0)
} >"$OUT/sbom-stub.txt"

{
  echo "# SHA-256"
  shasum -a 256 \
    "$ROOT/Cargo.lock" \
    "$ROOT/pnpm-lock.yaml" \
    "$ENTITLEMENTS" \
    "$OUT/arch.txt" \
    "$OUT/release-arches.txt" \
    "$OUT/sbom-stub.txt"
} >"$OUT/SHA256SUMS"

echo "wrote $OUT (arch=$ARCH; release arches arm64 + x86_64; ADR-002 Accepted)"
