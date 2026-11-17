#!/usr/bin/env bash
# Build every demo for the web into dist/<demo>/, write dist/index.html linking to them, and
# render HOW-IT-WORKS.md into dist/how-it-works.html.
#
# A demo is any directory at the repo root holding both Cargo.toml and index.html, so a new
# demo appears on the index without editing this script (add it to site/order.txt to place
# its card, and to the root README's table). Its card's title is the <title> of
# its index.html, its text the `description` in its Cargo.toml, and its picture is drawn by
# site/thumbs.js from the directory name.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

found=()
for manifest in */Cargo.toml; do
  dir="${manifest%/Cargo.toml}"
  [[ -f "$dir/index.html" ]] && found+=("$dir")
done
if [[ ${#found[@]} -eq 0 ]]; then
  echo "no demos found" >&2
  exit 1
fi

# The cards' order and groups are site/order.txt's (see it for why): a "== Name" line starts
# a group. groups[i] is the group of demos[i]. A demo the file doesn't list goes last, in a
# group of its own.
demos=()
groups=()
group=""
while read -r line; do
  [[ -z "$line" || "$line" == \#* ]] && continue
  if [[ "$line" == "== "* ]]; then
    group="${line#== }"
    continue
  fi
  if [[ " ${found[*]} " != *" $line "* ]]; then
    echo "site/order.txt lists '$line', which is not a demo" >&2
    exit 1
  fi
  if [[ -z "$group" ]]; then
    echo "site/order.txt: '$line' comes before any '== group' line" >&2
    exit 1
  fi
  demos+=("$line")
  groups+=("$group")
done < site/order.txt
for dir in "${found[@]}"; do
  if [[ " ${demos[*]} " != *" $dir "* ]]; then
    echo "warning: $dir is not in site/order.txt; its card goes last" >&2
    demos+=("$dir")
    groups+=("More")
  fi
done

# The root README lists the demos too, in tables under "### Group" headings: keep both the
# demos and the headings in site/order.txt's order.
readme_order="$(sed -n 's/^| \[[^]]*\](\([^/)]*\)\/) |.*/\1/p' README.md | tr '\n' ' ')"
if [[ "$readme_order" != "${demos[*]} " ]]; then
  echo "README.md's demo tables are not in site/order.txt's order:" >&2
  echo "  README:     $readme_order" >&2
  echo "  order.txt:  ${demos[*]}" >&2
  exit 1
fi
readme_groups="$(sed -n 's/^### //p' README.md | paste -sd '|' -)"
order_groups="$(printf '%s\n' "${groups[@]}" | uniq | paste -sd '|' -)"
if [[ "$readme_groups" != "$order_groups" ]]; then
  echo "README.md's ### group headings differ from site/order.txt's:" >&2
  echo "  README:     $readme_groups" >&2
  echo "  order.txt:  $order_groups" >&2
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
current=""
for i in "${!demos[@]}"; do
  dir="${demos[$i]}"
  if [[ "${groups[$i]}" != "$current" ]]; then
    [[ -n "$current" ]] && entries+="    </ul></section>"$'\n'
    current="${groups[$i]}"
    entries+="    <section class=\"group\"><h2>$current</h2><ul class=\"cards\">"$'\n'
  fi
  echo "== building $dir"
  (cd "$dir" && trunk build --release --public-url ./ --dist "$dist/$dir")
  # The script that reloads a page cached from before a deploy (see the file), straight
  # after <meta charset>, so it is listening before trunk's script asks for the demo's code.
  # Not before the charset: a browser looks for that only in a page's first 1,024 bytes,
  # which the script would fill. The favicon's links (site/icons.html) go after <title>.
  page="$dist/$dir/index.html"
  sed -e "/<meta charset/r $root/site/stale-page.html" -e "/<\/title>/r $root/site/icons.html" \
    "$page" > "$page.new"
  mv "$page.new" "$page"
  if ! grep -q 'stale-page-reloaded' "$page"; then
    echo "$page: no <meta charset> line to put the stale-page script after" >&2
    exit 1
  fi
  # The "Loading… n%" count (site/loading.mjs) divides the bytes received by the size
  # trunk writes into the page, but trunk measures the .wasm before wasm-opt shrinks it
  # (5.9 MB against 4.5 MB), so the count would stop near 76% and then jump. Put in the
  # size of the file actually served.
  wasm="$(cd "$dist/$dir" && ls ./*_bg.wasm)"
  size="$(wc -c < "$dist/$dir/$wasm" | tr -d ' ')"
  sed -E "s|(__trunkInitializer\\(init, '$wasm', )[0-9]+|\\1$size|" "$page" > "$page.new"
  mv "$page.new" "$page"
  if ! grep -q "__trunkInitializer(init, '$wasm', $size," "$page"; then
    echo "$page: could not set the .wasm's size for the loading count" >&2
    exit 1
  fi
  title="$(sed -n 's:.*<title>\(.*\)</title>.*:\1:p' "$dir/index.html" | head -1)"
  blurb="$(sed -n 's/^description *= *"\(.*\)"/\1/p' "$dir/Cargo.toml" | head -1)"
  entries+="      <li><a class=\"card\" href=\"$dir/\"><canvas class=\"thumb\" data-demo=\"$dir\" aria-hidden=\"true\"></canvas><span class=\"card-title\">${title:-$dir}</span><span class=\"card-text\">$blurb</span></a></li>"$'\n'
done
entries+="    </ul></section>"$'\n'

template="$root/site/index.html"
{
  sed -n '1,/<!-- DEMOS -->/p' "$template" | sed '$d'
  printf '%s' "$entries"
  sed -n '/<!-- DEMOS -->/,$p' "$template" | sed '1d'
} > "$dist/index.html"
cp "$root/site/style.css" "$root/site/thumbs.js" "$dist/"
# The Degel favicon, at the site's root so that every page's root-relative links find it
# (copied from the degel.com repo, which holds the originals and their provenance).
cp "$root/site/favicon.ico" "$root/site/favicon.svg" "$root/site/apple-touch-icon.png" "$dist/"
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
# Every page carries the favicon's three links.
missing="$(grep -L 'rel="apple-touch-icon"' "$dist"/*.html "$dist"/*/index.html || true)"
if [[ -n "$missing" ]]; then
  echo "pages without the favicon links:" >&2
  echo "$missing" >&2
  exit 1
fi

command rm -rf "$final"
mv "$dist" "$final"
trap - EXIT
echo "== built ${#demos[@]} demo(s) into $final"
