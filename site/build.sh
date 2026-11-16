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

# Build into a directory of our own, and swap it in at the end. Two builds at once (say a
# `make serve` while another build runs) used to share dist/: each began by deleting it,
# pulling it from under the other, whose trunk then failed copying its wasm into a
# directory that no longer existed.
final="$root/dist"
dist="$(mktemp -d "$root/dist.building.XXXXXX")"
trap 'command rm -rf "$dist"' EXIT

entries=""
for dir in "${demos[@]}"; do
  echo "== building $dir"
  (cd "$dir" && trunk build --release --public-url ./ --dist "$dist/$dir")
  # The script that reloads a page cached from before a deploy (see the file), straight
  # after <meta charset>, so it is listening before trunk's script asks for the demo's code.
  # Not before the charset: a browser looks for that only in a page's first 1,024 bytes,
  # which the script would fill.
  page="$dist/$dir/index.html"
  sed "/<meta charset/r $root/site/stale-page.html" "$page" > "$page.new"
  mv "$page.new" "$page"
  if ! grep -q 'stale-page-reloaded' "$page"; then
    echo "$page: no <meta charset> line to put the stale-page script after" >&2
    exit 1
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

# GitHub Pages lets browsers keep any file for 10 minutes. After a deploy, a browser could
# pair the new index.html with its cached old thumbs.js (it did, on 2026-11-16: the new
# Mandelbrot card got the generic picture until a forced refresh). So the pages ask for
# each shared file by a URL carrying a fingerprint of its contents: a deploy that changes
# the file changes the URL, and no stale copy can match it. (The demos' own .js and .wasm
# already have hashed names, from trunk.)
fingerprint() { cksum < "$1" | cut -d' ' -f1; }
style_v="$(fingerprint "$root/site/style.css")"
thumbs_v="$(fingerprint "$root/site/thumbs.js")"
for page in "$dist/index.html" "$dist/how-it-works.html"; do
  sed -e "s|href=\"style.css\"|href=\"style.css?v=$style_v\"|" \
      -e "s|src=\"thumbs.js\"|src=\"thumbs.js?v=$thumbs_v\"|" "$page" > "$page.new"
  mv "$page.new" "$page"
done
if grep -q -e 'href="style.css"' -e 'src="thumbs.js"' "$dist/index.html" "$dist/how-it-works.html"; then
  echo "a page still loads style.css or thumbs.js without a fingerprint" >&2
  exit 1
fi

command rm -rf "$final"
mv "$dist" "$final"
trap - EXIT
echo "== built ${#demos[@]} demo(s) into $final"
