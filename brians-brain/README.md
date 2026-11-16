# Brian's Brain

A grid of cells modelled, loosely, on nerve cells. Each cell is in one of three states:

- **Ready** (dark): resting, able to fire.
- **Firing** (white): just fired.
- **Recovering** (blue): just fired and can't fire again yet. Neuroscientists call this the
  *refractory* period.

Every generation, all cells update at once:

- A **ready** cell fires if **exactly 2** of its 8 neighbours are firing.
- A **firing** cell starts recovering.
- A **recovering** cell becomes ready again.

The recovery step changes everything. A cell that has just fired can't fire again at once,
so activity can't stay where it is: it has to move on. Almost every pattern in Brian's
Brain ends up travelling across the grid.

## History

Brian's Brain was invented by **Brian Silverman**, who also devised the Seeds rule (B2/S,
one of the rules in this project's Life demo). Brian's Brain closely resembles
Seeds; the recovery state is the difference. Silverman also created Wireworld, another demo in this project.

- **Moving patterns everywhere.** Because recovering cells block activity from turning
  back, almost every pattern is a *spaceship*, a shape that travels. Many are *rakes*,
  spaceships that leave other spaceships behind them. Patterns that move steadily are
  usually called *gliders*. Larger random starts tend to explode into sweeping diagonal
  waves.
- **Oscillators exist too**, though moving patterns far outnumber them. One has just 4 firing
  and 4 recovering cells and repeats every 3 generations.
- **2020: it can compute.** Yoel Matveyev built a pattern in Brian's Brain that emulates
  Rule 110, the one-dimensional rule proved universal (see the Elementary demo). That
  makes Brian's Brain Turing-complete: in principle it can carry out any computation.

## The glider

The simplest spaceship is two firing cells side by side, with two recovering cells behind
them. The two cells just ahead are the only ones touching both firing cells, so they fire
next. The recovering cells can't fire, so the pair moves forward one cell every
generation. *Add four gliders* places four of them, heading in four directions. A firing
pair *without* the recovering pair behind it behaves quite differently: try drawing one.

## Things to try

- **A small seed.** Press *Run*. The random patch in the middle throws off gliders and
  waves in all directions.
- **Gliders.** *Clear*, press *Add four gliders*, then *Run*. With *Wrap around the edges*
  on, they come back from the opposite side, and pairs travelling along the same line
  collide.
- **Draw.** *Clear* and drag a short line or a small blob, then *Step* through it a
  generation at a time to see how the recovery state steers it.
- **A big soup.** Raise *seed size* to 1 and *density* to 0.3, then press *Restart* for a
  world-sized start.

## Further reading

- The Wikipedia article [Brian's Brain](https://en.wikipedia.org/wiki/Brian%27s_Brain).
