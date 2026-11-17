# Falling sand

Paint a picture in sand, water, walls, plants and fire, and set it going: sand falls and
piles up, water runs downhill and finds its level, plants grow into the water, and fire
races through the plants. Every cell of the grid holds one material, or nothing, and each
step every cell follows a few simple rules about the cells around it.

It isn't quite a cellular automaton. A classic cellular automaton updates every cell at
once, from the picture as it was. Here the cells are updated one at a time, and each sees the moves
already made in the same step, as in the falling-sand games below. And one rule, pressure,
looks at a whole body of water at once. Petri Purho, one of the makers of the game Noita,
put it this way: "Essentially, it's complex cellular automata."

## The rules

A step goes through the rows from the bottom up, so a grain that falls lands in a row
already done and doesn't fall again. Within a row it goes left to right on one step and
right to left on the next, and whenever a cell could go either way, a coin toss decides.
That way neither side is favoured: the cells done first in a row get first pick of the
empty cells, so the scan alternates. No cell is updated twice in a step.

- **Sand** falls into the cell below if that is empty, water or fire: sand sinks, and
  what was there rises in its place. If it can't fall, it slides into one of the two
  cells diagonally below, if the cell beside it on that side is open too, so sand can't
  slip between two walls that only touch at a corner. So in a pile of sand alone, a grain
  comes to rest only when the three cells under it are full, and a pile at rest never
  stands more than one cell higher than the column next to it. Its slope, the **angle of repose**, is 45 degrees.
- **Water** moves only into empty cells: down if it can, else diagonally down, else
  sideways.
- **Pressure.** Those rules let water fall and spread, but never rise, so the two arms of
  a U-shaped tube would never even out. So after them, each step looks at every body of
  water (cells joined edge to edge) and finds its *surface*: the water cells with empty
  space above that can't move by themselves. If the highest of them is two or more rows
  above the lowest, water moves from the highest row to just above the lowest, as many
  cells as both rows can supply, chosen at random. Every move lowers some water, so the
  levels even out to within one row, as in real *communicating vessels*.
- **Walls** never change.
- **Plants** don't move. Now and then, about one step in 128, a plant cell picks the cell
  above it, or less often one beside it, and if that is water, the water becomes plant:
  plants grow up and out through water, like reeds, by drinking.
- **Fire** next to water goes out, and the water stays. Fire next to a plant, edge or
  corner, stays put and sets each such plant alight with a coin toss every step, so it
  burns through every plant joined to it. Otherwise a flame rises, straight up or to one
  side, and burns down: it lasts 9 to 16 steps once there is nothing left beside it to
  burn.

So sand is never made or lost. Water is lost only by turning into plant, and plants only
by burning, so until something burns, water and plant together stay the same. Only fire
destroys anything. The edges of the world are walls.

The genre takes its name from the *Falling Sand Game*, which grew out of a web game of
2005.

## History

- **Sand in blocks.** Cellular automata for sand have also been built on blocks of 2 x 2
  cells, like Critters, another demo in this project. Wikipedia's article on block
  cellular automata describes a sand rule on the Margolus neighbourhood: each block keeps
  its number of grains and moves them as far down inside it as they can go, and a block
  with two grains stacked one on the other turns into one with them side by side, so tall
  piles topple and spread. It cites B. Chopard and M. Droz, *Cellular Automata Modeling of
  Physical Systems* (Cambridge University Press, 1998), for the rule's conservation of
  grains.
