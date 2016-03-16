#!/bin/bash -e

echo -e "Installing and building musl"
mkdir musldist
PREFIX=$(pwd)/musldist
# Build musl
curl -O http://www.musl-libc.org/releases/musl-1.1.14.tar.gz
tar xf musl-1.1.14.tar.gz
cd musl-1.1.14/
CFLAGS=-fPIC ./configure --disable-shared --prefix=$PREFIX &> /dev/null
make &> /dev/null
make install &> /dev/null
cd ..
echo -e "Installing and building nightly rustc compiler with musl target"
mkdir muslrust
cd muslrust
# hack
curl -sSf https://static.rust-lang.org/rustup.sh > rustup.sh
chmod +x rustup.sh
./rustup.sh --disable-sudo --channel=nightly --with-target=x86_64-unknown-linux-musl --prefix=$PREFIX
