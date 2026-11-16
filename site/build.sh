#!/usr/bin/env bash
# Build every demo for the web into dist/<demo>/, write dist/index.html linking to them, and
# render HOW-IT-WORKS.md into dist/how-it-works.html.
#
# A demo is any directory at the repo root holding both Cargo.toml and index.html, so a new
# demo appears on the index without editing this script. Its card's title is the <title> of
# its index.html, its text the `description` in its Cargo.toml, and its picture is drawn by
# site/thumbs.js from the directory name.
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
  # trunk occasionally fails copying the optimised wasm into dist, and a rerun succeeds;
  # retry once rather than fail the whole site (cause unknown: cellular-automata-z02).
  if ! (cd "$dir" && trunk build --release --public-url ./ --dist "$dist/$dir"); then
    echo "== trunk failed for $dir; retrying once (see bead cellular-automata-z02)" >&2
    (cd "$dir" && trunk build --release --public-url ./ --dist "$dist/$dir")
  fi
  title="$(sed -n 's:.*<title>\(.*\)</title>.*:\1:p' "$dir/index.html" | head -1)"
  blurb="$(sed -n 's/^description *= *"\(.*\)"/\1/p' "$dir/Cargo.toml" | head -1)"
  entries+="      <li><a class=\"card\" href=\"$dir/\"><canvas class=\"thumb\" data-demo=\"$dir\" aria-hidden=\"true\"></canvas><span class=\"card-title\">${title:-$dir}</span><span class=\"card-text\">$blurb</span></a></li>"$'\n'
done

template="$root/site/index.html"
{
  sed -n '1,/<!-- DEMOS -->/p' "$template" | sed '$d'
  printf '%s' "$entries"
  sed -n '/<!-- DEMOS -->/,$p' "$template" | sed '1d'
} > "$dist/index.html"
cp "$root/site/style.css" "$root/site/thumbs.js" "$dist/"
cargo run --release --quiet -p mdpage -- \
  "$root/HOW-IT-WORKS.md" "$root/site/page.html" "$dist/how-it-works.html"

echo "== built ${#demos[@]} demo(s) into $dist"
