# - uses: extractions/setup-just@v2


generate_favicon:
    typst compile assets/favicon.typ --format svg

# Generate new matching `index.fat.html` from `index.html` and `index.fat.html.patch`
patch_fat_html:
    patch index.html -o index.fat.html < index.fat.html.patch

# Generate new patch file from matched `index.html` and `index.fat.html`
diff_fat_html:
    diff -u index.html index.fat.html > index.fat.html.patch

ci_fmt:
    cargo +stable fmt --all -- --check

ci_clippy:
    cargo +stable clippy --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings

ci_clippy_wasm:
    cargo clippy --workspace --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings

ci_cargo_deny:
    cargo deny check

ci_test:
    cargo +stable test --locked --target x86_64-unknown-linux-gnu --lib

ci: patch_fat_html ci_fmt ci_clippy ci_clippy_wasm ci_cargo_deny ci_test
