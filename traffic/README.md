# Traffic

Cars drive round a one-lane ring road. The road is a row of cells, each empty or holding
one car, and each car has a whole-number **speed**, from 0 up to a *maximum speed*: the
number of cells it moves in a step. Every step, every car does four things, all at once
and in this order:

1. **Speed up.** If it is below the maximum speed, its speed goes up by one.
2. **Slow down for the car ahead.** If its speed is more than the number of empty cells in
   front of it, it drops to that number, so it can't hit the car ahead.
3. **Dawdle.** If it is moving, then with probability *dawdling* its speed drops by one
   more.
4. **Move.** It moves on by its speed. The road's end joins its start.

That is the whole rule. Nothing ever blocks the road, yet stop-and-go **jams** appear by
themselves, and drift **backwards** against the traffic while every car in them moves
forwards. The picture is the road's history: the road runs across, one row per step, and
time runs down the screen, so a jam is a streak of stopped cars (coral) slanting down to
the left, between cars at full speed (teal). The strip underneath is the road now.

The order matters. A car that has just braked for the car ahead can still dawdle, and so
slow down more than it had to. These overreactions pass from car to car down a queue
until one car stops: that is how a jam starts. With steps 2 and 3 swapped there are no
overreactions, and no jams start of their own accord.

With maximum speed 1 and no dawdling (the *Rule 184* preset), a car moves exactly when the
cell ahead is empty. That is *Rule 184* in Stephen Wolfram's numbering of the
one-dimensional automata, which you can also run in Elementary cellular automata, another demo in
this project.

The model is named after Kai Nagel and Michael Schreckenberg, the two physicists who
introduced it in 1992.

## Flow against density

Traffic engineers measure a road by its **flow**: how many cars pass a point in a given
time. Here it is the number of cars passing a point per step. Plotted against the
**density**, the fraction of cells holding a car, it makes the *fundamental diagram*,
which the demo's chart measures as you watch:

- At low density, cars rarely meet. Each drives at about its maximum speed, so the flow
  rises in proportion to the number of cars.
- At high density, cars are held up by the cars ahead, and more cars means less flow.
- In between is a density of greatest flow: the road's capacity.

Without dawdling, the answer is exact. Once the road has settled, every car drives at
full speed as long as there is room, and the flow is the density times the maximum speed.
Past a density of 1/(maximum speed + 1) there isn't room, and the cars move into every
empty cell each step, so the flow is 1 minus the density. These are the two straight gold
lines on the chart. Dawdling lowers the curve and rounds off its peak. With maximum speed
5 and dawdling 0.5 the top is flat: over long runs Nagel's thesis finds the greatest flow,
0.318 cars a step, at density 0.085, half the 1/6 it would be without dawdling, and from
about 0.08 to 0.10 the flow is nearly the same.

The blue curve is measured on 49 test roads of 2,000 cells, one for each density from 0.02
to 0.98, each run for 500 steps to settle and then 1,000 more, at the current *maximum
speed* and *dawdling*; near the top, from 0.04 to 0.20, each density averages 8 roads. The
coral dot is the road on screen, averaged over its last 200 steps. One road's flow
wanders: near the top, two roads at the same settings can differ by 0.02 or more over
1,000 steps, which is more than the top of the curve rises or falls. The thesis also notes
that the maximum depends on the road's length. Near the peak a short road flows faster
than a long one: at maximum speed 5, dawdling 0.5 and density 0.08, roads of 500 cells
average about 0.35 cars a step, and long ones 0.32. So on a short road the dot can sit
above the curve.

## History

- **1986: the first traffic automaton.** M. Cremer and J. Ludwig, *A fast simulation
  model for traffic flow on the basis of Boolean operations*, Mathematics and Computers in
  Simulation 28, 297–303 (1986), is the first cellular automaton model of traffic that the
  review below knows of.
