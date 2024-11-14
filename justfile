mod executable

# this ensures that just running just give a list of commands
_list:
    just --list

generate_favicon:
    just executable/generate_favicon

# Generate new matching `index.fat.html` from `index.html` and `index.fat.html.patch`
patch_fat_html:
    just executable/patch_fat_html

# Generate new patch file from matched `index.html` and `index.fat.html`
diff_fat_html:
    just executable/diff_fat_html

ci_fmt:
    cargo +stable --locked fmt --all -- --check

alias fmt := ci_fmt

ci_clippy:
    cargo +stable --locked clippy --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings

ci_clippy_wasm:
    cargo --locked clippy --workspace --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings

clippy: ci_clippy ci_clippy_wasm

ci_cargo_deny:
    cargo +stable --locked deny check --hide-inclusion-graph --graph duplicates_tree

alias deny := ci_cargo_deny

duplicates_graphs: ci_cargo_deny
    rm -r duplicates_tree/rendered_graph_output || true
    mkdir -p duplicates_tree/rendered_graph_output
    find duplicates_tree/graph_output/ -name '*.dot' | xargs -I {} sh -c 'dot -Tpdf "{}" -o "duplicates_tree/rendered_graph_output/$(basename "{}" .dot).pdf"'

ci_test:
    cargo +stable --locked test --target x86_64-unknown-linux-gnu --lib

alias test := ci_test

ci_semver_updates:
    cargo +stable --locked generate-lockfile

# This isn't included in other things, as its slow and doesn't work without network
semver_updates:
    cargo +stable generate-lockfile

ci_deploy_tests: patch_fat_html clippy ci_test

ci_typo:
    typos

ci_qa: ci_fmt ci_cargo_deny ci_typo

ci: ci_qa ci_deploy_tests semver_updates

# compression level 4 is my best guess to the compression level couldflare defaults to on pages
# https://blog.cloudflare.com/results-experimenting-brotli/
benchmark_wasm_size compression_level='4':
    just executable/benchmark_wasm_size {{compression_level}}

tokio_console:
    RUSTFLAGS="--cfg tokio_unstable" cargo +stable build --features tokio_console,debounce_async_loops --target x86_64-unknown-linux-gnu
    ./target/x86_64-unknown-linux-gnu/debug/mcmc-demo &
    # spawn tokio-console in another terminal window
    konsole -e tokio-console

ci_easy_autofixes: diff_deploy_html diff_fat_html semver_updates
    cargo fmt
    typos --write-changes

alias eaFix := ci_easy_autofixes

ci_staged_autofixes:
    cargo --locked clippy --allow-staged --workspace --target wasm32-unknown-unknown --all-features --all-targets --fix
    cargo +stable --locked clippy --allow-staged --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets --fix
    cargo +stable --locked fmt --all

trunk_fat +cmd="serve":
    just executable/trunk_fat {{cmd}}

alias tf := trunk_fat

trunk_slim +cmd="serve":
    just executable/trunk_slim {{cmd}}

alias ts := trunk_slim

trunk_deploy cmd="serve" +flags="":
    just executable/trunk_deploy {{cmd}} {{flags}}

caddy_prepare:
    just trunk_deploy build --release
    just executable/dist/precompress

caddy_serve:
    caddy run

alias td := trunk_deploy

diff_deploy_html:
    just executable/diff_deploy_html

patch_deploy_html:
    just executable/patch_deploy_html

# I have yet to find a practical use for this, but how to do this under wayland isn't well documented, so lets keep this around in case it ever becomes helpful.
renderdoc:
    WAYLAND_DISPLAY="" qrenderdoc renderdoc_settings.cap
