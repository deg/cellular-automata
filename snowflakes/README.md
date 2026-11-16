# Snowflakes

A snow crystal grows from a single seed of ice in a field of water vapour. This demo runs
**Clifford Reiter's local cellular model** of that growth. With a very simple rule and just
two parameters, it grows the main flat forms of real snow crystals: six-sided plates, stars,
and fern-like dendrites, with branches on their branches.

The name: snow crystals are commonly known as snowflakes. Reiter titled his paper's
companion web page "Snowflake Automaton".

## The rule

The grid is hexagonal, and each cell holds a real number: an amount of water. A cell
holding **1 or more is ice**. Every step, all cells update at once:

1. A cell is **receptive** if it is ice or has an ice neighbour. Receptive cells are the
   crystal and the skin of cells around it.
2. Receptive cells **keep their water** and gain a constant **γ** (gamma).
3. The water in all other cells **diffuses**: each cell's new value is half its own value
   plus a twelfth of each of its six neighbours'. Receptive cells count as 0 in this
   average.

Then each cell's new water is its share of the diffusion plus, if it is receptive, its
kept water and γ. Receptive cells keep everything that reaches them, so in this model the
crystal is a sink: vapour flows into it and never comes back out. A receptive cell
freezes once it has gathered 1.

The crystal starts as one ice cell in a field where every cell holds **β** (beta), the
background vapour. Outside a circle around the seed the water is held at β, as in Reiter's
runs, so fresh vapour keeps arriving from all sides. Growth stops when the ice nears that
circle.

The crystal's tips reach out into the vapour, and γ is added evenly over its whole skin.
Reiter offered γ as the simplest extension of the model, and as a rough stand-in for water
arriving from outside the plane of growth. Between them, β and γ decide the form:

- Low γ and low β: **dendrites**. Higher β: stars, or more abstract forms.
- High γ: **plates**, sectors or broad stars, whatever β is.
- β = 0 and γ = 0: no growth at all.
- β = 0 and any γ > 0: plates.

Those outcomes are Reiter's. His Figure 3 maps them over β from 0 to 0.95 and γ from 0 to 1.
The demo's tests check the last two exactly.

A third parameter, **α** (alpha), sets how fast vapour diffuses. Reiter's standard model
has α = 1: the weights a half and a twelfth above. In general a neighbour weighs α/12 and
the cell itself 1 − α/2, a step of the diffusion equation on the hexagonal grid. He found
the growth very sensitive to α: 0.5, 0.501 and 0.502 give very different crystals,
although all three are dendrites. Above α = 2 a cell's own weight turns negative, and at
2.69 growth turns explosive.

## History

- **1611: Kepler.** Johannes Kepler wrote on why snowflakes are six-cornered, in *Strena
  seu de nive sexangula* (translated as *The Six-Cornered Snowflake*). He suggested the
  symmetry came from the hexagonal packing of spheres. Reiter calls this remarkable
  foresight, given today's understanding of the hexagonal molecular packing of ice.
- **1931: Bentley and Humphreys.** Wilson Bentley, a Vermont farmer, assembled the most
  extensive early collection of photographs of snow crystals, published as *Snow
  Crystals*, with W. J. Humphreys (McGraw-Hill, 1931). Gravner and Griffeath credit him
  with making the idealised snowflake an icon of winter.
- **1954: Nakaya.** Ukichiro Nakaya studied snow crystals scientifically, both natural and
  grown in the laboratory: *Snow Crystals: Natural and Artificial* (Harvard University
  Press, 1954). He was the first to grow them artificially, on the end of a rabbit hair.
  He found that a crystal's form depends heavily on the temperature and the saturation of
  the air it grows in. He controlled them by setting the air temperature and that of a
  water bath supplying the vapour.
- **1984: Packard's digital snowflakes.** Norman Packard's *Lattice models for
  solidification and aggregation* (an Institute for Advanced Study preprint, reprinted in
  1986) used a hexagonal automaton whose cells are ice or not: a cell freezes when exactly
  one of its neighbours is ice. Stephen Wolfram popularised it in *A New Kind of Science*
  (2002). It makes plates and sectors, but not dendrites or stars.
- **2005: Reiter.** Clifford A. Reiter, *A local cellular model for snow crystal growth*,
  Chaos, Solitons & Fractals 23, 1111–1119
  ([doi](https://doi.org/10.1016/S0960-0779(04)00374-1),
  [preprint](https://webbox.lafayette.edu/~reiterc/mvp/sfn/sfn_pp.pdf)). Following Nakaya,
  he aimed for a simple local model with just two parameters, β and γ, and did not try to
  fit the equations of the physics. His dendrites appear remarkably natural. The model is
  flat, so it cannot make the three-dimensional forms, such as needles and capped columns.
- **2008–2009: Gravner and Griffeath's "snowfakes".** Janko Gravner and David Griffeath
  wrote that Reiter's model strikes the best balance so far between speed of computation
  and physical realism, and refined it. Theirs adds anisotropic attachment kinetics
  (how readily vapour freezes on, depending on the local shape of the surface) and an
  idealised quasi-liquid layer, using eight parameters. It captures the tip instabilities
  and faceted branches of real crystals, which Reiter's lacks. They call its crystals
  *snowfakes*: *Modeling snow crystal growth II: A mesoscopic lattice map with plausible
  dynamics*, Physica D 237, 385–404, 2008
  ([doi](https://doi.org/10.1016/j.physd.2007.09.008)). They extended it to three
  dimensions in *Modeling snow-crystal growth: A three-dimensional mesoscopic approach*,
  Physical Review E 79, 011601, 2009
  ([doi](https://doi.org/10.1103/PhysRevE.79.011601)).

## Things to try

- **The presets** are Reiter's own examples: his Figure 4's thin dendrite, thick dendrite,
  stellar and sector forms; a plate; and *Many levels*, his Figure 5's α = 2.003, "a
  pretty growth pattern with many levels of dendrites". Each starts again from a seed.
- **Change conditions as it grows.** A real crystal falls through air of changing
  temperature and humidity. γ and α take effect at once, so you can do the same. Reiter's
  Figure 7 grew a plate with β = 0.4 and γ = 0.5, then lowered γ to 0.001 to grow
  dendrites from it. Set those, press Restart, let a small plate form, then drag γ down.
- **Explore β and γ.** At β = 0.4, raise γ from 0 to 1 a step at a time: the dendrites
  thicken into stars, then become a plate.
- **Nudge α** a little away from 1, or above 2, and compare.
- **Break the symmetry.** The model is deterministic and treats all six directions alike,
  so a single seed grows a perfectly symmetric crystal. Drag on the field to freeze a few
  extra cells near it, and the arms come out different. Reiter's Figure 6 started from
  small random clusters instead of a single seed.
- **Room to grow.** A bigger field gives finer detail, but takes longer: the thin dendrite
  needs about 9,000 steps at radius 200.

## Further reading

- Reiter's [auxiliary page](https://webbox.lafayette.edu/~reiterc/mvp/sfn/index.html) for
  the paper: tables of forms over β and γ, animations of the growth, and his program.
- Kenneth Libbrecht's [SnowCrystals.com](http://www.snowcrystals.com): photographs of real
  snow crystals, and the science of why they have the shapes they do.
- Jessica Li and Laura Schaposnik, *Interface control and snow crystal growth*
  (Physical Review E 93, 023302, 2016;
  [arXiv:1505.02042](https://arxiv.org/abs/1505.02042)): Reiter's model with two variables
  added, growth latency and growth direction, which extend it to plate-shaped crystals.
