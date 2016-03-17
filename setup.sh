#!/bin/bash -e

PREFIX=$(pwd)/rust
echo -e "Installing and building nightly rustc compiler with musl target into $PREFIX"
mkdir $PREFIX
cd $PREFIX
# hack
curl -sSf https://static.rust-lang.org/rustup.sh > rustup.sh
chmod +x rustup.sh
./rustup.sh --disable-sudo --channel=nightly --with-target=x86_64-unknown-linux-musl --prefix=$PREFIX
