# Forest fire

Every site of a square grid is **empty**, holds a **tree**, or holds a **burning tree**.
Trees grow, lightning strikes, and fire spreads:

- A burning tree burns out, leaving an empty site.
- A tree catches fire if one of its **nearest neighbours** (the four sites across an edge)
  is burning.
- A tree with no burning neighbour is struck by lightning, and catches fire, with a small
  probability *f*.
- An empty site grows a tree with probability *p*.

That is the model as Barbara Drossel and Franz Schwabl defined it in 1992. Its interest
lies in a limit: lightning much rarer than growth (*f* much smaller than *p*), and every
fire burning out long before new trees grow where it passed. Then nothing else happens
while a fire burns, and the only number left is the ratio *p*/*f*, the number of trees
grown between one lightning and the next. So the demo runs the model as the papers
simulate it: pick *p*/*f* sites at random and grow a tree on each one that is empty; then
strike one random site, and if it holds a tree, burn the whole **cluster** of trees
joined to it edge to edge. The fire spreads one ring of neighbours per step, so you can
watch it, and no tree grows until it is out.

The **size** of a fire is the number of trees it burns. In the steady state, fires burn
as many trees as grow, on average (1 − *ρ*) × *p*/*f* per lightning, where *ρ* is the
fraction of sites holding trees. But the average hides the point of the model: fires of
**every size** happen, from a single tree to a large part of the forest. Over a wide range
their sizes follow a **power law**: the share of fires burning *s* trees falls like
*s*^(1 − *τ*), with *τ* about 2.14. There is no typical fire, only a cut-off far out,
which moves further as *p*/*f* grows. The forest drives itself into this state: no
parameter is tuned to make it critical. This is **self-organised criticality**, the idea
behind the sandpile, another demo in this project.

Lightning strikes a site at random, so a big cluster is more likely to be hit than a
small one, in proportion to its size. That is why the fire sizes fall more slowly than
the cluster sizes do: as *s*^(1 − *τ*) rather than *s*^(−*τ*).

The name is the picture the rules paint. Per Bak, Kan Chen and Chao Tang, who published
the first version, titled their paper *A forest-fire model and some thoughts on
turbulence*.

## History

- **1987: self-organised criticality.** Per Bak, Chao Tang and Kurt Wiesenfeld proposed
  that some systems drive themselves to a critical point, where events of every size
  happen, with a sandpile as their example.
- **1990: Bak, Chen and Tang.** Their forest-fire model had no lightning: trees grow
  slowly, and fire spreads from burning trees to their neighbours but never starts on its
  own. Peter Grassberger and Holger Kantz showed in 1991 that as growth slows, its fire
  fronts become more and more regular, turning into spirals, and that in two dimensions
  the model is not critical.
- **1989–1992: lightning.** The model with lightning was introduced independently by
  Christopher Henley, in 1989, and by Drossel and Schwabl, in 1992. Henley's was already
  the extreme limit used here, in which a struck cluster burns at once. Drossel and
  Schwabl, introducing the lightning probability *f*, argued that the forest becomes critical as *f* goes to 0, with the time
  scales of growing and burning far apart. They derived scaling laws for the fires and
  checked them with simulations.
- **1993–1996: the exponents.** Drossel and Schwabl's simulations had shown the simple
  scaling of mean-field theory. Grassberger's of 1993, which introduced the method of
  growing *p*/*f* trees between lightnings and burning a cluster at once, and the others
  that followed, found something else: *τ* about 2.15 in two dimensions, and a forest
  density of about 0.408. In one dimension, Drossel, Siegfried Clar and Schwabl solved the
  model exactly in 1993, with *τ* = 2. Clar, Drossel and Schwabl's review of 1996
  collected the results: *τ* = 2.14 ± 0.03 in two dimensions.
- **2002: a power law in doubt.** Grassberger ran the model on grids of up to
  65,536 x 65,536, at *p*/*f* up to 256,000. Where his runs overlapped the earlier ones,
  they agreed; beyond them, the density went on rising and the apparent power laws
  drifted. He concluded that "all proposed scaling laws seem to be just transient", and
  conjectured that the density tends, very slowly, to 0.5927..., the density at which a
  random forest first has a cluster spanning it. By his "very crude estimate", reaching it
  would take *p*/*f* around 10^40.

## Things to try

- **The opening.** The forest grows from empty, and while its clusters are small,
  lightning burns only a few trees at a time. Once the density passes about 0.5, the
  clusters join into big ones, and a burst of large fires, some of them tens of thousands
  of trees, burns the forest back. The chart waits until the forest has burnt and grown
  back many times over, so that it shows the steady state, not the opening. Raise the
  speed to fill it.
- **The power law.** Let the chart fill. The dots follow the line, whose slope
  1 − *τ* = −1.14 uses the *τ* measured in the 1990s, but not exactly. Up to a few
  hundred trees they fall a little more steeply: Grassberger found the small fires falling
  faster than *s*^(−1.19), and estimated *τ* = 2.19 ± 0.01. Then, just before the cut-off
  where they drop away, they rise above the line in a bump, which he found growing as
  *p*/*f* grows. Choose a bigger *p*/*f* (it needs a bigger world): the cut-off moves out.
- **The mean density.** Grassberger measured it after each fire, before new trees grow,
  as the *mean density* row does. His values, for *p*/*f* from 125 to 4,000: 0.3798,
  0.3876, 0.3934, 0.3976, 0.4006 and 0.4027. His worlds were 1,024 to 16,384 sites
  across; on a smaller one the density comes out a little lower.
