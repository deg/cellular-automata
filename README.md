# Cellular automata

Demos of famous cellular automata, written in Rust. Each runs in a native window or in
the browser, and explains itself: the rule, its history, and things to try.

**Try them in your browser: <https://deg.github.io/cellular-automata/>**

| Demo | What it shows |
|---|---|
| [Abelian sandpile](sandpile/) | Grains dropped on one cell topple onto their 4 or 8 neighbours and settle into a fractal. |
| [Conway's Game of Life](life/) | Cells live or die by counting their neighbours; gliders, guns and other rules to try. |
| [Elementary cellular automata](elementary/) | All 256 one-dimensional rules, from the chaos of Rule 30 to the computer inside Rule 110. |
| [Langton's ant](langtons-ant/) | An ant follows two rules, wanders for about 10,000 steps, then builds a highway. |
| [Brian's Brain](brians-brain/) | Cells fire, rest and recover like neurons; almost every pattern sets off moving. |
| [Wireworld](wireworld/) | Draw wires and watch electrons run through them: diodes, clocks and logic gates. |

Each demo's own `README.md` holds its explanation and history; the app shows the same
text in a side panel.

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

- `ca_core/`: shared code: the grid, drawing it, the controls, and the native/web entry point.
- One directory per demo. In each, `src/sim.rs` is the automaton, tested natively, and
  `src/main.rs` is the user interface (egui).
- `site/`: builds every demo plus the index page into `dist/`.

## License

MIT. See [LICENSE](LICENSE).
