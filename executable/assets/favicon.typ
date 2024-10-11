#import "@preview/suiji:0.3.0": gen-rng, normal
#import "@preview/cetz:0.2.2"
#import "@preview/finite:0.3.0"

// #import draw_finite: state, transition,

#set page(
  width: auto,
  height: auto,
  margin: 0cm,
)

#let rng = gen-rng(42)

#let curr_rdm = ()

#let num_samples = 3000;
#let (_, rdms) = normal(rng, loc: 0.0, scale: 1.0, size: num_samples)

#let half_num_bins = 3;

#let num_bins = half_num_bins * 2;

#let hist_bins_limits = range(num_bins + 1).map(it => float(it - half_num_bins) / float(half_num_bins))

#let hist_bins = hist_bins_limits.slice(0, -1).zip(hist_bins_limits.slice(1))

#let hist = hist_bins.map(((start, end)) => {
  rdms.filter(rdm => start <= rdm and rdm < end).len()
})

#let hist_normf = 1 / float(num_samples) * half_num_bins;

#let hist_norm = hist.map(it => (float(it) * hist_normf));

#let hist_points = hist_bins_limits.slice(0, -1).zip(hist_norm)

#let normal_distr = x => 1.0/calc.sqrt(2.0* calc.pi) * calc.exp(-calc.pow(x, 2.0) / 2.0)

#let norm_samples = hist_bins_limits.map(it => normal_distr(it))

#let norm_min = normal_distr(1.0)
#let norm_max = normal_distr(0.0)

#import cetz: canvas

#canvas(length: 1cm, background: none, {
  import cetz: plot, draw
  import finite: automaton, layout
  // manually extracted from resulting svg. Target (16pt)^2
  // x: 16.0 pt / 132.489075874974 pt = 0.120764673573
  // y: 16.0 pt / 141.73249997918526 pt = 0.112888716437
  draw.scale(x: 0.120764673573, y: 0.112888716437)
  plot.plot(
    size: (5, 5),
    name: "plot",
    axis-style: none,
    y-min: norm_min,
    // y-max: norm_max,
    {
      plot.add(
        hypograph: true,
        style: (stroke: red, fill: red),
        domain: (-1.0, 1.0), 
        normal_distr,
      )
      plot.add-bar(
        style: (stroke: (paint: black, thickness: 0.6pt), fill: green.darken(30%)),
        bar-width: 0.9 / half_num_bins,
        (..hist_points, (1.0, 0.0)),
      )
      // plot.add-anchor("automata", (-0.2, (norm_max + norm_min)/2.0 - 0.02))
    }
  )

  // removed due to it 
  // * not being visible in most representations anyways
  // * transparency breaks svgcleaner
  // * it breaks draw.scale
  // draw.content(
  //   "plot.automata", 
  //   [#automaton(
  //     style:(
  //       state: (fill: black, stroke: none, radius: 0.40),
  //       transition: (stroke: (paint: black)),
  //       q1: (label: none),
  //       q2: (label: none, fill: black.transparentize(50%)),
  //       q1-q1: (anchor:top, curve: 0.3),
  //       q1-q2: (curve: 0),
  //     ),
  //     final: none,
  //     initial: none,
  //     (
  //       q1: (q1: none, q2: none),
  //       q2: (),
  //     )
  //   )]
  // )
})
