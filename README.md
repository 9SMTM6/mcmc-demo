# <img src="./executable/assets/favicon.svg" alt="MCMC-Demo-icon" width="50"/> MCMC-Demo

This application is still work in progress (WIP).

There is still next to no documentation, both for the users, as well as potential developers.

This is originally inspired by https://github.com/chi-feng/mcmc-demo. It's technologically different and more meant to explore the long-term results of different parameter settings and target distributions with batched execution.
While the JavaScript original struggles after running for a while, this can handle a lot more samples, and it also has different display options.

It's built using Rust and WebGPU, which allows execution in the Browser via Webassembly (WASM) similar to the JavaScript Project, while having more optimization potential and the creation of native applications.

Note that WebGPU currently isn't well supported even in up-to-date browsers, only Chromium supports it on Windows, MacOS and Android.
This hugely limits the potential users of the web version, but as this is simply a pet project I'll live with it.
Originally I intended to simply support older Browsers with WGPUs compatibility layer to WebGL, however that compatibility layer turned out to be very limited.
Also WGPU has some considerable differences when executing on the web and natively.

It's also utilizing Webassembly Threads with shared memory for efficient mid-level parallelism. This requires the use of nightly Rust when targeting the Web, and makes deployment harder (enabling shared memory on the Web requires certain headers that enforce measures against Spectre like sidechannel attacks).
Panics in background threads on the web lead to [issues](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/fn.future_to_promise.html#panics).

The project is using tokio synchronization primitives for async, however tokio doesn't support deployment of runtimes on the web.
There are efforts like [tokio-with-wasm](https://github.com/cunarist/tokio-with-wasm) but they've got their own limitations.
Thus execution on the web will be on the main thread with a LocalSet.
When running natively I utilize a full fat multithreaded tokio runtime.

For development information, see [Contributing.md](./Contributing.md).
