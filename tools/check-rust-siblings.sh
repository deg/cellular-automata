#!/usr/bin/env bash
# Build and test David's other Rust projects, printing one summary line each.
#
# Run it before and after updating the Rust toolchain (`rustup update stable`): the upgrade
# is fine only if every project still builds and passes, with no new warnings. Full logs go
# to target/sibling-check/<rustc version>/, so two runs can be diffed.
set -uo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
parent="$(dirname "$here")"
projects=(CPeMulator nscheme llm_wiki/src-tauri)

version="$(rustc --version | awk '{print $2}')"
logs="$here/target/sibling-check/$version"
mkdir -p "$logs"
echo "rustc $version; logs in $logs"

status=0
for p in "${projects[@]}"; do
  dir="$parent/$p"
  if [[ ! -f "$dir/Cargo.toml" ]]; then
    echo "$p: not found at $dir"
    status=1
    continue
  fi
  log="$logs/${p//\//_}.log"
  (cd "$dir" && cargo build --all-targets && cargo test) > "$log" 2>&1
  code=$?
  tests="$(grep -E '^test result:' "$log" | awk '{p += $4; f += $6} END {print p+0 " passed, " f+0 " failed"}')"
  warnings="$(grep -cE '^warning' "$log")"
  echo "$p: exit $code; $tests; $warnings warning lines"
  [[ $code -eq 0 ]] || status=1
done
exit $status
