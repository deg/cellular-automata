# How it works

The eleven demos are separate programs built on one shared library, all written in Rust.
Each runs in two places from the same source code: as a native desktop window
(`make run DEMO=life`), and in a web page, compiled to WebAssembly
(`make web DEMO=life`, or the live site at <https://automata.degel.com/>). This page
explains how that fits together, and why it is built this way.

## The big picture

```
  one demo (e.g. life/)                          ca_core/  (shared by all eleven)
  ---------------------                          ---------------------------------
  src/sim.rs   the automaton itself:            GridView   draws a grid, maps clicks
               cells, rules, one step.           Playback   Run / Pause / Step / Restart
               Pure Rust, no UI, tested.         About      renders the demo's README
                                                 Pace, Rng, line, moore_counts, ...
  src/main.rs  the controls and the frame        run()      opens a window or starts in
               loop, built from ca_core's                   the page
               pieces.
  README.md    the explanation and history,
               shown in the app too.
  index.html   the web page trunk builds.
                           |
                           v
         eframe + egui  (a Rust UI library: buttons, sliders, panels, textures)
                 |                                    |
                 v                                    v
     native: a window (winit),            web: a <canvas> in the page,
     drawn with OpenGL                     drawn with WebGL, running as WebAssembly
```

The rule throughout is to keep **the automaton apart from the screen**. Each demo's
`sim.rs` knows nothing about windows, browsers or pixels: it holds the cells and knows
how to advance them a step. That makes it easy to test (`make test` runs the whole
suite in seconds, with no window or browser) and easy to reason about. The UI
in `main.rs` only reads the simulation's state and calls its methods.

## Why Rust

- **Speed.** Cellular automata are pure number-crunching: millions of cells, updated
  thousands of times. Rust compiles to machine code, with speed comparable to C and no
  garbage collector pausing the animation. The sandpile shows what that buys: a million
  grains dropped on one cell settle in about 3 seconds natively (see the case study
  below). An interpreted language would typically be many times slower.
- **One codebase, desktop and browser.** Rust compiles to WebAssembly as easily as to a
  native program, and the UI library used here (egui) runs in both. There is no
  JavaScript version to keep in step with a desktop version: the same `sim.rs` and
  `main.rs` run in both.
- **Safety without a garbage collector.** Rust checks at compile time that memory is
  used correctly: no dangling pointers, no data races. At run time it checks every
  array access, so nothing is read past the end unnoticed. Bugs that would silently
  corrupt a C program become compile errors, or a clean crash with a message, in Rust.
- **Overflow is caught, not ignored.** Integer overflow stops a debug build with an
  error, and checked arithmetic is easy to write. The sandpile caps the pile's total
  grains so that no cell's 32-bit count can overflow, and its tests check that cap.
- **Good tooling.** `cargo` builds, tests and fetches libraries; `clippy` lints; `rustfmt`
  formats. `make check` runs all three.

The costs are real too. Rust is harder to learn than most languages, compiling takes
longer, and each demo's WebAssembly file is about 4.5 MB, so the first load takes a moment.

## What WebAssembly is

**WebAssembly** (WASM) is a compact binary instruction format that every modern browser
can run, alongside JavaScript but much closer to the speed of native code. Compilers for
languages like Rust, C and C++ can target it, so programs written in them can run in a
web page, safely sandboxed like any other page content.

Here is how a demo gets from Rust to the browser:

1. **Compile.** `cargo` builds the demo for the target `wasm32-unknown-unknown`: a 32-bit
   WebAssembly machine with no operating system underneath. The result is a `.wasm` file.
2. **Glue.** WebAssembly can't touch the page directly; it calls out to JavaScript for
   that. The `wasm-bindgen` tool generates the small JavaScript file that loads the
   `.wasm`, and passes calls between the two: the canvas, keyboard and mouse events, WebGL
   drawing.
3. **Bundle.** [trunk](https://trunkrs.dev/) runs those steps, optimises the `.wasm` with
   `wasm-opt`, and writes the demo's `index.html`, `.js` and `.wasm` into `dist/`. It is
   what `make web` and `make site` call.
4. **Run.** The page holds a single `<canvas>`. When the `.wasm` loads, `ca_core::run()`
   finds the canvas and starts egui in it. egui draws everything, buttons and grid alike,
   with WebGL, just as it draws with OpenGL in the native window.

A few things are different inside the browser, and the code allows for them:

- **No threads here.** Code in a page runs on one thread unless set up specially, so the
  simulations are written to do a bounded amount of work per frame and keep the page
  responsive (see "The frame loop").
- **SIMD must be switched on.** WebAssembly can process several numbers in one
  instruction (SIMD), but only if asked: `.cargo/config.toml` enables it for the browser
  build. The sandpile's fastest stage depends on it.
- **Some standard features are missing.** For example, `std::time::Instant` (a clock)
  isn't available, so frame timing uses the time egui reports for each frame.
- **Texture size limits.** Browsers limit how large a texture can be, and the limit varies;
  4,096 pixels is a safe ceiling, so grids are capped at sizes whose picture fits within it.

## egui and eframe

The user interface is built with [egui](https://github.com/emilk/egui), a GUI library
written in Rust by Emil Ernerfeldt, together with **eframe**, its companion framework.

- **egui** draws the interface: panels, buttons, sliders, drop-down menus, text boxes,
  scroll areas, hover tips and images (textures). It is pure Rust, with no JavaScript or
  HTML involved, even in the browser.
- **eframe** gives egui somewhere to draw. It opens the native window or takes over the
  web page's canvas, runs the event loop and renders egui's output with a graphics
  backend; this project uses *glow*, which speaks OpenGL natively and WebGL in the
  browser. Because eframe hides that difference, the same `App::ui` function serves
  both. `ca_core::run()` is the one place that sets eframe up, either way.

egui is an **immediate-mode** UI. There are no button objects kept around, and no
callbacks. Instead, every frame, the program describes the whole interface again, and
each call reports what happened to it at once (simplified from `ca_core/src/playback.rs`):

```rust
if ui.button("Restart").clicked() {
    self.restart_requested = true;
}
ui.add(egui::Slider::new(&mut self.speed, 1..=1_000).text("generations / second"));
```

The state lives only in the demo's own structs (`Playback`, the simulation), and the
interface is redrawn from them every frame, so the screen can never fall out of step with
the model. The cost is redrawing the interface every frame, which is small next to the
simulation, and egui sleeps while nothing changes.

The grid itself is not made of egui widgets. It is one picture (a texture), which egui
scales and places. Anything drawn over it, such as the sandpile's drip-point rings, the
ants and the elementary rule table, is drawn with egui's `Painter`, its low-level drawing
API. The in-app explanations use `egui_commonmark`, a Markdown renderer built on egui.

Why egui rather than a web framework: it keeps the whole project in one language, and the
same code runs natively and in the browser. It suits controls like these: sliders,
buttons, number fields. The trade-off is looks. An egui app looks like a tool rather than
a styled web page, and doesn't use the browser's own text and form controls.

egui changes its API between releases. This project uses 0.37, where `App::ui` receives a
`Ui` (older code used `update` with a `Context`), and panels are `egui::Panel::left(..)`.
Examples written for older versions often won't compile as they are.

## The frame loop

Following the immediate-mode pattern, each demo's `main.rs` describes and runs one frame
at a time, in `App::ui`:

1. **Controls.** Draw the left panel (`ca_core::controls_panel`): buttons, sliders, stats.
   A click is noticed here and recorded (say, "Step was pressed").
2. **Advance.** Ask `Playback` what to do: nothing, one step, or run. When running, the
   speed slider sets how much work to do. Slow automata use `Pace` to turn "15
   generations a second" into whole steps per frame, carrying the fraction over. Heavy
   ones cap the work per frame by cells processed, so a big world slows down rather than
   freezing the page.
3. **Draw.** If anything changed, turn the grid into a picture, one pixel per cell, and
   upload it as a single texture (`GridView::set_image`). Drawing one texture is far
   cheaper than drawing thousands of squares. It is shown with nearest-neighbour scaling,
   so cells stay crisp at any zoom.
4. **Interact.** `GridView::show` places the texture in the window and maps the pointer
   back to a cell, giving the cell hovered, clicked or dragged over.
5. **Repaint.** While running, ask egui for another frame straight away. While paused,
   egui sleeps until the next mouse or keyboard event.

The hex sandpile uses the same machinery with a different `Layout`. Its cells are drawn
as offset bricks, two pixels wide, with rows squashed to the spacing of real hexagons, and
the same layout maps clicks back to cells.

The Mandelbrot set is the exception. It is not a cellular automaton, and its picture is a
window on the plane that zooms and pans, not a grid of cells, so it draws with a view of its
own (`mandelbrot/src/view.rs`) that maps the screen to the plane exactly, both ways. It
still runs like the others: each pixel keeps its state (its z and its step count), so
raising the iteration limit carries on rather than starting again, and the work per frame
is sized from the speed measured as it goes, since the browser and native builds run at
different speeds. A new view is filled in coarse pixels first, with the old
view drawn scaled underneath until it is.

Zoomed past about 3 × 10^9, a few hundred times short of where f64 can no longer hold
each pixel's point, a deep view switches to *perturbation* (`sim::Reference`): one reference orbit is computed in double-double
arithmetic (`mandelbrot/src/dd.rs`, two f64s, about 32 digits), and every pixel iterates
only its small difference from it, in f64, rebasing onto the start of the reference when
its orbit drifts away. Shallow views keep the plain loop, unchanged; a deep pixel costs
about 1.7 times as much a step. The tests check deep pictures against the plain loop run
entirely in double-double, pixel by pixel.

Points inside the set never escape, so left alone each would run to the stop limit, and
near the set's edge that is most of the work. Instead each pixel saves its z at steps 1, 2,
4, 8, ... and stops once a later z comes back within a hundredth of a pixel of it: it has
settled into a cycle (on a view all inside, at x8,103 and 5,000 steps, 13 s became 0.04 s
natively). Deep pixels can only do this when the reference orbit cycles too, comparing
their small differences where they line up with it. The tests run every stopped pixel on,
in the plain loop, to many times the limit, and check that none escapes.

## Explanations inside the app

Each demo's `README.md` is compiled into the program with `include_str!` and rendered by
`ca_core::About`, using the `egui_commonmark` library. So the text you read in the app's
side panel and the one on GitHub are the same file, and can't drift apart.

## Building and publishing the site

- `make site` runs `site/build.sh`. For each top-level directory holding a `Cargo.toml`
  and an `index.html`, it runs trunk with relative paths (`--public-url ./`) into
  `dist/<demo>/`. It then writes `dist/index.html`, one card per demo, taking the title
  from the demo's page title and the text from its `Cargo.toml`. A new demo appears
  without touching the script.
- Each card's picture is drawn when the page loads, by a few lines of JavaScript in
  `site/thumbs.js`: a tiny run of that automaton in its own colours. There are no image
  files to keep up to date.
- This page is `HOW-IT-WORKS.md`, turned into HTML at build time by `tools/mdpage`, a
  small Rust program using the `pulldown-cmark` Markdown parser.
- GitHub Pages lets browsers keep any file for 10 minutes, so a deploy must not depend on
  browsers fetching everything afresh. Each demo's code files already have names that
  change with their contents (trunk adds a hash). The shared `style.css` and `thumbs.js`
  are loaded by addresses carrying a fingerprint of their contents, so a new page can't be
  paired with an old cached copy. And every demo page starts with a small script
  (`site/stale-page.html`) that reloads the page once if its code files are missing: what
  happens to a page cached from before a deploy, which deleted them.
- `make serve` builds the site and serves it locally at <http://localhost:8080/>.
- On every push to `main`, a GitHub Actions workflow (`.github/workflows/pages.yml`)
  runs the same `site/build.sh` and publishes `dist/` to GitHub Pages, served at
  <https://automata.degel.com/>. Everything is static files, so no server is needed.

## How the simulations are tested

The tests don't check that the code does what the code does. They check that it does
what the automaton is **known** to do:

- **Famous patterns.** In Life, the diehard pattern vanishes after exactly 130
  generations, and the glider gun fires a glider every 30.
- **Hand-worked steps.** The ant's first moves, Rule 30's first rows, the Brian's Brain
  glider.
- **Mathematics.** Rule 90 must reproduce Pascal's triangle modulo 2. The sandpile must
  conserve grains, keep its symmetries, and give the same pile whatever order cells
  topple in. The Mandelbrot set is its own mirror image, and the Julia set of 0 is the unit
  disc.
- **Fast against slow.** Each of the sandpile's fast paths is compared, cell for cell,
  with the plain method. A Mandelbrot picture computed a step at a time, in uneven slices,
  or panned partway, must match one computed in one go, bit for bit.
- **Designed by simulation.** Wireworld's diodes and clocks were found by simulating
  candidate shapes and keeping the ones that worked. The tests pin down that they still do.

Slow checks (`make slow`) and timings (`make timing`) are kept out of the everyday test
run.

## Case study: making the sandpile fast

Settling a million grains takes tens of billions of single topplings, and no choice of
order changes that number. So speed has to come from doing each one cheaply,
or many at once. In order:

1. **Sweeps.** Topple cells in passes across the grid, in place, and let a cell holding
   12 grains topple 3 times at once: 2 to 3 times faster than keeping a list of unstable
   cells. A million grains took 102 seconds.
2. **Symmetry.** A pile dropped on one cell is identical when mirrored across either axis
   or diagonal. So compute one eighth of it and mirror the rest: 13 seconds.
3. **Simultaneous updates in 16 bits.** In an in-place sweep, each cell feeds the next one,
   which is read straight back. That forms a chain the processor can't work ahead of: 2.5
   nanoseconds a cell. Computing every cell's next value from the previous values, into a
   second buffer (a *Jacobi* sweep), breaks the chain. With 16-bit cells the compiler
   then processes many cells per instruction (SIMD): 0.10 nanoseconds a cell. It takes
   twice as many sweeps, but a million grains settle in **3.3 seconds**.

Two ideas that sounded good were measured and dropped: skipping quiet regions (about half
the pile topples on every sweep, so there is little to skip), and removing branches from
the inner loop (slower).

Symmetry only helps while every grain has gone to the centre of a growing grid with 4 or 8
neighbours. Any other pile (a grain dropped elsewhere, a hex grid, a sink edge) gets
step 3 alone, over a rectangle holding every grain. On a hex grid each cell's share of
grains is worked out once rather than once per neighbour, because dividing by 6, unlike
by 4 or 8, costs a multiplication. A million grains, measured on one day on one machine:

| Pile | Plain | Fast |
|---|---|---|
| Off the centre | 132 s | 24 s |
| Hex grid | 65 s | 17 s |
| Sink edge, 501 cells wide | 76 s | 12 s |

The centred pile took 6.7 s that day, so this machine ran about twice as slow as for the
3.3 seconds above.

## Adding a demo

1. Make a directory with a `Cargo.toml` (whose `description` becomes the index blurb)
   depending on `ca_core`, and add it to the workspace `members` in the root `Cargo.toml`.
2. Write `src/sim.rs`, the automaton, with tests of known behaviour.
3. Write `src/main.rs`, using `ca_core::run`, `controls_panel`, `Playback`, `GridView`
   and `About`. Copy an existing demo, such as `brians-brain/src/main.rs`, as a start.
4. Copy another demo's `index.html` and change its title and `data-bin`. Keep its
   `data-initializer="../site/loading.mjs"`, which shows the download's progress.
5. Write `README.md`: the rule, its history (with checked sources), things to try.
6. Optionally, add a function to `site/thumbs.js` that draws its card's picture; otherwise
   it gets a generic one.
7. `make check`, then `make serve`. It appears on the index page by itself, and on the
   live site after the next push.

## Credits

Developed with:

- [Claude Code](https://github.com/anthropics/claude-code), Anthropic's AI coding agent,
  which wrote much of the code and documentation under David's direction.
- [Beads](https://github.com/gastownhall/beads), an issue tracker built for coding
  agents. Every task in this project was a bead: filed, worked and closed.
- [Beads Utils](https://github.com/deg/beads-utils), David's tools for managing agentic
  work with Beads and Claude Code: including the finalize and close-out routines
  this project was run with.

© 2026 David Goldfarb, deg@degel.com.
