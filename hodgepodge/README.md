# Hodgepodge machine

Every cell holds a number from 0 to *n*, its state, and the states are described as an
illness:

- **0 is healthy.**
- **n is ill.**
- **Anything in between is infected**, and the higher the number, the worse the infection.

All cells update at once, every generation, by looking at their **neighbours**: either the
4 cells that share an edge with them, or those and the 4 that share a corner, 8 in all.
Three rules, one for each kind of cell, use four numbers, *n*, *k1*, *k2* and *g*:

- **A healthy cell** counts its infected neighbours, *A*, and its ill neighbours, *B*, and
  becomes [*A* / *k1*] + [*B* / *k2*], where the brackets mean rounding down. So it catches
  one level of infection for every *k1* infected neighbours, and one for every *k2* ill
  ones. With too few of either around, it stays healthy.
- **An infected cell** adds up the states of itself and its neighbours, divides by the
  number of infected cells among them (itself included), rounds down, and adds *g*. It
  takes on the average infection of its neighbourhood, and then gets worse by *g*. It
  cannot go above *n*: when it reaches *n*, it is ill.
- **An ill cell** becomes healthy at the next generation.

So each cell goes round the same loop: healthy, infected at a level that rises
generation by generation, ill, and healthy again, but it needs infected or ill neighbours
to start. Infected cells drag each other's levels together, so neighbouring cells fall ill
at nearly the same time, and the infection moves across the world in waves. Start from
random states, and with the right numbers the waves organise themselves into rings and
turning **spirals**.

Its designers called it the hodgepodge machine; A. K. Dewdney, who described it in
*Scientific American*, calls the random disorder at the start of a run "the hodgepodge
phase".

The cyclic cellular automaton, another demo in this project, also makes spirals from
random noise, by a different rule. There, a cell steps on to the next of a few colours
only when enough of its neighbours already have that colour, and otherwise waits; nothing
is averaged. Here, an infected cell gets worse every generation whatever its neighbours
do, and its level is pulled towards theirs.

## History

- **A model of a catalyst.** Martin Gerhardt and Heike Schuster of the University
  of Bielefeld, in what was then West Germany, designed the hodgepodge machine to mimic a
  *heterogeneous catalytic reaction*, inspired by the work of the chemists Nils Jaeger and
  Peter Plath of the University of Bremen. In that reaction, carbon monoxide and oxygen
  combine into carbon dioxide on thousands of tiny palladium crystals spread through a
  porous material. The heat of the reaction changes the state of the catalyst, until a
  sudden change in the crystal frees the carbon monoxide on its surface; the catalyst
  cools, and the reaction starts again. Each cell of the grid stands for a particle of
  catalyst, and "infection" for its surface gradually filling up. Neighbouring particles
  affect each other by passing on heat, or carbon monoxide. With their mentor at
  Bielefeld, Andreas W. M. Dress, and with Jaeger and Plath, they published proposals for
  modelling such oscillating catalytic reactions in 1985, and with Dress a paper on
  cellular automata in which oscillators synchronise in 1988. By August 1988 they had also
  enlisted the help of S. C. Müller and John J. Tyson in studying the machine.
- **Waves like a chemical clock.** The machine turned out to mimic the
  *Belousov–Zhabotinsky reaction* just as well. In that reaction malonic acid is oxidised
  by potassium bromate, with cerium or iron as the catalyst, and in a dish it shows rings
  and spirals of colour that travel through the liquid. Dewdney's column of August 1988
  set a photograph of the reaction beside the machine's spirals. Gerhardt and Schuster did
  not claim that the chemistry is a cellular automaton: they saw the machine as a tool of
  approximation, a discrete version of a partial differential equation.
- **Four types of behaviour.** Dewdney reports a sample experiment of theirs: a 20 x 20
  grid, 4 neighbours, *n* = 100, *k1* = 2 and *k2* = 3, a random start, and 10,000
  generations at each value of *g*, recording only how many cells were infected each
  generation. At low *g*, activity died out and the cells became "boringly and
  everlastingly healthy". As *g* rose, four types of behaviour appeared in turn, with some
  overlap:
  - **Type 1:** most cells infected, and staying so, with irregular appearances of
    healthy cells;
  - **Type 2:** plateaus of infection about 30 generations long, broken by large numbers
    of cells becoming healthy;
  - **Type 3:** a very regular alternation between nearly every cell infected and almost
    none, every 20 generations or so;
  - **Type 4:** within a few generations, the share of infected cells fluctuating about a
    steady value, about 75 percent; on large grids, the waves form spirals.
