# TODO:

## Current prograss blockers:

* wgpu pipeline-overridable constants are not supported on glsl-out
  * but that is required via https://github.com/gfx-rs/wgpu/blob/7b4cbc26192d6d56a31f8e67769e656a6627b222/wgpu/Cargo.toml#L148C1-L151C20 (maybe removable via patch?)
  * issue: https://github.com/gfx-rs/wgpu/issues/3514
  * this is what I considered for the compute shader to set the compute_group dimensions.
  * AAActually its a naga issue. The source is in that file: https://github.com/gfx-rs/wgpu/blob/7b4cbc26192d6d56a31f8e67769e656a6627b222/naga/src/back/wgsl/writer.rs#L111 ([commit](https://github.com/gfx-rs/wgpu/commit/2929ec333cee981ef4cbf783c0e33d208c651c4d))
    * its surfaced via naga_oil
    * it might be an oversight. In that commit wgsl did not support `pipeline-overridable constants`, but later there was another PR that merged support, but it might've forgotten about these `writer`s. Or it was an accepted shortcoming, since it doesnt seem to be possible to do that stuff entirely without work (all the valid backends added a `pipeline_constant.rs` file).
    * actually, from my understanding, fixing this for naga wont fix the issue for naga_oil, since naga_oil wants to use naga as a preprocessor.
* wgpu 22.1 update brings some perhaps helpful updates, is blocked on wgsl_bindgen
* using shared memory multithreading on the web is blocked by https://github.com/emilk/egui/issues/4914
  * may also be patchable
  * introducing merge commit: https://github.com/emilk/egui/commit/bfadb90d429c9e6aa1beba37c6c38335e7462eb0
  * original commit: https://github.com/emilk/egui/pull/3595/commits/c5746dbd37a31d9a90c8987449b4089eb910ad8c
  * note that nothing was changed but the unification of imports. If concurrency wasn used in another commit building on this, it should be easy to patch

## Compute shader

I dont know if any of the below ideas for speeding up the diff rendering would work out. And in the end, dont think I'll get much use out of knowing how to do that (meanign I'll forget it anyways).

One thing where the probability of reuse is much higher, and that more connects to my past knowledge, is compute shaders.
Its a bit annoying to go away from the ability to render everything in real time always, but at the same time that lifts hard limits that were always going to be there with the previous approach - whether we were close to reaching them or not.

