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

```bash
cargo test --workspace                       # all tests, native, no browser
cargo clippy --workspace --all-targets       # lint; keep it warning-free
cargo run --release -p sandpile              # a demo in a native window
cd sandpile && trunk serve --release --open  # the same demo in the browser
```

- Use `--release` for anything but small grids: debug builds simulate far slower,
  in wasm even more so.
- Toolchain: rustc 1.95 with the `wasm32-unknown-unknown` target, and `trunk` 0.21
  (`cargo install trunk --version '^0.21' --locked`). trunk 0.22 needs rustc 1.96.1 or later.
- `trunk` downloads `wasm-bindgen` and `wasm-opt` on its first build.

## Architecture Overview

- `ca_core/`: the shared crate, package `ca_core`. Don't name it `core`, which clashes
  with Rust's built-in crate. It holds `Grid<T>`, `GridView` (draws a grid as one texture
  and reports the cell under the pointer), `Playback` (run, pause, step and speed), and
  `run(title, make_app)`, which opens a native window or starts in the page's
  `<canvas id="the_canvas_id">`. It re-exports `eframe` and `egui`, so demos don't
  depend on them directly.
- One directory per automaton, each a binary crate with an `index.html` for trunk:
  - `main.rs` holds the UI.
  - `sim.rs` holds the rule, kept free of UI code so it can be tested natively.
- The UI is egui/eframe 0.37, with the `glow` backend. In 0.37, `App::ui` takes a
  `&mut Ui` (not `update(ctx)`), and panels are `egui::Panel::left(id).show(ui, ..)`.
  Code from memory or older examples uses the old API.

## Conventions & Patterns

- Keep the simulation in `sim.rs`, separate from the UI, and test its invariants there.
- Move code into `ca_core` only once a second demo needs it.
