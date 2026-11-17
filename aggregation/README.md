# Diffusion-limited aggregation

One particle sits in the middle of a grid: the seed of a cluster. Another particle is let
go far away and wanders at random, one step at a time up, down, left or right, until it
lands on a cell **next to** the cluster. There it **sticks**, and the next particle is let
go. That's the whole rule.

The cluster that grows is not a blob but a **branching fractal**, like the copper that
grows on an electrode, or a starving colony of bacteria. A wandering particle almost
always meets a tip of the cluster before it can get deep into the gaps between branches,
so the tips grow and the gaps, the *fjords*, stay open. The outer branches shield the
inner ones.

This isn't quite a cellular automaton, where every cell updates at once: only one particle
moves at a time. The name says what limits the growth: particles reach the cluster only by
diffusing, that is, by a random walk, and the cluster is an *aggregate* of the particles
that have stuck.

## The rule, in detail

- The grid is square. A particle sticks when one of its four nearest neighbours (not a
  diagonal one) belongs to the cluster, so the cluster always stays in one piece.
- With the *sticking probability* below 1, a particle next to the cluster sticks only
  with that chance at each step, and otherwise wanders on.
- With the *line* seed, the whole bottom row is the seed, its left and right ends are
  joined round, and particles come down from above.

The colours show when each particle joined: gold at the seed, then coral, pink and blue,
to white for the newest. Leonard Sander coloured his pictures of DLA the same way.

### The fractal dimension

A disc of radius R holds a number of particles in proportion to R², and a line in
proportion to R. A DLA cluster is in between: the number of particles N grows as R to a
power D of about 1.7, its **fractal dimension**. The chart plots N against the cluster's
radius of gyration (the root mean square distance of its particles from their centre) on
logarithmic scales, where the power law is a straight line of slope D. The demo fits the
line from 100 particles on, and shows its slope.

No one has found D by theory: the only way to know it is to grow clusters and measure.
Susan Tolman and Paul Meakin measured 1.715 ± 0.004 for clusters of up to a million
particles grown off the lattice, and Sander quotes 1.71. On the square lattice the
measured value depends a little on the size: for clusters of 40,000 to 80,000 sites, like
this demo's biggest, Meakin found the radius of gyration growing as N to the power 0.60,
a dimension of about 1.67. Clusters grown here measure about 1.70.

### Making it fast

Wandering at random is slow: most of a walk is spent getting near the cluster, and in two
dimensions a particle can wander very far before coming back. The demo uses three standard
shortcuts. The first two are exact for a random walk in continuous space and very close on
the grid; the third is exact on the grid. The demo's tests compare each with walks that
don't use it, and find no difference in where particles stick:

- Particles start on a circle just outside the cluster, at a random point. A particle
  coming from far away meets that circle first anyway, equally likely at any point.
- A particle that wanders well outside is put straight back on the circle, at a point
  drawn with the chance that it would have come back there.
- A particle with no part of the cluster near it jumps straight to the edge of a square
  around it, at a point drawn with the chance that its walk would have reached that point
  first.

## History