- **1989: Physica D.** Gerhardt and Schuster described the automaton in Physica D in 1989,
  and with Tyson they published cellular automaton models of *excitable media*, which
  carry waves like these, in 1990.
- **Later.** The machine is still used to model the reaction: from 2015 Dalibor Štys and
  colleagues compared modified versions of it with the patterns the reaction forms in a
  Petri dish.

The presets keep the sample's *n* = 100, *k1* = 2 and *k2* = 3, and raise *g*. Dewdney
gives no values of *g*, so the ones here were found by running this demo; each shows what
he describes for one type. They use 8 neighbours: with 4, *k1* = 2 and *k2* = 3, every
random start tried here on Dewdney's 20 x 20 grid died out, at each *g* tried up to 30.
With 4 neighbours and rounding down, a straight front of infection cannot move: the
healthy cell just ahead of it has only 1 infected or 1 ill neighbour, and
[1/2] = [1/3] = 0, so only cells in a concave corner of the front catch the infection.
With *k1* = *k2* = 1 a single neighbour is enough, and the front moves.

The *Spirals* preset holds about 60% of the cells infected, not Dewdney's 75%. Some later
programs swap *k1* and *k2*, dividing the ill neighbours by *k1*; with that order, 8
neighbours and *g* of 19 or 20, the share settled at 71–72% on every start tried here.
This demo keeps Dewdney's order.

## Things to try

- **Spirals.** Run the *Spirals* preset. The noise breaks up into rings and spiral waves,
  and the *infected cells* chart swings wildly at first, then settles to a nearly level
  line: Dewdney's Type 4. Watch a spiral's centre turn.
- **Pulses.** Choose *Pulses*: the whole world beats, filling with infection, falling ill
  and mostly clearing every 14 generations or so, like his Type 3. Compare the chart with
  the steady line of *Spirals*.
- **Plateaus.** Choose *Plateaus*, and watch the chart: long stretches with nearly every
  cell infected, cut short when most of the world falls ill at once.
- **Rings.** Choose *Rings* for rings of infection spreading from many small centres.
- **Too slow to last.** Choose the smallest world and set *g* to 1: within a few hundred
  generations every cell is healthy for good, on every random start tried here. On the
  larger worlds the infection may survive.
- **Start a wave.** Choose *Spirals*, press *Clear*, and drag a short stroke on the empty
  world. Waves spread out from it, one cell a generation, and its centre keeps sending out
  more. Shift-drag through the waves to cut them, and see what grows from the broken ends.
- **Broader spirals.** Set *n* to 200, *k1* and *k2* to 3 and *g* to about 40.
- **4 neighbours.** Choose 4 neighbours with any preset, and the infection usually dies
  out within a few hundred generations. Lower *k1* and *k2* to 1, so that a single
  neighbour can infect, and it lives: with *g* around 15, diamond-shaped waves fill the
  world.
- **Edges.** Untick *Wrap around the edges*, as in Gerhardt and Schuster's sample: the
  cells on the edge then simply have fewer neighbours.

## Further reading

- The Wikipedia article
  [Belousov–Zhabotinsky reaction](https://en.wikipedia.org/wiki/Belousov%E2%80%93Zhabotinsky_reaction),
  on the reaction the machine resembles. Wikipedia has no article on the machine itself.
- A. K. Dewdney, *The hodgepodge machine makes waves*, in his *Computer Recreations*
  column, Scientific American 259, 104–107 (August 1988),
  [doi:10.1038/scientificamerican0888-104](https://doi.org/10.1038/scientificamerican0888-104),
  [PDF](https://people.sc.fsu.edu/~jburkardt/classes/ysp_2003/dewdney_hodgepodge.pdf).
- M. Gerhardt and H. Schuster, *A cellular automaton describing the formation of spatially
  ordered structures in chemical systems*, Physica D 36, 209–221 (1989),
  [doi:10.1016/0167-2789(89)90081-X](https://doi.org/10.1016/0167-2789(89)90081-X).
- M. Gerhardt, H. Schuster and J. J. Tyson, *A cellular automaton model of excitable media
  including curvature and dispersion*, Science 247, 1563–1566 (1990),
  [doi:10.1126/science.2321017](https://doi.org/10.1126/science.2321017).
- D. Štys, T. Náhlík, A. Zhyrova, R. Rychtáriková, Š. Papáček and P. Císař, *Model of the
  Belousov-Zhabotinsky reaction* (2015),
  [arXiv:1507.08783](https://arxiv.org/abs/1507.08783).
