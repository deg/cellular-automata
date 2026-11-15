#!/usr/bin/env bash
# Build every demo for the web into dist/<demo>/, and write dist/index.html linking to them.
#
# A demo is any directory at the repo root holding both Cargo.toml and index.html, so a new
# demo appears on the index without editing this script. Its link text is the <title> of
# its index.html, and its blurb is the `description` in its Cargo.toml.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dist="$root/dist"
cd "$root"

demos=()
for manifest in */Cargo.toml; do
  dir="${manifest%/Cargo.toml}"
  [[ -f "$dir/index.html" ]] && demos+=("$dir")
done
if [[ ${#demos[@]} -eq 0 ]]; then
  echo "no demos found" >&2
  exit 1
fi

command rm -rf "$dist"
mkdir -p "$dist"

entries=""
for dir in "${demos[@]}"; do
  echo "== building $dir"
  (cd "$dir" && trunk build --release --public-url ./ --dist "$dist/$dir")
  title="$(sed -n 's:.*<title>\(.*\)</title>.*:\1:p' "$dir/index.html" | head -1)"
  blurb="$(sed -n 's/^description *= *"\(.*\)"/\1/p' "$dir/Cargo.toml" | head -1)"
  entries+="      <li><a href=\"$dir/\">${title:-$dir}</a><p>$blurb</p></li>"$'\n'
done

template="$root/site/index.html"
{
  sed -n '1,/<!-- DEMOS -->/p' "$template" | sed '$d'
  printf '%s' "$entries"
  sed -n '/<!-- DEMOS -->/,$p' "$template" | sed '1d'
} > "$dist/index.html"

echo "== built ${#demos[@]} demo(s) into $dist"
