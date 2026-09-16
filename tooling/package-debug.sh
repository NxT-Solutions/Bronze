#!/usr/bin/env bash
# Local debug packaging (story 9.1, SEC-005). No notarization, no network updater.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${BRONZE_PACKAGE_OUT:-$ROOT/dist/debug-pack}"
DG01_INTEL="${DG01_INTEL:-0}"
ARCH="arm64"
if [[ "$DG01_INTEL" == "1" ]]; then
  echo "DG-01 Intel support is not accepted silently (ADR-002 remains Proposed)." >&2
  exit 1
fi

mkdir -p "$OUT"
printf '%s\n' "$ARCH" >"$OUT/arch.txt"

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
    "$OUT/sbom-stub.txt"
} >"$OUT/SHA256SUMS"

echo "wrote $OUT (arch=$ARCH)"
