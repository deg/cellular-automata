# Lattice gas

A gas made of particles that live on a lattice. Each particle moves one step at a time along
the lines of a hexagonal (strictly, triangular) lattice, from node to node. When particles
meet at a node, they collide and scatter. Nothing about one particle is fluid-like: it just
hops and bounces. But average the particles over a few hundred nodes, and the averages flow
like water, with eddies behind an obstacle.

## The rule

Each node of the lattice has six neighbours, in six directions 60 degrees apart. A node can
hold at most one particle moving in each direction, so its whole state is six bits: which of
the six directions have a particle. Every step has two phases.

1. **Collision.** At every node at once, the particles there may scatter. Only two kinds of
   meeting change anything (the model's authors call this rule set FHP-I); every other
   arrangement passes through unchanged, so 5 of the 64 possible arrangements collide and
   59 don't. Both collisions keep the number of particles and their total momentum (the sum
   of their velocity arrows) exactly, as real collisions do.
   - **Head-on pair.** Exactly two particles, coming from opposite directions. They leave
     along another of the three lines through the node, turned 60 degrees one way or the
     other, with equal chance.
   - **Triple.** Exactly three particles, 120 degrees apart. They leave in the three
     directions between the ones they arrived in.
2. **Streaming.** Every particle moves one node along its direction.

**Walls** send every particle back the way it came ("bounce-back"), so the gas next to a
wall has no average velocity: it sticks to the wall, as a real fluid does.

The demo also runs the older **HPP** model on a square lattice, for comparison: four
directions, and one collision, a head-on pair leaving at 90 degrees.

**The wind** comes in at the left edge: every step, the first few columns are refilled with
fresh gas moving right. The world wraps around, so the gas leaving at the right edge runs
into those same columns, and the westward-moving particles in them flow back in at the
right edge, keeping the density there steady: a wind tunnel. d'Humières and Lallemand fed
their outlet the same way, injecting fresh particles there too "in order to keep the
density almost constant".

## Why the averages flow

One node holds at most six particles, so on its own it says almost nothing: look at the
*particles* view, which is noise. The fluid appears in averages. The other views average
each block of nodes (16 across, by default) over the last hundred steps or so, and draw the
mean velocity as arrows and colour.

Mass and momentum are conserved in every collision, and those two conservation laws are,
at large scales, most of what the equations of fluid flow (the Navier-Stokes equations) say.
The details of the collisions matter mostly through how fast momentum spreads between
neighbouring layers of gas: the **viscosity**. So a gas this crude still flows like a real
one, as long as the lattice is symmetric enough. That is where the hexagon matters.

## History

- **1973: HPP.** J. Hardy, Y. Pomeau and O. de Pazzis introduced the first fully discrete
  lattice gas, on a square lattice: four directions, with head-on pairs turning through
  90 degrees. Their papers are *Time evolution of a two-dimensional model system. I.
  Invariant states and time correlation functions*, Journal of Mathematical Physics 14,
  1746–1759 (1973), and *Molecular dynamics of a classical lattice gas: transport
  properties and time correlation functions*, Physical Review A 13, 1949–1961 (1976).
  They built it to study basic questions in statistical mechanics, such as how transport
  coefficients behave in two dimensions. It conserves mass and momentum and carries sound
  waves, but its large-scale flow does not obey the Navier-Stokes equations: the square
  lattice is not symmetric enough. The flow of momentum in the averaged gas depends on the
  direction of flow relative to the lattice, so the gas behaves differently along the
  lattice and diagonally across it.
- **1986: FHP.** Uriel Frisch, Brosl Hasslacher and Yves Pomeau showed that the same kind of
  model on a hexagonal lattice does obey the Navier-Stokes equations at large scales, in
  *Lattice-gas automata for the Navier-Stokes equation*, Physical Review Letters 56,
  1505–1508 (1986). Six directions are enough to
  make the averaged flow the same in every direction; four are not. The model is known by
  their initials. Its rules are a few bits of logic per node, so they proposed it for
  simple, massively parallel fluid computers.
- **1987: theory and simulations.** The full theory, with the exact collision rules used
  here, is in U. Frisch, D. d'Humières, B. Hasslacher, P. Lallemand, Y. Pomeau and
  J.-P. Rivet, *Lattice gas hydrodynamics in two and three dimensions*, Complex Systems 1,
  649–707 (1987). In the same issue, in *Numerical simulations of hydrodynamics with
  lattice gas automata in two dimensions*, Complex Systems 1, 599–632 (1987),
  D. d'Humières and P. Lallemand simulated lattices of about a million nodes and compared
  them with known solutions and experiments: flow down a channel, the flow past a
  backward step, and eddies growing behind a flat plate, which they compared with
  experiments in water. The same issue describes RAP1, a cellular automaton machine
  built for fluid dynamics, by A. Clouqueur and D. d'Humières.
