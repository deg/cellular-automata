# The Mandelbrot set

Pick a point c of the plane, and treat it as a number (a *complex* number: the point
(a, b) is the number a + b*i*). Start with z = 0, and repeat one rule:

> z → z² + c

Either z stays close to 0 forever, or at some step it gets further than 2 from 0, after
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
points beyond 1 grow, and the set is where the shrinking and the nudging balance forever.

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

The other demos on this site are cellular automata, or close to one, like Langton's ant:
a grid of cells, each changing according to its neighbours. The Mandelbrot set is not
one. Each point follows its own rule, and no point ever looks at another.

It is here because it is the same kind of surprise. One rule a child could follow, repeated,
makes a shape of endless complexity that nobody would guess from the rule. The demo also
runs it like the automata: each pixel is a cell with a state (its z), and each step every
cell takes one more turn of z → z² + c. *Run* shows the set emerging step by step. (The
sandpile demo makes a fractal too, from a rule that is a true cellular automaton.)

## Julia sets

Turn the question round. Fix c, and ask which *starting points* z₀ stay near 0 when you
repeat z → z² + c. Those points make the (filled) **Julia set** of c, and every c has its
own. The inset in the corner shows the Julia set of the c under the pointer. Click the
picture to pin c, and click the inset (or *Enlarge*) to see the Julia set full size, where
you can zoom and follow orbits too.

The Mandelbrot set is a map of all the Julia sets. Fatou and Julia proved that the Julia
set of c is in one piece exactly when the orbit of 0 stays bounded, that is, when c is in
the Mandelbrot set. For any c outside it, the Julia set falls apart into a dust of
separate points. Move the pointer across the edge of the Mandelbrot set and watch the
inset shatter.

## History

- **1915–1920: Fatou and Julia.** In 1915 the French Académie des Sciences set the
  iteration of rational functions as the subject of its Grand Prix for 1918. Gaston Julia
  won it, with his *Mémoire sur l'itération des fonctions rationnelles*, Journal de
  Mathématiques Pures et Appliquées, 8th series, 1, 47–245 (1918). Pierre Fatou, who did
  not enter, worked on the same questions in notes in the Comptes Rendus in 1917 and in
  *Sur les équations fonctionnelles*, Bulletin de la Société Mathématique de France 47,
  161–271 (1919) and 48, 33–94 and 208–314 (1920). Between them they proved the fact the Mandelbrot set
  rests on: for each c, the set of starting points whose orbit under z → z² + c stays
  bounded (now called the *filled Julia set* of c; the Julia set proper is its edge) is
  all in one piece if the orbit of 0 stays bounded, and otherwise a dust of separate
  points.
- **1978–1981: Brooks and Matelski.** At a 1978 conference at Stony Brook, Robert Brooks
  and J. Peter Matelski presented a paper on groups of transformations that included a
  computer printout, in asterisks, of "the set of C's such that f(z) = z² + C has a stable
  periodic orbit": in effect, the inside of the set. It was published in the conference's
  proceedings (the chapter is dated 1980, the volume 1981).
- **1979–1980: Mandelbrot.** Benoit Mandelbrot, a mathematician at IBM's research centre
  in Yorktown Heights, used its computers to draw the set in late 1979 or early 1980
  (sources disagree on the date), and published pictures in *Fractal aspects of the
  iteration of z ↦ λz(1−z) for complex λ and z*, Annals of the New York Academy of
  Sciences 357, 249–259 (1980). His printouts showed small islands apparently separate from the main
  body, which he at first took for specks of dirt, and he thought the set was in several
  pieces. Others drew pictures of it about the same time, and who saw it first was later
  disputed. His book *The Fractal Geometry of Nature* followed in 1982.
- **1982: Douady and Hubbard.** Adrien Douady and John H. Hubbard proved that the set is
  **connected**, all in one piece: the islands are joined to the main body by filaments too
  fine for the computer to find. Their note is *Itération des polynômes quadratiques
  complexes*, Comptes Rendus de l'Académie des Sciences, Série I, 294, 123–125 (1982).
  They named it after
  Mandelbrot, for his observations of it, and their lecture notes from Orsay (1984–1985)
  worked out much of its structure.
- **1985: fame.** A. K. Dewdney's *Computer Recreations* column in Scientific American,
  August 1985, called it "the most complex object in mathematics" and brought it to a wide
  audience. It has been a favourite thing to program ever since.
- **1998: Shishikura.** Mitsuhiro Shishikura proved that the set's boundary is as wiggly as
  a curve can be: its Hausdorff dimension is 2, the same as a filled-in area. His paper is
  *The Hausdorff dimension of the boundary of the Mandelbrot set and Julia sets*, Annals
  of Mathematics 147, 225–267 (1998).
