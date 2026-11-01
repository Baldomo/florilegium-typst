#!/usr/bin/env bash
# Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
# usage: tools/oracle/gen.sh [prefix]
# Runs every case whose id starts with prefix in its own fresh lua process and rewrites
# those lines of core/tests/fixtures/oracle.tsv (id<TAB>bytes<TAB>sha256, sorted).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
lib="$(cd "$here/../.." && pwd)"
tsv="$lib/core/tests/fixtures/oracle.tsv"
prefix="${1:-}"
cd "$lib"
LUA=(lua)
command -v mise >/dev/null 2>&1 && LUA=(mise exec -- lua)
export here
export LUA_CMD="${LUA[*]}"

one() {
  set -euo pipefail
  local id="$1"; tmp=""
  tmp="$(mktemp)"
  trap 'rm -f "${tmp:-}"' EXIT
  $LUA_CMD "$here/case.lua" "$id" > "$tmp" || { echo "gen.sh: case failed: $id" >&2; exit 1; }
  printf '%s\t%s\t%s\n' "$id" "$(wc -c < "$tmp" | tr -d ' ')" "$(sha256sum "$tmp" | cut -d' ' -f1)"
}
export -f one

new="$(mktemp)"
trap 'rm -f "$new" "$tsv.tmp"' EXIT
$LUA_CMD "$here/list.lua" "$prefix" | xargs -d '\n' -P "$(nproc)" -I{} bash -c 'one "$1"' _ {} > "$new" \
  || { echo "gen.sh: some cases failed; $tsv left unchanged" >&2; exit 1; }
mkdir -p "$(dirname "$tsv")"
touch "$tsv"
{
  awk -v p="$prefix" 'index($0, p) != 1' "$tsv"
  cat "$new"
} | LC_ALL=C sort > "$tsv.tmp"
mv "$tsv.tmp" "$tsv"
echo "oracle.tsv: $(grep -c '' "$tsv") cases ($(grep -c '' <<<"$(cut -f1 "$tsv" | awk -v p="$prefix" 'index($0,p)==1')") with prefix '$prefix')"
