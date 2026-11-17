# Simple rules, strange worlds

**Cellular automata and other simple rules.** Demos of famous simple rules, written in
Rust: cellular automata, and a few relatives such as the Mandelbrot set. Each runs in a
native window or in the browser, and explains itself: the rule, its history, and things to
try. They are grouped as on the site's home page.

**Try them in your browser: <https://automata.degel.com/>**

### The classics

| Demo | What it shows |
|---|---|
| [Conway's Game of Life](life/) | Cells live or die by counting their neighbours; gliders, guns and other rules to try. |
| [Elementary cellular automata](elementary/) | All 256 one-dimensional rules, from the chaos of Rule 30 to the computer inside Rule 110. |
| [Langton's ant](langtons-ant/) | An ant follows two rules, wanders for about 10,000 steps, then builds a highway. |

### Brains, machines and creatures

| Demo | What it shows |
|---|---|
| [Brian's Brain](brians-brain/) | Cells fire, rest and recover like neurons; almost every pattern sets off moving. |
| [Wireworld](wireworld/) | Draw wires and watch electrons run through them: diodes, clocks and logic gates. |
| [Critters](critters/) | Blocks of four cells change by a fixed table, and the world can run backwards to its exact start; gliders, billiard balls and a gas. |
| [Cyclic cellular automaton](cyclic/) | Each colour can only advance to the next one round a colour wheel; random noise organises itself into turning spirals. |
| [Hodgepodge machine](hodgepodge/) | Cells fall sick, infect their neighbours and recover; from random noise come waves and turning spirals, like those of an oscillating chemical reaction. |
| [Langton's loops](langtons-loops/) | A loop of signals builds a copy of itself, and the copies go on copying into a growing colony. |
| [Lenia](lenia/) | Cells hold any value from 0 to 1 and grow by a smooth ring around them; gliding, turning creatures appear that look alive. |
| [Traffic](traffic/) | Cars on a ring road speed up, brake and dawdle at random, and stop-and-go jams appear from nowhere and drift backwards. |

### Physics on a grid

| Demo | What it shows |
|---|---|
| [Abelian sandpile](sandpile/) | Grains dropped on one cell topple onto their neighbours, on a square or hex grid, and settle into a fractal. |
| [Forest fire](forest-fire/) | Trees grow, lightning strikes now and then, and fires of every size sweep through the forest. |
| [Snowflakes](snowflakes/) | Water freezes where the vapour reaches it first, and a seed grows into a snow crystal: plates, stars and branching dendrites. |
| [Diffusion-limited aggregation](aggregation/) | Particles wander at random until they touch the cluster and stick; it grows into a branching fractal, like copper on an electrode. |
| [Lattice gas](lattice-gas/) | Particles hop and collide on a hexagonal lattice; averaged over many, they flow like a fluid around an obstacle. |
| [Falling sand](falling-sand/) | Paint sand, water, walls, plants and fire: sand piles up, water finds its level, plants drink and fire burns. |

### Relatives

| Demo | What it shows |
|---|---|
| [Percolation](percolation/) | Not a cellular automaton but chance: each cell is open with chance p, and at a critical p a cluster of open cells first reaches across the grid. |
| [Reaction-diffusion](reaction-diffusion/) | Not a cellular automaton but chemistry: two chemicals react and spread; by how fast one is fed and the other removed, they form spots that divide, stripes, waves or chaos. |
| [The Mandelbrot set](mandelbrot/) | Not a cellular automaton but a fractal: repeat z → z² + c and see which points stay near 0, step by step, zooming in. |

Each demo's own `README.md` holds its explanation and history; the app shows the same
text in a side panel.

For how it all fits together (the shared library, the frame loop, WebAssembly, and why
Rust), see [HOW-IT-WORKS.md](HOW-IT-WORKS.md).

## Running

You need Rust (stable) and, for the browser, the `wasm32-unknown-unknown` target and
[trunk](https://trunkrs.dev/). `make setup` installs both.

```bash
make                         # list every target
make run DEMO=life           # one demo in a native window
make web DEMO=life           # one demo in the browser
make serve                   # every demo, with an index page, at http://localhost:8080/
make check                   # formatting, lint and tests
```

## Layout

In brief; [HOW-IT-WORKS.md](HOW-IT-WORKS.md) has the full picture.

- `ca_core/`: shared code: the grid, drawing it, the controls, and the native/web entry point.
- One directory per demo. In each, `src/sim.rs` is the automaton, tested natively, and
  `src/main.rs` is the user interface (egui).
- `site/`: builds every demo plus the index page into `dist/`.

## Licence

Copyright © 2026 David Goldfarb, deg@degel.com.

MIT licensed. See [LICENSE](LICENSE).

The Lenia demo's creature patterns (in `lenia/src/sim.rs` and `site/thumbs.js`) come from
Bert Chan's Lenia repository, © 2018 Bert Chan, under the MIT licence; its full notice is at
the end of [lenia/README.md](lenia/README.md).
