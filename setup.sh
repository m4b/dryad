#!/bin/bash -e

cd /tmp
curl https://sh.rustup.rs > rustup-setup
chmod +x rustup-setup
./rustup-setup -y --default-toolchain nightly
source ~/.cargo/env
rustup default nightly
rustup target add x86_64-unknown-linux-musl
rustup show
ld -v
