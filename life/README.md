# Conway's Game of Life

A grid of cells, each alive or dead. Every generation, all the cells update at once, each
looking only at its eight neighbours:

- A **dead** cell with exactly **3** live neighbours comes to life (birth).
- A **live** cell with **2 or 3** live neighbours stays alive (survival).
- Every other cell dies, or stays dead: of loneliness with fewer than 2 neighbours, of
  overcrowding with more than 3.

That's the whole game. There are no players. You set up the first generation and watch.
From these three rules come patterns that sit still, patterns that blink, patterns that
crawl across the grid, and patterns that build other patterns.

## History

- **1970: the game.** The British mathematician John Horton Conway devised Life. It
  reached the world through Martin Gardner's *Mathematical Games* column in Scientific
  American, October 1970, which was based on Gardner's conversations with Conway.
- **The glider.** While Conway's group was tracking the R-pentomino (one of the patterns
  in this demo), Richard K. Guy spotted a five-cell shape that crawls diagonally across
  the grid: the **glider**. Sources disagree on whether that was 1969 or 1970. It is often
  credited to Conway, but Conway himself said it was Guy.
- **1970: the $50 prize.** Conway conjectured that no pattern could grow forever, and
  Gardner's column offered **$50** to the first person to prove or disprove it before the
  end of 1970. In November 1970 a team at MIT led by Bill Gosper won it with the
  **Gosper glider gun**, which fires a new glider every 30 generations forever.
- **1982: Life can compute.** In *Winning Ways for your Mathematical Plays* (volume 2,
  1982), Elwyn Berlekamp, Conway and Guy showed that Life can carry out any computation a
  computer can. Gliders act as signals, and collisions between them act as logic gates.
  In 2000 Paul Rendell built an actual Turing machine out of Life patterns.

## Patterns in this demo

- **Glider**: the smallest spaceship, one cell diagonally every 4 generations.
- **Lightweight spaceship**: travels sideways.
- **Gosper glider gun**: the prize-winner. Press Run and watch it fire.
- **Pulsar**: a large oscillator that repeats every 3 generations.
- **R-pentomino, Acorn, Diehard**: *methuselahs*, tiny patterns that take a very long
  time to settle. On an unbounded grid the R-pentomino (5 cells) settles after 1,103
  generations and the acorn (7 cells) after 5,206. The diehard vanishes completely after
  exactly 130. This demo's world is finite, so the first two end differently here: their
  gliders wrap around or hit the edge.

## Other rules

Life is one of a family of *Life-like* rules, written **B**/**S**. B lists the neighbour
counts that bring a dead cell to life, and S the counts that keep a live cell alive. Life
is B3/S23. Try:

- **HighLife, B36/S23**: Life plus birth on 6 neighbours. Nathan Thompson studied it in
  1994. It has a **replicator**, a pattern that makes copies of itself.
- **Seeds, B2/S**: credited to Brian Silverman. No cell ever survives, yet many patterns
  explode into chaotic growth.
- **Day & Night, B3678/S34678**: credited to Nathan Thompson. Live and dead cells play
  symmetric roles: invert a pattern and it evolves exactly as the inverse of the original.

## Things to try

- **The gun.** It starts placed: press Run. Turn *Wrap around the edges* off, so gliders
  die at the border. With wrap on, they come back round, and on the default world they
  wreck the gun at generation 6,390.
- **Draw.** Clear, drag on the grid to draw live cells, shift-drag to erase, then Run.
  What do your scribbles settle into?
- **Random soup.** *Random fill* at density 0.3 on the largest world, then Run at a high
  speed. Watch how the soup evolves, and whether gliders escape from it.
- **A methuselah.** Clear, place the R-pentomino, and Run. Watch how much five cells
  make.
- **Paste a pattern.** LifeWiki lists thousands, each with its RLE text. Paste one into
  *Paste an RLE pattern*.

## Further reading

- The Wikipedia article
  [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life).
- [LifeWiki](https://conwaylife.com/wiki/Conway's_Game_of_Life), the community's
  encyclopedia of patterns and history.
- Martin Gardner, *The fantastic combinations of John Conway's new solitaire game
  "life"*, in his *Mathematical Games* column, Scientific American 223, 120–123
  (October 1970), [transcription](https://www.ibiblio.org/lifepatterns/october1970.html).
- [Golly](https://golly.sourceforge.io/), a fast open-source Life simulator that can run
  enormous patterns.
