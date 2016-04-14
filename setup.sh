#!/bin/bash -e

cd /tmp
curl -O https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-setup
chmod +x rustup-setup
./rustup-setup -y --default-toolchain nightly
source ~/.cargo/env
rustup default nightly
rustup target add x86_64-unknown-linux-musl
