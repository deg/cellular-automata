# Critters

Every cell is a 0 or a 1, but cells don't change one at a time here. The world is cut into
**blocks** of 2 x 2 cells, and each block as a whole is replaced by another, looked up in a
table. A block of four cells can be in only 16 ways, so the table has 16 entries. On the
next step the blocks are drawn again, shifted one cell right and one cell down, so that
each new block takes one cell from each of four old ones, and on the step after that they
shift back. This alternating pattern of blocks is the **Margolus neighbourhood**, after
Norman Margolus, who devised it. Here the world wraps round at its edges.

**Critters** is a rule of Tommaso Toffoli and Norman Margolus. For each block, count its
1s:

- **Two:** the block stays as it is.
- **Zero, one or four:** every cell flips, 0 to 1 and 1 to 0.
- **Three:** every cell flips, and the block is turned half a turn.

The name comes from the book that introduced the rule, where, from a lump of random cells,
"small 'critters' race horizontally and vertically; when they collide they may bounce
back or execute a right-angle turn".

No two blocks turn into the same block, so every step can be **undone**: from any picture
there is exactly one picture it came from, and a table that leads back to it. Run backwards
and the world retraces its history step by step, back to the start exactly, however tangled
it had become. Critters keeps something else too: a block with *k* 1s always becomes one
with *k* 0s. So, as Margolus puts it, call the 1s particles on even steps and the 0s
particles on odd steps, and the number of particles never changes. The demo draws
particles, so the empty space stays dark instead of flashing on every step.

Two other rules from the same work are here, both reversible and both keeping their 1s,
which are the particles throughout:

- **Billiard balls.** A lone particle moves to the opposite corner of its block. When the
  blocks shift, it finds itself in the same corner of its new block, and so runs on along a
  diagonal. Two particles on a diagonal of one block leave on the other diagonal. Every
  other block stays as it is, so a solid square of four, straddling two blocks, never
  moves: it is a **mirror**. A **ball** is two particles one behind the other, with an odd
  number of empty cells between them. When two balls meet they are held for a step, then
  leave, each knocked off its path.
- **HPP gas.** Every particle moves to the opposite corner of its block, whatever else is
  there, except that two particles meeting head-on, on a diagonal, leave on the other
  diagonal, at right angles to how they came. The letters are the initials of its
  inventors, J. Hardy, Y. Pomeau and O. de Pazzis.

## History

- **1982: computing with billiard balls.** Edward Fredkin and Tommaso Toffoli showed that a
  computer could, in principle, be built from perfectly elastic balls bouncing off each
  other and off fixed mirrors. Whether two balls collide at a point depends on whether
  both arrive, so a collision is a logic gate, and the whole computer is reversible: run
  every ball backwards and it uncomputes. (E. Fredkin and T. Toffoli, *Conservative
  logic*, International Journal of Theoretical Physics 21, 219–253 (1982).)
- **1983–1984: the Margolus neighbourhood.** Norman Margolus, at MIT, wanted a cellular
  automaton that behaved like the billiard balls. A cell that looks at its neighbours to
  decide its own next state is hard to make reversible: a particle moving from one cell to
  the next must vanish from one and appear in the other, and the two cells decide
  separately. Margolus's answer was to update blocks instead of cells. If the table for a
  block is reversible, the whole world is, and if it keeps the number of 1s, so does the
  world. His billiard-ball rule is the one here, and he showed that it is *universal*: it
  can compute anything a computer can. (N. Margolus, *Physics-like models of
  computation*, Physica D 10, 81–95 (1984).)
- **1984: the HPP gas on a machine.** J. Hardy, Y. Pomeau and O. de Pazzis had introduced
  the HPP gas in the 1970s as a model of a gas on a square lattice (the lattice gas, another
  demo in this project, tells that story). Toffoli and Margolus rediscovered it and wrote it
  as a block rule. When they first ran it, Margolus recalled, they were amazed to see a
  round pressure wave. They had built machines that ran automata fast enough to watch.
  Toffoli and Margolus report Pomeau's account: seeing his gas run on one of them, at a
  1984 workshop, showed him that what he had conceived as a model on paper could actually
  be computed, and set him looking for lattice gases that would model fluids better.
