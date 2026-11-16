# Langton's loops

A small square loop of cells carries a train of signals round and round. Each time the
train passes a branch at one corner, a copy of it runs out along an **arm** and extends
it. After 151 generations the arm has been bent into a second loop, a cell-for-cell copy of
the first, which is cut free and starts doing the same. The copies keep copying, and a
colony grows outward like a coral reef: a fringe of busy, reproducing loops around a core
of finished, empty ones.

Each cell is in one of 8 states and sees only itself and its four nearest neighbours,
above, below, left and right. Every generation, all cells look up those five states in a
table of 219 rules and change at once. Nothing in the table mentions loops or copying: the
loop does the copying, using the rules the way a machine uses physics.

## The rule

The states, with their colours in this demo:

- **0**, navy: empty.
- **1**, teal: the **core**, the inside of a path, along which signals travel.
- **2**, indigo: the **sheath**, the walls on either side of the core.
- **3**, sky blue: appears briefly at several stages of the construction.
- **4**, coral: with a 0 behind it, a "4 0" signal. Two of them turn the arm left.
- **5**, pink: travels back into the parent when a copy is finished, pulling the arm in
  behind it, then starts the parent's next arm.
- **6**, white: with a 0 behind it, starts the new loop's own arm.
- **7**, gold: with a 0 behind it, a "7 0" signal, which extends the arm by one cell.

A **signal** is a pair of cells moving along the core, a signal state followed by a 0, one
cell per generation. Where a path branches, a signal is copied down both branches. The
loop's "genome" is the sequence of signals circulating in it, **7 0 - 7 0 - 7 0 - 7 0 - 7 0 - 7 0 - 4 0 - 4 0**.

Each time the sequence passes the junction where the arm leaves the loop, a copy of it runs
out along the arm: the six "7 0" signals extend the arm by six cells, and the two "4 0"
signals turn it left. So each trip of the genome builds one side and one corner of the new
loop. After four trips the arm has come round and joined itself, and the collision of
signals at the new junction makes two new signals. A "5" travels back into the parent,
pulling the "umbilical cord" in behind it, and on to the parent's next corner, where it
starts a new arm, turned 90 degrees counterclockwise from the last. A "6 0" travels into
the new loop and starts its arm.

The table lists each neighbourhood once, and it also applies turned through a quarter, a
half and three-quarters of a turn (but not mirrored). A neighbourhood the table doesn't
list leaves the cell as it is. Beyond the edge of the world every cell is empty.

## History

- **About 1950: von Neumann.** John von Neumann set out to show that a machine could
  reproduce itself, and did it with a cellular automaton of 29 states per cell and the
  same five-cell neighbourhood. His construction is a *universal constructor*: a machine
  that can build any machine described on a tape, and copy the tape. Given its own
  description, it builds itself. It was completed and published after his death by Arthur
  Burks, as *Theory of Self-Reproducing Automata* (1966).
- **1965: Codd.** In his doctoral thesis at the University of Michigan (1965), published
  as the book *Cellular Automata* (1968), Edgar F. Codd found a universal constructor that
  needs only 8 states. His
  machines are built from **data paths**, a core of 1s between sheaths of 2s, along which
  signals travel. One of his components is the **periodic emitter**, a path that loops back
  on itself so that its signals circulate forever.
- **1984: Langton.** Christopher Langton, at the University of Michigan, asked the reverse
  question: how *simple* can a self-reproducing machine be? Universal construction is
  enough for self-reproduction, he argued, but not necessary: no living thing has been
  shown to be a universal constructor. Instead he required that the copying be directed by
  the structure itself, and that it use its information in two ways, as instructions to
  carry out (as a cell *translates* its genes) and as data to copy (as it *transcribes*
  them). He then took Codd's periodic emitter and stored the construction instructions in
  the loop rather than on a tape. He made the loop a square, so it needs instructions for
  only one side and one corner, and changed the rules so that each signal does more:
  extending a path takes the two signals "7 0 - 6 0" in Codd's automaton, and "7 0" alone
  here. The result reproduces itself in 151 time steps and fits in a rectangle of 10 by 15
  cells, where von Neumann's and Codd's machines take many tens of thousands.
