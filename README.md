# TODO:

## Compute shader

I dont know if any of the below ideas for speeding up the diff rendering would work out. And in the end, dont think I'll get much use out of knowing how to do that (meanign I'll forget it anyways).

One thing where the probability of reuse is much higher, and that more connects to my past knowledge, is compute shaders.
Its a bit annoying to go away from the ability to render everything in real time always, but at the same time that lifts hard limits that were always going to be there with the previous approach - whether we were close to reaching them or not.

I currently envison this approach (lets see how much of this I'll get):

0. still use max-scaling. With near-uniform distributions we otherwise get a far to depressed dynamic range where things actually happen.
1. determine device limits to divide work accordingly
2. since we don't render directly anymore, I've got much more freedom in splitting up the workload, concretely optimizing for typical buffers. So I intend to break up the determination of the approx distribution into multiple sets of reference points.
3. do it in a cumpute shader
4. The result can be stored either:
    * in a texture.
    * a storage buffer
5. the storage will never have to leave the GPU. Compute it once, read the result it in a fragment shader where the actual colors are determined
6. with that I could also consider decoupling calculation resolution and render resolution, but I think for now I'll keep them coupled
7. In order to avoid numerical stability issues I'll probably add some normalization after N steps. I have to decide on a proper strategy for that. Perhaps I can actually do it based on current maximum instead. Most of these strategies will lead to systemctic errors in the precision, since rounding might happen in different situations, but I'm fine with that.
8. I should probably introduce some kind of loading (compute) visualization...

## Scaling of distributions and approximations

For the approximation, distribution scaling doesn't work currently, since we've got difficulties scaling it. For distribution scaling we would have to integrate it for normalization. For every change, currently every render.
Alternatives: Max-Scaling. Still problematic for approximation, but perhaps doable if we scan all values with some "lower resolution" compute shader. But that would excascerbate the current scaling issue, since then this would have to happen before render and for every subsequent one.

Alternatively we could use the render shader as a compute shader of sorts, and use it for normalization with 1 frame delay. IDK if thats gonna go well with eguis energy saving render approach, and it also leads to artifacts/flickering, which may be an accessibility issue (this application is already not accessible, but this would hurt another groups than so far).

## Problematic approach to diff rendering

The approach I took to rendering/calculating the approximation doesn't scale at all.
The issue is (probably) that EVERY fragment shader (every pixel) will read every approximation point. That won't do.
We would need to split this up, but thats not really easy, and probably goes kind of deep into game development adjacent topics, which I dont really want to do.

## Find a way to Profile performance issues

Generally I should find a way to profile webgpu render. Currently I'm mostly guessing from past reference points, and while I'm decently certain in my conclusions, it would be nice to have confirmation, and some foresight into upcoming issues ("will solving this just lead to another very close bottleneck", which is currently stopping me from some experimentations).

Currently working:
* with feature "profile"
  * we start puffin, which can measure scope duration and gives a generic flame-graph from the rust side (no GPU)
  * there appears a "backend" button which was largely copied from the egui demo and offers frametimes as well as informations about GPU allocations etc.

Future ideas:
* implement tracing from rust side for wgpu actions, blocked by: https://github.com/gfx-rs/wgpu/issues/5974
* implement the stuff from https://webgpufundamentals.org/webgpu/lessons/webgpu-timing.html

## Non-Batched execution

Currently I only support batched execution, to quickly see results of different configurations.
In the future I also want to support a substep execution such as in the [original inspiration](https://chi-feng.github.io/mcmc-demo/app.html?algorithm=RandomWalkMH&target=banana).

## Execution of batches on web

Update: I've now setup a parallel deployment to cloudflare pages (mcmc-webgpu-demo.pages.dev).

Note: This is not actually the primary issue, though annoying. In memory size was problematic occasionally, but primary issues were render speed / size on VRAM / GPU Caches (i think?). Especially in the diff approach. More to come about this.

To be able to efficiently execute batches in the background on the web we would need a bunch of things to fall into place.
We want to be able to execute that task in a background thread.
However for that to work we need wasm-threads, and theres a few issues with using this.

WASM threads rely on sharedarraybuffers, and these need some headers to be active on the webpage, as described here:
* https://web.dev/articles/webassembly-threads
* https://web.dev/articles/coop-coep

Checking the deployed github page, these headers are not set by default, and setting headers for github pages isnt allowed by default:
https://stackoverflow.com/questions/14798589/github-pages-http-headers

One of the answers suggests setting the headers with `<meta http-equiv="HEADER">`, however that doesnt work for these headers:
* not listed here: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta#http-equiv
* someone tried it and it did not work: https://www.reddit.com/r/WebAssembly/comments/tuinyg/run_wasm_from_github_pages_possible/

There are suggestes solutions, but AFAICT these require you to serve (or proxy) the artifacts on another domain (pages.dev if using cloudflare). Note, this may also allow me to use brotli instead of gzip compression.

Even with these headers, compatibility is questionable, as at least in the beginning mobile browsers did not enable this feature, because it eats resources (as it leads to another process for that page specifically):

> [https://web.dev/articles/webassembly-threads] However, this mitigation was still limited only to Chrome desktop, as Site Isolation is a fairly expensive feature, and couldn't be enabled by default for all sites on low-memory mobile devices nor was it yet implemented by other vendors.

Also note the difficulties in using threads in Rust because of the generic `wasm32-unknown-unknown` target rust uses:

> [https://web.dev/articles/webassembly-threads] If Wasm is intended to be used in a web environment, any interaction with JavaScript APIs is left to external libraries and tooling like wasm-bindgen and wasm-pack. Unfortunately, this means that the standard library is not aware of Web Workers and standard APIs such as std::thread won't work when compiled to WebAssembly.


Altogether I'm not sure of the right route. Supporting threads seems like a lot of work with unknown end result.

An alternative might be simply using a promise with wasm_bindgen_futures and hoping that the browsers manage to make it not suck. Its unlikely this will use proper threads, but ...

## Support more PRNG and low-discrepancy randomness

* actually make PRNG generic and allos choice in UI, currently hard-coded
* https://en.wikipedia.org/wiki/Quasi-Monte_Carlo_method
* https://crates.io/crates/sobol_burley
* https://crates.io/crates/halton
* https://crates.io/crates/quasirandom

# eframe template

### Learning about egui

The official egui docs are at <https://docs.rs/egui>. If you prefer watching a video introduction, check out <https://www.youtube.com/watch?v=NtUkr_z7l84>. For inspiration, check out the [the egui web demo](https://emilk.github.io/egui/index.html) and follow the links in it to its source code.

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> (DISABLED) `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.
>
> If you renamed the `main` branch to something else (say you re-initialized the repository with `master` as the initial branch), be sure to edit the github workflows `.github/workflows/pages.yml` file to reflect the change
> ```yml
> on:
>   push:
>     branches:
>       - <branch name>
> ```

You can test the template app at <https://emilk.github.io/eframe_template/>.

## Updating egui

As of 2023, egui is in active development with frequent releases with breaking changes. [eframe_template](https://github.com/emilk/eframe_template/) will be updated in lock-step to always use the latest version of egui.

When updating `egui` and `eframe` it is recommended you do so one version at the time, and read about the changes in [the egui changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md) and [eframe changelog](https://github.com/emilk/egui/blob/master/crates/eframe/CHANGELOG.md).
