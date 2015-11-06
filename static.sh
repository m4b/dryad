#!/bin/bash

PREFIX=musldist
SONAME=dryad.so.1

export LD_LIBRARY_PATH=$PREFIX/lib:$LD_LIBRARY_PATH

clang -c -o start.o src/arch/x86/asm.s

$PREFIX/bin/rustc --target=x86_64-unknown-linux-musl src/main.rs -O -g --emit obj -o dryad.o

ld -I/tmp/$SONAME -soname $SONAME -pie -static -Bsymbolic -nostdlib -shared -e _start -o $SONAME start.o dryad.o $PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib/libcore-71b07a99.rlib $PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd-71b07a99.rlib

cp $SONAME /tmp/

