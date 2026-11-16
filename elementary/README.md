# Elementary cellular automata

The simplest cellular automata there are. A single row of cells, each **0** or **1**.
Every generation, each cell looks at three cells, its left neighbour, itself and its right
neighbour, and becomes 0 or 1 according to a rule. The demo draws each new generation as
the next row down, so the picture is the automaton's whole history: space runs across,
time runs down.

## 256 rules, numbered

Three cells, each 0 or 1, can form 8 patterns: 111, 110, 101, 100, 011, 010, 001, 000. A
rule only has to say, for each of those 8 patterns, whether the cell becomes 0 or 1. That
makes 2^8 = **256** possible rules.

To number them, write the 8 answers in that order (from 111 down to 000) as a binary
number. **Rule 30** is 00011110 in binary: the patterns 100, 011, 010 and 001 give 1, and
the rest give 0. The demo draws a rule's table the same way. Click an output square to
flip it and watch the rule number change.

## History

- **1983: a systematic study.** The physicist Stephen Wolfram analysed these one-row,
  two-state automata in *Statistical mechanics of cellular automata*, Reviews of Modern
  Physics 55, 601–644 (1983). He used them as simple mathematical models for studying
  self-organisation, the way order arises by itself. Its abstract calls them
  "elementary" cellular automata, in quotation marks.
- **1984: four classes.** In *Universality and complexity in cellular automata*, Physica
  D 10, 1–35 (1984), he argued that one-dimensional automata fall into four classes, and
  suggested that the fourth was probably capable of universal computation:
  1. those that settle to a uniform state;
  2. those that settle into stable or repeating structures;
  3. those that stay chaotic;
  4. those that build complex, localised structures that interact.
- **2004: Rule 110 is universal.** Wolfram conjectured in 1985 that Rule 110 could carry
  out any computation. Matthew Cook proved it, in *Universality in elementary cellular
  automata*, Complex Systems 15, 1–40 (2004). Cook showed that Rule 110 can emulate
  *cyclic tag systems*, a simple model of computation known to be universal. Publication was held up
  for several years by a legal dispute with Wolfram Research over a non-disclosure
  agreement.

## Rules to try

- **Rule 30**: start from a single cell and you get a triangle of apparent chaos. Its
  centre column looks so random that Rule 30 has been used as the random-number generator
  in Mathematica. A similar pattern appears on the shell of the sea snail *Conus
  textile*.
- **Rule 90**: each cell becomes the exclusive-or (XOR) of its two neighbours. From one
  cell this draws Pascal's triangle with odd numbers on and even numbers off, which is the
  **Sierpinski triangle**, a fractal. (The demo's tests check this row by row.)
- **Rule 110**: the universal one. From a single cell it grows to one side only. From a
  random row it settles into a regular background crossed by moving structures, which are
  the "signals" Cook's proof uses.
- **Rule 184**: a model of traffic on a one-lane road, the basis of many more detailed
  traffic models. Each 1 is a car that moves one step right whenever the cell ahead is
  empty. Start from a *random row*. A queue of stopped cars clears from the front, so a jam
  moves *backwards* while every car in it moves forwards, as in real stop-and-go traffic. Cars are never created or
  destroyed, so on a ring the count of cells on never changes.
- **Rule 150**: each cell becomes the XOR of all three cells, giving another nested,
  fractal triangle.

## Things to try

- **Edit a rule.** Click an output square in its table. Small changes can turn chaos into
  order.
- **Change the start.** Switch between *single cell* and *random row*. Many rules show
  their character only from a random start.
- **Change the rule mid-run.** The history keeps the old rule's rows above the new
  ones.

## Further reading

- The Wikipedia articles
  [Elementary cellular automaton](https://en.wikipedia.org/wiki/Elementary_cellular_automaton),
  [Rule 30](https://en.wikipedia.org/wiki/Rule_30),
  [Rule 110](https://en.wikipedia.org/wiki/Rule_110) and
  [Rule 184](https://en.wikipedia.org/wiki/Rule_184).
- David Eppstein's notes on
  [Wolfram's classification](https://ics.uci.edu/~eppstein/ca/wolfram.html).
