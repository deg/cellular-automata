# Simple rules, strange worlds

**Cellular automata and fractals.** Demos of famous simple rules, written in Rust: ten
cellular automata and the Mandelbrot set. Each runs in a native window or in the browser,
and explains itself: the rule, its history, and things to try.

**Try them in your browser: <https://automata.degel.com/>**

| Demo | What it shows |
|---|---|
| [Abelian sandpile](sandpile/) | Grains dropped on one cell topple onto their neighbours, on a square or hex grid, and settle into a fractal. |
| [Conway's Game of Life](life/) | Cells live or die by counting their neighbours; gliders, guns and other rules to try. |
| [Elementary cellular automata](elementary/) | All 256 one-dimensional rules, from the chaos of Rule 30 to the computer inside Rule 110. |
| [Langton's ant](langtons-ant/) | An ant follows two rules, wanders for about 10,000 steps, then builds a highway. |
| [Brian's Brain](brians-brain/) | Cells fire, rest and recover like neurons; almost every pattern sets off moving. |
| [Wireworld](wireworld/) | Draw wires and watch electrons run through them: diodes, clocks and logic gates. |
| [Cyclic cellular automaton](cyclic/) | Each colour can only advance to the next one round a colour wheel; random noise organises itself into turning spirals. |
| [Snowflakes](snowflakes/) | Water freezes where the vapour reaches it first, and a seed grows into a snow crystal: plates, stars and branching dendrites. |
| [Langton's loops](langtons-loops/) | A loop of signals builds a copy of itself, and the copies go on copying into a growing colony. |
| [Lattice gas](lattice-gas/) | Particles hop and collide on a hexagonal lattice; averaged over many, they flow like a fluid around an obstacle. |
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

## License

Copyright © 2026 David Goldfarb, deg@degel.com.

MIT licensed. See [LICENSE](LICENSE).