I currently envison this approach (lets see how much of this I'll get):

0. still use max-scaling. With near-uniform distributions we otherwise get a far to depressed dynamic range where things actually happen.
1. determine device limits to divide work accordingly
2. since we don't render directly anymore, I've got much more freedom in splitting up the workload, concretely optimizing for typical buffers. So I intend to break up the determination of the approx distribution into multiple sets of reference points.
3. do it in a compute shader - todo: with pipeline-overridable constants for render size (https://github.com/gfx-rs/wgpu/releases/tag/v0.20.0)
4. The result can be stored either:
    * in a texture.
    * a storage buffer (as long as that is efficient, textures are optimized to only load parts)
5. execute that compute shader (and the original computation) in a separate thread (see [wasm-threads](README#wasm-threads)) with a nice loading animation - `egui::widgets::ProgressBar` - while waiting. This works around inefficient diff approach, though perhaps it can still be improved with some space partition like Quadtree.
6. the storage will never have to leave the GPU. Compute it once, read the result it in a fragment shader where the actual colors are determined
7. with that I could also consider decoupling calculation resolution and render resolution, but I think for now I'll keep them coupled
8. In order to avoid numerical stability issues I'll probably add some normalization after N steps. I have to decide on a proper strategy for that. Perhaps I can actually do it based on current maximum instead. Most of these strategies will lead to systemctic errors in the precision, since rounding might happen in different situations, but I'm fine with that.

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
  * actually, perhaps instead use https://github.com/Wumpf/wgpu-profiler

## Non-Batched execution

Currently I only support batched execution, to quickly see results of different configurations.
In the future I also want to support a substep execution such as in the [original inspiration](https://chi-feng.github.io/mcmc-demo/app.html?algorithm=RandomWalkMH&target=banana).

## wasm-threads

To be able to efficiently execute batches in the background on the web we would need a bunch of things to fall into place.
We want to be able to execute that task in a background thread.
However for that to work we need wasm-threads, and theres a few issues with using this.

WASM threads rely on sharedarraybuffers, and these need some headers to be active on the webpage, as described here:
* https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements
* https://web.dev/articles/webassembly-threads
* https://web.dev/articles/coop-coep

Github pages cant do that.
I've now setup a parallel deployment to cloudflare pages (mcmc-webgpu-demo.pages.dev), that is able to set these headers (and as added bonus also deploys brotli compressed artifacts, not just gzip).

I can test for the availability with `self.crossOriginIsolated`.

Even with these headers, compatibility is questionable, as at least in the beginning mobile browsers did not enable this feature, because it eats resources (as it leads to another process for that page specifically):

> [https://web.dev/articles/webassembly-threads] However, this mitigation was still limited only to Chrome desktop, as Site Isolation is a fairly expensive feature, and couldn't be enabled by default for all sites on low-memory mobile devices nor was it yet implemented by other vendors.

Also note the difficulties in using threads in Rust because of the generic `wasm32-unknown-unknown` target rust uses:

> [https://web.dev/articles/webassembly-threads] If Wasm is intended to be used in a web environment, any interaction with JavaScript APIs is left to external libraries and tooling like wasm-bindgen and wasm-pack. Unfortunately, this means that the standard library is not aware of Web Workers and standard APIs such as std::thread won't work when compiled to WebAssembly.

I've found 2 libraries that solve this well enough for my purposes:
* wasm-mt (supports generic futures? hard requirement on wasm-pack)
  * actually no: 
    > wasm-mt is not efficient in that it does not include support of the standard thread primitive operations: 
    > 1. shared memory based message passing and mutexes,
* [wasm_thread](https://github.com/chemicstry/wasm_thread)
  * updated 5 months ago
  * API Clone of std::thread
  * works both native and on web
  * build process already integrated on here
  * loads a bunch of js scripts, would be nice to avoid, but ATM it seems like the best deal 
* [wasm-futures-executor](https://github.com/wngr/wasm-futures-executor)
  * 2 years ago last update
  * API Clone of futures_executor::ThreadPool 
  * identical build process to wasm_thread
  * though I think it embeds the snippets differently.
  * i dont think it has a native compat layer, but easy to address with a reexport
  * I think it works differently when spawning tasks:
    > There is a significant overhead of sending and spawning futures across the thread boundary. It makes most sense for long-lived tasks (check out the factorial demo, which is a rough 3x performance increase). As always, make sure to profile your use case.

Both will use nightly to rebuild the standard library, and a bunch of other flags (the same seems to be true for the fairly polular `wasm-bindge-rayon`). Which makes things annoying and at least difficult with trunk.

Thunk has a few examples about web-workers (undocumented at root):
* https://github.com/trunk-rs/trunk/tree/main/examples/webworker
  * seems to be basic example. Manually creates a js script in rust and loads it
* https://github.com/trunk-rs/trunk/tree/main/examples/webworker-module
  * seems to be using trunk to bundle an antumatically created js loader to avoid the managing in the previous example.
  * has an identical worker script
* https://github.com/trunk-rs/trunk/tree/main/examples/webworker-gloo
  * uses gloo-worker, an abstraction on the web api that makes some annoying parts in app.rs go away. Though that requires a 3-way split, into lib with common impl, app.rs and worker.rs
  * this 3 way split avoids the need for the loading of another module. However it likely also means that A LOT more data has to be loaded, since lib will be used in both, and my lib code is likely to require depending on the major size contributors (eg. wgpu). wasm component model may solve that in the future, but now is the present.
None of these seem to work with shared memory.
Considering that I generate a bunch of data in a serial fashion (not sped up by parallelism, this is unavoidable for markov chains AFAIK) and want to then transport data with a shared bufferbinding, this might be problematic. Unless the bufferbinding can survive the serialization over the border between.

See also https://github.com/trunk-rs/trunk/issues/680

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