- **1979: Forrest and Witten.** S. R. Forrest and T. A. Witten measured long-range
  correlations in aggregates of smoke particles: *Long-range correlations in smoke-particle
  aggregates*, Journal of Physics A 12, L109–L117 (1979),
  [doi:10.1088/0305-4470/12/5/008](https://doi.org/10.1088/0305-4470/12/5/008). This was
  the experiment Witten and Sander set out to describe, though by Sander's account DLA
  turned out to model a different process.
- **1981: Witten and Sander.** Thomas Witten and Leonard Sander introduced the model, on
  the square lattice, in *Diffusion-limited aggregation, a kinetic critical phenomenon*,
  Physical Review Letters 47, 1400–1403 (1981),
  [doi:10.1103/PhysRevLett.47.1400](https://doi.org/10.1103/PhysRevLett.47.1400). In its
  clusters, as in aggregates of metal particles, density falls off with distance as a
  power law, and the radius of gyration grows as a power of the number of particles.
  Sander's first program let particles go far from the cluster and spent most of its
  time on walks that found nothing. Over dinner, his companion asked why he didn't start
  them on a circle near the cluster (he credits the idea to M. E. Sander), and soon they
  had clusters to look at.
- **1983: other dimensions, sticking, and surfaces.** Paul Meakin grew clusters on
  lattices of two to six dimensions and off the lattice: *Diffusion-controlled cluster
  formation in 2–6-dimensional space*, Physical Review A 27, 1495–1507 (1983),
  [doi:10.1103/PhysRevA.27.1495](https://doi.org/10.1103/PhysRevA.27.1495). He tried
  sticking probabilities from 0.1 to 1: the smaller ones gave denser clusters, with the
  same scaling. In *Diffusion-controlled deposition on fibers and surfaces*, Physical
  Review A 27, 2616–2623 (1983),
  [doi:10.1103/PhysRevA.27.2616](https://doi.org/10.1103/PhysRevA.27.2616), he grew them
  on lines and surfaces.
- **1984: DLA in nature.** Robert Brady and Robin Ball grew fractal deposits of copper:
  *Fractal growth of copper electrodeposits*, Nature 309, 225–229 (1984),
  [doi:10.1038/309225a0](https://doi.org/10.1038/309225a0). In Sander's account, they
  grew it on the end of a thin wire in diffusion-limited conditions, and it looked
  self-similar over five orders of magnitude in size. The same year Lincoln
  Paterson linked DLA to viscous fingering, where one fluid pushed into another through
  a porous medium branches the same way: *Diffusion-limited aggregation and two-fluid
  displacements in porous media*, Physical Review Letters 52, 1621–1624 (1984),
  [doi:10.1103/PhysRevLett.52.1621](https://doi.org/10.1103/PhysRevLett.52.1621). L.
  Niemeyer, L. Pietronero and H. J. Wiesmann modelled the branching patterns of
  electrical breakdown, and by Sander's account found real ones to have DLA's fractal
  dimension: *Fractal dimension of dielectric breakdown*, Physical Review Letters 52,
  1033–1036 (1984),
  [doi:10.1103/PhysRevLett.52.1033](https://doi.org/10.1103/PhysRevLett.52.1033). In 1989
  Hiroshi Fujikawa and Mitsugu Matsushita grew colonies of the bacterium *Bacillus
  subtilis* with random branches like DLA's, of fractal dimension 1.716 ± 0.008, whose
  inner branches stopped growing: *Fractal growth of Bacillus subtilis on agar plates*,
  Journal of the Physical Society of Japan 58, 3875–3878 (1989),
  [doi:10.1143/JPSJ.58.3875](https://doi.org/10.1143/JPSJ.58.3875).
- **1985–1987: the lattice shows.** By Sander's account, small clusters grown on the
  square and triangular lattices looked alike: under about 100,000 particles, the growth
  appeared to ignore the lattice. Then Ball and Brady's hierarchical maps of the cluster
  made much bigger clusters possible, and lattice effects began to show: R. C. Ball and
  R. M. Brady, *Large scale lattice effect in diffusion-limited aggregation*, Journal of
  Physics A 18, L809–L813 (1985),
  [doi:10.1088/0305-4470/18/13/014](https://doi.org/10.1088/0305-4470/18/13/014). Growing
  hundreds of clusters of 40,000 to 80,000 sites, Meakin found the square lattice's
  exponent already differing from the triangular and hexagonal lattices', which matched
  off-lattice DLA: *Universality, nonuniversality, and the effects of anisotropy on
  diffusion-limited aggregation*, Physical Review A 33, 3371–3382 (1986),
  [doi:10.1103/PhysRevA.33.3371](https://doi.org/10.1103/PhysRevA.33.3371). At about a
  million particles, square-lattice clusters stretch their arms out along the lattice's
  axes: Paul Meakin, Robin Ball, P. Ramanlal and Leonard Sander, *Structure of large
  two-dimensional square-lattice diffusion-limited aggregates: Approach to asymptotic
  behavior*, Physical Review A 35, 5233–5239 (1987),
  [doi:10.1103/PhysRevA.35.5233](https://doi.org/10.1103/PhysRevA.35.5233). This demo's
  biggest world holds a few tens of thousands.
- **1989: the dimension.** Susan Tolman and Paul Meakin, *Off-lattice and
  hypercubic-lattice models for diffusion-limited aggregation in dimensionalities 2–8*,
  Physical Review A 40, 428–437 (1989),
  [doi:10.1103/PhysRevA.40.428](https://doi.org/10.1103/PhysRevA.40.428), measured 1.715 ±
  0.004 in two dimensions.
- **2000: still open.** By Sander's review the record stood at about 30,000,000
  particles, and there was still no satisfactory theory of DLA. In 2021 Eviatar and Itamar
  Procaccia found the exact dimension of DLA grown on a line, off the lattice: 3/2, in
  *Dimension of diffusion-limited aggregates grown on a line*, Physical Review E 103,
  L020101 (2021),
  [doi:10.1103/PhysRevE.103.L020101](https://doi.org/10.1103/PhysRevE.103.L020101),
  [arXiv:2010.05709](https://arxiv.org/abs/2010.05709).

## Things to try

- **Watch the tips.** Almost every new particle, in white, joins near a tip. Hover over a
  fjord: the particles at its bottom joined long ago, before the branches around them
  grew past.
- **Measure the dimension.** Choose the *1000 x 1000* world and let a cluster grow to the
  edge. The slope wanders at first and settles near 1.7. *Restart* a few times: each
  cluster gives a slightly different slope, and their average is close to 1.70.
- **Lower the sticking probability** to 0.1 and *Restart*. Particles get further into the
  fjords before they stick, and the branches grow thick. Meakin found that it changes the
  density, but not the dimension, of large clusters.
- **Grow a forest.** Choose the *line* seed. Trees start everywhere, but the tall ones
  catch the particles and the short ones stop growing. With the ends joined round, the
  Procaccias point out, a single tree would in the end take all the growth.
- **Compare with the snowflakes**, another demo in this project. A snow crystal grows
  from a seed too, but its vapour diffuses everywhere at once, by a rule with no
  randomness, so its six arms grow alike. Here each particle's walk is random, and no two
  clusters are the same.

## Further reading

- The Wikipedia article
  [Diffusion-limited aggregation](https://en.wikipedia.org/wiki/Diffusion-limited_aggregation).
- Leonard M. Sander, *Diffusion-limited aggregation: a kinetic critical phenomenon?*,
  Contemporary Physics 41, 203–218 (2000),
  [doi:10.1080/001075100409698](https://doi.org/10.1080/001075100409698): a personal
  review by one of its inventors, with pictures, examples from nature, and how to simulate
  it fast.
- T. A. Witten and L. M. Sander, *Diffusion-limited aggregation*, Physical Review B 27,
  5686–5697 (1983), [doi:10.1103/PhysRevB.27.5686](https://doi.org/10.1103/PhysRevB.27.5686):
  the longer paper that followed the first.
