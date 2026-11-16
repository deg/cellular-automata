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

  // Pictures are 80 x 50 cells by default, matching the cards' 16 : 10 frame. A picture
  // that needs more room, more detail or bigger cells uses its own size in the same
  // proportions.
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

    "lattice-gas"(canvas) {
      // FHP-I particles blowing past a plate, then averaged over blocks and steps: each
      // pixel is the swirl (vorticity) of the mean flow, coral one way and blue the other.
      const STONE = [0xb8, 0xbc, 0xc8];
      const w = 64, h = 40, random = rng(1986);
      const wind = 0.3, density = 0.2, plateX = 16, plateRows = 5;
      // A node is a byte. Bits 0-5: particles moving at 0°, 60°, ... 300° (y up); bit 6 a
      // wall; bit 7 the random bit choosing which way a head-on pair turns.
      const rot = (n, k) => ((n << k) | (n >> (6 - k))) & 63;
      const table = new Uint8Array(256);
      // Per byte: particles, and momentum in units of (cos / 2, sin * sqrt 3 / 4). Walls 0.
      const count = new Uint8Array(256), momX = new Int8Array(256), momY = new Int8Array(256);
      const units = [[2, 0], [1, 2], [-1, 2], [-2, 0], [-1, -2], [1, -2]];
      for (let s = 0; s < 256; s++) {
        const n = s & 63, wall = s & 64;
        let out = n;
        if (wall) out = rot(n, 3);
        else if (n === 21 || n === 42) out = rot(n, 1);
        else if (n === 9 || n === 18 || n === 36) out = rot(n, s & 128 ? 5 : 1);
        table[s] = ((s & 192) ^ (out !== n && !wall ? 128 : 0)) | out;
        for (let k = 0; k < 6 && !wall; k++) {
          if (n >> k & 1) { count[s]++; momX[s] += units[k][0]; momY[s] += units[k][1]; }
        }
      }
      // Fresh gas at the equilibrium of the wind (d'Humières and Lallemand's (2.5)).
      const chance = [0, 1, 2, 3, 4, 5].map((i) => density * (1 + 2 * wind * Math.cos(i * Math.PI / 3)));
      const fresh = () => {
        let b = 0;
        for (let i = 0; i < 6; i++) if (random() < chance[i]) b |= 1 << i;
        return b;
      };
      let cells = new Uint8Array(w * h), next = new Uint8Array(w * h);
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
        const wall = Math.abs(x + (y & 1) / 2 - plateX) < 1 && Math.abs(y - h / 2) < plateRows;
        cells[y * w + x] = (random() < 0.5 ? 128 : 0) | (wall ? 64 : fresh());
      }
      // Blocks of 8 nodes by 8 rows, summed over every third step from step 45.
      const bw = 8, bh = 8, nx = w / bw, ny = Math.floor(h / bh);
      const n = new Float64Array(nx * ny), jx = new Float64Array(nx * ny), jy = new Float64Array(nx * ny);
      // One step: collide at every node, then stream every particle to its neighbour.
      // Bit i arrives from the neighbour in direction i + 3. Odd rows sit half a node right,
      // so a row's diagonal neighbours above and below are at x + p - 1 and x + p.
      const gather = (row, up, dn, x, l, r, a, b) => (cells[row + x] & 192) |
        (cells[row + l] & 1) | (cells[row + r] & 8) | (cells[dn + a] & 2) |
        (cells[dn + b] & 4) | (cells[up + b] & 16) | (cells[up + a] & 32);
      function step() {
        for (let i = 0; i < w * h; i++) cells[i] = table[cells[i]];
        for (let y = 0; y < h; y++) {
          const p = y & 1, row = y * w, up = (y + h - 1) % h * w, dn = (y + 1) % h * w;
          const da = dn - 1 + p, ua = up - 1 + p; // the lower-left and upper-left neighbours
          for (let x = 1; x < w - 1; x++) {
            next[row + x] = (cells[row + x] & 192) | (cells[row + x - 1] & 1) |
              (cells[row + x + 1] & 8) | (cells[da + x] & 2) | (cells[da + x + 1] & 4) |
              (cells[ua + x + 1] & 16) | (cells[ua + x] & 32);
          }
          next[row] = gather(row, up, dn, 0, w - 1, 1, p ? 0 : w - 1, p);
          next[row + w - 1] = gather(row, up, dn, w - 1, w - 2, 0, w - 2 + p, p ? 0 : w - 1);
        }
        const swap = cells;
        cells = next;
        next = swap;
        // The wind: the two left-hand columns refilled with fresh gas.
        for (let y = 0; y < h; y++) for (let i = y * w; i < y * w + 2; i++) {
          if (!(cells[i] & 64)) cells[i] = (cells[i] & 128) | fresh();
        }
      }
      function accumulate() {
        for (let by = 0; by < ny; by++) for (let y = by * bh; y < (by + 1) * bh; y++) {
          for (let bx = 0; bx < nx; bx++) {
            let sn = 0, sx = 0, sy = 0;
            for (let i = y * w + bx * bw, end = i + bw; i < end; i++) {
              const c = cells[i];
              sn += count[c]; sx += momX[c]; sy += momY[c];
            }
            const k = by * nx + bx;
            n[k] += sn; jx[k] += sx; jy[k] += sy;
          }
        }
      }
      for (let t = 0; t < 120; t++) {
        step();
        if (t >= 45 && t % 3 === 0) accumulate();
      }
      // Mean velocity per block, then its curl, du_y/dx - du_x/dy (y up; rows run down),
      // across blocks bw nodes wide and bh rows of sqrt(3)/2 tall.
      const ux = jx.map((v, k) => v / 2 / Math.max(1, n[k]));
      const uy = jy.map((v, k) => v * Math.sqrt(3) / 4 / Math.max(1, n[k]));
      const at = (a, x, y) => a[Math.min(ny - 1, Math.max(0, y)) * nx + Math.min(nx - 1, Math.max(0, x))];
      const dy = bh * Math.sqrt(3) / 2;
      const curl = new Float64Array(nx * ny);
      for (let y = 0; y < ny; y++) for (let x = 0; x < nx; x++) {
        curl[y * nx + x] = (at(uy, x + 1, y) - at(uy, x - 1, y)) / (2 * bw) +
          (at(ux, x, y + 1) - at(ux, x, y - 1)) / (2 * dy);
      }
      // Interpolated between block centres, one value per pixel.
      const swirl = new Float64Array(W * H);
      for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
        const gx = (x + 0.5) * nx / W - 0.5, gy = (y + 0.5) * ny / H - 0.5;
        const x0 = Math.floor(gx), y0 = Math.floor(gy), fx = gx - x0, fy = gy - y0;
        swirl[y * W + x] = (at(curl, x0, y0) * (1 - fx) + at(curl, x0 + 1, y0) * fx) * (1 - fy) +
          (at(curl, x0, y0 + 1) * (1 - fx) + at(curl, x0 + 1, y0 + 1) * fx) * fy;
      }
      // The demo's scale: shear across an eddy about 30 nodes wide. Colours in 32 steps
      // each way, blue for clockwise and coral for counter-clockwise.
      const scale = 1 / (wind / 15);
      const mix = (a, b, t) => a.map((v, i) => Math.round(v + (b[i] - v) * t));
      const shades = [BLUE, CORAL].map((c) => Array.from({ length: 33 }, (_, k) => mix(NAVY, c, k / 32)));
      const plate = [plateX * W / w, plateRows * H / h];
      paint(canvas, W, H, swirl, (v, i) => {
        const x = i % W, y = (i / W) | 0;
        if (Math.abs(x + 0.5 - plate[0]) < 0.8 && Math.abs(y + 0.5 - H / 2) < plate[1]) return STONE;
        const t = Math.max(-1, Math.min(1, v * scale));
        return shades[t < 0 ? 0 : 1][Math.round(Math.abs(t) * 32)];
      });
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

    "langtons-loops"(canvas) {
      // One loop after 450 generations: a small colony. Langton's 1984 table, CTRBL then
      // the next state, each entry also standing for its rotations (see the demo's sim.rs).
      const table = "000000 000012 000020 000030 000050 000063 000071 000112 000122 000132 " +
        "000212 000220 000230 000262 000272 000320 000525 000622 000722 001022 001120 002020 " +
        "002030 002050 002125 002220 002322 005222 012321 012421 012525 012621 012721 012751 " +
        "014221 014321 014421 014721 016251 017221 017255 017521 017621 017721 025271 100011 " +
        "100061 100077 100111 100121 100211 100244 100277 100511 101011 101111 101244 101277 " +
        "102026 102121 102211 102244 102263 102277 102327 102424 102626 102644 102677 102710 " +
        "102727 105427 111121 111221 111244 111251 111261 111277 111522 112121 112221 112244 " +
        "112251 112277 112321 112424 112621 112727 113221 122244 122277 122434 122547 123244 " +
        "123277 124255 124267 125275 200012 200022 200042 200071 200122 200152 200212 200222 " +
        "200232 200242 200250 200262 200272 200326 200423 200517 200522 200575 200722 201022 " +
        "201122 201222 201422 201722 202022 202032 202052 202073 202122 202152 202212 202222 " +
        "202272 202321 202422 202452 202520 202552 202622 202722 203122 203216 203226 203422 " +
        "204222 205122 205212 205222 205521 205725 206222 206722 207122 207222 207422 207722 " +
        "211222 211261 212222 212242 212262 212272 214222 215222 216222 217222 222272 222442 " +
        "222462 222762 222772 300013 300022 300041 300076 300123 300421 300622 301021 301220 " +
        "302511 401120 401220 401250 402120 402221 402326 402520 403221 500022 500215 500225 " +
        "500232 500272 500520 502022 502122 502152 502220 502244 502722 512122 512220 512422 " +
        "512722 600011 600021 602120 612125 612131 612225 700077 701120 701220 701250 702120 " +
        "702221 702251 702321 702525 702720";
      const key = (c, t, r, b, l) => (c << 12) | (t << 9) | (r << 6) | (b << 3) | l;
      // Unlisted neighbourhoods keep the centre state.
      const next = Uint8Array.from({ length: 1 << 15 }, (_, k) => k >> 12);
      for (const e of table.split(" ")) {
        const [c, t, r, b, l, i] = [...e].map(Number);
        next[key(c, t, r, b, l)] = next[key(c, l, t, r, b)] = i;
        next[key(c, b, l, t, r)] = next[key(c, r, b, l, t)] = i;
      }
      const loop = [" 22222222", "2170140142", "2022222202", "272    212", "212    212",
        "202    212", "272    212", "21222222122222", "207107107111112", " 2222222222222"];
      let cells = new Uint8Array(W * H);
      loop.forEach((row, y) => [...row].forEach((ch, x) => {
        cells[(y + 30) * W + x + 24] = ch === " " ? 0 : +ch;
      }));
      const at = (x, y) => (x >= 0 && y >= 0 && x < W && y < H ? cells[y * W + x] : 0);
      for (let g = 0; g < 450; g++) {
        const now = (i) => {
          const x = i % W, y = (i / W) | 0;
          return next[key(cells[i], at(x, y - 1), at(x + 1, y), at(x, y + 1), at(x - 1, y))];
        };
        cells = Uint8Array.from(cells, (_, i) => now(i));
      }
      const SHEATH = [0x4a, 0x4f, 0x80], MARKER = [0x5a, 0xb4, 0xff];
      const PINK = [0xd1, 0x6b, 0xa5];
      paint(canvas, W, H, cells, (v) => [NAVY, TEAL, SHEATH, MARKER, CORAL, PINK, SPARK, GOLD][v]);
    },

    snowflakes(canvas) {
      // Reiter's model, the demo's Stellar preset, grown from one seed on a small hex grid
      // (axial coordinates) until the ice nears the circle held at the background.
      const VAPOUR = [0x6c, 0x8c, 0xc4], OLD_ICE = [0x8f, 0xc1, 0xe3];
      // R counts hex rows, as Reiter's program does: the circle is R sqrt(3)/2 cells wide.
      const R = 35, beta = 0.6, gamma = 0.01;
      const c = R + 2, n = 2 * c + 1;
      let water = new Float64Array(n * n).fill(beta), next = new Float64Array(n * n).fill(beta);
      const field = new Float64Array(n * n).fill(beta), ice = new Uint8Array(n * n);
      const rec = new Uint8Array(n * n), list = [];
      for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
        const q = x - c, r = y - c;
        if (4 * (q * q + q * r + r * r) < 3 * R * R) list.push(y * n + x);
      }
      const inside = Int32Array.from(list), m = inside.length;
      water[c * n + c] = 1;
      for (let reach = 0; 4 * reach < 3 * (R - 2) * (R - 2);) {
        for (let k = 0; k < m; k++) { const i = inside[k]; ice[i] = water[i] >= 1 ? 1 : 0; }
        for (let k = 0; k < m; k++) {
          const i = inside[k];
          rec[i] = ice[i] | ice[i + 1] | ice[i - 1] | ice[i + n] | ice[i - n] | ice[i + n - 1] | ice[i - n + 1];
          field[i] = rec[i] ? 0 : water[i];
        }
        for (let k = 0; k < m; k++) {
          const i = inside[k], v = field[i];
          const s = field[i + 1] + field[i - 1] + field[i + n] + field[i - n] + field[i + n - 1] + field[i - n + 1];
          next[i] = v + (s - 6 * v) / 12 + (rec[i] ? water[i] + gamma : 0);
          if (next[i] >= 1 && !ice[i]) {
            const q = (i % n) - c, r = ((i / n) | 0) - c;
            reach = Math.max(reach, q * q + q * r + r * r);
          }
        }
        [water, next] = [next, water];
      }
      let most = 1;
      for (const i of inside) most = Math.max(most, water[i]);
      // Each pixel shows the hex cell its centre falls in (rounded in cube coordinates).
      const w = 160, h = 100, scale = R * Math.sqrt(3) / 2 / 46, cells = new Float64Array(w * h);
      for (let py = 0; py < h; py++) for (let px = 0; px < w; px++) {
        const r = (py + 0.5 - h / 2) * scale * 2 / Math.sqrt(3), q = (px + 0.5 - w / 2) * scale - r / 2;
        let rq = Math.round(q), rr = Math.round(r);
        const rs = Math.round(-q - r), dq = Math.abs(rq - q), dr = Math.abs(rr - r), ds = Math.abs(rs + q + r);
        if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs;
        const x = rq + c, y = rr + c;
        cells[py * w + px] = x >= 0 && y >= 0 && x < n && y < n ? water[y * n + x] : beta;
      }
      const mix = (a, b, t) => a.map((v, k) => Math.round(v + (b[k] - v) * t));
      paint(canvas, w, h, cells, (v) => (v < 1
        ? mix(NAVY, VAPOUR, Math.max(v, 0))
        : mix(SPARK, OLD_ICE, Math.sqrt(Math.min((v - 1) / (most - 1), 1)))));
    },

    cyclic(canvas) {
      // The basic cyclic automaton (range 1, 4 neighbours, threshold 1) with 8 colours,
      // from random colours until spirals have formed: a cell takes the next colour when a
      // neighbour has it. The edges wrap.
      const n = 8, random = rng(14);
      let cells = Uint8Array.from({ length: W * H }, () => Math.floor(random() * n));
      for (let g = 0; g < 250; g++) {
        cells = cells.map((v, i) => {
          const x = i % W, y = (i - x) / W, next = (v + 1) % n;
          const hit = [[x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]]
            .some(([nx, ny]) => cells[((ny + H) % H) * W + (nx + W) % W] === next);
          return hit ? next : v;
        });
      }
      // n colours round the loop blue, teal, gold, coral and back, evenly by distance.
      const loop = [BLUE, TEAL, GOLD, CORAL];
      const len = loop.map((a, i) => Math.hypot(...a.map((c, j) => c - loop[(i + 1) % 4][j])));
      const total = len.reduce((s, l) => s + l);
      const colours = Array.from({ length: n }, (_, k) => {
        let along = (k * total) / n, i = 0;
        while (i < 3 && along > len[i]) along -= len[i++];
        const f = along / len[i], a = loop[i], b = loop[(i + 1) % 4];
        return a.map((c, j) => Math.round(c + (b[j] - c) * f));
      });
      paint(canvas, W, H, cells, (v) => colours[v]);
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
