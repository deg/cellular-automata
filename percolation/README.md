# Percolation

Every cell of a square grid is **open** with probability *p*, or **closed**, each one
independently of the others. Open cells that share an edge (not just a corner) belong to
the same **cluster**. That is the whole model: site percolation on the square lattice.

The question is how big the clusters get. With *p* small they are specks. As *p* rises
they grow and join, and at a **threshold** the change is sudden: in the limit of an
endless grid, a cluster forms that reaches from one side to the other. On a finite grid
the sign of it is a cluster that **spans** the grid, joining the top row to the bottom
row. For the square lattice the threshold is known only numerically:
*p* = 0.59274605079210, give or take 2 in the last place.

This is not a cellular automaton: no cell changes by a rule of its neighbours. Each cell
draws its own random number, once, and is open while *p* is above it. So the demo can raise *p* slowly
with the numbers fixed, and the grid only fills in: you watch the clusters grow and
merge, until the gold one, the largest, reaches across.

The name is from Simon Broadbent and John Hammersley, who in 1957 called a fluid's spread
through a random medium a *percolation process* when the randomness lies in the medium,
not in the fluid.

## The threshold on a finite grid

On a grid of finite size the threshold is blurred. A grid can span below the threshold,
or fail to span above it: the chance that it spans rises smoothly from 0 to 1 over a band
of *p* around the threshold. The band narrows as the grid grows, its width in proportion
to *L*^(−3/4) on an *L* x *L* grid.

On a square grid, the chance of spanning at the threshold tends to exactly 1/2 as the
grid grows. Robert Ziff confirmed this in 1992, and the same year John Cardy derived the
chance of crossing a rectangle at the threshold, which depends on the rectangle's shape
but not its size. That's why the worlds here are square.

Stephan Mertens computed the chance of spanning exactly for every square grid up to
24 x 24. Half of all 16 x 16 grids span by *p* = 0.58989, and half of all 24 x 24 grids by
0.59128: still a little short of the threshold, which they approach from below as the
grids grow.

## History

- **1957: Broadbent and Hammersley.** They contrasted two ways a fluid can spread through
  a medium. In a **diffusion** process the randomness belongs to the fluid, as in a random
  walk. In a **percolation** process it belongs to the medium, and once the medium is
  drawn, it alone decides where the fluid goes. Their medium was a *random maze*: a
  lattice with some of its connections dammed at random. Among their examples were a gas
  soaking into a porous solid, and blight spreading from tree to tree in an orchard
  planted on a square lattice, which stays contained only if the chance of passing it on
  is below a critical value.
- **1963–1975: early estimates.** Computer simulations of site percolation on the
  square lattice put the threshold at 0.569 ± 0.013 in 1963, on a 78 x 78 grid, and at
  0.593 ± 0.001 by 1975, on 256 x 256 grids.
- **1976: labelling clusters.** Hoshen and Kopelman published an algorithm for labelling
  the clusters of a grid.
- **1980: the bonds.** Harry Kesten proved that when the *connections* between
  neighbouring points of the square lattice are open at random, instead of the points
  themselves (bond percolation), the threshold is exactly 1/2. For the sites, as here, no
  exact value is known.
- **1992: crossing a square.** Ziff confirmed that a square grid spans with chance 1/2 at
  the threshold, in agreement with universality, the idea that such quantities don't
  depend on the lattice's details. Cardy derived the chance of crossing at the threshold
  for any rectangle.
- **2000: one cell at a time.** Newman and Ziff found how to follow the clusters over
  every *p* from 0 to 1 in a single run, in time proportional to the number of sites.
  Others had already added the sites one at a time in random order; Newman and Ziff
  joined each new site to its neighbours' clusters with a *union-find* structure, which
  makes each step take almost constant time. This demo works that way. With it they put
  the threshold at 0.59274621 ± 0.00000013.
- **2002: forest fires.** Peter Grassberger simulated the forest-fire model, another demo
  in this project, on grids of up to 65,536 x 65,536, and conjectured that the density of
  its forest in the critical state is this threshold, that of site percolation in
  two dimensions: a density still far beyond what simulations could reach.
- **2015–2022: more digits.** Jesper Jacobsen found the threshold to 14 decimal places,
  the value quoted above, by extrapolating exact calculations on small grids. Mertens's
  exact spanning probabilities, in 2022, gave estimates close to his.

Percolation's threshold is a critical point reached by tuning a parameter, here *p*, to
one exact value. Grassberger contrasted points like that with *self-organised
criticality*, in which a system drives itself into a critical state, and named the
sandpile, another demo in this project, as its "paradigmatic model".

## Things to try

