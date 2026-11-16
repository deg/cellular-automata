# The fisheye trial (branch `fisheye-trial`)

This branch keeps an experiment that was tried and **not kept**: a fisheye ("hyperbolic")
view for the Mandelbrot demo. It is commit `01395d3` (bead cellular-automata-x51.8), on
top of `main` as it was then (`0bb2aa0`). David looked at it on 2026-11-16, judged that
it does not look good, and it was rolled back from `main`. It is kept here in case the
idea is revisited, so the work and what it taught are not lost.

## The problem it was meant to solve

Zoomed in, the Mandelbrot view loses its context: you can't tell where in the set you
are. Orbits, drawn as lines over the picture, ran mostly offscreen.

## What it did

A **Fisheye** checkbox. The middle half of a circle as wide as the picture's shorter side
showed the view at its own zoom, flat. Beyond that the scale grew exponentially, so the
rim reached 4 from the centre and took in the whole set. The lens lived in `Viewport`
(`Lens::Flat | Fisheye { k }`, with `k` solved by bisection from the zoom), and its
inverse had a closed form, for drawing orbits and markers. Through the lens, zoom centred
on the middle, a drag moved the centre at the core's scale, and every move recomputed the
whole picture. Orbit steps were cut into pieces so they bent through the lens.

## What we learned

- **A pure exponential lens magnifies unevenly almost from the centre.** At ×100 only a
  small patch in the middle showed the true zoom. A flat core (half the radius) fixed
  that, and is the version on this branch.
- **The ring doesn't give useful context.** The whole set appears bent round the rim, so
  its familiar shape is gone and it doesn't help you see where you are. Deeper in, the
  ring packs more and more scales into concentric bands, which read as clutter, with
  speckle where the compression is strongest (there is no supersampling).
- **It costs the cheap navigation.** A flat view pans by shifting pixels it has already
  computed, and shows the old picture scaled while a new one computes. Through a fisheye
  every pixel's point changes with any move, so all of it has to be recomputed.
- **Simpler tools solved the problem instead**, and are on `main`. The overview map
  (x51.7) shows where the view is on the whole set. The orbit now belongs to the Julia
  inset (x51.6, x51.9): its points are points of the Julia set's plane, which the inset
  shows whole, so a bounded orbit is always in view.

## If it is revisited

Judge it in the demo itself, not from still pictures. Ideas not tried:
- supersampling the ring
- a narrower ring
- a log-polar strip in a side panel instead of a lens (angle across, log of the distance
  from the centre down), so zooming becomes scrolling