- **1988 and after: lattice Boltzmann.** Guy McNamara and Gianluigi Zanetti proposed
  simulating the gas's average populations directly, as real numbers, instead of
  individual particles, in *Use of the Boltzmann equation to simulate lattice-gas
  automata*, Physical Review Letters 61, 2332–2335 (1988). With no particles, there is
  none of the noise you see here. Lattice gases were the precursor of these lattice
  Boltzmann methods.

## What this one can and can't do

- **Noise.** Every average is over a finite number of particles, so the flow views shimmer.
  Bigger blocks or longer time averages smooth it, at the cost of detail or of lag.
- **Low Reynolds numbers.** The Reynolds number says how much a flow is dominated by
  inertia (high: eddies and turbulence) rather than viscosity (low: smooth, syrupy flow).
  It grows with the wind's speed and the obstacle's size, and falls with the viscosity.
  FHP-I's viscosity is high, and the wind has to stay well below the speed of sound, so on
  a lattice this size the Reynolds number stays in the tens: the readout shows it. That is
  enough for a pair of eddies behind a plate, but not for the alternating vortices that
  d'Humières and Lallemand saw shed from their plate at Reynolds numbers of a few hundred.
  They used a richer rule set, FHP-III, which adds a resting particle and many more
  collisions and reaches Reynolds numbers about six times higher, on a lattice of almost
  three million nodes.
- **Not quite incompressible.** The default wind is 0.42 of the speed of sound, and it
  speeds up round the plate's ends to about two thirds of it, so the gas's density varies
  by about 15% across the flow, where a real liquid's would hardly change. A slower wind
  would compress it less, but would give a lower Reynolds number and weaker eddies: with
  FHP-I that is the trade-off. The flow is also a row of plates, not one: the world wraps
  around top to bottom, so each plate has copies of itself a world's height above and
  below it, and fills a quarter of the gap.
- **Not quite Galilean.** The lattice gas's flow carries itself along more slowly than a
  real fluid's would, by a factor that depends on the density; time is rescaled to compare
  with real flows. The Reynolds number shown allows for it.

## Things to try

- **Plate.** The demo opens on it, running, in the *swirl* view. Two layers of opposite
  spin peel off the plate's ends, and between them, behind the plate, the arrows shrink
  and turn back: a pair of eddies, growing with time. Switch to *speed* to see the still
  bubble of the wake.
- **Particles.** Switch to the *particles* view while the plate runs: the flow you just saw
  is invisible in the raw automaton. The arrows still show it.
- **Sound pulse.** A dense disc of gas at rest spreads as a ring at the speed of sound,
  1/√2 node spacings per step. The ring stays round, though the particles move in only
  six directions. On the square lattice a pulse spreads nearly round too: sound alone
  doesn't show HPP's flaw. Flow does; see the next item.
- **The square lattice (HPP).** Choose *square (HPP)* and run the *Plate* again. HPP keeps
  the momentum of every row, and of every column, separately: "far too many conserved
  quantities for physical modeling", as Frisch and his colleagues put it in 1987. So
  momentum never passes from one row to the next, and nothing like viscosity acts across
  the flow. The plate's shadow runs straight downstream with razor-sharp edges and never
  fills in, and there are no eddies. On the hexagonal lattice the same wake has almost
  filled in 400 nodes behind the plate.
- **Draw your own obstacle.** Drag on the lattice to draw walls; shift-drag to erase. A
  wall across the whole channel blocks the wind; a gap in it makes a jet.
- **Channel.** Walls along the top and bottom. The gas sticks to them, and the slow layers
  next to the walls thicken downstream.
- **Change the averaging** (under *Averaging*). Small blocks and short windows show detail
  but noise; large ones show smooth flow. Try 6 *nodes per block* and 10 *steps averaged*,
  then 32 and 500.
- **Change the wind.** A faster wind means a higher Reynolds number and bigger eddies, but
  also a denser pile-up of gas in front of the obstacle (see *density*).

## Further reading

- The Wikipedia article [Lattice gas automaton](https://en.wikipedia.org/wiki/Lattice_gas_automaton).
- The two 1987 Complex Systems papers above are free to read on the
  [journal's website](https://www.complex-systems.com/), and explain both the theory and
  the practice.
- D. A. Wolf-Gladrow, *Lattice-Gas Cellular Automata and Lattice Boltzmann Models*
  (Springer, 2000): a textbook on both.