- **The opening.** The grid fills in from empty. Up to about *p* = 0.5 the clusters are
  small, scattered among the closed cells. Then they join faster and faster, the gold
  cluster takes in the others, and somewhere near 0.59 it reaches from top to bottom.
  The chart's line, its share of the grid, stays near zero until close to the gold line,
  then climbs steeply.
- **Where does this grid span?** The *spans* row gives the *p* at which it did. *Restart*,
  then *Run*, several times and compare: the values scatter around the threshold. On a 100 x 100
  world they scatter widely; on 800 x 800 much less.
- **Step through the threshold.** *Pause* at about 0.58 and *Step*. Most steps change
  little, but now and then one cell joins two big clusters, and the largest jumps.
- **Click a cluster.** Just below the threshold, click the big clusters that are not gold:
  they are nothing like blobs, but long and branched, full of holes and inlets.
- **Above the threshold.** Drag *p* to 0.7: nearly every open cell is gold, and the other
  clusters are small ones, shut in the holes of the big one. Drag it back down, and the
  same grid comes apart in the same places.

## Further reading

- The Wikipedia articles [Percolation theory](https://en.wikipedia.org/wiki/Percolation_theory)
  and [Percolation threshold](https://en.wikipedia.org/wiki/Percolation_threshold), whose
  tables list the thresholds of many lattices.
- S. R. Broadbent and J. M. Hammersley, *Percolation processes: I. Crystals and mazes*,
  Mathematical Proceedings of the Cambridge Philosophical Society 53, 629–641 (1957),
  [doi:10.1017/S0305004100032680](https://doi.org/10.1017/S0305004100032680).
- M. E. J. Newman and R. M. Ziff, *Efficient Monte Carlo algorithm and high-precision
  results for percolation*, Physical Review Letters 85, 4104–4107 (2000),
  [doi:10.1103/PhysRevLett.85.4104](https://doi.org/10.1103/PhysRevLett.85.4104),
  [arXiv:cond-mat/0005264](https://arxiv.org/abs/cond-mat/0005264).
- M. E. J. Newman and R. M. Ziff, *Fast Monte Carlo algorithm for site or bond
  percolation*, Physical Review E 64, 016706 (2001),
  [doi:10.1103/PhysRevE.64.016706](https://doi.org/10.1103/PhysRevE.64.016706),
  [arXiv:cond-mat/0101295](https://arxiv.org/abs/cond-mat/0101295). The method in detail.
- R. M. Ziff, *Spanning probability in 2D percolation*, Physical Review Letters 69,
  2670–2673 (1992),
  [doi:10.1103/PhysRevLett.69.2670](https://doi.org/10.1103/PhysRevLett.69.2670).
- J. L. Cardy, *Critical percolation in finite geometries*, Journal of Physics A 25,
  L201–L206 (1992),
  [doi:10.1088/0305-4470/25/4/009](https://doi.org/10.1088/0305-4470/25/4/009),
  [arXiv:hep-th/9111026](https://arxiv.org/abs/hep-th/9111026).
- S. Mertens, *Exact site-percolation probability on the square lattice*, Journal of
  Physics A 55, 334002 (2022),
  [doi:10.1088/1751-8121/ac4195](https://doi.org/10.1088/1751-8121/ac4195),
  [arXiv:2109.12102](https://arxiv.org/abs/2109.12102).
- J. L. Jacobsen, *Critical points of Potts and O(N) models from eigenvalue identities in
  periodic Temperley–Lieb algebras*, Journal of Physics A 48, 454003 (2015),
  [doi:10.1088/1751-8113/48/45/454003](https://doi.org/10.1088/1751-8113/48/45/454003),
  [arXiv:1507.03027](https://arxiv.org/abs/1507.03027).
- H. Kesten, *The critical probability of bond percolation on the square lattice equals
  1/2*, Communications in Mathematical Physics 74, 41–59 (1980),
  [doi:10.1007/BF01197577](https://doi.org/10.1007/BF01197577).
- J. Hoshen and R. Kopelman, *Percolation and cluster distribution. I. Cluster multiple
  labeling technique and critical concentration algorithm*, Physical Review B 14,
  3438–3445 (1976),
  [doi:10.1103/PhysRevB.14.3438](https://doi.org/10.1103/PhysRevB.14.3438).
- P. Grassberger, *Critical behaviour of the Drossel-Schwabl forest fire model*, New
  Journal of Physics 4, 17 (2002),
  [doi:10.1088/1367-2630/4/1/317](https://doi.org/10.1088/1367-2630/4/1/317),
  [arXiv:cond-mat/0202022](https://arxiv.org/abs/cond-mat/0202022).
