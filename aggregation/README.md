# Diffusion-limited aggregation

One particle sits in the middle of a grid: the seed of a cluster. Another particle is let
go far away and wanders at random, one step at a time up, down, left or right, until it
lands on a cell **next to** the cluster. There it **sticks**, and the next particle is let
go. That's the whole rule.

The cluster that grows is not a blob but a **branching fractal**, like the copper that
grows on an electrode, or a starving colony of bacteria. A wandering particle almost always meets a
tip of the cluster before it can get deep into the gaps between branches, so the tips
grow and the gaps, the *fjords*, stay open. The outer branches shield the inner ones.

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
particles grown off the lattice, and Sander quotes 1.71.

### Making it fast

Wandering at random is slow: most of a walk is spent getting near the cluster, and in two
dimensions a particle can wander very far before coming back. The demo uses three standard
shortcuts, none of which changes where particles stick (its tests compare each with the
plain walk):

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
  [doi:10.1103/PhysRevLett.47.1400](https://doi.org/10.1103/PhysRevLett.47.1400). Its
  clusters' density falls off with distance as a power law, as in metal-particle
  aggregates, and so does their radius of gyration. Sander's first program let
  particles go far from the cluster and spent most of its time on walks that found
  nothing. Over dinner, his companion asked why he didn't start them on a circle near
  the cluster (he credits the idea to M. E. Sander), and soon they had clusters to look
  at.
- **1983: other dimensions, sticking, and surfaces.** Paul Meakin grew clusters on
  lattices of two to six dimensions and off the lattice: *Diffusion-controlled cluster
  formation in 2–6-dimensional space*, Physical Review A 27, 1495–1507 (1983),
  [doi:10.1103/PhysRevA.27.1495](https://doi.org/10.1103/PhysRevA.27.1495). He tried
  sticking probabilities from 0.1 to 1: the smaller ones gave denser clusters, with the
  same scaling. In *Diffusion-controlled deposition on fibers and surfaces*, Physical
  Review A 27, 2616–2623 (1983),
  [doi:10.1103/PhysRevA.27.2616](https://doi.org/10.1103/PhysRevA.27.2616), he grew them
  on lines and surfaces.
- **1984: DLA in nature.** Robert Brady and Robin Ball grew copper on the end of a thin
  wire in diffusion-limited conditions and found it self-similar over five orders of
  magnitude in size: *Fractal growth of copper electrodeposits*, Nature 309, 225–229
  (1984), [doi:10.1038/309225a0](https://doi.org/10.1038/309225a0). The same year Lincoln
  Paterson linked DLA to viscous fingering, where one fluid pushed into another through
  a porous medium branches the same way (Physical Review Letters 52, 1621–1624), and L.
  Niemeyer, L. Pietronero and H. J. Wiesmann found DLA's fractal dimension in the
  branching patterns of electrical breakdown (Physical Review Letters 52, 1033–1036). In 1989 Hiroshi Fujikawa and Mitsugu
  Matsushita found that colonies of the bacterium *Bacillus subtilis*, short of food, grow
  like DLA: *Fractal growth of Bacillus subtilis on agar plates*, Journal of the Physical
  Society of Japan 58, 3875–3878 (1989).
- **1985–1987: the lattice shows.** Ball and Brady's hierarchical maps of the cluster made
  much bigger clusters possible. On the square lattice, clusters of under about 100,000
  particles looked like those grown off it. At about a million, their arms stretch out
  along the lattice's axes: Paul Meakin, Robin Ball, P. Ramanlal and Leonard Sander,
  *Structure of large two-dimensional square-lattice diffusion-limited aggregates:
  Approach to asymptotic behavior*, Physical Review A 35, 5233–5239 (1987),
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
  *The dimension of diffusion limited aggregates grown on a line*, Physical Review E 103,
  L020101 (2021), [arXiv:2010.05709](https://arxiv.org/abs/2010.05709).

## Things to try

- **Watch the tips.** Almost every new particle, in white, joins near a tip. Hover over a
  fjord: the particles at its bottom joined long ago, before the branches around them
  grew past.
- **Measure the dimension.** Choose the *1000 x 1000* world and let a cluster grow to the
  edge. The slope wanders at first and settles near 1.7. *Restart* a few times: each
  cluster gives a slightly different slope, and their average is close to 1.71.
- **Lower the sticking probability** to 0.1 and *Restart*. Particles get further into the
  fjords before they stick, and the branches grow thick. Meakin found that it changes the
  density, but not the dimension, of large clusters.
- **Grow a forest.** Choose the *line* seed. Trees start everywhere, but the tall ones
  catch the particles and the short ones stop growing. With the ends joined round, the
  Procaccias point out, a single tree would in the end take all the growth.
- **Compare with the snowflakes**, another demo here. A snow crystal grows from a seed too,
  but its vapour diffuses everywhere at once, by a rule with no randomness, so its six arms
  grow alike. Here each particle's walk is random, and no two clusters are the same.

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
