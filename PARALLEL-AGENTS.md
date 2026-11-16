# Four demos built in parallel by agents

On 2026-11-16 four new demos were built at once by four Claude Code agents, each in its own
git worktree, and then reviewed, fixed and merged by a fifth session that coordinated them:
the cyclic cellular automaton, snowflakes (Reiter's model), Langton's loops and the FHP
lattice gas. This is an account of how that was organised, what went well, and what went
wrong, written so the method can be reused. The beads are cellular-automata-cnf (the
epic, Four new demos, built in parallel by agents) and its children .1 to .5.

## The shape of it

```
                      coordinator (one Claude Code session)
                plans, briefs, merges, writes shared files, reviews
       ┌───────────────┬───────────┴───────────┬───────────────┐
   builder         builder                 builder         builder
   cyclic          snowflakes              langtons-loops  lattice-gas
   worktree        worktree                worktree        worktree
   demo/cyclic     demo/snowflakes         demo/...        demo/lattice-gas
       │               │                       │               │
   reviewer        reviewer                reviewer        reviewer      (fresh agents,
       │               │                       │               │          read-only)
       └─── fixes go back to the SAME builder, which still has its context ───┘
                                    │
                         integration branch (worktree)
                  merge each demo, then the shared docs, then ca_core
                                    │
                         main (only with David's go-ahead)
```

Eight agents in all: four builders and four reviewers. The builders were resumed for their
fixes rather than replaced: a resumed agent keeps its whole context, so a fix costs
minutes, not a rebuild.

## Timeline

| Time | Event |
|---|---|
| about 16:10 | Another session found committing on `main`. Rule from David: don't disturb it; don't touch `main` or its checkout. |
| 16:10-16:27 | Five worktrees, build cache cloned into each, beads filed, plan reviewed. |
| 16:27 | Four builders launched in one message. |
| 18:11-18:27 | All four builders finish (104-120 minutes each, 99-143 tool calls). Each gets a reviewer as soon as it finishes. |
| 18:19 | First three branches merged onto `integration`, before their reviews, to find conflicts early. |
| 18:26-18:49 | Reviews come back (12-14 minutes each); fixes relayed to the builders and merged as they arrive. |
| 18:38 | Shared code moved into `ca_core`. |
| 18:49-18:55 | `main`'s one new commit merged in; `make check`, `make slow` and `make site` pass; reported as done. |
| after 18:55 | David: "'Choices' is not an excuse for bugs or inconsistencies." A second round (below). |
| 19:34-19:47 | Second-round fixes merged, plus a tooltip fix for three older demos and `main`'s two newer commits; all checks pass again. |
| after 19:47 | David asks for a consistency review across all eleven demos; two read-only reviewers compare them; David settles four conventions. |
| 20:32 | Shared readout, note, number format and colour added to `ca_core`; seven workers start. |
| 20:38-20:53 | The seven workers' branches rebased and merged one by one, with conventions settled on the way. |
| 20:54-22:32 | The README sweep; a pause when the session hit its usage limit. |
| 22:53 | The last outliers from a final read-only check fixed; ready to land on `main`. |

Times are from the merge commits of `integration` as it was first built, before it was
made linear (see "Merging"), except the first rows, which are approximate.

The building took about two hours of wall-clock time for all four. One after another it
would have taken roughly eight, though that is an estimate: nobody built them in sequence
to compare.

## Decisions, and why

### Worktrees, outside the repository

`git worktree add -b demo/<name> ../cellular-automata-wt/<name> <base>` gives each agent a
full checkout on its own branch, sharing one `.git`. Nothing an agent does in its directory
can touch another's files or the main checkout, where another session was working.

- **Outside the repo directory**, not in it: Claude Code's own `isolation: "worktree"`
  option creates worktrees inside the repository, which would have put new files in the
  checkout the other session was using.
- **All from one fixed commit** (71a2d46), so every branch has the same base and the
  ownership check below can diff against it.
- **An `integration` worktree** for merging. `main` was checked out in the other session's
  directory, and git won't check out one branch in two worktrees; merging there would also
  have mixed our work with its uncommitted edits. `integration` takes in `main` only at
  the end.

### Clone the build cache instead of rebuilding it

Each worktree has its own `target/` (separate targets let four `cargo` builds run at once;
a shared one would serialise them on its lock). Instead of four cold builds of eframe,
`cp -cR target <worktree>/target` made APFS copy-on-write clones: 5.2 GB "copied" four
times in 11 seconds, using no disk until a file changes. A test build in a worktree then
recompiled only the workspace's own crates (8 seconds).

### The trap: the shell's directory resets

In Claude Code the Bash tool's working directory returns to the session's start directory
after each command. A subagent that ran a bare `cargo build` or `git commit` would have run
it in the main checkout, and a commit would have landed on `main`. So the brief made it a
hard rule that every command begins `cd <worktree> &&`, every git command is
`git -C <worktree>`, and the agent's first act is to check `git branch --show-current`.
(The advisor review of the plan caught this before launch.)

### Shared state: beads and git hooks

The beads database is shared by every worktree (`bd where` from a worktree points at the
main checkout's `.beads/`), and so are the git hooks, which run `bd hooks run ...`.

- A throwaway commit on `integration` showed the hooks took 0.2 s and wrote nothing; it was
  then reset away. Builders still committed with `git -c core.hooksPath=/dev/null commit`,
  a per-command setting that changes no shared configuration.
- Builders and reviewers ran no `bd` commands at all; only the coordinator touched beads,
  and rarely.

### Who owns which file

Most conflicts in parallel work come from files everyone touches. Here those were:

| File | Owner | How |
|---|---|---|
| `<demo>/` | that demo's builder | anything |
| `Cargo.toml` (`members`) | builder adds its own name | coordinator resolves the conflict |
| `Cargo.lock` | cargo | coordinator takes `integration`'s side and lets cargo re-add the path crate |
| `site/thumbs.js` | builder adds one entry | coordinator keeps both sides |
| `README.md`, `CLAUDE.md`, `AGENTS.md`, `HOW-IT-WORKS.md` | coordinator only | written once, after merging |
| `ca_core/` | coordinator only | builders implement locally and report candidates |

The home page needed nothing: `site/build.sh` finds demos by themselves.

Before each merge a script (`check-branch.sh` below) lists every file the branch changed
outside its own directory and flags anything not on that list. All four branches were
clean every time. It is cheaper to check than to trust the report.

No new crates.io dependencies were allowed, which kept each branch's `Cargo.lock` change to
one path-crate entry.

### Briefs: one shared, one per demo, and no facts in them

A shared brief (below) held the rules every builder needed: where it may work, which files
it owns, how to commit, the project's conventions, the quality gates, and the shape of the
final report. Each agent's own prompt then gave only its worktree, branch, bead, port, and
the demo.

The demo part named **what to find and where to find it, not the answer**. No rule tables,
parameter values or published step counts went into a brief: an agent handed a number tends
to "confirm" it. Each builder had to find the primary source and make its tests check
against it. This paid off; see the reviews.

Other rules that mattered:

- **Stage risky work.** The lattice gas had to commit a tested particle-level version
  before its flow view, so there would be something to merge even if it ran long. (It
  finished all three stages.)
- **Build only your own page.** Each builder ran `trunk build` for its own demo and checked
  it headless on its own port (8771-8774). The full-site build ran once, on `integration`.
  Four full builds at once would have starved the machine: one builder still measured a
  load average of 34 with four agents and the other session running.
- **A fixed report format**: commits, test results, screenshots, every file touched outside
  the demo, wished-for `ca_core` changes, unsourced claims, design choices to revisit,
  doubts. That made each report quick to act on.

### Independent review, then fixes by the original builder

A builder that wrote the code is anchored on it. Each finished demo got a **fresh** agent,
read-only, given the code and a review brief (below), but not the builder's conclusions,
except where the builder had flagged an open puzzle. The reviewer's first job was to find
the primary source itself and compare the code with it.

Fixes went back to the **builder**, by resuming it with `SendMessage`: it still had its
context, so the fixes took between 30 seconds and 7 minutes.

What the reviews found:

- **Snowflakes**: the builder had matched Reiter's hand-worked figure and found its growth
  times within 1% of his tables after rescaling, but its "Sector" crystal grew a plain
  hexagon instead of his scalloped one, and it couldn't find out why. The reviewer ported
  Reiter's J program to Python and found the rule exact (agreement to 1e-15). The bug was
  the **unit of radius**: Reiter counts hex rows (√3/2 of a cell width), so every crystal
  was grown 15% too big, and at that size the Sector crystal's scallops fill in. With his
  unit, seven step counts match his tables exactly. The reviewer also showed by mutation
  that no test checked the diffusion constant α.
- **Langton's loops**: the rule checked three independent ways (the paper's text, Golly's
  rule tree for all 32,768 neighbourhoods, a 1999 applet). But changing one digit of a
  table entry first used at generation 879 passed every test; pinned cell counts at
  generations 151 and 1000, computed from Golly's tree, now catch it. Also two wrong dates,
  one number out, and a likely painting bug.
- **Cyclic**: rule, fast counting and pacing correct (three deliberate mutations all
  caught); every preset's parameters matched the source. The findings were all credits and
  dates: who published a rule first, a 1991 heading over 1989 work.
- **Lattice gas**: the reviewer downloaded both 1987 papers and found the physics right:
  collisions for all 64 states, the lattice, walls, the wind's inlet, the Reynolds number,
  the speed of sound. Everything it found was around the simulation: the index card's
  picture dropped half its samples (a random bit left unmasked) and took 121 ms on every
  load of the home page, against about 1-10 ms for each other card; drawing the flow cost
  more per frame than simulating it at the larger sizes; two parts of the rule were
  untested (mutations passed); a slider left the view blank while paused; and the default
  wind was faster, relative to the speed of sound, than the README said it must be.

**Mutation testing** (break the rule on purpose in a scratch copy; see whether a test fails)
was the reviewers' sharpest tool. Twice it showed that a test suite which looked thorough
could not catch a plausible bug.

**Reviewers can be wrong too.** One claimed a paper "cites itself" as 1989; the builder
checked, found that reference pointed to a different paper, and fixed the date on other
evidence. The builders were told to verify each finding before applying it. (The
lattice-gas builder checked all twelve of its findings and found every one held up, so
verifying is not the same as doubting.)

### Merging

While the work was under way, `integration` merged each demo with `--no-ff`, merged each
round of review fixes the same way, and merged `main` in twice. The first three demos
merged before their reviews were done, to find the conflicts early.

That made a history Magit draws as a tangle of crossing lines. David's rule, now in
`~/CLAUDE.md` for every project, is a *semi-linear* history: rebase a branch onto the tip
of `main`, then merge it with `--no-ff`, so each branch is one short second track beside
`main` and appears to start after the previous one finished; never merge `main` into a
branch. So `integration` was rebuilt at the end:

1. Backups first: `backup/before-linear/<branch>` for `integration` and each demo branch.
2. A fresh worktree on a new branch at `main`'s tip.
3. For each demo, in the order they first merged: rebase its branch, in its own worktree,
   onto the new branch's tip (`rebase-demo.sh` below), running `cargo check` on every
   rebased commit so the history can be bisected; then `git merge --no-ff` it.
4. The coordinator's own commits (shared docs, `ca_core`, write-up) cherry-picked on top,
   each checked to build.
5. Proof: the rebuilt tree differs from the old `integration` only by `main`'s newer
   commit, and `make check` passes.
6. The commit hashes quoted in beads were updated to the new commits.

Rebasing also meant resolving the same conflicts once per commit rather than once per
merge, which the script does by file type:

- `Cargo.toml` and `site/thumbs.js` conflicted for every branch: every branch changed the
  same `members` line and inserted its picture at the same place. Both are pure additions.
  A small script (`union.py` below) resolves a conflict by keeping both sides **only when
  the common base is empty**, and refuses otherwise. The `members` line is rebuilt in a
  fixed order.
- `Cargo.lock`: take the new base's side and let cargo add the new path crate back.

### The consistency round

After the second round David asked for a consistency review across all the demos, the
seven older ones included. It ran as its own small parallel effort:

1. **Two read-only reviewers**, each given every demo and a brief (below) asking for a
   table with one column per demo: one for the interface and behaviour (titles, panel
   order, start state, Restart, pointer, readout, notes, presets, labels, colours, fit on
   the laptop screen), one for the text (README structure, citation style, spelling,
   card blurbs, the index page, the project docs). They found about 25 outliers, and seven
   places where the older demos disagreed among themselves.
2. **David decided the four disagreements that mattered**, through one multiple-choice
   question each: Restart goes back to the opening (with a Clear button where you draw from
   scratch), the cell under the pointer is a readout row everywhere, the home page's cards
   follow a curated order, and presets are highlighted chips with their note below.
3. **Shared pieces first.** Before any worker started, the coordinator added the pieces
   every demo would need to `ca_core` (the readout grid with its pointer row, the speed-cap
   note, number grouping, the background colour), so the workers would all use one
   wording rather than eleven copies.
4. **Seven workers in parallel**, each owning whole demo directories: the four builders
   (resumed, for their own demos) and three new agents for the older demos. All shared a
   conventions brief (below). The coordinator did the shared files (card order, root
   README, HOW-IT-WORKS, CLAUDE.md) meanwhile.
5. **Questions settled as they came up**, and written into CLAUDE.md at once: a random
   opening is drawn afresh by Restart (one builder had added a button to replay the same
   one, which no other demo had); the speed-cap note names no number (one demo's budget was
   five times another's); dimensions like "1200 x 1040" aren't grouped with commas.
6. **Merged one by one**, each branch rebased onto `integration` with every commit checked,
   then `--no-ff`, as the semi-linear rule says.
7. **A final sweep** by one agent across all eleven READMEs for what no single-demo worker
   could see (one format for Further reading, citations verified against Crossref), then a
   fresh read-only check of the whole.

## What went wrong

- **A merge committed conflict markers.** On the lattice-gas merge, `Cargo.lock` conflicted
  for the first time, and a one-line command of mine committed the file with its markers in:
  `cargo check | tail` reported success because `tail` succeeded. It was caught at once and
  fixed by amending the merge. The fix was a merge script (`merge-demo.sh` below) with
  `set -euo pipefail`, a check of the branch's files first, conflict handling by file type,
  and a refusal to commit if any marker remains. Lesson: in a pipeline, check the exit
  status of the command that matters, not the last one.
- **House style drifted.** Two builders used Title Case for their page titles; the site uses
  sentence case ("Langton's ant"). The brief said to copy the style of other demos'
  descriptions but not their titles. A line in the brief would have prevented it; the
  coordinator caught it on the rendered index page.
- **"Choices" passed on unchecked.** Each builder ended its report with "design choices
  David might want to revisit", and I relayed those lists to David as they were. David's
  answer: "'Choices' is not an excuse for bugs or inconsistencies." Checked against the
  other demos, most were defects:
  - Two demos started paused where the project's own rule (elementary and Mandelbrot start
    running, each with a comment: a fixed start best seen emerging) says they should run.
  - The lattice gas's controls panel was the only one of eleven that scrolled at laptop
    size (found by rendering every demo's panel side by side), and its readout was prose
    where every other demo uses a label/value grid.
  - The cyclic "Turbulent phase" preset froze on the default world, on every one of 10
    seeds. No world size we offer fixed it, so the builder switched to the paper's other
    example of turbulence, and gave every preset a minimum world size checked by test.
  - A painting fix had never been tried; trying it by a headless drag showed it was not
    enough.
  - A misplaced tooltip had been noticed and left. Its cause, egui anchoring a tooltip to
    the widget rather than the pointer, turned out to affect three demos already on `main`
    (Life, Langton's ant, the sandpile); those were fixed too.

  One item I had called inconsistent (snowflakes starting to run) was in fact the house
  rule. The lesson: a builder's list of choices is the coordinator's review queue, to be
  checked against the existing code before anything is reported, not an item for the
  report.
- **Fixed item by item, then declared consistent.** After the second round I reported the
  listed items fixed, then found one more myself in the final side-by-side render (the
  snowflakes showed the pointer's value as a sentence, the other new demos as a grid row),
  and David asked for a consistency review across all the demos. The cause is structural:
  each builder and each reviewer saw one demo, so nobody's job was the differences
  *between* demos, and fixing the items someone happened to notice doesn't make the set
  consistent. Consistency needs its own review, by an agent that reads every demo side by
  side against a list of the conventions, as a planned step before reporting, not as a
  reaction to a complaint.
- **Times written from memory.** The first version of the timeline above gave times I had
  estimated as I went, and several were wrong by most of an hour. Recorded times (here,
  the merge commits) are the only ones to quote.
- **Shared code was planned but didn't fit.** The plan was to move the hex neighbour offsets
  into `ca_core` for the snowflakes and the lattice gas. The lattice gas used offset rows
  (as its source paper does), not axial coordinates, so only the sandpile and the
  snowflakes share it. Plans about shared code are guesses until the code exists.

## What to do the same way next time

1. Worktrees outside the repo, one per agent, from one base commit, plus an integration
   worktree. Clone the build cache.
2. Every command in the worktree's directory, said as a hard rule; first action checks the
   branch.
3. A written table of which files each agent owns; the coordinator alone writes shared docs
   and shared code.
4. A shared brief plus a short per-agent part; no facts in briefs, only where to find them.
5. A fixed report format.
6. An ownership check by script before every merge.
7. A fresh, read-only reviewer per piece, told to check against the primary source and to
   use mutation testing; fixes back to the original builder, who verifies before applying.
8. Brief the house conventions explicitly (title case, start running or paused, the
   readout grid, panels that fit the laptop screen), and check each builder's "choices"
   against the other demos before reporting. Render every demo side by side to see
   inconsistencies.
9. A cross-demo consistency review as a planned step, after the per-demo reviews: a fresh
   agent that reads every demo against the conventions, with the coordinator fixing or
   dispatching what it finds.
10. Merge early, merge often, with a script that refuses to commit markers or a failing build;
    rebase each branch onto the tip first, so the history stays semi-linear.
11. One full build and one rendered check of the whole site at the end.

## Cost

Builders used about 240-350 thousand tokens each, reviewers about 135-185 thousand, and the
resumed builders' fixes about 14-47 thousand. That is several times what one agent building the demos in sequence
would spend, because each builder reads the project and its conventions afresh. What it
buys is wall-clock time (about two hours instead of an estimated eight) and the independent
reviews, which found real bugs the builders' own checks had passed (the snowflakes' radius
unit, the lattice-gas card's dropped samples, a blank view after a slider change, a painting
offset) and four gaps in the tests that mutation testing exposed.

## Appendix: the briefs and scripts

<details>
<summary>The shared builder brief</summary>

# Common brief for every demo builder

You are one of four agents, each building one new demo for David's cellular-automata project
(Rust, egui/eframe 0.37, native + WebAssembly; public site https://automata.degel.com/). A
coordinating session briefed you and will merge your branch. The other three agents are
building other demos in parallel, in their own worktrees. Your brief's demo-specific part names
your demo directory `<demo>`, your worktree `<wt>` and your bead id.

## Hard rules: where you may work (read twice)

Another Claude session, which does not know about you, is working in the main checkout
`/Users/deg/Documents/degel/cellular-automata` on branch `main`. Disturbing it is forbidden.

- **The Bash tool's working directory resets to that main checkout after every call.** So
  EVERY Bash command must start with `cd <wt> &&` (or be fully absolute), and every git command
  must be `git -C <wt> ...`. A bare `cargo build`, `make check` or `git commit` would run in the
  other session's checkout. Never do that.
- Every file path you Read/Edit/Write is absolute and under `<wt>`. The CLAUDE.md files loaded
  into your context quote paths in the main checkout; do not copy those paths.
- Your very first action: `cd <wt> && git branch --show-current` must print `demo/<demo>`.
  If it does not, stop and report.
- Never touch branch `main`, never push, never run `git worktree`, `git checkout <other
  branch>`, `git rebase`, or anything with `--force`.
- Do **not** run any `bd` (beads) command. The coordinator handles beads; the beads database
  is shared with the other session.
- Commit with the beads hooks off, so commits never touch the shared beads database:
  `git -C <wt> -c core.hooksPath=/dev/null commit -m "..."`.
  End each commit message with a blank line and
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Mention your bead id in the
  subject, e.g. `Cyclic automaton demo: the rule and its tests (cellular-automata-cnf.1)`.
  Commit at each finished, tested step (David's standing instruction for this project).
- Scratch files (screenshots, notes, probe scripts) go in
  `<scratchpad>/<demo>/`,
  never in the repo.

## Which files you may change

- Anything inside `<wt>/<demo>/` (your new demo).
- `<wt>/Cargo.toml`: add `"<demo>"` to the workspace `members` list (just before
  `"tools/mdpage"`). Nothing else in that file.
- `<wt>/Cargo.lock`: only the change cargo makes for your new path crate.
- `<wt>/site/thumbs.js`: add ONE entry to the `pictures` object for your demo's index card
  (keyed by the directory name), reusing the helpers and palette constants already there.
  Add constants you need inside your own function. Change nothing else in that file.

Do **not** change `ca_core/`, other demos, `README.md`, `CLAUDE.md`, `AGENTS.md`,
`HOW-IT-WORKS.md`, `Makefile`, `site/` (other than your thumbs.js entry), `tools/` or
`.github/`. The coordinator updates the shared docs and lists. If you want something in
`ca_core`, implement it locally in your demo and list it in your report as a candidate.

**No new crates.io dependencies.** Use what `ca_core` re-exports (eframe, egui) and the
standard library. If you think you truly need one, say so in the report instead.

## How to build a demo here

Read these first (all under `<wt>`):
- `CLAUDE.md` (project conventions: sim.rs vs main.rs, README requirements, testing by known
  answers, egui 0.37 API notes, `controls_panel` width rules).
- `HOW-IT-WORKS.md`, especially "Adding a demo" near the end, and the parts on the frame
  loop, `Pace` and `generations_per_frame`.
- `brians-brain/` (a complete small demo: `Cargo.toml`, `index.html`, `README.md`,
  `src/main.rs`, `src/sim.rs`) as your template, and `ca_core/src/*.rs` for the API
  (`run`, `Grid`, `GridView`/`Layout`/`GridResponse`, `Playback`, `Pace`,
  `generations_per_frame`, `Rng`, `About`, `controls_panel`, `PaintStroke`,
  `moore_counts`).
- Copy `brians-brain/index.html`, changing `<title>` and `data-bin`; keep
  `data-initializer="../site/loading.mjs"`.
- The index card blurb is your Cargo `description` (one sentence, in the style of the others:
  `grep -h ^description <wt>/*/Cargo.toml`). The card's link text is your page `<title>`.
- Use the project's palette (see the colour constants in other demos and `site/thumbs.js`:
  navy background 0x141830, teal, gold, coral, spark, blue) so the demo looks like a sibling.
- Release builds only for anything but tiny grids.

## The README

Every demo has a `README.md` for a reader who has never heard of the automaton: the rule,
where the name comes from, its history (who, when, why it mattered, **with citations**),
things to try in the demo, and further reading. `main.rs` passes it to `ca_core::About` with
`include_str!`.

**Check every fact and citation against a source before writing it; do not fill gaps from
memory.** WebSearch is a deferred tool: load it with ToolSearch (`select:WebSearch`) before
use. For pages, use the `web-fetch` agent type via the Agent tool if you have no WebFetch.
Anything you could not source: leave it out, or list it in your report. A separate reviewer
will check the README against sources.

## Tests

Test the simulation in `sim.rs` by behaviour with known answers: hand traces of the first
steps, invariants (conservation, symmetry), and published behaviour (with the source cited in
the test's doc comment). **Find the known answers in a source; do not take numbers from this
brief or from memory.** Put slow tests behind `#[ignore]` (they run in `make slow`).

## Quality gates before you report

1. `cd <wt> && make check` (fmt-check, clippy -D warnings, all workspace tests) passes.
2. Your demo runs in the browser. Build only your own page:
   `cd <wt>/<demo> && trunk build --release` (output in `<wt>/<demo>/dist/`; trunk reuses the
   cloned `target/`). Do NOT run `make site` or `site/build.sh` (the coordinator does that once,
   after merging; four parallel full-site builds would starve the machine).
   Make sure `<wt>/<demo>/dist/` is not committed (check `git -C <wt> status`); add a
   `<demo>/.gitignore` with `dist/` if needed.
3. Check it visually, headless (never screenshot the real screen). Serve your page on your
   assigned port and render it with Playwright's headless Chromium:
   ```
   cd <wt>/<demo> && python3 -m http.server --directory dist <port>   # run_in_background
   B=~/Library/Caches/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-mac-arm64/chrome-headless-shell
   $B --headless --hide-scrollbars --window-size=1236,840 --force-device-scale-factor=2 \
      --use-angle=swiftshader --enable-unsafe-swiftshader --virtual-time-budget=8000 \
      --screenshot=<scratch>/<demo>-1.png http://localhost:<port>/
   ```
   (Trunk's `index.html` may reference `../site/loading.mjs`; if the page fails to load because
   of that path, serve `<wt>` itself and open `/<demo>/dist/`, or copy `site/loading.mjs`
   into place in your scratch dir. Work it out; don't edit `site/`.) 1236 x 840 at 2x is
   David's laptop: the grid must not be squeezed by the side panels. To click or drag, use
   Playwright's Python API with `executable_path` set to that binary, launch args
   `--force-device-scale-factor=2 --window-size=1236,840`, and
   `browser.new_context(no_viewport=True)` (Playwright's own device_scale_factor breaks egui
   input). Headless frame rates are software-rendered: don't judge speed from them. Kill your
   http.server when done.
4. Also check your thumbs.js picture: render it in isolation (a tiny scratch HTML with a
   `<canvas data-demo="<demo>" width=80 height=50>` and a `<script src=<wt>/site/thumbs.js>`),
   and look at the screenshot.

## Scope

One solid, well-explained, well-tested demo, in the spirit of the existing ones. A few
presets and sliders, painting where it makes sense, Run/Pause/Step/Restart via `Playback`.
Don't gold-plate. If you hit a real design choice David should make, pick the sensible
default, note it in the report, and carry on.

## Your final report (your last message, which goes to the coordinator)

- Commits on your branch, with short hashes and subjects.
- `make check` result, and the names of your tests (which known answers they check, and
  sources).
- Screenshot paths (absolute).
- Every file you touched outside `<demo>/` (should be only Cargo.toml, Cargo.lock,
  site/thumbs.js).
- ca_core changes you wanted, with the reason.
- README claims you could not source, and design choices you made that David might want
  to revisit.
- Anything you are unsure works.

</details>

<details>
<summary>An example per-demo prompt (Langton's loops)</summary>

You are building a new demo for David's cellular-automata project.

FIRST read the common brief, which is part of these instructions and has the hard rules on where you may work:
<scratchpad>/briefs/common.md

Your values for that brief:
- `<demo>` = `langtons-loops`
- `<wt>` = `/Users/deg/Documents/degel/cellular-automata-wt/langtons-loops` (branch `demo/langtons-loops`)
- bead: `cellular-automata-cnf.3` (Langton's loops demo: a self-copying pattern)
- headless port: 8773
- scratch dir: <scratchpad>/langtons-loops/

## The demo

Christopher Langton's self-reproducing loop (Physica D, 1984): an 8-state automaton on the von Neumann neighbourhood (4 neighbours) in which a sheathed loop carrying a "genome" of signals builds an arm, copies itself, and the copies go on copying until a colony forms. It descends from von Neumann's self-reproducing automaton and Codd's simplification; Langton's point was that self-reproduction does not need universal construction.

The rule table is the whole automaton, so it must be exact:
- Transcribe the transition table and the initial loop from a named, citable source file. Langton's 1984 paper is the primary source; Golly's rule file for Langton's loops (`Langtons-Loops.rule` in the Golly distribution, and its pattern file for the initial loop) is a widely used machine-readable transcription. Find them, cite the exact source (URL, file, version) in a comment in the code and in the README, and check the license allows transcribing a table (a table of facts is generally fine; note what you found).
- Apply the table's rotational symmetry the way the source specifies, and test that the expanded table has no conflicting entries.
- Decide (from the source) what happens for neighbourhoods missing from the table, and say so in the code.

Known-answer tests (find the facts in sources, don't take them from memory): the loop really reproduces: after the number of generations the source states, a second complete loop exists, identical to the first up to rotation/translation; later generations keep making copies. If a source gives the period of reproduction or the size of the colony at some generation, test that too (slow tests behind #[ignore]).

What David should be able to do: start from one loop and watch the colony grow (it grows outward forever; give it room: big worlds, a sensible default speed, maybe a "fit to colony" view or several world sizes), Run/Pause/Step/Restart, speed, a generation counter. Colours for the 8 states harmonising with the project palette, chosen so the sheath, the core and the signals are easy to tell apart. Painting is optional (editing cells breaks loops in interesting ways, so it could be fun with a state picker). Further presets (e.g. Byl's loop, Chou-Reggia, Sayama's evoloops) are optional extras ONLY if their tables come from a citable source too; the README can mention them as further reading either way.

Follow the common brief's quality gates and finish with the report it specifies.

</details>

<details>
<summary>The reviewer brief</summary>

# Brief for a demo reviewer

You are reviewing one new demo for David's cellular-automata project (Rust, egui 0.37, native +
WebAssembly). Another agent built it on its own branch, in its own git worktree. You did not
write it; your job is an independent read, not a confirmation. Your brief's specific part names
`<demo>`, `<wt>` and the bead id.

## Hard rules

- **Read-only.** Do not edit, commit, or create files in the repo. Write notes and probe
  scripts only in
  `<scratchpad>/review-<demo>/`.
- The Bash tool's working directory resets to `/Users/deg/Documents/degel/cellular-automata`,
  the main checkout, where another session is working. Never run anything there. Start every
  Bash command with `cd <wt> &&`, use `git -C <wt>`, and use absolute paths under `<wt>`.
- No `bd` commands. No pushes, no checkouts, nothing touching `main`.
- You may run tests and builds in `<wt>`: `cd <wt> && cargo test --release -p <demo>`, and the
  slow ones with `-- --ignored`.

## What to check, most important first

1. **The rule is right.** Find the primary source for the automaton yourself (paper, or the
   source file the code cites) and compare the implementation with it line by line: the
   update rule, neighbourhood, boundary handling, symmetry expansion, parameter meanings, and
   preset values. Use WebSearch (deferred tool: load it with ToolSearch `select:WebSearch`)
   and the `web-fetch` agent type to read pages. A mismatch with the source is the most
   valuable finding you can make.
2. **The tests prove what they claim.** Does each known-answer test actually check the known
   answer, cite its source, and fail if the rule were subtly wrong? Try mutating the rule in a
   scratch copy (outside the repo, e.g. `git -C <wt> archive HEAD | tar -x -C <scratch>/copy`,
   then build there with its own `CARGO_TARGET_DIR` under your scratch dir) to see whether the
   tests catch it, if that is cheap.
3. **The README's facts.** Every date, name, attribution, title, journal, and claim about
   history or behaviour: check against a source. List each claim you could not confirm, and each
   that is wrong, with what the source says.
4. **Correctness bugs in the code**: off-by-ones at edges, hex/axial coordinate mistakes,
   painting mapped to the wrong cells, overflow, panics on resize, a slider that can reach a
   value that breaks the sim, UI rows that overflow the 290-point controls panel.
5. Project conventions (see `<wt>/CLAUDE.md`): sim.rs free of UI, release-speed, egui 0.37 API,
   `controls_panel` width rules, README structure.

Don't report style nits or matters of taste unless they hurt a reader.

## Report (your final message)

A ranked list of findings. For each: file:line, what is wrong, the evidence (quote the source,
or the failing scenario), and a suggested fix. Separate "confirmed" (you have evidence) from
"plausible" (you suspect). End with the README claims you checked and found correct, so the
coordinator knows what has been verified.

</details>

<details>
<summary>check-branch.sh: what did a branch change outside its own directory?</summary>

```bash
#!/bin/bash
# Before merging demo/<demo>: list files the branch changed outside what it may touch.
# Usage: check-branch.sh <demo>
set -u
demo=$1
base=71a2d46
repo=/Users/deg/Documents/degel/cellular-automata-wt/integration
echo "== demo/$demo: commits since $base"
git -C "$repo" log --oneline "$base..demo/$demo"
echo "== files outside $demo/ (allowed: Cargo.toml, Cargo.lock, site/thumbs.js)"
git -C "$repo" diff --name-only "$base...demo/$demo" | grep -v "^$demo/" | while read -r f; do
  case "$f" in
    Cargo.toml|Cargo.lock|site/thumbs.js) echo "  ok   $f" ;;
    *) echo "  BAD  $f" ;;
  esac
done
echo "== Cargo.toml diff"
git -C "$repo" diff "$base...demo/$demo" -- Cargo.toml
echo "== Cargo.lock: packages added or changed"
git -C "$repo" diff "$base...demo/$demo" -- Cargo.lock | grep -E '^[+-](name|version) ' || echo "  (none)"
```

</details>

<details>
<summary>union.py: resolve a conflict where both sides only added text</summary>

```python
# Resolve diff3 conflicts where both sides only ADDED text: keep ours, then theirs.
# Refuses if the base section is non-empty (a real edit on both sides needs a human look).
import sys
path = sys.argv[1]
out, state, base = [], None, []
for line in open(path):
    if line.startswith("<<<<<<< "): state = "ours"; continue
    if line.startswith("||||||| "): state = "base"; continue
    if line.startswith("=======") and state: state = "theirs"; continue
    if line.startswith(">>>>>>> "):
        if any(l.strip() for l in base): sys.exit(f"{path}: base not empty, resolve by hand")
        state, base = None, []; continue
    if state == "base": base.append(line); continue
    out.append(line)
open(path, "w").write("".join(out))
print("resolved", path)
```

</details>

<details>
<summary>merge-demo.sh: the safe merge, written after the Cargo.lock mistake</summary>

```bash
#!/bin/bash
# Merge demo/<demo> onto integration, safely: ownership check first, stop on any conflict
# that isn't a known-safe kind, and never commit with conflict markers or a failing build.
# Usage: merge-demo.sh <demo> "<merge message>"
set -euo pipefail
demo=$1; msg=$2
S=<scratchpad>  # where these scripts live
I=/Users/deg/Documents/degel/cellular-automata-wt/integration
cd "$I"
[ "$(git branch --show-current)" = integration ] || { echo "not on integration"; exit 1; }
if bash "$S/check-branch.sh" "$demo" | grep -q BAD; then echo "ownership check failed"; exit 1; fi
if ! git -c core.hooksPath=/dev/null merge --no-ff -q -m "$msg" "demo/$demo"; then
  for f in $(git diff --name-only --diff-filter=U); do
    case "$f" in
      site/thumbs.js) python3 "$S/union.py" "$f" ;;
      Cargo.lock) git show 'HEAD:Cargo.lock' > Cargo.lock ;;   # cargo re-adds path crates below
      *) echo "conflict in $f: resolve by hand"; exit 1 ;;
    esac
  done
fi
cargo check --workspace -q
node --check site/thumbs.js
if git grep -n -E '^(<<<<<<<|>>>>>>>) ' -- . ':!dist' ':!target' >/dev/null; then echo "conflict markers left"; exit 1; fi
git add -A Cargo.lock site/thumbs.js
if git rev-parse -q --verify MERGE_HEAD >/dev/null; then   # a merge left for us to commit
  git -c core.hooksPath=/dev/null commit -q --no-edit
fi
git log --oneline -1
```

</details>

<details>
<summary>rebase-demo.sh: rebase a demo branch onto the integration tip, checking every commit</summary>

```bash
#!/bin/bash
# Rebase demo/<demo> (in its own worktree) onto <onto>, so it can then be merged with
# --no-ff as a clean second track. Resolves the three known kinds of conflict:
#   Cargo.toml  the workspace members line: rebuilt in a fixed order
#   Cargo.lock  take the new base's side; cargo re-adds the path crate
#   site/thumbs.js  both sides only added an entry: keep both (union.py refuses otherwise)
# Anything else stops for a human. Every rebased commit must pass `cargo check`, so the
# result can be bisected.
# Usage: rebase-demo.sh <demo> <onto-commit> [<old-base>]
set -euo pipefail
demo=$1; onto=$2; old_base=${3:-71a2d46}
S=<scratchpad>
W=/Users/deg/Documents/degel/cellular-automata-wt/$demo
cd "$W"
[ "$(git branch --show-current)" = "demo/$demo" ] || { echo "not on demo/$demo"; exit 1; }
[ -z "$(git status --porcelain)" ] || { echo "worktree not clean"; exit 1; }
export GIT_EDITOR=true
G() { git -c core.hooksPath=/dev/null "$@"; }

members() {   # the members line, with the new demos in a fixed order
  python3 - <<'EOF'
import re
s = open("Cargo.toml").read()
names = set(re.findall(r'"([^"]+)"', "".join(re.findall(r"members = \[[^\]]*\]", s))))
order = ["ca_core", "sandpile", "life", "elementary", "langtons-ant", "brians-brain",
         "wireworld", "mandelbrot", "cyclic", "snowflakes", "langtons-loops", "lattice-gas",
         "tools/mdpage"]
assert names <= set(order), names - set(order)
line = "members = [" + ", ".join(f'"{n}"' for n in order if n in names) + "]\n"
s = re.sub(r"<<<<<<< .*?\n.*?>>>>>>> [^\n]*\n", line, s, flags=re.S)
open("Cargo.toml", "w").write(s)
EOF
}

resolve() {
  for f in $(git diff --name-only --diff-filter=U); do
    case "$f" in
      Cargo.toml) members ;;
      Cargo.lock) git checkout --ours Cargo.lock ;;
      site/thumbs.js) python3 "$S/union.py" "$f" ;;
      *) echo "conflict in $f: resolve by hand"; exit 1 ;;
    esac
    git add "$f"
  done
  cargo metadata -q --format-version 1 >/dev/null   # lets cargo re-add a path crate to the lock
  git add Cargo.lock
}

if ! G rebase --onto "$onto" "$old_base" --exec 'cargo check --workspace -q'; then
  while [ -d "$(git rev-parse --git-path rebase-merge)" ]; do
    if [ -z "$(git diff --name-only --diff-filter=U)" ]; then
      echo "rebase stopped with no conflict: a commit failed cargo check"; exit 1
    fi
    resolve
    if git grep -q -E '^(<<<<<<<|>>>>>>>) ' -- . ':!dist' ':!target'; then echo "markers left"; exit 1; fi
    G rebase --continue || true
  done
fi
git log --oneline "$onto..HEAD"
```

</details>

<details>
<summary>The consistency reviewers' brief</summary>

# Brief for a cross-demo consistency reviewer

David's cellular-automata project (Rust, egui 0.37, native + WebAssembly; site
https://automata.degel.com/) has eleven demos: `sandpile`, `life`, `elementary`,
`langtons-ant`, `brians-brain`, `wireworld`, `mandelbrot` (the seven older ones, which David
has used and shaped) and `cyclic`, `snowflakes`, `langtons-loops`, `lattice-gas` (four new
ones, built today by separate agents). Each was reviewed on its own. Nobody has yet compared
them with each other, and David has found inconsistencies between them (title case, which
demos start running, a panel that scrolled, a readout written as prose where the others use a
grid). Your job is the differences BETWEEN demos.

## Hard rules

- **Read-only.** Work in `/Users/deg/Documents/degel/cellular-automata-wt/integration`
  (`<wt>`), commit 19482b7. Do not edit, commit, or create files there. Notes, scripts and
  screenshots go in
  `<scratchpad>/consistency-<your area>/`.
- The Bash tool's working directory resets to `/Users/deg/Documents/degel/cellular-automata`,
  the main checkout, where another session is working. Never run anything there. Start every
  Bash command with `cd <wt> &&` and use absolute paths.
- No `bd` commands, no git writes, no pushes.
- The built site is in `<wt>/dist/` (one directory per demo, plus `index.html`). To see a
  demo, serve it on your assigned port and render headless, never a screenshot of the real
  screen:
  ```
  cd <wt> && python3 -m http.server --directory dist <port>      # run_in_background
  B=~/Library/Caches/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-mac-arm64/chrome-headless-shell
  $B --headless --hide-scrollbars --window-size=1236,840 --force-device-scale-factor=2 \
     --use-angle=swiftshader --enable-unsafe-swiftshader --virtual-time-budget=8000 \
     --screenshot=<file> http://localhost:<port>/<demo>/
  ```
  Run one Chromium at a time (parallel runs fail). To hover, click or drag, use Playwright's
  Python API with `executable_path` set to that binary, launch args
  `--force-device-scale-factor=2 --window-size=1236,840 --use-angle=swiftshader
  --enable-unsafe-swiftshader`, and `browser.new_context(no_viewport=True)`. Kill your
  http.server when done.

## How to judge

- Build a table: one row per thing to compare, one column per demo. Fill every cell from the
  code or a render, not from memory or from another demo's pattern.
- The convention is what the older seven do (most of them, or all), unless they disagree
  among themselves; then say so and propose one, with the reason.
- A difference is a finding only if a reader would notice it or it would surprise the next
  person to edit the code. A difference with a real reason (a demo whose nature needs it)
  is not a finding, but say what the reason is. Matters of taste are not findings.
- `<wt>/CLAUDE.md` states some conventions (README contents, controls panel width rules,
  sim.rs vs main.rs, testing). Check every demo against them.

## Report (your final message)

1. The table (compact; abbreviations fine).
2. Findings, ranked by how visible they are, each with: the demos that differ, the
   convention, file:line, and the fix. Group findings that one change would fix.
3. Places where the older seven disagree among themselves, with a proposed convention.
4. What you checked and found consistent (one line each), so the coordinator knows the
   coverage.

</details>

<details>
<summary>The conventions brief for the consistency round's workers</summary>

# Brief: bring demos into line with the project's conventions

Two consistency reviewers compared all eleven demos side by side, and David chose the
conventions where the older demos disagreed. Your prompt names the demos you own and lists
their findings. This brief has the rules and the conventions themselves.

## Hard rules

- Work only in your worktree `<wt>`, on your branch. The Bash tool's working directory
  resets to `/Users/deg/Documents/degel/cellular-automata` (the main checkout, where another
  session works) after every call: start EVERY Bash command with `cd <wt> &&`, use
  `git -C <wt>`, and absolute paths under `<wt>` for every file you read or edit. First
  action: `cd <wt> && git branch --show-current` must print your branch.
- Change only the files of the demos you own (`<demo>/...`) and, if your prompt says so,
  their entry in `site/thumbs.js`. Not `ca_core`, not the root README, CLAUDE.md, AGENTS.md,
  HOW-IT-WORKS.md, `site/` otherwise: the coordinator does those. If you need a `ca_core`
  change, say so in the report.
- No `bd` commands. Never push, never touch `main`, never merge anything into your branch.
  Commit with hooks off: `git -C <wt> -c core.hooksPath=/dev/null commit`, ending the message
  with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Subjects
  like `Life: Restart goes back to the opening; a Clear button (cellular-automata-<id>)`
  (use the bead id in your prompt). Commit per demo or per kind of change, each building and
  passing tests: the branch will be rebased and every commit checked.
- Scratch files in
  `<scratchpad>/<your name>/`.
- Facts: anything you add to a README (a name's origin, a date) must come from a source you
  checked (WebSearch is deferred: load it with ToolSearch `select:WebSearch`; read pages with
  the `web-fetch` agent type). Don't fill gaps from memory.

## Shared pieces in ca_core (use them; commit e8bc199)

- `ca_core::readout(ui, |r| { r.row("generation", ...); r.pointer(...); })`: the readout
  grid at the bottom of the panel. `r.row` returns the label's response (for a hover text).
- `r.pointer(Option<String>)`: the "under the pointer" row, "-" off the grid.
- `ca_core::speed_cap_note(ui, "world")`: the one wording of the speed-cap note.
- `ca_core::grouped(n)`: numbers as 1,234,567.
- `ca_core::NAVY`: the background colour; replace a demo's own navy constant with it.

## The conventions

UI:
1. **Start state**: paused, unless the opening is itself the show (a fixed start best seen
   developing); then `playback.running = true;` with a one-line comment saying why.
2. **Restart** returns to the demo's opening (its chosen start, pattern or circuit) with the
   current settings. A demo where you draw from scratch (Life, Brian's Brain, Wireworld) gets
   a separate **Clear** button that empties the world. Every demo has a short note saying what
   Restart does (and Clear, where there is one), next to the other playback notes.
3. **Pointer**: every demo with a 2D picture shows the cell under the pointer as the last
   row of its readout grid via `r.pointer(..)`; no tooltip on the grid. Request a repaint
   when the hovered cell changes, so the row doesn't lag a frame while paused.
4. **Readout**: `ca_core::readout`, lowercase labels, singular counters ("generation",
   "step"), numbers through `grouped`. The speed unit uses the readout's noun.
5. **Speed-cap note**: `speed_cap_note` in every demo whose cap can bite at its slider's
   range on a world it offers; no home-made variants.
6. **Presets**: highlighted chips (`selectable_label`) showing the current choice, with the
   chosen one's note below and no hover repeating it. Drop-downs stay only for long lists
   (Life's patterns, Wireworld's circuits).
7. **Sections**: a strong heading on its own line, then its controls. Option chips and radios
   in lowercase ("single cell", "wire"); named presets keep their names ("Gosper glider gun").
   The world/size block comes last before the readout. Brush controls sit with the drawing
   hint (a "Draw" section where there is one).
8. **Labels**: "World:" for a 2D world's size row (Wireworld keeps "Board:"); "Wrap around
   the edges" with Life's hover text; drawing hints end "; shift-drag to erase" (or say what
   shift-drag does where it doesn't erase). Status messages with `ui.small`.

README and text (READMEs are read by people new to the automaton, in the app's About panel):
9. **Structure**: the rule (in the opening, or under `## The rule` when it doesn't fit), where
   the name comes from (one sentence in the opening or History; a heading only if the name
   needs explaining), History, Things to try (bold leads), Further reading LAST.
10. **No paragraph after a nested list inside a list item**: the About panel's renderer draws
    it outside the bullet. Put it before the sublist, or after the list.
11. **Citations**: Author(s), *Title in italics*, Journal Name spelled out in plain text
    Volume, pages (year). En dash for every range, full numbers (pp. 135–144, 1984–1985).
    Links labelled `doi:10....` or `arXiv:....`. Scientific American plain, the column's name
    in italics. Refer to papers in running text by authors and year, without parenthesised
    author-date citations.
12. **Further reading** starts with "The Wikipedia article [Title](url)." when there is one.
13. **Provenance for editors** (which file a table was transcribed from, commit hashes, plate
    and section numbers, rule-list credits) goes in sim.rs doc comments, not the README;
    the README may end Further reading with one short "Sources" line if needed.
14. **Style**: British spelling; "anticlockwise"; "forever"; "degrees" in prose; commas in
    numbers of 1,000 and up; control names in *italics*; "Rule 184" capitalised as a name.
15. The app's notes and the README must not contradict each other.
16. The Cargo `description` (the index card) matches the root README's table row word for
    word; the coordinator owns the README table, so if you change a description, report the
    new text.

## Before you report

- `cd <wt> && make check` passes.
- For each demo you changed: `cd <wt>/<demo> && trunk build --release`, serve it on your
  port, and render headless at 1236 x 840, 2x (see `<wt>/CLAUDE.md` and the memory in your
  context about Playwright; one Chromium at a time; `--use-angle=swiftshader
  --enable-unsafe-swiftshader`). Check the panel still fits without scrolling in its tallest
  ordinary state, and hover the grid to see the pointer row. Kill your server after.
- Report: commits (hash, subject), what you changed per finding, what you didn't and why,
  any Cargo description you changed, screenshot paths, anything you're unsure of.

</details>
