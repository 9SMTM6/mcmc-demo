# <img src="./executable/assets/favicon.svg" alt="MCMC-Demo-icon" width="50"/> MCMC-Demo

This application is still work in progress (WIP).

There is still next to no documentation, both for the users, as well as potential developers.

This is originally inspired by https://github.com/chi-feng/mcmc-demo. It's technologically different and more meant to explore the long-term results of different parameter settings and target distributions with batched execution.
While the JavaScript original struggles after running for a while, this can handle a lot more samples, and it also has different display options.

It's built using Rust and WebGPU, which allows execution in the Browser via Webassembly (WASM) similar to the JavaScript Project, while having more optimization potential and supporting deployment as native application.

> Note that WebGPU currently isn't well supported even in up-to-date browsers, only Chromium supports it on Windows, MacOS and Android.
This hugely limits the potential users of the web version, but as this is simply a pet project I'll live with it.
Originally I intended to simply support older Browsers with WGPUs compatibility layer to WebGL, however that compatibility layer turned out to be very limited.
Also WGPU has some considerable differences when executing on the web and natively.

The project uses two different concurrency tools for distinct purposes:

- **Multithreading for compute tasks**: Used for parallel execution of computational workloads (e.g., MCMC sampling batches) using threads with shared memory.
  - On **native platforms**: This is straightforward using standard Rust threads with shared memory.
  - On **web platforms**: This requires WebAssembly threads with shared memory, but these come with significant challenges: it requires nightly Rust, makes deployment harder (enabling shared memory on the Web requires certain headers that enforce measures against Spectre-like sidechannel attacks), and panics in background threads on the web lead to [issues](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/fn.future_to_promise.html#panics).

- **Async task runtime**: Both platforms use Tokio synchronization primitives for async operations.
  - On **native platforms**: Utilizes a full multithreaded Tokio runtime for distributing async tasks across multiple threads, enabling parallelism for I/O-bound operations.
  - On **web platforms**: Uses a single-threaded runtime with a LocalSet on the main thread. Tokio doesn't support multithreaded runtimes on the web; there are efforts like [tokio-with-wasm](https://github.com/cunarist/tokio-with-wasm) but they've got their own limitations.

For development information, see [Contributing.md](./Contributing.md).
