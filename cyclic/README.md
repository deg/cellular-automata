# Cyclic cellular automaton

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
once, or churns forever without settling. The name comes from the wheel: Robert Fisch,
Janko Gravner and David Griffeath, who studied the rule, described each colour as a
creature in a "cyclic food chain" that can eat only the colour just before it.

## History

- **1988–1989: random cyclic competition.** The automaton grew out of a random version,
  in which each site picks a neighbour at random and eats it if it can. In 1989 Bramson
  and Griffeath proved that in one dimension it settles into fixed colours when there are
  5 or more colours, but forms ever-growing patches of one colour with 3 or 4. In two
  dimensions it was a mystery until Fisch and Griffeath simulated it on the *Cellular
  Automaton Machine* (CAM), a plug-in board for running automata quickly, designed by
  Toffoli and Margolus. To their surprise, the screen filled with large spiral waves, as
  Griffeath reported in 1988.
- **1989–1991: the cyclic cellular automaton.** Hoping a rule without randomness would be
  easier to analyse, Fisch, Gravner and Griffeath turned to the deterministic version:
  range 1, diamond neighbourhood, threshold 1, the *CCA* preset here. They proved that
  once a *clock* exists (a loop of cells described below), every cell eventually cycles
  through all the colours, one step per generation, however many colours there are.
  A. K. Dewdney described their work in his *Computer Recreations* column in Scientific
  American in August 1989, and their paper appeared in 1991. With 14 colours on a
  256 x 256 grid, they saw four stages, named with four Ds:
  - **Debris.** Almost every cell soon has nothing to eat, and stops: random debris.
  - **Droplets.** A few active patches survive, and grow as droplets of waves that spread
    until they have eaten all the debris.
  - **Defects.** Where droplets meet the debris and each other, loops of cells form
    whose colours wind round the whole colour wheel, and spirals grow around them.
  - **Demons.** The most efficient spirals have at their centre a *clock*: a loop of cells
    on which each colour has its next colour beside it, so it advances every generation
    whatever happens around it. A clock and the cells it drives make a *demon*. Demons
    slowly take over from the other spirals, until every cell of the grid cycles through
    all 14 colours, one step per generation.
- **1991: range and threshold.** In a second paper, Fisch, Gravner and Griffeath added the
  range, the threshold, and the box and diamond neighbourhoods, and mapped what different
  rules do from random noise, using a program they wrote for PCs, *EXCITE!*. When there
  are many colours, it is mostly the threshold divided by the square of the range that
  decides between three behaviours. Among their examples are the rules here named
  *Perfect*, *Turbulence*, *Bootstrap fixation* and *Lava lamp* (their "lava light"). The
  three behaviours:
  - with a low threshold, waves bend round into **spirals**, and the world becomes
    periodic: every cell ends up repeating a cycle of colours forever;
  - with a higher threshold, would-be spirals keep unravelling, leaving debris that seeds
    new waves: a state they called **turbulence**, which never settles;
  - with a higher one still, the world **fixates**: it stops changing for good.
- **Why it mattered.** These rules are simple mathematical stand-ins for *excitable media*,
  through which waves of activity travel and turn into spirals. The papers list spiral
  waves in oscillating chemical reactions (the Belousov-Zhabotinsky reaction), in heart
  muscle during atrial fibrillation, and in nerve networks. Fisch, Gravner and Griffeath
  offered the cyclic automaton as a model simple enough to prove things about. A closely
  related rule, the *Greenberg-Hastings model*, differs only in that every colour except 0
  advances automatically.

Most presets take their names from the rule list of **MCell**, Mirek Wójtowicz's program
for exploring cellular automata, which writes a rule as, for example, `R1/T3/C4/NM`: range
1, threshold 3, 4 colours, Moore (box) neighbourhood (`NN` is von Neumann). The panel shows
each rule in this notation, and who published it or added it to MCell's list.

## Things to try

- **The four Ds.** Choose *CCA* and run fast. First the noise freezes into debris, then
  droplets of waves eat it, then spirals appear and grow. Leave it running: the
  *changed last generation* count climbs until every cell changes every generation.
- **More colours.** Fisch, Gravner and Griffeath found that with 20 colours the droplets
  become distinct and "drop-like". Set *colours* to 20 with the CCA rule, on the largest
  world. They expected a 256 x 256 world to fixate once there are more than about 20
  colours, because droplets that survive get too rare to appear: on the middle-sized
  world, some random starts already freeze with 20.