- **Patches.** Grassberger observed that the steady forest is made of patches of roughly
  even density, most of them far below or far above the density at which a fire can cross
  them. Pause, and look for them: thick stands that have not burnt for a long time, and
  thin ones that burnt recently. Click a tree in each to light it, and compare the fires.
- **Watch a fire spread.** Set the speed to a few steps a second and watch a fire ring by
  ring: the front stops wherever it runs out of neighbouring trees.

## Further reading

- The Wikipedia article [Forest-fire model](https://en.wikipedia.org/wiki/Forest-fire_model).
- B. Drossel and F. Schwabl, *Self-organized critical forest-fire model*, Physical Review
  Letters 69, 1629–1632 (1992),
  [doi:10.1103/PhysRevLett.69.1629](https://doi.org/10.1103/PhysRevLett.69.1629).
- S. Clar, B. Drossel and F. Schwabl, *Forest fires and other examples of self-organized
  criticality*, Journal of Physics: Condensed Matter 8, 6803–6824 (1996),
  [doi:10.1088/0953-8984/8/37/004](https://doi.org/10.1088/0953-8984/8/37/004),
  [arXiv:cond-mat/9610201](https://arxiv.org/abs/cond-mat/9610201). A review.
- P. Grassberger, *Critical behaviour of the Drossel-Schwabl forest fire model*, New
  Journal of Physics 4, 17 (2002),
  [doi:10.1088/1367-2630/4/1/317](https://doi.org/10.1088/1367-2630/4/1/317),
  [arXiv:cond-mat/0202022](https://arxiv.org/abs/cond-mat/0202022).
- P. Bak, K. Chen and C. Tang, *A forest-fire model and some thoughts on turbulence*,
  Physics Letters A 147, 297–300 (1990),
  [doi:10.1016/0375-9601(90)90451-S](https://doi.org/10.1016/0375-9601(90)90451-S).
- P. Grassberger and H. Kantz, *On a forest fire model with supposed self-organized
  criticality*, Journal of Statistical Physics 63, 685–700 (1991),
  [doi:10.1007/BF01029205](https://doi.org/10.1007/BF01029205).
- P. Grassberger, *On a self-organized critical forest-fire model*, Journal of Physics A
  26, 2081–2089 (1993),
  [doi:10.1088/0305-4470/26/9/007](https://doi.org/10.1088/0305-4470/26/9/007).
- C. L. Henley, *Statics of a "self-organized" percolation model*, Physical Review
  Letters 71, 2741–2744 (1993),
  [doi:10.1103/PhysRevLett.71.2741](https://doi.org/10.1103/PhysRevLett.71.2741).
- B. Drossel, S. Clar and F. Schwabl, *Exact results for the one-dimensional
  self-organized critical forest-fire model*, Physical Review Letters 71, 3739–3742
  (1993), [doi:10.1103/PhysRevLett.71.3739](https://doi.org/10.1103/PhysRevLett.71.3739).
- P. Bak, C. Tang and K. Wiesenfeld, *Self-organized criticality: an explanation of the
  1/f noise*, Physical Review Letters 59, 381–384 (1987),
  [doi:10.1103/PhysRevLett.59.381](https://doi.org/10.1103/PhysRevLett.59.381).
