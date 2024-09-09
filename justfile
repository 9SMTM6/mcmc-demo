generate_favicon:
    typst compile assets/favicon.typ --format svg

# Generate new matching `index.fat.html` from `index.html` and `index.fat.html.patch`
patch_fat_html:
    patch index.html -o index.fat.html < index.fat.html.patch

# Generate new patch file from matched `index.html` and `index.fat.html`
diff_fat_html:
    diff -u index.html index.fat.html > index.fat.html.patch || 0

ci_fmt:
    cargo +stable --locked fmt --all -- --check

ci_clippy:
    cargo +stable --locked clippy --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings

ci_clippy_wasm:
    cargo --locked clippy --workspace --target wasm32-unknown-unknown --all-features --all-targets -- -D warnings

ci_cargo_deny:
    cargo +stable --locked deny check

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

ci_qa: ci_fmt ci_cargo_deny ci_typo ci_semver_updates

ci: ci_qa ci_required_for_deploy

fix_ci_unstaged:
    cargo +stable generate-lockfile
    typos --write-changes

fix_ci_staged:
    cargo --locked clippy --allow-staged --workspace --target wasm32-unknown-unknown --all-features --all-targets --fix
    cargo +stable --locked clippy --allow-staged --workspace --target x86_64-unknown-linux-gnu --all-features --all-targets --fix
    cargo +stable --locked fmt --all

# Note that this WILL stage current changes
fix_ci: fix_ci_unstaged && fix_ci_staged
    git add .

trunk_fat: patch_fat_html
    trunk serve --config Trunk.fat.toml

alias tf := trunk_fat

trunk_slim:
    trunk serve

alias ts := trunk_slim
