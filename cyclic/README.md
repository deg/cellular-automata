# Cyclic Cellular Automaton

Every cell holds one of a few colours, numbered 0, 1, 2, and so on. The colours are
arranged in a **wheel**: after colour 0 comes 1, then 2, ..., and after the last colour
comes 0 again. A cell can only ever move one step on round the wheel, to its **next
colour**, and only when it is pushed:

- A cell moves on to its next colour if **at least *threshold*** of the cells **within
  *range*** of it already have that next colour.
- Otherwise it keeps its colour.

All cells update at once, every generation. Four numbers make a rule:

- **Range**: how far a cell looks. Range 1 is the cells touching it.
- **Neighbourhood**: what "within range" means. The **box** (Moore) takes every cell up to
  *range* steps away across, down or diagonally: a square. The **diamond** (von Neumann)
  counts steps north, south, east and west only. At range 1 the box holds 8 cells and the
  diamond 4.
- **Threshold**: how many next-colour cells it takes to push a cell on.
- **Colours**: how many colours the wheel has.

Start from random noise, and with the right numbers the noise organises itself into
waves of colour and then into turning **spirals**. With other numbers it freezes almost at
once, or churns forever without settling.

## Where the name comes from

Each colour can only "eat" the colour just before it on the wheel. Robert Fisch, Janko
Gravner and David Griffeath, who studied the rule, described it as creatures competing in
a "cyclic food chain": colour 1 eats 0, 2 eats 1, and 0 eats the last colour, closing the
cycle.

## History

