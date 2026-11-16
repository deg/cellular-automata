// Small pictures for the index page: each card's canvas gets a tiny run of its own
// automaton, in the demo's colours. Canvases are
// matched by data-demo (the demo's directory name); an unknown name gets a generic
// picture, so a new demo still has one.
(() => {
  "use strict";

  const NAVY = [0x14, 0x18, 0x30];
  const TEAL = [0x2a, 0x9d, 0x8f];
  const GOLD = [0xe9, 0xc4, 0x6a];
  const CORAL = [0xe7, 0x6f, 0x51];
  const SPARK = [0xf4, 0xf1, 0xde];
  const BLUE = [0x3a, 0x6e, 0xa5];
  const WIRE = [0xd9, 0x8c, 0x2b];
  const HEAD = [0x5a, 0xb4, 0xff];
  const TAIL = [0xe0, 0x45, 0x3a];
  const ANT = [0xff, 0x30, 0x30];

  // Pictures are 80 x 50 cells, matching the cards' 16 : 10 frame.
  const W = 80, H = 50;

  // A small seeded random generator, so the pictures are the same on every visit.
  function rng(seed) {
    let s = seed >>> 0;
    return () => ((s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 4294967296);
  }

  // Draw a w x h grid of values into the canvas, one pixel per cell.
  function paint(canvas, w, h, cells, colour) {
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");
    const img = ctx.createImageData(w, h);
    for (let i = 0; i < w * h; i++) {
      const [r, g, b] = colour(cells[i], i);
      img.data.set([r, g, b, 255], i * 4);
    }
    ctx.putImageData(img, 0, 0);
  }

  // Rows of an elementary rule from a first row; returns the w x h cells. Beyond the row's
  // ends, cells are always 0.
  function elementary(rule, first, h) {
    const w = first.length;
    const cells = new Uint8Array(w * h);
    cells.set(first);
    for (let y = 1; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const at = (dx) => {
          const nx = x + dx;
          return nx >= 0 && nx < w ? cells[(y - 1) * w + nx] : 0;
        };
        const left = at(-1), right = at(1);
        cells[y * w + x] = (rule >> ((left << 2) | (at(0) << 1) | right)) & 1;
      }
    }
    return cells;
  }

  // Count each cell's live 8-neighbours (no wrap), for the Life-like pictures.
  function neighbours(cells, w, h, alive) {
    const n = new Uint8Array(w * h);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      if (!alive(cells[y * w + x])) continue;
      for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
        const nx = x + dx, ny = y + dy;
        if ((dx || dy) && nx >= 0 && ny >= 0 && nx < w && ny < h) n[ny * w + nx]++;
      }
    }
    return n;
  }

  const pictures = {
    sandpile(canvas) {
      const cells = new Uint32Array(W * H);
      const c = (H >> 1) * W + (W >> 1);
      cells[c] = 3000;
      for (let busy = true; busy;) {
        busy = false;
        for (let y = 1; y < H - 1; y++) for (let x = 1; x < W - 1; x++) {
          const i = y * W + x, v = cells[i];
          if (v < 4) continue;
          const k = v >> 2;
          cells[i] = v & 3;
          cells[i - 1] += k; cells[i + 1] += k; cells[i - W] += k; cells[i + W] += k;
          busy = true;
        }
      }
      paint(canvas, W, H, cells, (v) => [NAVY, TEAL, GOLD, CORAL][v] || SPARK);
    },

    elementary(canvas) {
      const first = new Uint8Array(W);
      first[W >> 1] = 1;
      paint(canvas, W, H, elementary(30, first, H), (v) => (v ? GOLD : NAVY));
    },

    life(canvas) {
      // The Gosper glider gun, run until it has fired a stream of gliders.
      const rle = "24bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bobo$10bo5bo7bo$11bo3bo$12b2o!";
      let cells = new Uint8Array(W * H);
      let x = 0, y = 0, count = "";
      for (const ch of rle) {
        if (ch >= "0" && ch <= "9") { count += ch; continue; }
        const n = count ? +count : 1;
        count = "";
        if (ch === "b") x += n;
        else if (ch === "o") for (let k = 0; k < n; k++) cells[(y + 2) * W + 2 + x++] = 1;
        else if (ch === "$") { y += n; x = 0; }
      }
      for (let g = 0; g < 150; g++) {
        const n = neighbours(cells, W, H, (v) => v === 1);
        cells = cells.map((v, i) => (n[i] === 3 || (v && n[i] === 2) ? 1 : 0));
      }
      paint(canvas, W, H, cells, (v) => (v ? GOLD : NAVY));
    },

    "langtons-ant"(canvas) {
      // Run past step 10,000, when the ant starts building its highway.
      const w = 120, h = 75;
      const cells = new Uint8Array(w * h);
      let x = w >> 1, y = h >> 1, d = 0;
      const dirs = [[0, -1], [1, 0], [0, 1], [-1, 0]];
      for (let s = 0; s < 11500; s++) {
        const i = y * w + x;
        d = (d + (cells[i] ? 3 : 1)) % 4;
        cells[i] ^= 1;
        x = (x + dirs[d][0] + w) % w;
        y = (y + dirs[d][1] + h) % h;
      }
      const ant = y * w + x;
      paint(canvas, w, h, cells, (v, i) => (i === ant ? ANT : v ? TEAL : NAVY));
    },

    "brians-brain"(canvas) {
      // A random patch in the middle, a few generations after it starts to spread.
      const random = rng(1987);
      let cells = new Uint8Array(W * H); // 0 ready, 1 firing, 2 recovering
      for (let y = 15; y < 35; y++) for (let x = 26; x < 54; x++) {
        if (random() < 0.3) cells[y * W + x] = 1;
      }
      for (let g = 0; g < 7; g++) {
        const n = neighbours(cells, W, H, (v) => v === 1);
        cells = cells.map((v, i) => (v === 1 ? 2 : v === 2 ? 0 : n[i] === 2 ? 1 : 0));
      }
      paint(canvas, W, H, cells, (v) => [NAVY, SPARK, BLUE][v]);
    },

    mandelbrot(canvas) {
      // The whole set, each outside point coloured by the step its z passes 2 at, in
      // bands. Twice the other pictures' detail: at 80 x 50 the set is a blob.
      const w = 160, h = 100, step = 2.8 / h, limit = 200;
      const cells = new Uint16Array(w * h); // the escape step, or 0 for the set
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        const cr = -0.65 + (x + 0.5 - w / 2) * step, ci = (y + 0.5 - h / 2) * step;
        let zr = 0, zi = 0, n = 0;
        while (zr * zr + zi * zi <= 4 && n < limit) {
          [zr, zi] = [zr * zr - zi * zi + cr, 2 * zr * zi + ci];
          n++;
        }
        cells[y * w + x] = zr * zr + zi * zi > 4 ? n : 0;
      }
      const DUSK = [0x1f, 0x2a, 0x55];
      paint(canvas, w, h, cells, (n) =>
        n < 3 ? NAVY : n < 5 ? DUSK : [BLUE, TEAL, GOLD, CORAL][n % 4]);
    },

    wireworld(canvas) {
      // The demo's OR gate: two clocks feeding one output through diodes.
      const rows = [
        " #@~", "#   #      ##", "#   ######## ##########", "#   #      ##         #",
        " ###                  #", "                      ###########", " @~                   #",
        "#  #       ##         #", "#  ######### ##########", " ##        ##",
      ];
      const w = 40, h = 25, ox = 3, oy = 7;
      let cells = new Uint8Array(w * h); // 0 empty, 1 head, 2 tail, 3 wire
      rows.forEach((row, y) => [...row].forEach((ch, x) => {
        const v = { "#": 3, "@": 1, "~": 2 }[ch];
        if (v) cells[(y + oy) * w + x + ox] = v;
      }));
      for (let g = 0; g < 37; g++) {
        const n = neighbours(cells, w, h, (v) => v === 1);
        cells = cells.map((v, i) => (v === 1 ? 2 : v === 2 ? 3 : v === 3 && (n[i] === 1 || n[i] === 2) ? 1 : v));
      }
      paint(canvas, w, h, cells, (v) => [NAVY, HEAD, TAIL, WIRE][v]);
    },
  };

  // Any other demo: Rule 110 from a random row.
  function generic(canvas) {
    const random = rng(110);
    const first = Uint8Array.from({ length: W }, () => (random() < 0.5 ? 1 : 0));
    paint(canvas, W, H, elementary(110, first, H), (v) => (v ? TEAL : NAVY));
  }


  for (const canvas of document.querySelectorAll("canvas[data-demo]")) {
    (pictures[canvas.dataset.demo] || generic)(canvas);
  }
})();
