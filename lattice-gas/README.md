# Lattice gas

A gas made of particles that live on a lattice. Each particle moves one step at a time along
the lines of a hexagonal (strictly, triangular) lattice, from node to node. When particles
meet at a node, they collide and scatter. Nothing about one particle is fluid-like: it just
hops and bounces. But average the particles over a few hundred nodes, and the averages flow
like water, with eddies behind an obstacle.

## The rule

Each node of the lattice has six neighbours, in six directions 60° apart. A node can hold at
most one particle moving in each direction, so its whole state is six bits: which of the six
directions have a particle. Every step has two phases.

1. **Collision.** At every node at once, the particles there may scatter. Only two kinds of
   meeting change anything (the model's authors call this rule set FHP-I):
   - **Head-on pair.** Exactly two particles, coming from opposite directions. They leave
     along another of the three lines through the node, turned 60° one way or the other,
     with equal chance.
   - **Triple.** Exactly three particles, 120° apart. They leave in the three directions
     between the ones they arrived in.

   Every other arrangement passes through unchanged. Both collisions keep the number of
   particles and their total momentum (the sum of their velocity arrows) exactly, as real
   collisions do.
2. **Streaming.** Every particle moves one node along its direction.

**Walls** send every particle back the way it came ("bounce-back"), so the gas next to a
wall has no average velocity: it sticks to the wall, as a real fluid does.

## History

- **1973: HPP.** Jean Hardy, Yves Pomeau and Olivier de Pazzis introduced the first fully
  discrete lattice gas, on a square lattice, with head-on pairs turning through 90°
  (J. Math. Phys. 14, 1746 (1973); Phys. Rev. A 13, 1949 (1976)). They built it to study
  basic questions in statistical mechanics, such as how transport coefficients behave in
  two dimensions. It conserves mass and momentum and carries sound waves, but its
  large-scale flow does not obey the equations of real fluids: a square lattice is not
  symmetric enough, and the averaged flow "feels" the lattice's directions.
- **1986: FHP.** Uriel Frisch, Brosl Hasslacher and Yves Pomeau showed that the same kind of
  model on a hexagonal lattice does obey the Navier-Stokes equations, the equations of
  real fluid flow, at large scales ("Lattice-gas automata for the Navier-Stokes equation",
  Phys. Rev. Lett. 56, 1505 (1986)). The model is known by their initials. Its rules are
  only a few bits of logic per node, so they proposed it as the basis for simple, massively
  parallel fluid computers.
- **1987: the full theory and simulations.** The theory, with the exact collision rules
  used here, is in U. Frisch, D. d'Humières, B. Hasslacher, P. Lallemand, Y. Pomeau and
  J.-P. Rivet, "Lattice gas hydrodynamics in two and three dimensions", Complex Systems 1,
  649–707 (1987). In the same issue, D. d'Humières and P. Lallemand simulated flows on
  lattices of millions of nodes and compared them with real experiments: channel flow,
  the flow past a backward step, and eddies behind a flat plate ("Numerical simulations of
  hydrodynamics with lattice gas automata in two dimensions", Complex Systems 1, 599–632).

## Further reading

- The two Complex Systems papers above are free to read on the journal's site,
  <https://www.complex-systems.com/>.
- [Lattice gas automaton](https://en.wikipedia.org/wiki/Lattice_gas_automaton) on
  Wikipedia.
