# Langton's ant

An ant stands on a grid of blank cells. Each step it follows two rules:

- On a **blank** cell, it turns **right**, colours the cell, and steps forward.
- On a **coloured** cell, it turns **left**, clears the cell, and steps forward.

That's all. For the first few thousand steps the ant scribbles an irregular, growing blob,
with no sign of order. Then, at about step 10,000, it suddenly starts building a
**highway**: a strip it extends diagonally across the grid, repeating the same 104-step
cycle forever.

Nothing in the two rules mentions highways, diagonals or the number 104. The highway is
*emergent*: it isn't written into the rule, it arises from it. That's why Langton's ant is
a favourite example of simple rules producing complex and surprising behaviour.

Unlike the other demos, this isn't quite a cellular automaton, where every cell updates at
once. Only the cell under the ant changes. Automata like this, with a moving head that
reads and writes cells, are called **turmites**, short for "Turing machine termites".

## History

- **1986.** Chris Langton invented the ant.
- **The highway.** Starting from a blank grid, the ant settles into highway-building
  after about 10,000 steps. Each 104-step cycle moves it two cells diagonally. (This
  demo's tests measure the onset and the cycle.)
- **What's proved, and what isn't.** It has been proved that the ant's path is always
  unbounded, whatever finite pattern of coloured cells it starts among: it never stays
  trapped in a finite region. But whether it *always* ends up building a highway is an open
  question. Every finite starting pattern that has been tried leads to one, yet no one has
  proved that it must.
- **More colours.** Greg Turk and Jim Propp generalised the ant to more than two colours.
  Cells cycle through the colours in order, and the rule says, for each colour, whether
  to turn left or right. The rule is written as a string of letters, so Langton's ant is
  `RL`. The paper *Further travels with my ant* (David Gale, James Propp, Scott Sutherland
  and Serge Troubetzkoy, Mathematical Intelligencer 17, 1995) explores this family.

## Things to try

- **The highway.** Press Run, or press *Jump ahead 10,000 steps* to skip the chaos and
  watch the highway being built.
- **Collisions.** The grid wraps around, so the highway eventually runs back into the
  ant's own earlier mess. What happens then?
- **Two ants.** Restart, click to add a second ant near the first, and Run. Their paths
  interfere, and the highway may never come.
- **Other rules.** Try the presets, or type any string of L and R (2 to 12 letters). Some
  make chaos, some grow in regular shapes, some build highways of their own.

## Further reading

- The Wikipedia article [Langton's ant](https://en.wikipedia.org/wiki/Langton%27s_ant).
- Gale, Propp, Sutherland and Troubetzkoy, *Further travels with my ant*
  ([arXiv:math/9501233](https://arxiv.org/abs/math/9501233)).