- **Perfect spirals.** Choose *Perfect*, which switches to the largest world. It churns
  for a while (MCell says "often after more than one hundred updates") before perfect
  spiral cores appear and slowly take over. On smaller worlds they seldom appear.
- **Turn up the threshold.** Shalizi and Shalizi describe four colours on the range-1 box:
  threshold 1 gives a jumble of small oscillations, 2 gives spiral waves, 3 turbulence (in
  which spirals may still win in the end), and 4 or more fixation. Choose *Imperfect*
  (threshold 2), then try 1, 3 and 4.
- **Spirals overtaken.** Choose *Imperfect* and run fast. Fisch, Gravner and Griffeath
  found that in this rule spirals can rule for a long time, their cells taking some
  multiple of 4 generations to go once round the 4-colour wheel. Then a small structure
  that cycles on its own suddenly appears, and quickly takes over from the spirals. Watch
  *changed last generation*: while spirals rule it stays well below the whole world; once
  the new structures have taken over, every cell changes every generation.
- **Turbulence.** Choose *Turbulence*: waves form, break up, and leave debris that starts
  new ones, forever. Fisch, Gravner and Griffeath gave two representative turbulent rules:
  this one (range 2, diamond, threshold 4, 5 colours) and range 2, box, threshold 5, 8
  colours (MCell's *Turbulent phase*). Turbulence works on large length scales, so a
  world that is too small freezes: they found the second rule often fixates on a
  256 x 256 grid but kept going on 512 x 512. In this demo it froze on 2 of 10 random
  starts even at 640 x 400, so the preset is the first rule, which keeps going from
  320 x 200 up (on 160 x 100 it freezes too). Set the second by hand on the largest world
  to compare.
- **Freezing.** Choose *Bootstrap fixation* and watch the noise freeze. Then drag across
  it to paint fresh noise and see whether anything starts up again.
- **Paint.** Drag on any running world to paint random colours; shift-drag paints a patch
  of one colour. See what the waves do to each.
- **Range.** Choose *Cyclic spirals* (range 3), then raise the range and the threshold
  together.

## Further reading

- The Wikipedia article
  [Cyclic cellular automaton](https://en.wikipedia.org/wiki/Cyclic_cellular_automaton).
- R. Fisch, J. Gravner and D. Griffeath, *Cyclic cellular automata in two dimensions*, in
  Spatial Stochastic Processes, a festschrift for T. E. Harris, Birkhäuser,
  pp. 171–185 (1991), [PDF](https://www.math.ucdavis.edu/~gravner/papers/cca.pdf).
- R. Fisch, J. Gravner and D. Griffeath, *Threshold-range scaling of excitable cellular
  automata*, Statistics and Computing 1, pp. 23–39 (1991),
  [PDF](https://www.math.ucdavis.edu/~gravner/papers/fgg.pdf),
  [arXiv:patt-sol/9304001](https://arxiv.org/abs/patt-sol/9304001).
- M. Bramson and D. Griffeath, *Flux and fixation in cyclic particle systems*, Annals of
  Probability 17, pp. 26–45 (1989).
- D. Griffeath, *Cyclic random competition: a case history in experimental mathematics*,
  Notices of the American Mathematical Society, December, pp. 1472–1480 (1988).
- A. K. Dewdney, *A cellular universe of debris, droplets, defects and demons*, in his
  *Computer Recreations* column, Scientific American, August, pp. 102–105 (1989).
- R. Durrett and D. Griffeath, *Asymptotic behavior of excitable cellular automata*,
  Experimental Mathematics 2, no. 3 (1993),
  [PDF](https://sites.math.duke.edu/~rtd/reprints/paper79.pdf).
- C. R. Shalizi and K. L. Shalizi, *Quantifying self-organization in cyclic cellular
  automata*, in Noise in Complex Systems and Stochastic Dynamics, Proceedings of SPIE
  5114 (2003), [doi:10.1117/12.485805](https://doi.org/10.1117/12.485805); a fuller
  version (2005), [arXiv:nlin/0507067](https://arxiv.org/abs/nlin/0507067).
- MCell's [Cyclic CA family](https://mcell.ca/pages/families.html#cyclic-ca) and its
  [rule list](https://mcell.ca/app/data/rules.txt).
