// Download progress for a demo page's "Loading…" text. Every demo's index.html names this
// file in its <link data-trunk rel="rust" data-initializer=...>, and trunk then streams the
// demo's .wasm through these hooks. The download is a few megabytes and GitHub Pages can be
// slow to serve it, so without a count the page looks stuck.
//
// `current` counts the bytes as they arrive, already unzipped, and `total` is the .wasm's
// size, which trunk knows at build time, so their ratio is the true fraction even though
// the server sends it compressed.
export default function () {
  const text = () => document.getElementById("loading_text");
  const show = (message) => {
    const element = text();
    if (element) element.textContent = message;
  };
  return {
    onProgress({ current, total }) {
      if (total > 0) {
        show(`Loading… ${Math.min(100, Math.floor((100 * current) / total))}%`);
      }
    },
    onComplete() {
      show("Starting…");
    },
    onFailure(reason) {
      show("The demo didn’t load. Try reloading the page.");
      console.error(reason);
    },
  };
}
