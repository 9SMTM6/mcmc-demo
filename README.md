# TODO:

## Current progress blockers:

* [Fixedish, still needs search-replace code] wgpu pipeline-overridable constants are not supported on glsl-out
  * but that is required via https://github.com/gfx-rs/wgpu/blob/7b4cbc26192d6d56a31f8e67769e656a6627b222/wgpu/Cargo.toml#L148C1-L151C20 (maybe removable via patch?)
  * issue: https://github.com/gfx-rs/wgpu/issues/3514
  * this is what I considered for the compute shader to set the compute_group dimensions.
  * AAActually its a naga issue. The source is in that file: https://github.com/gfx-rs/wgpu/blob/7b4cbc26192d6d56a31f8e67769e656a6627b222/naga/src/back/wgsl/writer.rs#L111 ([commit](https://github.com/gfx-rs/wgpu/commit/2929ec333cee981ef4cbf783c0e33d208c651c4d))
    * its surfaced via naga_oil
    * it might be an oversight. In that commit wgsl did not support `pipeline-overridable constants`, but later there was another PR that merged support, but it might've forgotten about these `writer`s. Or it was an accepted shortcoming, since it doesnt seem to be possible to do that stuff entirely without work (all the valid backends added a `pipeline_constant.rs` file).
    * actually, from my understanding, fixing this for naga wont fix the issue for naga_oil, since naga_oil wants to use naga as a preprocessor.
  * [wgsl-bindgen issue](https://github.com/Swoorup/wgsl-bindgen/issues/39)
  * [naga_oil issue](https://github.com/bevyengine/naga_oil/issues/102)
* [Fixedish] using shared memory multithreading on the web is blocked by https://github.com/emilk/egui/issues/4914
  * currently using a patched version
  * reverts relevant changes from: https://github.com/emilk/egui/pull/3595/commits/c5746dbd37a31d9a90c8987449b4089eb910ad8c


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

### Webgpu in a background thread

Note that a compute shader in a webworker is supposed to work according to [spec](https://www.w3.org/TR/webgpu/#navigator-gpu), but [apparently firefox doesnt support that](https://developer.mozilla.org/en-US/docs/Web/API/WorkerNavigator/gpu), even on nightly. So here's hoping that they will eventually support it when they release.

Uuuuh. Just saw that it apparently explicitly isn't supported on Chromium Linux either...

I've validated that moving Buffers etc over the webworker boundary isn't going to happen anytime soon. Wgpu stance is that its 'not doable' with webgpu objects, and the official specs don't really menthion things in that capacity.
Even if it were to be doable, in a broadly supported way, it could very well end up being restricted again after the fact, considering e.g. the story with sharedarraybuffer.
So I'll work under the assumption thats not going to happen.
If I share buffers etc. over the boundary, itll have to go over CPU, at least on the web.

So the choices are:

* compute cluster on background thread, serialize to CPU to share to main thread
  * according to official info (linked somewhere in comments) only supported on Chromium
  * allows full control over the adapter-settings etc, and should use separate resources from egui.
* find a way to dispatch that work on the main thread.
  * this is somewhat annoying to integrate in egui
  * avoids going over the CPU
  * in App::new the creation_context holds the wgpu device and queue that are also used by egui.
    I believe that its required to use these to use buffers etc without going over the CPU.
    I could perhaps immediately stick these references into another object that then recieves gpu tasks.
    This object could dispatch that work from a background thread in native as a later optimization (with tests whether that actually does anything for perf).

### WebGPU Synchronization

Originally I was under the impression that global synchronization on the GPU was impossible, from some article that I might look for again in the future.

But it seems I was mistaken.
Barriers are indeed only allowed on a workgroup level.
Atomics however seem to be synchronized on the entire GPU.
That is at least whats done [here](https://webgpufundamentals.org/webgpu/lessons/webgpu-compute-shaders-histogram.html).

Look that up in the future.

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
With a history of past proposals etc.
Maybe also with history navigation, if that actually fits the workflow (atm I dont thik so, since RNG is seeded once currently, and we should probably reset on changes to e.g. target distribution).

## Support more PRNG and low-discrepancy randomness

Note that theres a fundamental difference in low-discrepancy RNGs compared to `normal` RNGs.
They have to be aware of the output distribution they're supposed to resemble. 
Concretly this means that (to work properly) they need to sample in 2D space immediately.
The implementations also are all uniform samplers. 
IDK how to transform that yet while retaining proper low-discrepancy. A normal transformation probably suffices, but that is to be explored.

* actually make PRNG generic and allow choice in UI, currently hard-coded
* https://en.wikipedia.org/wiki/Quasi-Monte_Carlo_method
* https://crates.io/crates/sobol_burley
* https://crates.io/crates/halton
* https://crates.io/crates/quasirandom

## PWA

* currently not offline capable. I removed the serviceworker:
  * did not work with the autogenerated file hashes
  * disabling the file hashes makes the task of actually getting a newer version problematic, I looked into this but eventually gave up.
* consider the experimental related_applications attribute https://developer.mozilla.org/en-US/docs/Web/Manifest/related_applications

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