- **1989: random cyclic competition.** The automaton grew out of a random version, in
  which each site picks a neighbour at random and eats it if it can. Bramson and
  Griffeath proved that in one dimension it settles into fixed colours when there are
  5 or more colours, but forms ever-growing patches of one colour with 3 or 4
  ([Bramson and Griffeath, 1989](#further-reading)). In two dimensions it was a mystery
  until Fisch and Griffeath simulated it on the *Cellular Automaton Machine* (CAM), a
  plug-in board for running automata quickly, designed by Toffoli and Margolus. To their surprise, the screen filled with large spiral waves
  ([Griffeath, 1988](#further-reading)).
- **1991: the cyclic cellular automaton.** Hoping a rule without randomness would be easier
  to analyse, Fisch, Gravner and Griffeath turned to the deterministic version: range 1,
  diamond neighbourhood, threshold 1. This is the *CCA* preset here
  ([Fisch, Gravner and Griffeath, 1991a](#further-reading)). With 14 colours on a 256 x 256
  grid, they saw four stages, named with four Ds:
  - **Debris.** Almost every cell soon has nothing to eat, and stops: random debris.
  - **Droplets.** A few active patches survive, and grow as droplets of waves that spread
    until they have eaten all the debris.
  - **Defects.** Loops of cells whose colours wind round the whole colour wheel form
    inside the waves, and spirals grow around them.
  - **Demons.** The most efficient spirals have at their centre a *clock*: a loop of cells
    on which each colour has its next colour beside it, so it advances every generation
    whatever happens around it. A clock and the cells it drives make a *demon*. Demons
    slowly take over from the other spirals, until every cell of the grid cycles through
    all 14 colours, one step per generation.

  They proved that once a clock exists, every cell eventually cycles like this, however
  many colours there are. A. K. Dewdney described the rule in his *Scientific American*
  column of August 1989, "A cellular universe of debris, droplets, defects, and demons".
- **1991: range and threshold.** In a second paper, Fisch, Gravner and Griffeath added the
  range, the threshold, and the box and diamond neighbourhoods
  ([Fisch, Gravner and Griffeath, 1991b](#further-reading)). They mapped what different
  rules do from random noise, using a program they wrote for PCs, *EXCITE!*:
  - with a low threshold, waves bend round into **spirals**, and the world becomes
    periodic: every cell ends up repeating a cycle of colours forever;
  - with a higher threshold, would-be spirals keep unravelling, leaving debris that seeds
    new waves: a state they called **turbulence**, which never settles;
  - with a higher one still, the world **fixates**: it stops changing for good.

  When there are many colours, it is mostly the threshold divided by the square of the range
  that decides which of these happens. Their colour plates include the rules here named
  *Perfect* (perfect spirals appearing out of a churning soup), *Turbulent phase*, and
  *Bootstrap fixation*. The rule named *Lava lamp* here they called the "lava light".
- **Why it mattered.** These rules are simple mathematical stand-ins for *excitable media*,
  through which waves of activity travel and turn into spirals. The papers list spiral waves in oscillating chemical reactions (the Belousov-Zhabotinsky
  reaction), in heart muscle during atrial fibrillation, and in nerve networks. Fisch,
  Gravner and Griffeath offered the cyclic automaton as a model simple enough to prove
  things about. A closely related rule, the *Greenberg-Hastings model*, differs only in
  that every colour except 0 advances automatically.

The presets' names, rules and credits come from the rule list of **MCell**, Mirek
Wójtowicz's program for exploring cellular automata. It writes a rule as, for example,
`R1/T3/C4/NM`: range 1, threshold 3, 4 colours, Moore (box) neighbourhood (`NN` is von
Neumann). The panel shows each rule in this notation.

## Things to try

- **The four Ds.** Choose *CCA* and run fast. First the noise freezes into debris, then
  droplets of waves eat it, then spirals appear and grow. Leave it running: the
  *changed last step* count climbs until every cell changes every generation.
- **More colours.** Fisch, Gravner and Griffeath found that with 20 colours the droplets
  become distinct and "drop-like". Set *Colours* to 20 with the CCA rule. With still
  more colours, droplets get rarer, and a small world may never get one: it fixates.
- **Perfect spirals.** Choose *Perfect* on the largest world. It churns for a while (MCell
  says "often after more than one hundred updates") before perfect spiral cores appear and
  slowly take over.
- **Turn up the threshold.** Shalizi and Shalizi describe four colours on the range-1 box:
  threshold 1 gives a jumble of small oscillations, 2 gives spiral waves, 3 turbulence (in
  which spirals may still win in the end), and 4 or more fixation. Choose *Imperfect*
  (threshold 2), then try 1, 3 and 4.
- **Clocks take over.** Choose *Imperfect* and run fast. Fisch, Gravner and Griffeath
  found that in this rule spirals can rule for a long time, their cells taking some
  multiple of 4 generations to go once round the 4-colour wheel. Then a small structure
  that cycles on its own suddenly appears, and quickly takes over from the spirals. Watch
  *changed last step*: while spirals rule it stays well below the whole world; once the
  new structures have taken over, every cell changes every generation.
- **Turbulence.** Choose *Turbulent phase* on the largest world. Fisch, Gravner and
  Griffeath found that this rule often fixates on a 256 x 256 grid, only because the grid is
  too small: it kept going on 512 x 512. This demo's largest world is 640 x 400, so it may
  freeze here too.
- **Freezing.** Choose *Bootstrap fixation* and watch the noise freeze. Then drag across
  it to paint fresh noise and see whether anything starts up again.
- **Paint.** Drag on any running world to paint random colours; shift-drag paints a patch
  of one colour. See what the waves do to each.
- **Range.** Choose *Cyclic spirals* (range 3), then raise the range and the threshold
  together.

## Further reading

- R. Fisch, J. Gravner and D. Griffeath (1991a), "Cyclic cellular automata in two
  dimensions", in *Spatial Stochastic Processes*, a festschrift for T. E. Harris,
  Birkhäuser, 171-185. [PDF](https://www.math.ucdavis.edu/~gravner/papers/cca.pdf).
- R. Fisch, J. Gravner and D. Griffeath (1991b), "Threshold-range scaling of excitable
  cellular automata", *Statistics and Computing* 1, 23-39.
  [PDF](https://www.math.ucdavis.edu/~gravner/papers/fgg.pdf),
  [arXiv:patt-sol/9304001](https://arxiv.org/abs/patt-sol/9304001).
- M. Bramson and D. Griffeath (1989), "Flux and fixation in cyclic particle systems",
  *Annals of Probability* 17, 26-45.
- D. Griffeath (1988), "Cyclic random competition: a case history in experimental
  mathematics", *Notices of the American Mathematical Society*, December, 1472-1480.
- A. K. Dewdney (1989), "Computer recreations: a cellular universe of debris, droplets,
  defects and demons", *Scientific American*, August, 102-105.
- R. Durrett and D. Griffeath (1993), "Asymptotic behavior of excitable cellular
  automata", *Experimental Mathematics* 2, no. 3.
  [PDF](https://sites.math.duke.edu/~rtd/reprints/paper79.pdf).
- C. R. Shalizi and K. L. Shalizi, "Quantifying self-organization in cyclic cellular
  automata", [arXiv:nlin/0507067](https://arxiv.org/abs/nlin/0507067).
- MCell's [Cyclic CA family](https://mcell.ca/pages/families.html#cyclic-ca) and its
  [rule list](https://mcell.ca/app/data/rules.txt).
- The Wikipedia article
  [Cyclic cellular automaton](https://en.wikipedia.org/wiki/Cyclic_cellular_automaton).
