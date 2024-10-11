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

ci_clippy:
    cargo +stable --locked clippy --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings

ci_clippy_wasm:
    cargo --locked clippy --workspace --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings

ci_cargo_deny:
    cargo +stable --locked deny check --hide-inclusion-graph --graph duplicates_tree

render_duplicates: ci_cargo_deny
    rm -r duplicates_tree/rendered_graph_output || true
    mkdir -p duplicates_tree/rendered_graph_output
    find duplicates_tree/graph_output/ -name '*.dot' | xargs -I {} sh -c 'dot -Tpdf "{}" -o "duplicates_tree/rendered_graph_output/$(basename "{}" .dot).pdf"'

ci_test:
    cargo +stable --locked test --target x86_64-unknown-linux-gnu --lib

ci_semver_updates:
    cargo +stable --locked generate-lockfile

# This isn't included in other things, as its slow and doesn't work without network
fix_ci_semver_updates:
    cargo +stable generate-lockfile

ci_required_for_deploy: patch_fat_html ci_clippy ci_clippy_wasm ci_test

ci_typo:
    typos

ci_qa: ci_fmt ci_cargo_deny ci_typo

ci: ci_qa ci_required_for_deploy

full_ci: ci_qa ci_required_for_deploy ci_semver_updates trunk_slim trunk_fat

# compression level 4 is my best guess to the compression level couldflare defaults to on pages
# https://blog.cloudflare.com/results-experimenting-brotli/
benchmark_wasm_size compression_level='4':
    # ignore files in my global ~/.cargo/config.toml
    # wont work on other setups!
    export RUSTFLAGS=
    export CARGO_BUILD_INCREMENTAL=false
    trunk build --release
    trunk build --config Trunk.fat.toml --release
    brotli -q {{compression_level}} -f dist/*/mcmc-demo*_bg.wasm
    lsd -lah dist/*/mcmc-demo*_bg.was*

fix_ci_unstaged:
    typos --write-changes

fix_ci_staged:
    cargo --locked clippy --allow-staged --workspace --target wasm32-unknown-unknown --all-features --all-targets --fix
    cargo +stable --locked clippy --allow-staged --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets --fix
    cargo +stable --locked fmt --all

tokio_console:
    RUSTFLAGS="--cfg tokio_unstable" cargo +stable build --features tokio_console --target x86_64-unknown-linux-gnu
    ./target/x86_64-unknown-linux-gnu/debug/mcmc-demo &
    # spawn tokio-console in another terminal window
    konsole -e tokio-console

# Note that this WILL stage current changes
fix_ci: fix_ci_unstaged && fix_ci_staged
    git add .

fix_full_ci: fix_ci fix_ci_semver_updates

trunk_fat: patch_fat_html
    just mcmc_demo/trunk_fat

alias tf := trunk_fat

trunk_slim: patch_fat_html
    just mcmc_demo/trunk_slim

alias ts := trunk_slim

# I have yet to find a practical use for this, but how to do this under wayland isn't well documented, so lets keep this around in case it ever becomes helpful.
renderdoc:
    WAYLAND_DISPLAY="" qrenderdoc renderdoc_settings.cap
