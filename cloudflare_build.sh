#!/bin/env sh

set -x
set -eo

apt update

apt install -y curl wget brotli gzip gcc

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup_install.sh

sh ./rustup_install.sh --target wasm32-unknown-unknown -y

export PATH=$PATH:$HOME/.cargo/bin

wget -qO- https://github.com/thedodd/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-

./trunk build --release --public-url mcmc-webgpu-demo.pages.dev