- **The colony.** Langton also described how a colony grows. Each loop builds its
  offspring on each of its sides in turn. A loop that finds a site taken retracts its arm
  and blocks its own core, and its signals are then erased one by one, leaving a dead,
  empty loop. So the first, "Adam" loop reproduces four times, each of its immediate offspring
  only twice, and the colony grows at its edge for as long as there is room.

Langton went on to coin the name "artificial life", and organised the first workshop on
it, at Los Alamos National Laboratory in 1987. Langton's ant, another demo in this project,
is also his.

This demo's tests check the paper's central claims: the first copy is complete at
generation 151 and is exactly the starting loop, its own copy follows 151 generations
later, and the Adam loop has four offspring and then dies.

**Sources.** The table is Table I of Langton's paper (page 140), and the starting loop is
its fig. 6. The demo's copy of the table was taken from the machine-readable transcription
in Golly, a free cellular-automaton program: `Rules/Langtons-Loops.rule` and
`Patterns/Loops/Langtons-Loops.rle` at its git commit
[53bdb74e](https://sourceforge.net/p/golly/code/ci/53bdb74e48cd0792f87a2e3e6a5bb0c5d7d5af0e/tree/).
It matches the paper's Table I entry for entry, and also the list in Eli Bachmutsky's Java
applet
[`loops.java`](https://web.archive.org/web/20010521213854/necsi.org/postdocs/sayama/sdsr/java/loops.java)
(1999), which Golly's file credits. The paper doesn't say what happens to a neighbourhood
missing from the table; Golly and the applet both leave the cell unchanged, and so does
this demo.

## Things to try

- **The first copy.** Press Run and watch the arm grow out of the bottom right corner,
  turn left four times and close. Pause near generation 120 and Step on to 151 to see the
  two loops separate: watch for the pink 5 and the white 6.
- **The arm moves on.** Watch the parent: after each copy it builds a new arm on its next
  side, a quarter turn counterclockwise from the last.
- **The colony.** Raise the speed to a few hundred and let the colony spread, with
  "Follow the colony" zooming out as it grows. The dead loops inside keep only their teal
  core and indigo sheath; the signals live in the fringe.
- **Interfere.** Pause, pick a state under *Draw*, and paint over part of a loop: erase a
  piece of sheath, add a signal to the genome, or change a gold 7 into a coral 4. Then run
  and see whether it still reproduces, and what its offspring look like.
- **The edge.** On the smallest world the colony reaches the edge after about 1,800
  generations. Loops there have nowhere to build, and the fringe stalls one loop after
  another until, around generation 4,100, nothing in the world moves at all.

## Further reading

- The Wikipedia article [Langton's loops](https://en.wikipedia.org/wiki/Langton%27s_loops),
  with a table comparing the loops that followed.
- C. G. Langton, *Self-reproduction in cellular automata*, Physica D 10 (1984) 135-144,
  [doi:10.1016/0167-2789(84)90256-2](https://doi.org/10.1016/0167-2789(84)90256-2), with a
  free copy at the University of Michigan's
  [Deep Blue](https://hdl.handle.net/2027.42/24968).
- J. von Neumann, *Theory of Self-Reproducing Automata*, edited and completed by A. W.
  Burks (University of Illinois Press, 1966).
- E. F. Codd, *Cellular Automata* (Academic Press, 1968).
- Smaller loops: J. Byl, *Self-reproduction in small cellular automata*, Physica D 34
  (1989) 295-299, removes the inner sheath; J. A. Reggia, S. L. Armentrout, H.-H. Chou and
  Y. Peng, *Simple systems that exhibit self-directed replication*, Science 259 (1993)
  1282-1287, remove the sheath altogether.
- Loops that die and evolve: H. Sayama, *Introduction of structural dissolution into
  Langton's self-reproducing loop*, Artificial Life VI (1998), adds a state that dissolves
  loops, and his *evoloops* (1999) evolve. H. Sayama and C. L. Nehaniv,
  *Self-reproduction and evolution in cellular automata: 25 years after evoloops*,
  Artificial Life 31 (2025) 81-95,
  [doi:10.1162/artl_a_00451](https://doi.org/10.1162/artl_a_00451), reviews
  what followed.
- [Golly](https://golly.sourceforge.io/), which runs these and many other rules, and its
  [Rule Table Repository](https://github.com/GollyGang/ruletablerepository).