- **Still open.** Whether the set is *locally connected* (roughly: whether every bit of it
  is joined to its neighbours by paths that don't wander far) has been asked since Douady
  and Hubbard, and is still unproved. Much else would follow from it. Even its area is
  known only approximately: counting pixels gives about 1.50659.

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
- **Follow an orbit.** The white path in the Julia inset is the orbit of 0 under the
  inset's c: z₀ = 0 (ringed), z₁ = c, z₂, and so on, one dot per step. Its points belong
  to the Julia set's plane, not the Mandelbrot set's, which is why it is drawn there, at a
  scale where all of it fits however far you zoom. For c in the main cardioid it spirals
  in to a single point. In the disc to its left it ends up hopping between two points
  (gold). Each bulb round the edge has its own cycle length. For c outside the set, the
  path flies off. Enlarge the Julia set, and the path follows the pointer's z₀ instead.
- **Period doubling.** Move left along the real axis: z settles on a single point at
  −0.5, on a cycle of 2 at −1, 4 at −1.3, 8 at −1.38 and 16 at −1.3995 (zoom in to
  reach those, and raise the stop limit for the longer cycles). The doublings come faster
  and faster, and pile up at the *Feigenbaum point*, −1.401155. Beyond it lies the spike.
- **Cross the edge with the Julia inset showing.** Inside the set, the Julia set is one
  piece; step outside and it breaks into dust. Pin c in the bulb above the main cardioid,
  enlarge the Julia set, and hover inside it: its orbits settle into the same cycle of
  three points.
- **Zoom into the edge.** Scroll the mouse wheel or pinch to zoom in on the pointer; drag
  or swipe to move. Places with names:
  - **Seahorse Valley**, the pinched waist between the main cardioid and the disc to its
    left, around −0.745 + 0.117i: rows of seahorse-tail spirals.
  - **Elephant Valley**, just right of the cardioid's cusp, around 0.298 + 0.001i: trunks.
  - **A small copy of the whole set** on the spike to the left, at −1.7549.
- **Find π.** Points just above the pinch at −0.75 take longer and longer to escape as you
  approach it: −0.75 + 0.001i escapes at step 3,143, and −0.75 + 0.0001i at step 31,417.
  The count times the distance tends to π. Many people noticed this independently in the
  early 1990s. To see it, zoom in on the pinch until the pointer can sit that close, raise
  the stop limit past the count you expect, and hover.
- **Keep your bearings with the overview map** (top right). It shows the whole set, with
  the view marked in gold: a rectangle, or cross hairs once you are too deep for one. Click
  the map to jump the view there, at the same zoom.
- **Raise the stop limit as you zoom.** Deep in, the edge is made of points that take
  hundreds or thousands of steps to escape. With too low a limit they all look inside, and
  the picture goes blobby. *Go there now* jumps straight to the limit.
- **Look for small copies of the whole set**, along the spike to the left and in the
  tendrils round the edge. They are connected to the main set by threads too thin to see.
- **Zoom as far as it goes.** Ordinary 64-bit numbers can't tell neighbouring pixels
  apart past about a trillion times, so beyond a few billion times the demo switches to a trick
  called *perturbation*: it follows one point's orbit in double-length numbers (about 32
  digits) and works out every other pixel as a small difference from it. That takes it to
  about 10^27 times, where even those run out. Raise the stop limit as you go: deep
  pictures need thousands of steps, and take a while to fill in. (An enlarged Julia set
  still stops at a trillion.)
- **Try the other palettes**, and the escape bands, at the same spot.

## Further reading

- The Wikipedia article [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set).
- Numberphile, *The Mandelbrot Set*, with Holly Krieger,
  [video](https://www.youtube.com/watch?v=NGMRB4O922I): why the rule makes the shape.
- A. K. Dewdney, *A computer microscope zooms in for a look at the most complex object in
  mathematics*, in his *Computer Recreations* column, Scientific American 253, 16–24
  (August 1985),
  [article](https://www.scientificamerican.com/article/computer-recreations-1985-08/).
- John Horgan, *Mandelbrot Set-To*, Scientific American 262, 30–34 (April 1990),
  [article](https://www.scientificamerican.com/article/mandelbrot-set-1990-horgan/): who
  discovered it.
- A. Douady and J. H. Hubbard, *Exploring the Mandelbrot set: the Orsay notes*,
  [PDF](https://pi.math.cornell.edu/~hubbard/OrsayEnglish.pdf): the mathematics.
- Robert Munafo's [Mu-Ency](https://mrob.com/pub/muency.html), an encyclopedia of the
  set's places and lore.