- **1992: Nagel and Schreckenberg.** Kai Nagel and Michael Schreckenberg, *A cellular
  automaton model for freeway traffic*, Journal de Physique I 2, 2221–2229 (1992),
  [doi:10.1051/jp1:1992277](https://doi.org/10.1051/jp1:1992277). In most of the
  traffic automata physicists had studied, as the 1995 paper below describes them, a car
  moved at most one cell a step. Nagel and Schreckenberg let it go faster, up to a speed
  limit, added the random dawdling step, and found in simulations the change from smooth
  flow to stop-and-go waves seen on real motorways. Nagel's thesis argues that the model is
  minimal: any further simplification loses something essential, such as jams that form
  by themselves. Its scale comes from a dense jam, in which each car takes up about 7.5 m:
  that is one cell. Cars in free traffic at maximum speed 5 and dawdling 0.5 average 4.5
  cells a step, which matches about 120 km/h if a step is 1 second, about a driver's
  reaction time.
- **1993–1995: an exact answer.** A. Schadschneider and M. Schreckenberg, *Cellular
  automation models and traffic flow* (sic), Journal of Physics A 26, L679–L683 (1993),
  [doi:10.1088/0305-4470/26/15/011](https://doi.org/10.1088/0305-4470/26/15/011), found the
  flow for maximum speed 1 and any dawdling probability p with an improved mean-field
  approximation: at density c, it is (1 − √(1 − 4(1 − p)c(1 − c)))/2. Schreckenberg,
  Schadschneider, Nagel and Ito proved it exact in 1995, in the paper below. For higher
  maximum speeds with dawdling, there was still no exact answer when the review below was
  written, in 2000.
- **1994–1995: fast enough for a country.** Nagel's thesis, *High-speed microsimulations
  of traffic flow* (University of Cologne, 1995), and M. Schreckenberg, A. Schadschneider,
  K. Nagel and N. Ito, *Discrete stochastic models for traffic flow*, Physical Review E
  51, 2939–2949 (1995), [arXiv:cond-mat/9412045](https://arxiv.org/abs/cond-mat/9412045),
  report the model running on supercomputers. On a 64-node Intel Paragon it could
  simulate 280,000 km of single-lane road as fast as real time, when the whole of
  Germany's motorways come to about 60,000 km of single lanes. The work was part of a
  collaboration between Cologne and TRANSIMS, a transportation simulation project at Los
  Alamos National Laboratory.
- **2000: a review.** D. Chowdhury, L. Santen and A. Schadschneider's long review of the
  statistical physics of traffic gives the model a chapter of its own, and another to
  the many models that extend it: more lanes, traffic both ways, cars slow to start.

How realistic is it? Nagel's thesis compares it with motorway measurements. Motorways carry
up to about 2,000 cars an hour per lane. The model's greatest flow, at maximum speed 5
and dawdling 0.5, is 0.318 cars a step, about 1,150 an hour at a step a second; Nagel
suggests that the practitioners' "maximum flow" is not measured the way the model's
long-run average is. Real jams move backwards at about 15 km/h; matching the model's to
that gives a step of about 0.7 seconds. All the ways of matching it agree that a step is
roughly a second and a cell roughly 7.5 m.

## Things to try

- **Watch a jam start.** Choose *Jams* and watch the newest rows. A car dawdles, the car
  behind brakes for it and dawdles too, and a few steps later a short line of coral, a
  queue of stopped cars, is drifting down to the left. Cars leave its front and join its
  back, so it moves backwards while every car moves forwards.
- **Take the dawdling away.** Set *dawdling* to 0. The jams already there melt away. If
  the density is below 1/(maximum speed + 1), 1/6 for speed 5, every car ends up at full
  speed forever, and the picture becomes parallel stripes. Then set *dawdling* to 0.05.
- **Too dense to flow freely.** Choose *No dawdling*: at density 0.5 the cars can't all
  drive at full speed. The jams of the random start settle into a pattern that moves back
  one cell a step, forever.
- **Find the capacity.** Choose *Jams*, then move the *density* slider and watch the coral
  dot walk along the measured curve. The top is flat: anywhere from about 0.08 to 0.10
  carries nearly the most traffic. Then lower *dawdling* and watch the curve rise: careful
  drivers carry more traffic.
- **Speed limits.** At density 0.15 with dawdling 0.5, lowering the *maximum speed* from 5
  to 4 hardly changes the flow, and 3 costs only a little. Below that it falls fast.
- **Rule 184.** Choose *Rule 184*, then open Elementary cellular automata, choose Rule 184 and a
  random row: the same kind of picture, in one colour.
- **A longer road.** Set *Road length* to 800 to see many jams at once.

## Further reading

- The Wikipedia article
  [Nagel–Schreckenberg model](https://en.wikipedia.org/wiki/Nagel%E2%80%93Schreckenberg_model).
- D. Chowdhury, L. Santen and A. Schadschneider, *Statistical physics of vehicular traffic
  and some related systems*, Physics Reports 329, 199–329 (2000),
  [doi:10.1016/S0370-1573(99)00117-9](https://doi.org/10.1016/S0370-1573(99)00117-9),
  [arXiv:cond-mat/0007053](https://arxiv.org/abs/cond-mat/0007053). Section 8 is about this
  model.
- K. Nagel, *High-speed microsimulations of traffic flow*, PhD thesis, University of
  Cologne (1995),
  [PDF](https://svn.vsp.tu-berlin.de/repos/public-svn/publications/kn-old/thesis/complete/thesis-nagel.pdf).
- M. Schreckenberg, A. Schadschneider, K. Nagel and N. Ito, *Discrete stochastic models
  for traffic flow*, Physical Review E 51, 2939–2949 (1995),
  [doi:10.1103/PhysRevE.51.2939](https://doi.org/10.1103/PhysRevE.51.2939),
  [arXiv:cond-mat/9412045](https://arxiv.org/abs/cond-mat/9412045).
