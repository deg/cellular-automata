# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:1105d646 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/core-concepts/sync-concepts.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->


## Build & Test

All Rust, in one Cargo workspace rooted here. Each demo runs natively or in the browser.
Everything goes through the Makefile; `make` alone lists the targets.

```bash
make check                  # before every commit: fmt-check, clippy -D warnings, tests
make slow                   # slow checks the READMEs quote (release build, a few minutes)
make run DEMO=sandpile      # a demo in a native window
make web DEMO=sandpile      # the same demo alone in the browser (trunk serve)
make serve                  # every demo plus an index page, at localhost:8080
make siblings               # before/after a Rust update: build+test David's other Rust projects
```

- `make site` (`site/build.sh`) builds every demo into `dist/<demo>/` and writes
  `dist/index.html` from `site/index.html`. A demo is any top-level directory with both
  `Cargo.toml` and `index.html`. Its link text is its page `<title>`, and its blurb is
  its Cargo `description`, so a new demo needs no edit to the site or the Makefile.
- A new command worth repeating gets a Makefile target with a `## description`, so
  `make help` lists it.
- Use `--release` for anything but small grids: debug builds simulate far slower,
  in wasm even more so.
- Toolchain: stable rustc (1.100 as of 2026-11-15) with the `wasm32-unknown-unknown`
  target, and `trunk` 0.22 (`cargo install trunk --locked`). Rustc 1.95.0 stays installed
  as a fallback (`rustup default 1.95.0`). Before updating Rust again, check that David's
  other Rust projects still build and pass their tests: `../CPeMulator`, `../nscheme`
  and `../llm_wiki/src-tauri`.
- `trunk` downloads `wasm-bindgen` and `wasm-opt` on its first build.

## Deployment

- The repo is public: https://github.com/deg/cellular-automata. Its beads sync there too
  (`refs/dolt/data`), so they are public: keep them to engineering.
- Every push to `main` rebuilds the site and deploys it to GitHub Pages at
  https://automata.degel.com/, via `.github/workflows/pages.yml` (it runs `site/build.sh`).
  A push therefore redeploys a public site: push only with David's go-ahead.
- `automata.degel.com` is a CNAME to `deg.github.io` at GoDaddy, and `degel.com` is verified
  on the `deg` GitHub account, which stops anyone else claiming the name. How it was set up,
  and how to add another site, is in `../degel-ai-server/GITHUB_PAGES.md`.

## Architecture Overview

- `ca_core/`: the shared crate, package `ca_core`. Don't name it `core`, which clashes
  with Rust's built-in crate. It re-exports `eframe` and `egui`, so demos don't depend on
  them directly. What it holds:
  - `run(title, make_app)`: opens a native window, or starts in the page's
    `<canvas id="the_canvas_id">`.
  - `Grid<T>`, plus `line` (the cells between two points, for gap-free drag painting)
    and `moore_counts` (each cell's count of 8-neighbours in some state, with edges
    handled once; Life, Brian's Brain and Wireworld use it).
  - `GridView`: draws a grid as one texture. Its `GridResponse` gives `hovered`,
    `pressed` (every frame held, for painting), `clicked` (once per click), and
    `cell_rect`/`cell_size` for drawing markers over cells. Both directions of the
    mapping go through its `Layout` (square, or hex as offset bricks), defined as plain
    functions with round-trip tests; `Layout::image` builds the picture without a window.
  - `Playback`: Run, Pause, Step, Restart and a speed slider. Each frame,
    `take_restart()`, then `advance()` returns `Idle`, `Step` or `Run(speed)`.
  - `Pace`: turns steps per second into steps per frame, carrying the fraction.
    `generations_per_frame` caps the work per frame by world area, so big worlds slow
    down rather than freeze.
  - `Rng`: a small seeded generator for random fills.
  - `About`: the side panel rendering the demo's `README.md`.
  - `controls_panel`: the scrolling left panel each demo puts its controls in.
  - `PaintStroke`: turns `GridResponse::pressed` into solid lines of cells to paint.
- One directory per automaton (`sandpile`, `life`, `elementary`, `langtons-ant`,
  `brians-brain`, `wireworld`), each a binary crate with an `index.html` for trunk:
  - `main.rs` holds the UI.
  - `sim.rs` holds the rule, kept free of UI code so it can be tested natively.
- The UI is egui/eframe 0.37, with the `glow` backend. In 0.37, `App::ui` takes a
  `&mut Ui` (not `update(ctx)`), and panels are `egui::Panel::left(id).show(ui, ..)`.
  `Ui::id` and `Id::new` are deprecated: use `ui.make_persistent_id(..)`. Code from
  memory or older examples uses the old API.

## Conventions & Patterns

- Keep the simulation in `sim.rs`, separate from the UI, and test its invariants there.
- Move code into `ca_core` only once a second demo needs it.
- `HOW-IT-WORKS.md` explains the architecture and quotes timings and `ca_core` APIs:
  update it when those change, or it drifts from the code.
- Every demo has a `README.md` for a reader who has never heard of the automaton: the
  rule, where the name comes from, its history (who, when, why it mattered, with
  citations), things to try in the demo, and further reading. `main.rs` passes it to
  `ca_core::About` with `include_str!`, so the same text is shown inside the app.
- Check facts and citations in a README before writing them; don't fill gaps from memory.
- `.cargo/config.toml` turns on WebAssembly SIMD for the browser builds; the sandpile's
  16-bit Jacobi stage (`sandpile/src/jacobi.rs`) relies on it to be fast there.
- The sandpile has three sweeps (plain, symmetric octant, 16-bit Jacobi on a quarter).
  Any change to them must keep the `without_symmetry()` equivalence tests in
  `sandpile/src/sim.rs` passing: they compare the fast paths with the plain sweep exactly.
- Test a demo's simulation by behaviour with known answers where possible: famous
  patterns doing what they are known to do (the diehard dying at 130), hand traces of the
  first steps, or mathematical identities (Rule 90 against Pascal's triangle mod 2).
  Design patterns (Wireworld circuits) by simulating candidates, not from memory.