- **2000: cellular gravity.** Frédéric Gruau and John Tromp asked how to make particles
  fall and pile up at the bottom of areas enclosed by walls, with simple, quick rules.
  They proposed a block rule, and proved that it piles up the particles on a grid of
  height *h* in at most 3*h* steps: *Cellular gravity*, Parallel Processing Letters 10,
  383–393 (2000), [doi:10.1142/S0129626400000354](https://doi.org/10.1142/S0129626400000354).
- **2005: an applet, then the Falling Sand Game.** According to Wikipedia's article on the
  genre, the first known popular game of this kind was a Java applet on a Japanese
  website, Dofi-Blog, in 2005. Later expanded and put up again as the *Falling Sand Game*,
  it set off a trend and gave the genre its name. Many followed, among them *Powder Game*
  (2007) and *The Powder Toy* (2010).
- **2018: Sandspiel.** Max Bittker built his falling-sand game *Sandspiel* in late 2018.
  He described its update in *Making Sandspiel* (2019): cells are updated one at a time,
  left to right and top to bottom, and each carries a clock "to prevent a single particle
  from being simulated multiple times in a single tick", which would happen when it moves
  the way the scan is going. This demo's cells carry a bit for the same reason.
- **2019: Noita.** The game *Noita*, by Nolla Games, came out in early access on
  24 September 2019 and in full on 15 October 2020. Nearly every pixel of its world is
  simulated. In an interview, Petri Purho said that it "is based on a falling sand style
  simulation", and that "you have to update the world from the bottom up", as this demo
  does. At GDC 2019 he spoke on scaling the simulation up to large worlds.
- **2020: the lean of blocks.** Jonathan Devlin and Micah Schuster found that on the plain
  Margolus neighbourhood, sand topples "in one direction only" until it can no longer fall
  that way. They shifted the blocks in a cycle of four steps, which makes grains take
  turns toppling each way, and let a stack of two grains topple only with some
  probability: the lower it is, the taller and thinner the piles. They call rules like
  these *side-profile* sandpiles, unlike the sandpile of Bak, Tang and Wiesenfeld (the
  sandpile, another demo here), which they note is studied for self-organised criticality
  "rather than for video games": *Probabilistic cellular automata for granular media in
  video games*, [arXiv:2008.06341](https://arxiv.org/abs/2008.06341) (2020).

## Things to try

- **The hourglass.** The opening runs at once: sand pours through the neck and builds a
  pile below at 45 degrees. Shift-drag to widen the neck, or paint a wall across it.
- **Break the dam.** Choose *Dam*, then shift-drag across the dam's wall, low down, to
  open a gap. The lake pours into the valley, and its level and the flood's even out. The
  grass on the sand drinks the water and grows.
- **The forest fire.** Choose *Forest fire* and the flames spread through every tree joined by the
  grass. Paint *water* on the grass between two trees, ahead of the fire, to make a
  firebreak, or paint *fire* anywhere to start another.
- **The U-tube.** Water poured into one arm rises in the other until the two are level:
  that is the pressure rule at work. Paint more *water* into either arm and watch it share
  out. Paint *sand* into one arm: it sinks to the bottom and pushes the water up both.
- **Sand under water.** *Clear*, draw a box of *wall*, half fill it with *water*, then
  pour *sand* in. The sand sinks, and piles up under the water just as steeply as in air.
- **A garden.** Draw a line of *plant* along the bottom of a pond and run for a while: it
  grows up through the water and drinks the pond dry. Then set it alight.

## Further reading

- The Wikipedia article [Falling-sand game](https://en.wikipedia.org/wiki/Falling-sand_game).
- The Wikipedia article [Block cellular automaton](https://en.wikipedia.org/wiki/Block_cellular_automaton),
  with its sand rule.
- M. Bittker, *Making Sandspiel* (2019), [maxbittker.com](https://maxbittker.com/making-sandspiel):
  how one falling-sand game works, with code.
- *Noita: a Game Based on Falling Sand Simulation*, 80.lv (2019),
  [80.lv](https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation/): Petri
  Purho on how Noita's world is simulated, in chunks of 64 x 64 pixels, on several
  threads at once.
- P. Purho, *Exploring the Tech and Design of Noita*, GDC 2019,
  [GDC Vault](https://gdcvault.com/play/1025695/Exploring-the-Tech-and-Design).
- J. Devlin and M. D. Schuster, *Probabilistic cellular automata for granular media in
  video games* (2020), [arXiv:2008.06341](https://arxiv.org/abs/2008.06341).
