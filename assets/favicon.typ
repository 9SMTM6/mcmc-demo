#import "@preview/suiji:0.3.0": gen-rng, normal
#import "@preview/cetz:0.2.2"
#import "@preview/finite:0.3.0"

// #import draw_finite: state, transition,

#set page(
  width: auto,
  height: auto,
  margin: 0cm,
  // fill: black,
)

#let style = (stroke: black, fill: blue.darken(20%), background: black)

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

#canvas(length: 1cm, background: black, {
  import cetz: plot, draw
  import finite: automaton, layout
  // manually extracted from resulting svg. Target (16pt)^2
  // x: 16.0pt / 131.60874982452165pt = 0.1215724639990376
  // y: 16.0pt / 141.73249997918526pt = 0.1128887164365953
  // Disabled for now since content doesn't scale, and the alternative element functions cause compilation errors even using the examples...
  // draw.scale(x: 0.1215724639990376, y: 0.1128887164365953)
  plot.plot(
    size: (5, 5),
    name: "plot",
    axis-style: none,
    y-min: norm_min,
    // y-max: norm_max,
    {
      plot.add-bar(
        style: style,
        bar-width: 1.0 / half_num_bins,
        (..hist_points, (1.0, 0.0)),
      )
      plot.add(
        style: (stroke: red),
        domain: (-1.0, 1.0), 
        normal_distr,
      )
      plot.add-anchor("automata", (-0.2, (norm_max + norm_min)/2.0 - 0.02))
    }
  )

  draw.content(
    "plot.automata", 
    [#automaton(
      style:(
        state: (fill: olive, stroke: none, radius: 0.5),
        transition: (stroke: (dash:"dashed", paint: olive)),
        q1: (label: none),
        q2: (label: none),
        q3: (label: none),
        // q1-q2: (anchor: left),
        q1-q1: (anchor:left),
        q2-q2: (anchor:top),
        q3-q3: (anchor:right),
      ),
      final: none,
      initial: none,
      layout: layout.circular.with(offset: -25deg, dir: right, radius: 0.7),
      (
        q1: (q1: none, q2: none),
        q2: (q2: none, q3: none),
        q3: (q3: none),
      )
    )]
  )
})
