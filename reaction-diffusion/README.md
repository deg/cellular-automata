# Reaction-diffusion

This one is **not a cellular automaton**, but chemistry. Two chemicals, U and V, are spread
through a thin dish. Everywhere in it, all the time:

- **They react.** Where a U meets two Vs, it turns into a third V: U + 2V → 3V. V makes
  more of itself, and uses up U to do it.
- **V decays** into an inert product, at a rate k.
- **The dish is fed.** Fresh U flows in at a rate F, which also washes a little of
  everything out.
- **Both spread out** (diffuse), U twice as fast as V.

That is all, and yet a small square of V in a dish of plain U can grow into spots that
divide like cells, stripes that stop short of each other, waves, or chaos. Which one
depends on the feed rate F and the decay rate k.

It is not a cellular automaton because the chemicals are amounts, not states, and the rule
is a pair of equations for how those amounts change, continuously, in space and time:

> ∂U/∂t = Du ∇²U − UV² + F(1 − U)

> ∂V/∂t = Dv ∇²V + UV² − (F + k)V

UV² is the reaction, F(1 − U) the feed, (F + k)V the loss of V, and the ∇² terms the
diffusion, with Du and Dv the two chemicals' rates of diffusion. The demo computes the
equations on a grid, the way the automata here run: each cell holds an amount of U and of
V, and each step every cell updates from itself and its four neighbours. ∇² becomes the
four neighbours' amounts minus four times the cell's own: how much more the neighbours
hold. Diffusion moves chemical down that difference.

The picture shows U: navy where U = 1, the plain fed dish, through blue and teal to gold
where V has eaten U down to about 0.2.

The name says what happens: chemicals react and diffuse. This model is the Gray–Scott
model, after the chemists Peter Gray and Stephen Scott.

## Pearson's numbers

The demo runs the model as John Pearson did in 1993. His dish is 2.5 units square, on a
256 x 256 grid. U diffuses at Du = 2 × 10⁻⁵ and V at Dv = 10⁻⁵; each step is 1 time unit
long; and the edges wrap around, so the dish has no walls. At the start the whole dish is
plain U (U = 1, V = 0) except a 20 x 20 square in the middle at U = ½, V = ¼, and every
value is nudged by up to 1% at random, "to break the square symmetry".

Each cell here is one of Pearson's grid cells, at every world size: a bigger world is a
bigger dish, so the patterns stay the same size and there are more of them.

## How spots divide

In much of the map, the only uniform state the dish can settle into is plain U. A spot of V
can last there only because it is not uniform: it needs the slopes of U and V at its edge.
So, Pearson argued, a spot has a largest size, since a big one would have a middle with no
slope, which would fall back to plain U. A spot grows until its middle does just that,
and leaves two spots. When spots crowd together, the slopes are too weak for them all,
and a whole patch of them dies at once, leaving a hole that the spots round it fill in a
few thousand steps.

None of this is a Turing pattern, the kind that grows by itself from a uniform state when
one chemical diffuses enough faster than the other. With U only twice as fast as V there
are no stable ones, Pearson noted: about 2.8 times is the least that gives stable ones. His patterns
need a push, like the square of V.

## History

