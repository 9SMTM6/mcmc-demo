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
Panics in background threads on the web lead to [issues](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.future_to_promise.html#panics).

The project is using tokio synchronization primitives for async, however tokio doesn't support deployment of runtimes on the web.
There are efforts like [tokio-with-wasm](https://github.com/cunarist/tokio-with-wasm) but they've got their own limitations.
Thus execution on the web will be on the main thread with a LocalSet.
When running natively I utilize a full fat multithreaded tokio runtime.

## Development documentation

### Running the project

If you want to run the project locally for development, ensure that `rustup` is installed (either via [the official method](https://rustup.rs/) or the system package manager) and run `cargo +stable run [--release]` in the top level of this project.
This will use the stable rust compiler to create a native executable and run it.

You have to override the toolchain to stable, as the WASM version of this application uses true multithreading on the web, which in turn requires the `build-std` flag, which in turn requires nightly rust, thus we set an override in [a configuration file](./rust-toolchain.toml).

`build-std` also requires manual specification of the target when active, so if you want to use the nightly (default) compiler, you have to run `cargo run --target host-tuple [--release]`.
This also propagates to your IDE language-server. 
We ship a configuration file that does this for `vscode` with `rust-analyzer` and targets the web at [.vscode/settings.json](.vscode/settings.json), and also one for debugging in `vscode` with `lldb` at [.vscode/launch.json](.vscode/launch.json).


Aside from the Rust toolchain (including `rust-analyzer`, `clippy`, `rustfmt`, most will be installed on demand if not already present) we use a bunch of other tools for different parts of the project:

* [`trunk-rs`](https://github.com/trunk-rs/trunk) as asset bundler for the web deployment
    * it will also download `wasm-bindgen`, `wasm-opt` etc. on demand - on most OS's
* Default POSIX `diff`, `patch` and `find` e.g., to avoid code duplication for some files
* [`just`](https://github.com/casey/just) as command runner, it's similar to `make`
* [`typst`](https://github.com/typst/typst) for the logo - yeah its a bit overkill
    * and [`svgcleaner`](https://github.com/RazrFalcon/svgcleaner) and
    * `rsvg-convert` for deployment of said logo
* [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) for dependency linting and cleanup
* [`typos`](https://github.com/crate-ci/typos) for spell-checking
* Additional tools you'll probably never use yourself:
    * `lldb` for debugging CPU code on native (usually distributed alongside `clang` or `llvm`)
    * `brotli` for compression on the web.
    * [`tokio-console`](https://github.com/tokio-rs/console) for debugging of async on native
        * The related just task uses `konsole` to open the tokio-console in a separate terminal emulator
    * I've got some experiments with `qrenderdoc`, `caddy` and `podman` (`docker`)

Most of the time though you should get away with just installing `trunk-rs` to test on the web by executing `trunk serve --config Trunk.fat.toml` in [./executable/](./executable/), or also install `just` and run `just trunk_fat` at the top level.

### Project structure

The project has to use a separate crate for rust proc-macros, thus there are multiple cargo projects at the crate root (which is a cargo workspace), but most things will happen in [./executable/](./executable/), the project root holds general config files.

* [./executable/src/](./executable/src/) contains the Rust code for the application
    * additional documentation for the internal structure might be found in rustdoc
* [./executable/assets/](./executable/assets/) contains the application logo and then assets for the web deployment
* [./executable/shaders/](./executable/shaders/) contains the shader code. It's using WGSL shaders with a primitive homebrew buildtime import system that works similar to C-Style `#include`s with `#pragma once`. The shader code is checked at build time for syntax errors by abusing `wgsl-bindgen`.
* [./executable/dist/](./executable/dist/) contains a trunk project that works as a wrapper around 2 variants of the Web-Version of this application.
    * One version with less features, that's smaller (still ~1.2 MB unfortunately)
    * One version with all features possible on the Web

The project has a lot of features that allow for different additional functionality to be added. The primary purpose of these is to allow to strip out 'wasteful' features in terms of deployment-size, such as persistence with serde, and/or disable debug tooling.
Documentation for the features may be found in Comments above them in [./executable/Cargo.toml](./executable/Cargo.toml).
