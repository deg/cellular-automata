# Wireworld

A cellular automaton designed for building circuits. Each cell is one of four things:

- **Empty** (black): nothing happens here, ever.
- **Wire** (copper): a conductor that electrons can run along.
- **Electron head** (blue) and **electron tail** (red): an electron is a head with a tail
  just behind it.

Every generation, all cells update at once:

- A **head** becomes a **tail**.
- A **tail** becomes **wire**.
- A **wire** cell becomes a **head** if exactly **1 or 2** of its 8 neighbours are heads.

That's all, but it's enough for electrons to travel along wires, one cell per generation.
They move away from their tails, because the cell behind a head is still a tail when the
head moves on. Where wires meet, the "1 or 2 heads" limit lets the geometry decide what
gets through. From there you can build diodes, clocks, logic gates and, in the end, whole
computers.

## History

- **1987.** Brian Silverman proposed Wireworld as part of his program *Phantom Fish
  Tank*. Silverman also invented Brian's Brain, another demo in this
  project.
- **1990.** Wireworld became widely known through A. K. Dewdney's *Computer
  Recreations* column in **Scientific American, January 1990**, which also introduced a
  rule called Rugworld.
- **2004: a Wireworld computer.** David Moore and Mark Owen released a working computer
  built entirely of Wireworld cells, with its results shown on seven-segment displays
  made of electrons. Wireworld is Turing-complete: with enough cells, it can carry out any
  computation.

## Circuits in this demo

Each circuit is checked by the demo's tests.

- **Clock**: an electron circling a 12-cell loop. Where the loop touches the output
  wire, a copy of the electron leaves every 12 generations, and the original keeps
  circling. Clocks are the power supply of Wireworld circuits.
- **Diodes**: two clocks. The top one feeds a **diode**, a small blob in the wire that
  lets electrons through in one direction. The bottom one feeds the same diode turned
  round, which blocks every electron.
- **OR gate**: two clocks, firing every 12 and every 8 generations, join into one output
  wire, which carries electrons from either. The diodes keep each clock's electrons from
  running back up the other input.

## Things to try

- **Watch the OR gate**, then press *Remove all electrons* and add a single electron by
  hand. Pick the *tail* brush and click a wire cell, then the *head* brush and click the
  next cell along.
- **Break a diode.** Erase one cell of its blob (shift-drag) and see whether it still
  blocks.
- **Build your own.** Draw a loop of wire with one electron in it: that's a clock. Its
  length sets its rhythm. Join two clocks of different lengths at a junction without
  diodes: what happens?

## Further reading

- The Wikipedia article [Wireworld](https://en.wikipedia.org/wiki/Wireworld).
- [The Wireworld computer](https://www.quinapalus.com/wi-index.html), by Mark Owen and
  David Moore.