- **1952: Turing.** Alan Turing proposed that chemicals reacting and diffusing could make
  patterns such as stripes from a uniform start, as a theory of morphogenesis, how living
  things develop their shapes: *The chemical basis of morphogenesis*, Philosophical
  Transactions of the Royal Society of London B 237, 37–72 (1952),
  [doi:10.1098/rstb.1952.0012](https://doi.org/10.1098/rstb.1952.0012).
- **1983–1985: Gray and Scott.** Peter Gray and Stephen Scott studied this pair of
  reactions in a continuously fed, stirred tank: stirred, so with no diffusion and no
  pattern in space, only amounts that change in time. Their papers include
  *Autocatalytic reactions in the isothermal, continuous stirred tank reactor*, Chemical
  Engineering Science 38, 29–43 (1983) and 39, 1087–1097 (1984), and *Sustained
  oscillations and other exotic patterns of behavior in isothermal reactions*, Journal of
  Physical Chemistry 89, 22–32 (1985).
- **1987: patterns with equal diffusion.** J. A. Vastano, J. E. Pearson, W. Horsthemke and
  H. L. Swinney showed that in one dimension the model can form steady patterns even when
  U and V diffuse at the same rate: *Chemical pattern formation with equal diffusion
  coefficients*, Physics Letters A 124, 320–324 (1987).
- **1993: Pearson's map.** John Pearson, at Los Alamos, ran the model in two dimensions on
  a Connection Machine, from the square of V, for 200,000 steps at each of dozens of
  settings of F and k. He sorted what he found into twelve patterns, named by Greek letters
  α to μ, and plotted where each appeared: the map in the demo's panel. Among them were
  spots that divide, and chaos, for which he measured how fast two nearly equal dishes
  drift apart. *Complex patterns in a simple system*, Science 261, 189–192 (1993),
  [doi:10.1126/science.261.5118.189](https://doi.org/10.1126/science.261.5118.189).
- **1993: the experiment beside it.** In the same issue, K. J. Lee, W. D. McCormick,
  Q. Ouyang and H. L. Swinney reported a real chemical reaction in a thin gel layer whose
  fronts travelled at a constant speed and then stopped, about 0.4 millimetres apart:
  *Pattern formation by interacting chemical fronts*, Science 261, 192–194 (1993),
  [doi:10.1126/science.261.5118.192](https://doi.org/10.1126/science.261.5118.192).
  Pearson found fronts that stop in his patterns θ, κ and μ.
- **1994: spots in a dish.** Lee, McCormick, Pearson and Swinney then saw spots divide in
  the laboratory: *Experimental observation of self-replicating spots in a
  reaction–diffusion system*, Nature 369, 215–218 (1994),
  [doi:10.1038/369215a0](https://doi.org/10.1038/369215a0). In a longer paper, Lee and
  Swinney describe the reaction (ferrocyanide, iodate and sulfite, in a thin gel fed from
  a stirred reservoir) and its spots, which grow by dividing and die of overcrowding:
  *Lamellar structures and self-replicating spots in a reaction-diffusion system*,
  Physical Review E 51, 1899–1915 (1995),
  [doi:10.1103/PhysRevE.51.1899](https://doi.org/10.1103/PhysRevE.51.1899).
- **Since: on the web.** Robert Munafo extended Pearson's catalogue of patterns, and Karl
  Sims's tutorial calls k the "kill" rate, as the demo's slider does (see Further
  reading).

## Things to try

- **ε, the opening.** The demo opens on ε, at F = 0.02, k = 0.059, the one setting Pearson
  gives in his text. The square spreads out as a ring of spots, and the spots reach the
  far corners after about 10,000 steps (Pearson: "on the order of 10,000–20,000"). After
  that they keep dividing and dying: *change per step* never settles.
- **λ: spots that divide and stop.** Choose *λ*. Watch a spot at the edge of the colony stretch,
  pinch in the middle and become two. The spots reach the far corners after about 20,000
  steps, and then the pattern settles: *change per step* falls towards 0.
- **μ: stripes that grow at their tips.** Choose *μ*. Pearson saw that two tips heading
  for each other turn aside rather than meet.
- **κ and θ: fronts that stop.** Waves spread outwards, and where they meet, they stop.
- **δ: hexagons.** Choose *δ*: the dish fills with spots in a regular hexagonal pattern.
- **The plain places.** Click a coral dot on the map, then *Restart*: the square of V dies
  away, and the dish goes back to plain U, navy. (Except, sometimes, at k = 0.045,
  F = 0.016: there it depends on the random noise.) Click a blue dot and *Restart*, and the
  whole dish turns into the other uniform state, about U = 0.3, V = 0.25. Between them lie
  the patterns.
- **Move a pattern.** The sliders and the map carry on from the dish as it is, so you can
  grow a pattern in one place and move it to another. Pearson's map is for his square, and
  a pattern can outlive it: λ's spots, moved to the coral dot at k = 0.0675, F = 0.04,
  live on.
- **Draw.** Choose *Clear*, then draw V on the empty dish: a line in *μ* grows into a
  long stripe. In *λ*, a small dot settles into a single spot that stays put, and a
  bigger one dies away, while Pearson's square grows into a colony.
- **A bigger dish.** Choose *512 x 512*: the same spots and stripes, four times as many.

## Further reading

- The Wikipedia article [Reaction–diffusion system](https://en.wikipedia.org/wiki/Reaction%E2%80%93diffusion_system).
- J. E. Pearson, *Complex patterns in a simple system*, Science 261, 189–192 (1993),
  [arXiv:patt-sol/9304003](https://arxiv.org/abs/patt-sol/9304003): short, and free to
  read.
- R. Munafo, *Reaction-Diffusion by the Gray-Scott Model: Pearson's Parametrization*,
  [mrob.com](https://mrob.com/pub/comp/xmorphia/): Pearson's patterns and many more.
- K. Sims, *Reaction-Diffusion Tutorial*, [karlsims.com](https://www.karlsims.com/rd.html).
- A. M. Turing, *The chemical basis of morphogenesis*, Philosophical Transactions of the
  Royal Society of London B 237, 37–72 (1952),
  [doi:10.1098/rstb.1952.0012](https://doi.org/10.1098/rstb.1952.0012).
