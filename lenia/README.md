# Lenia

Lenia is a cellular automaton in which nothing is whole. In the Game of Life, another demo
here, a cell is alive or dead, counts its eight neighbours, and changes all at once. In
Lenia a cell holds any number from 0 to 1, weighs every cell within a wide circle around
it, and changes a little at a time. Out of that come "creatures": smooth blobs that glide,
turn and pulse, and look more like something under a microscope than like Life's
patterns.

## The rule

Every step, for every cell at once:

- **Look around.** Take a weighted average of the cells within a radius R of it, R = 13
  cells for most creatures here. The weights (the *kernel*) form a smooth ring: nothing at
  the cell itself, most halfway out, fading to nothing at the edge. They add up to 1, so
  the average, called the *potential* U, is between 0 and 1 too.
- **Grow or shrink.** A *growth function* turns the potential into a growth between −1
  and 1. It is 1 when U is exactly the target μ (mu), and falls away on either side; σ
  (sigma) sets how far. Here U more than 3σ from μ gives −1.
- **Change a little.** Add a tenth of the growth to the cell's value, and keep it between
  0 and 1.

In symbols, with ∗ for the weighted sum over the ring and T = 10:

> U = K ∗ A

> A ← A + G(U) / T, kept between 0 and 1

A cell with a little but not too much around it grows; too little, or too much, and it
fades. That is Life's rule too, with smoothness everywhere: the states, the ring and the
growth are smooth, and T steps make one unit of time, so with large R and T the world
approaches a continuous one, in space, time and state.

The colours run from navy (0) through blue and teal to gold (1), as in reaction-diffusion,
another demo here, whose chemicals are amounts that vary smoothly too.

The name is from the Latin *lenis*, smooth.

## Life inside Lenia

Life is Lenia with everything made coarse again. Take radius 2, so the ring holds just the
eight neighbours, at distance 1 and √2, all weighted 1, plus the cell itself weighted ½.
Take growth +1 when the potential is within 0.07 of 0.35 and −1 otherwise, and add the
whole growth each step (T = 1). The potential is (neighbours + ½ × the cell) / 8.5:

- an empty cell with 3 live neighbours has potential 0.35: it is born;
- a live cell with 2 or 3 has 0.29 or 0.41: it survives;
- every other count gives a potential more than 0.07 from 0.35: the cell dies, or stays
  empty.

That is exactly Conway's rule. Choose *Life* to watch Lenia run Gosper's glider gun.

## Speed

Each cell's average covers 516 cells at radius 13, too many to add up one by one, every
cell, every step. The demo uses the Fourier transform instead, as Chan's programs do: it
turns the whole weighted sum into a multiplication, and a step of the 256 x 256 world
takes about half a millisecond (natively, on a laptop).

## History

- **1970: Life.** John Conway's Game of Life reached the world through Martin Gardner's
  *Mathematical Games* column: *The fantastic combinations of John Conway's new solitaire
  game "life"*, Scientific American 223, 120–123 (October 1970).
- **2011: SmoothLife.** Stephan Rafler made Life continuous in space and state, and found
  a "smooth glider" that can move in any direction: *Generalization of Conway's "Game of Life" to a continuous domain –
  SmoothLife* (2011), [arXiv:1111.1567](https://arxiv.org/abs/1111.1567). Chan arrived at
  Lenia independently, and calls the likeness between the two "convergent evolution".
- **2018–2019: Lenia.** Bert Wang-Chak Chan, in Hong Kong, had written a precursor,
  Primordia, around 2005. He put Lenia online in 2018, in a version that runs in the
  browser.
  His paper reports more than 400 species in 18 families, many found by hand: by mutating
  and selecting patterns, steering the parameters and keeping what lived. He sorted them
  into a taxonomy with Latin names, as a biologist would (*Orbium unicaudatus*,
  *Scutium solidus*), and studied how they move, collide and fuse. He counts his Scutium
  as SmoothLife's smooth glider rediscovered. *Lenia: Biology of Artificial
  Life*, Complex Systems 28, 251–286 (2019),
  [doi:10.25088/ComplexSystems.28.3.251](https://doi.org/10.25088/ComplexSystems.28.3.251),
  [arXiv:1812.05433](https://arxiv.org/abs/1812.05433).
- **2020: expanded.** Chan extended Lenia to more dimensions, several kernels and several
  channels, and found creatures that copy themselves:
  *Lenia and Expanded Universe*, Artificial Life Conference Proceedings 32, 221–229
  (2020), [doi:10.1162/isal_a_00297](https://doi.org/10.1162/isal_a_00297),
  [arXiv:2005.03742](https://arxiv.org/abs/2005.03742).

The creatures in the demo, with their patterns and settings, are from Chan's own catalogue
in his [Lenia repository](https://github.com/Chakazul/Lenia), © 2018 Bert Chan, under the
MIT licence.

## Things to try

- **Orbium.** The opening: Lenia's best-known creature glides in a straight line, about
  6 cells every 10 steps, and wraps round the world. Its *mass*, the sum of all the
  values, stays at about 74 as it goes.
- **Kill it gently.** Set *width σ* to 0.012 (click its number to type one) and press
  *Restart*: Orbium evaporates. At 0.024 it explodes and fills the world with a fizzing
  texture. Between about 0.013 and 0.02 it lives, a little smaller or bigger. *Centre μ*
  does the same between about 0.13 and 0.16. *Put back* restores Chan's values. The
  sliders also act on a running world, so you can push a creature over the edge.
- **Other creatures.** *Scutium* is a sturdier glider. *Gyrorbium* turns as it goes, so it
  circles round a small loop and never gets far. *Vagorbium* is bigger (radius 20) and
  zigzags. *Synorbium* is two orbs joined, gliding together. The *serrated Scutium*'s
  kernel has two rings, the inner one half height; it pulses as it wanders.
- **Soup.** Choose *Soup*, a scatter of random patches with Orbium's rule, and run it
  (faster helps). Most soups fill the world or die out; about one in five leaves an
  Orbium or two, crawling out of the mess. Press *Restart* for a fresh one.
- **Disturb a creature.** Drag across a creature's path to scatter random values in front
  of it, and see whether it survives the collision.
- **Draw your own.** *Clear*, then scribble with the brush and run. A single round blob
  dies; the soup's patches show how much more mess it takes to make a creature.
- **Life.** Choose *Life*: the same program, with radius 2 and whole steps, runs Conway's
  Game of Life, and the glider gun fires.
- **A bigger world.** *512 x 512* gives a soup room for more creatures; it runs slower.

## Further reading

- The Wikipedia article [Lenia](https://en.wikipedia.org/wiki/Lenia).
- Bert Wang-Chak Chan, *Lenia: Biology of Artificial Life*, Complex Systems 28,
  251–286 (2019), [PDF](https://content.wolfram.com/sites/13/2019/10/28-3-1.pdf). The
  paper, with pictures of dozens of species.
- [Chan's Lenia repository](https://github.com/Chakazul/Lenia), with versions in Python,
  JavaScript and MATLAB, and the full catalogue of creatures.
- Stephan Rafler, *Generalization of Conway's "Game of Life" to a continuous domain –
  SmoothLife* (2011), [arXiv:1111.1567](https://arxiv.org/abs/1111.1567).
