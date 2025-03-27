## TODO:

### Update / Fix Profiling

GPU Profiling with wgpu-profiler did not work out yet, and profiling egui required use of puffin until now. But now egui uses the profiling crate, allowing usage of other profilers. So try and unify these profilers, if possible, and then check back with wgpu-profiler.

### Compute shader

I dont know if any of the below ideas for speeding up the diff rendering would work out. And in the end, dont think I'll get much use out of knowing how to do that (meaning I'll forget it anyways).

One thing where the probability of reuse is much higher, and that more connects to my past knowledge, is compute shaders.
Its a bit annoying to go away from the ability to render everything in real time always, but at the same time that lifts hard limits that were always going to be there with the previous approach - whether we were close to reaching them or not.

I currently envison this approach (lets see how much of this I'll get):

0. still use max-scaling. With near-uniform distributions we otherwise get a far to depressed dynamic range where things actually happen.
1. determine device limits to divide work accordingly
2. since we don't render directly anymore, I've got much more freedom in splitting up the workload, concretely optimizing for typical buffers. So I intend to break up the determination of the approx distribution into multiple sets of reference points.
3. do it in a compute shader
4. The result can be stored either:
    * in a texture.
    * a storage buffer (as long as that is efficient, textures are optimized to only load parts)
5. execute that compute shader (and the original computation) in a separate thread (see [wasm-threads](README#wasm-threads)) with a nice loading animation - `egui::widgets::ProgressBar` - while waiting. This works around inefficient diff approach, though perhaps it can still be improved with some space partition like Quadtree.
6. the storage will never have to leave the GPU. Compute it once, read the result it in a fragment shader where the actual colors are determined
7. with that I could also consider decoupling calculation resolution and render resolution, but I think for now I'll keep them coupled
8. In order to avoid numerical stability issues I'll probably add some normalization after N steps. I have to decide on a proper strategy for that. Perhaps I can actually do it based on current maximum instead. Most of these strategies will lead to systemctic errors in the precision, since rounding might happen in different situations, but I'm fine with that.

#### Mostly outdated considerations

Also see [semi-official explainer](https://github.com/gpuweb/gpuweb/wiki/The-Multi-Explainer#multi-threading-javascript).
This concerns and links to official issues about accessing webgpu from multiple wasm threads, as well as having GPU work on multiple queues.
Also consider this note:

> Note: Queues do not directly provide GPU parallelism or concurrency. All programmable GPU work is structured to be parallelizable across GPU cores, and it is already possible for independent work on a single queue to be interleaved or run out-of-order at the discretion of the hardware/driver schedulers, within the constraints of ordering guarantees (which, in native APIs, are implicit or provided by the application via barriers). Multi-queue only improves the occupancy of available hardware on the GPU when the two tasks at the front of two queues can better occupy hardware resources than just one of the tasks would.

My observation was that compute tasks scheduled from a separate command_encoder did block egui rendering.

My understanding from the quote is that:
1. some tasks might be parallelizable even with a single queue
2. they might currently be sequential because either the compute task overloads the gpu or 
3. there is some implicit synchronization between these tasks (very possible since the render does render the buffer written from compute)

I might be able to create some abstraction that might or might not start the a gpu compute task from a background thread

Consider that moving any between threads is forbidden on the web:
* https://gpuweb.github.io/gpuweb/explainer/#multithreading-transfer
* https://github.com/gfx-rs/wgpu/issues/2652
* https://wgpu.rs/doc/wgpu/#other (fragile-send-sync-non-atomic-wasm)
* kinda conflicts with spec? https://www.w3.org/TR/webgpu/#canvas-hooks
* other than that I could not find any real mention of thread or web worker (other than availability of creation methods)
  in neither the wgsl nor the webgpu spec.

Note that a compute shader in a webworker is supposed to work according to [spec](https://www.w3.org/TR/webgpu/#navigator-gpu), but [apparently firefox doesnt support that](https://developer.mozilla.org/en-US/docs/Web/API/WorkerNavigator/gpu), even on nightly. So here's hoping that they will eventually support it when they release.

Uuuuh. Just saw that it apparently explicitly isn't supported on Chromium Linux either. So thats not going to happen.

#### WebGPU Synchronization

Originally I was under the impression that global synchronization on the GPU was impossible, from some article that I might look for again in the future.

But it seems I was mistaken.
Barriers are indeed only allowed on a workgroup level.
Atomics however seem to be synchronized on the entire GPU.
That is at least whats done [here](https://webgpufundamentals.org/webgpu/lessons/webgpu-compute-shaders-histogram.html).

Look that up in the future.

### Scaling of distributions and approximations

For the approximation, distribution scaling doesn't work currently, since we've got difficulties scaling it. For distribution scaling we would have to integrate it for normalization. For every change, currently every render.
Alternatives: Max-Scaling. Still problematic for approximation, but perhaps doable if we scan all values with some "lower resolution" compute shader. But that would excascerbate the current scaling issue, since then this would have to happen before render and for every subsequent one.

Alternatively we could use the render shader as a compute shader of sorts, and use it for normalization with 1 frame delay. IDK if thats gonna go well with eguis energy saving render approach, and it also leads to artifacts/flickering, which may be an accessibility issue (this application is already not accessible, but this would hurt another groups than so far).

### Problematic approach to diff rendering

The approach I took to rendering/calculating the approximation doesn't scale at all.
The issue is (probably) that EVERY fragment shader (every pixel) will read every approximation point. That won't do.
We would need to split this up, but thats not really easy, and probably goes kind of deep into game development adjacent topics, which I dont really want to do.

### Find a way to Profile performance issues

Generally I should find a way to profile webgpu render. Currently I'm mostly guessing from past reference points, and while I'm decently certain in my conclusions, it would be nice to have confirmation, and some foresight into upcoming issues ("will solving this just lead to another very close bottleneck", which is currently stopping me from some experimentations).

Currently working:
* with feature "profile"
  * we start puffin, which can measure scope duration and gives a generic flame-graph from the rust side (no GPU)
  * there appears a "backend" button which was largely copied from the egui demo and offers frametimes as well as information about GPU allocations etc.

Future ideas:
* implement tracing from rust side for wgpu actions, blocked by: https://github.com/gfx-rs/wgpu/issues/5974
* implement the stuff from https://webgpufundamentals.org/webgpu/lessons/webgpu-timing.html
  * actually, perhaps instead use https://github.com/Wumpf/wgpu-profiler

### Non-Batched execution

Currently I only support batched execution, to quickly see results of different configurations.
In the future I also want to support a substep execution such as in the [original inspiration](https://chi-feng.github.io/mcmc-demo/app.html?algorithm=RandomWalkMH&target=banana).
With a history of past proposals etc.
Maybe also with history navigation, if that actually fits the workflow (atm I dont think so, since RNG is seeded once currently, and we should probably reset on changes to e.g. target distribution).

### Support more PRNG and low-discrepancy randomness

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

# eframe template

### Learning about egui

The official egui docs are at <https://docs.rs/egui>. If you prefer watching a video introduction, check out <https://www.youtube.com/watch?v=NtUkr_z7l84>. For inspiration, check out the [the egui web demo](https://emilk.github.io/egui/index.html) and follow the links in it to its source code.

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

You can test the template app at <https://emilk.github.io/eframe_template/>.
