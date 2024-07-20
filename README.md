# TODO:

## Non-Batched execution

Currently I only support batched execution, to quickly see results of different configurations.
In the future I also want to support a substep execution such as in the [original inspiration](https://chi-feng.github.io/mcmc-demo/app.html?algorithm=RandomWalkMH&target=banana).

## Execution of batches on web

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

## Allocator for shipped linux binaries

OUT OF DATE: Because of build issues we switched to glibc anyways.

Linux releases are built with musl in the provided pipeline.

This may cause performance regressions compared to glibc.

See https://superuser.com/a/1820423.

Benchmark from 2021: https://github.com/BurntSushi/ripgrep/issues/1691. Not a huge difference IMO, and even if feature gated I'd like to avoid yet another dependency. I might benchmark this at some point, but since my application also does a bunch of other things benchmarking this isnt entirely trivial, so I'll get to it if I ever do.

## Support more PRNG and low-discrepancy randomness

https://en.wikipedia.org/wiki/Quasi-Monte_Carlo_method
https://crates.io/crates/sobol_burley
https://crates.io/crates/halton
https://crates.io/crates/quasirandom

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
