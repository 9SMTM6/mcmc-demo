# TODO:

## Rendering shapes

https://medium.com/@lumi_/render-any-2d-shape-using-a-single-shader-263be93879d9
    https://github.com/GeorgeAzma/silk-engine

alternative, use Painter::line_segment https://github.com/emilk/egui/blob/b1dc059ef3a18ec67c9283fed24c07bc2dfeefcc/crates/egui_demo_lib/src/demo/misc_demo_window.rs#L162


not really what I was looking for but gotta remember that for the future:
https://getcode.substack.com/p/massively-parallel-fun-with-gpus

## Size-reductions:

Possibilities:

* (DONE) wasm-opt
* disable webgl compat layer
* (DONE) disable persistence
* (DONE) disable default_fonts on egui
* (DONEISH, gzip) compression on browser

All together were able to reduce size from ~7.3MB to ~1MB

I enabled all by the webgl_compat disable and brotli by default in `trunk build --release`.

### Compression
GH-Pages doesnt support brotli, so gzip it is.

In addition to all other optimizations, brotli manages to reduce WASM size further from ~2.4MB to ~1MB without webgl_compat etc, ~5MB to ~1.5MB with.

`brotli dist/mcmc_demo-*_bg.wasm`

Integrating it would be harder, trunk doesnt support it, and I'm unsure if thats a great idea, since I dont think (?) brotli can stream decompression, which in turn means that the ability to stream-run wasm in browsers for faster interactivity is gone.

Just checked, but yeah, brotli supports streaming. Theres also a feature request for trunk that adds sompression, but there was no progress: https://github.com/trunk-rs/trunk/issues/91

Also note that zstd is also an option. Its better still than brotli in most tests. But Safari doesnt support it: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding
I also just checked, and with default settings, while compression was A LOT faster, it also lead to worse compression ratio with webgl_compat.

### Wasm-opt

`wasm-opt -O2 --fast-math mcmc_demo-95a84b4d1a0af565_bg.wasm -o mcmc_demo-95a84b4d1a0af565_bg.opt.wasm`

Went from ~7.5 MB to ~5.5 MB in my testing

Optimizing with `-Oz` did not make a difference in my testing. Neither for wasm-opt nor cargo.

### Disable WebGL compat

No good reason for that currently, and Serde takes up a bunch of space

went from ~7.5 MB to ~4.0 MB

### disable persistance

requires removal of serde and persisence feature on eframe

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
