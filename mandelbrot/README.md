# The Mandelbrot set

Pick a point c of the plane, and treat it as a number (a *complex* number: the point
(a, b) is the number a + b*i*). Start with z = 0, and repeat one rule:

> z → z² + c

Either z stays close to 0 for ever, or at some step it gets further than 2 from 0, after
which it races off to infinity. **The Mandelbrot set is the points c whose z stays close to
0.** In the demo they are the dark shape in the middle. Every other point is coloured by the
step at which its z escaped.

That's the whole rule. The shape it makes has detail at every scale: zoom in anywhere on
its edge and new spirals, spikes and tiny copies of the whole set keep appearing, for as
long as the computer's numbers last.

## Squaring a point

Complex numbers multiply in a way that is easy to picture. Every point has a distance from
0 and an angle (measured anticlockwise from the positive horizontal axis). Squaring a
point **squares its distance and doubles its angle**. Adding c then shifts the result by c.
So each step spins and stretches z, then nudges it. Points near 0 shrink when squared, and
points beyond 1 grow, and the set is where the shrinking and the nudging balance for ever.

## Why 2?

Once z is further than 2 from 0, it can never come back. Suppose |z| > 2 and |z| ≥ |c|.
Then

> |z² + c| ≥ |z|² − |c| ≥ |z|² − |z| = |z| (|z| − 1),

and |z| − 1 > 1, so the next z is further out still, by a factor that keeps growing. (And if
|c| > 2, then z is past 2 after the very first step, since z₁ = c.) So the moment |z| passes
2, the point is known to be outside the set, and the step at which that happened is its
**escape count**.

A point that hasn't escaped after many steps is *probably* in the set, but no number of
steps proves it: some points near the edge escape only after thousands. The demo stops at
a limit you choose, and draws everything that hasn't escaped by then as the set.

## Not a cellular automaton

The other demos on this site are cellular automata: a grid of cells, each changing
according to its neighbours. The Mandelbrot set is not one. Each point follows its own
rule, and no point ever looks at another.

It is here because it is the same kind of surprise. One rule a child could follow, repeated,
makes a shape of endless complexity that nobody would guess from the rule. The demo also
runs it like the automata: each pixel is a cell with a state (its z), and each step every
cell takes one more turn of z → z² + c. Run shows the set emerging step by step. (The
sandpile demo makes a fractal too, from a rule that is a true cellular automaton.)

## Colours

- **Escape bands.** Colour each point by its whole escape count, and the outside of the set
  falls into flat bands: every point in a band escaped at the same step. Watch the bands
  appear by restarting and running slowly. After 1 step only the disc of radius 2 is left
  dark, after 2 an oval, and each step after carves a closer outline of the set.
- **Smooth colouring** (the default) blends a fraction into the count, worked out from how
  far past 2 the escaping z landed, so the bands melt into a continuous gradient.
- **The colours repeat.** Near the edge of the set the escape counts climb into the
  thousands, so the palette cycles every few dozen steps (adjustable) rather than being
  stretched once over every count.

## Things to try

- **Restart, then Step.** Hover over a point to see its number and the step at which it
  escapes. Points in the big heart shape (the *main cardioid*) and the disc to its left are
  known to be inside without iterating, and the demo says so.
- **Zoom into the edge.** Scroll the mouse wheel or pinch to zoom in on the pointer; drag
  or swipe to move. The pinched waist between the main cardioid and the disc to its left is
  full of spirals.
- **Raise the stop limit as you zoom.** Deep in, the edge is made of points that take
  hundreds or thousands of steps to escape. With too low a limit they all look inside, and
  the picture goes blobby. "Go there now" jumps straight to the limit.
- **Look for small copies of the whole set**, along the spike to the left and in the
  tendrils round the edge. They are connected to the main set by threads too thin to see.
- **Zoom as far as it goes.** At about a trillion times, the 64-bit numbers the demo uses
  can no longer tell neighbouring pixels apart, and it stops.
- **Try the other palettes**, and the escape bands, at the same spot.