- **1987: Critters.** Toffoli and Margolus described Critters in their book *Cellular
  Automata Machines: A New Environment for Modeling* (MIT Press, 1987), section 12.8.2,
  pp. 132–134. It is reversible like the billiard balls, but needs no careful arrangement:
  start from "a blob of randomness" in an empty world, and gliders escape from it on their
  own. Unlike the billiard-ball rule, it is not its own inverse, though the book points
  out a shortcut: complement every cell, run Critters itself, and you see the history run
  backwards in complement.
- **1999: Critters is universal.** In a later account, Margolus showed how pairs of
  Critters gliders, suitably arranged, act just like the balls of his billiard-ball rule,
  so Critters can compute anything too. He also showed why its gliders are so hard to
  destroy. When two collide, they form a blob of eight particles. The blob can't settle
  into a fixed pattern or a cycle, because then running backwards would leave it there
  too, though it was made by a collision. So it must break up, and the only moving things
  eight particles can make are one or two gliders: at least one glider always comes out.
  (N. Margolus, *Crystalline computation*, in A. J. G. Hey (ed.), *Feynman and
  Computation*, Perseus Books, 267–305 (1999), [arXiv:comp-gas/9811002](https://arxiv.org/abs/comp-gas/9811002).)

## Things to try

- **There and back.** Run *Critters* for a few hundred generations, until the square has
  thrown gliders across the world. Choose *backwards* and Run: the gliders fly home, the
  mess untangles, and the run stops at generation 0 on the very square you started from.
  The readout's *cells unlike the start* shows 0.
- **The gliders.** Each is four particles: two side by side and two a little apart. It
  goes through four shapes as it moves two cells, along a row or a column. Watch two
  collide: something always comes out.
- **Draw your own start.** Choose *Clear*, then draw a few strokes with a brush of radius
  0. Run forwards, then backwards to your drawing. The *random* brush with a radius of 5
  makes a small random square, the kind of seed gliders come from.
- **Change the past.** Run forwards, pause, and draw a few particles in the busiest part.
  Now run backwards: the start doesn't come back, and *cells unlike the start* counts how
  far off it is.
- **Before the start.** Run backwards from generation 0, into negative generations. The
  random square has a past as well as a future, and it looks much the same: gliders fly
  out in both.
- **Billiard balls.** Balls bounce around a box of mirrors. They are dropped at random,
  so most meet askew rather than squarely, and break up into loose particles; in a
  billiard-ball computer every ball is placed with care. Run them for a while, then
  backwards: every collision un-happens in turn, and the balls come back whole. Draw your
  own mirrors with a brush of radius 0: a square of four particles is a mirror only where
  it straddles two blocks.
- **HPP gas.** A square of particles sits in a random gas, and a round wave spreads from
  it, though every particle moves along a diagonal. This rule gets the biggest world, so
  the wave has room to grow before it wraps round the edges. Run backwards, and the wave
  gathers itself back into the square.

## Further reading

- The Wikipedia article [Critters (cellular automaton)](https://en.wikipedia.org/wiki/Critters_(cellular_automaton)).
- The Wikipedia article [Block cellular automaton](https://en.wikipedia.org/wiki/Block_cellular_automaton),
  on rules of this kind.
- N. Margolus, *Crystalline computation*, in *Feynman and Computation* (1999),
  [arXiv:comp-gas/9811002](https://arxiv.org/abs/comp-gas/9811002): Critters, the
  billiard balls and the HPP gas in block form, with pictures, by Margolus himself.
- N. Margolus, *Physics-like models of computation*, Physica D 10, 81–95 (1984),
  [doi:10.1016/0167-2789(84)90252-5](https://doi.org/10.1016/0167-2789(84)90252-5).
- T. Toffoli and N. Margolus, *Invertible cellular automata: a review*, Physica D 45,
  229–253 (1990), [doi:10.1016/0167-2789(90)90185-R](https://doi.org/10.1016/0167-2789(90)90185-R).
- T. Toffoli and N. Margolus, *Cellular Automata Machines: A New Environment for
  Modeling* (MIT Press, 1987), [PDF](https://people.csail.mit.edu/nhm/cam-book.pdf):
  Critters is in section 12.8.2, and the Margolus neighbourhood in chapter 12.
