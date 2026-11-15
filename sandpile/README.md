# Abelian sandpile

Drop grains of sand, one at a time, onto a single cell of a grid. A cell can hold up to
three grains. When a fourth arrives, the cell **topples**: it sends one grain to each of its
four neighbours (up, down, left, right) and is left empty. A neighbour that now holds four
grains topples in turn, and so on, until every cell holds three or fewer. One dropped grain
can set off a chain of topplings, called an **avalanche**, that is tiny or that sweeps
across the whole pile.

Drop a few hundred thousand grains on one cell and let them settle, and the pile grows into
an intricate, nearly circular fractal. It is made of nested triangles and circles, and
there is order in it at every scale.

## The rule

- Every cell holds a whole number of grains.
- A cell holding at least 4 grains (as many as it has neighbours) is unstable and topples:
  it loses 4 grains and each of its 4 neighbours gains one.
- Repeat until no cell is unstable. The pile is then **stable**.

Toppling never creates or destroys a grain, except at the edge of a finite grid, where
grains can fall off.

## Why "abelian"?

Suppose several cells are unstable at once. Which should topple first? It doesn't matter:
however you order the topplings, the pile settles into exactly the same final pattern, and
each cell topples the same number of times along the way. Operations whose order doesn't
matter are called *commutative*, or *abelian*, after the Norwegian mathematician Niels
Henrik Abel. That's where the name comes from.

The demo relies on this. It topples cells in whatever order is fastest: it sweeps across
the grid row by row, and lets a cell holding, say, 12 grains topple three times at once.
The tests check that dropping grains one at a time gives exactly the same pile as dropping
them all at once.

## History

- **1987: self-organized criticality.** The physicists Per Bak, Chao Tang and Kurt
  Wiesenfeld introduced the sandpile in *Self-organized criticality: an explanation of the
  1/f noise* (Physical Review Letters 59, 381;
  [PDF](https://courses.physics.ucsd.edu/2016/Spring/physics235/BTW-PRL.pdf)). They were asking why so many natural
  systems show events of every size, with no typical scale: earthquakes, avalanches,
  forest fires, solar flares. Their answer was that some systems drive themselves to a
  "critical" state with no tuning from outside. Keep dripping sand onto the pile and it
  reaches a state where avalanche sizes follow a power law. Most are tiny, a few are huge,
  and there are all sizes in between. Bak later wrote a popular book on the idea, *How
  Nature Works* (1996).
- **1990: the abelian property.** Deepak Dhar proved that the order of topplings doesn't
  matter (the property that gives the model its name), and showed it has deep algebraic
  structure. On a finite grid with a sink edge, the stable piles that keep recurring as
  you add grains form a mathematical group, which Dhar named the **sandpile group**. The number of those
  piles equals the number of spanning trees of the grid
  (*Self-organized critical state of sandpile automaton models*, Physical Review
  Letters 64, 1613).
- **1991: chip-firing.** Combinatorialists study the same process on any graph, as a
  *chip-firing game*. Anders Björner, László Lovász and Peter Shor, *Chip-firing games on
  graphs* (European Journal of Combinatorics 12, 1991).
- **2010s: why the fractal looks the way it does.** For a long time the pattern from a
  single source was a well-known picture that nobody could explain. Wesley Pegden and
  Charles Smart proved that it has a definite limiting shape as the number of grains grows
  (*Convergence of the Abelian sandpile*, Duke Mathematical Journal 162, 2013;
  [arXiv:1105.0111](https://arxiv.org/abs/1105.0111)). With Lionel
  Levine they then explained its circles and triangles through **Apollonian circle
  packings**, named after the Greek geometer Apollonius of Perga, in which circles
  fill the gaps between circles, smaller and smaller without end (*Apollonian structure in the Abelian sandpile*, Geometric and
  Functional Analysis 26, 2016; [arXiv:1208.4839](https://arxiv.org/abs/1208.4839)).

## Things to try

- **Watch avalanches.** Restart, keep *Drip* on with 1 grain per drip, set the speed low,
  and press Run. Toppling cells flash white. Most grains do nothing; now and then one sets
  off an avalanche across the pile.
- **Step through one.** Pause, then press Step repeatedly. On a settled pile, Step drips
  first (one grain, unless you raise *grains per drip*); while an avalanche is running, each Step is one sweep across the pile.
- **Grow the fractal.** Restart and Drop 100,000 grains (a second or two). Then try
  1,000,000, which takes a couple of minutes. The pattern keeps the same character, finer
  and finer.
- **Sink edge.** Choose *grains fall off* and Restart. Grains now escape at the edge of a
  small grid, which is the setting of the original 1987 paper and of the sandpile group.
- **Several sources.** Shift-click the grid to add a drip point, marked with a pink ring;
  shift-click it again to remove it, and *Centre only* puts things back. Every drip feeds
  all the points at once. Add two or three, raise *grains per drip* to 100 or so, and Run:
  watch the piles grow into each other. Drip points stay put through Restart, so you can
  try the same arrangement with either edge. On a *grains fall off* grid, though, points
  beyond its edge drip straight off it: their grains count as lost, and the panel says how
  many points that is.
- **One-off drops.** A plain click drops *grains per drip* grains on that cell, once.

## Further reading

- Lionel Levine and James Propp, *What is a sandpile?*, Notices of the American
  Mathematical Society 57 (2010), pages 976-979: a short introduction
  ([free PDF](https://faculty.uml.edu/jpropp/whatisasandpile.pdf)).
- The Wikipedia article
  [Abelian sandpile model](https://en.wikipedia.org/wiki/Abelian_sandpile_model).
- Per Bak, *How Nature Works: The Science of Self-Organized Criticality* (1996).

## Variants

The rule works with any set of neighbours, as long as a cell topples when it holds one
grain for each of them, so toppling never creates or loses a grain. The abelian property
holds for every such choice.

- **8 neighbours.** Choose *8* under *Neighbours*, which starts a fresh pile. Diagonal cells now count as
  neighbours: a cell holds up to 7 grains and topples at 8, sending one grain in each of
  the eight directions. The colours spread over the eight heights. Drop 100,000 grains
  and compare the shape with the four-neighbour pile.
- **Hex grid** (planned): a cell topples at 6, sending one grain to each of its six
  neighbours.
